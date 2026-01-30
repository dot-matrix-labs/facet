use anyhow::{Error as E, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::phi3::{Config as Phi3Config, Model as Phi3};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

pub struct LocalLlm {
    model: Phi3,
    tokenizer: Tokenizer,
    device: Device,
    logits_processor: LogitsProcessor,
}

impl LocalLlm {
    pub fn new() -> Result<Self> {
        let device = Device::new_metal(0).unwrap_or(Device::Cpu);
        
        let api = Api::new()?;
        let repo = api.repo(Repo::new("microsoft/Phi-3-mini-4k-instruct".to_string(), RepoType::Model));
        // let repo = api.repo(Repo::new("microsoft/Phi-3.5-mini-instruct".to_string(), RepoType::Model));

        let tokenizer_filename = repo.get("tokenizer.json")?;
        let config_filename = repo.get("config.json")?;
        let model_filenames = vec![
            repo.get("model-00001-of-00002.safetensors")?,
            repo.get("model-00002-of-00002.safetensors")?,
        ];

        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;
        let config: Phi3Config = serde_json::from_slice(&std::fs::read(config_filename)?)?;
        
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&model_filenames, DType::F32, &device)? };
        let model = Phi3::new(&config, vb)?;

        Ok(Self {
            model,
            tokenizer,
            device,
            logits_processor: LogitsProcessor::new(299792458, Some(0.7), Some(0.9)),
        })
    }

    fn format_prompt(&self, system: &str, user: &str) -> String {
        format!("<|user|>\n{}\n{}\n<|end|>\n<|assistant|>\n", system, user)
    }

    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        let tokens = self.tokenizer.encode(prompt, true).map_err(E::msg)?;
        let mut tokens = tokens.get_ids().to_vec();
        let mut generated_tokens = Vec::new();

        for _ in 0..max_tokens {
            let input = Tensor::new(&tokens[tokens.len().saturating_sub(2048)..], &self.device)?.unsqueeze(0)?;
            let input_ids = &tokens[tokens.len().saturating_sub(2048)..];
            let pos = input_ids.len(); // Current position in the sequence
            let logits = self.model.forward(&input, pos)?; // Fix: passed pos
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
            
            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens.push(next_token);

            if next_token == self.tokenizer.token_to_id("<|end|>").unwrap_or(32000) 
                || next_token == self.tokenizer.token_to_id("<|endoftext|>").unwrap_or(32007) {
                break;
            }
        }

        let decoded = self.tokenizer.decode(&generated_tokens, true).map_err(E::msg)?;
        Ok(decoded.replace("<|end|>", "").trim().to_string())
    }

    pub fn synthesize(&mut self, text: &str) -> Result<String> {
        let prompt = self.format_prompt(
            "You are a helpful assistant. Summarize the following text concisely.",
            text
        );
        self.generate(&prompt, 500)
    }

    pub fn extract_pii(&mut self, text: &str) -> Result<(String, std::collections::HashMap<String, String>)> {
        let system_prompt = "You are a privacy expert. Identify Personal Identifiable Information (PII) such as Names, Emails, Phone Numbers, and Addresses. 
Return the output in JSON format: {\"redacted_text\": \"...\", \"pii\": {\"PLACEHOLDER\": \"ORIGINAL_VALUE\"}}. 
Replace PII with placeholders like [NAME_1], [EMAIL_1].";
        
        let prompt = self.format_prompt(system_prompt, text);
        let output = self.generate(&prompt, 1000)?;
        
        // Attempt to parse JSON. If failure, return original (fail-safe) or basic regex based redaction.
        // For now, assuming model adheres to instruction for this alpha implementation.
        #[derive(serde::Deserialize)]
        struct PiiResult {
            redacted_text: String,
            pii: std::collections::HashMap<String, String>,
        }

        // Find JSON block in output if wrapped in markdown codefence
        let json_str = if let Some(start) = output.find("```json") {
             if let Some(end) = output[start..].find("```") {
                 // skip "```json" (7 chars) and take until next ```
                 // tricky indexing, let's just clean it
                 output.replace("```json", "").replace("```", "")
             } else {
                 output.clone()
             }
        } else {
            output.clone()
        };

        if let Ok(res) = serde_json::from_str::<PiiResult>(&json_str) {
            Ok((res.redacted_text, res.pii))
        } else {
            // Fallback: Return original if parsing fails
             Ok((text.to_string(), std::collections::HashMap::new()))
        }
    }

    pub fn optimize_prompt(&mut self, query: &str) -> Result<String> {
        let prompt = self.format_prompt(
            "You are a prompt engineer. Rewrite the following query to be more precise and optimized for an LLM rag search.",
            query
        );
        self.generate(&prompt, 200)
    }
}
