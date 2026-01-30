use crate::claude::ClaudeClient;
use anyhow::Result;

// TODO: Re-enable OpenAI support by adding the `_api` feature to async-openai
// use async_openai::{
//     types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role},
//     Client,
// };

// Local module
pub mod local;
pub use local::LocalLlm;

pub enum LlmProvider {
    // OpenAI(Client<async_openai::config::OpenAIConfig>, String),
    Claude(ClaudeClient),
}

pub struct LlmClient {
    provider: LlmProvider,
}

impl LlmClient {
    // TODO: Re-enable OpenAI support
    // pub fn new_openai(api_key: String, model: Option<String>) -> Self {
    //     let config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);
    //     let client = Client::with_config(config);
    //
    //     Self {
    //         provider: LlmProvider::OpenAI(client, model.unwrap_or_else(|| "gpt-4o".to_string())),
    //     }
    // }

    pub fn new_claude(binary_path: Option<String>) -> Self {
        let client = if let Some(path) = binary_path {
            ClaudeClient::with_path(path)
        } else {
            ClaudeClient::new()
        };

        Self {
            provider: LlmProvider::Claude(client),
        }
    }

    pub fn from_env() -> Self {
        // Default to Claude for now
        Self::new_claude(None)
    }

    pub async fn complete(&self, prompt: &str, system_prompt: Option<&str>) -> Result<String> {
        match &self.provider {
            // TODO: Re-enable OpenAI support
            // LlmProvider::OpenAI(client, model) => {
            //     let mut messages = Vec::new();
            //
            //     if let Some(sys) = system_prompt {
            //         messages.push(ChatCompletionRequestMessage {
            //             role: Role::System,
            //             content: Some(sys.to_string()),
            //             name: None,
            //             function_call: None,
            //         });
            //     }
            //
            //     messages.push(ChatCompletionRequestMessage {
            //         role: Role::User,
            //         content: Some(prompt.to_string()),
            //         name: None,
            //         function_call: None,
            //     });
            //
            //     let request = CreateChatCompletionRequestArgs::default()
            //         .model(model)
            //         .messages(messages)
            //         .build()?;
            //
            //     let response = client.chat().create(request).await?;
            //
            //     let content = response
            //         .choices
            //         .first()
            //         .and_then(|choice| choice.message.content.clone())
            //         .unwrap_or_default();
            //
            //     Ok(content)
            // }
            LlmProvider::Claude(client) => client.complete(prompt, system_prompt).await,
        }
    }

    pub async fn check_claude_availability() -> bool {
        ClaudeClient::is_available().await
    }
}
