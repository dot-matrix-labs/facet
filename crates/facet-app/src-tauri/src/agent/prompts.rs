//! Prompt templates for different agent workflows

use serde::{Deserialize, Serialize};

/// Type of prompt/workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptType {
    /// Generate CDP automation script
    CdpGeneration,
    /// Update agent config based on feedback
    ConfigUpdate,
    /// Improve user feedback prompt
    FeedbackImprovement,
}

/// Prompt template builder
pub struct PromptTemplate {
    prompt_type: PromptType,
}

impl PromptTemplate {
    /// Create a new prompt template
    pub fn new(prompt_type: PromptType) -> Self {
        Self { prompt_type }
    }

    /// Get the common CDP command reference text
    fn cdp_commands_reference() -> &'static str {
        r#"
IMPORTANT INSTRUCTIONS:
- You are running in HEADLESS/NON-INTERACTIVE mode
- You MUST respond with a valid CDP JSON script
- DO NOT ask clarifying questions
- DO NOT request additional information
- Your response will be executed automatically without human review

VISUALDOM FORMAT:
When step frames are captured during automation, they include a VisualDom format we created.
VisualDom combines CDP DOMSnapshot with base64-encoded images to give you precise understanding
of page structure without OCR. It includes:
- Complete DOM tree with bounding boxes for every element
- All visible text content (no OCR needed)
- Computed CSS styles (position, size, visibility, colors, fonts)
- Embedded images as base64 data URIs
- Paint order and visual stacking hierarchy

Use the CDP command reference below:

AVAILABLE CDP COMMANDS:

1. Page.navigate - Navigate to URL
   {"method": "Page.navigate", "params": {"url": "https://example.com"}}

2. Page.captureScreenshot - Take screenshot
   {"method": "Page.captureScreenshot", "params": {"format": "png", "captureBeyondViewport": true}, "save_as": "screenshot.png"}

3. Runtime.evaluate - Execute JavaScript
   {"method": "Runtime.evaluate", "params": {"expression": "document.title", "returnByValue": true}, "save_as": "result.json"}

4. Input.insertText - Type text
   {"method": "Input.insertText", "params": {"text": "Hello World"}}

5. Input.dispatchMouseEvent - Mouse actions
   {"method": "Input.dispatchMouseEvent", "params": {"type": "mousePressed", "x": 100, "y": 200, "button": "left", "clickCount": 1}}

6. Input.dispatchKeyEvent - Keyboard actions
   {"method": "Input.dispatchKeyEvent", "params": {"type": "keyDown", "key": "Enter"}}

OUTPUT FORMAT (JSON only, no markdown, no explanations, no questions):

{
  "name": "descriptive-name",
  "description": "What this script does",
  "cdp_commands": [
    {
      "method": "Page.navigate",
      "params": {"url": "..."},
      "description": "Navigate to page"
    },
    {
      "method": "Runtime.evaluate",
      "params": {"expression": "...", "returnByValue": true},
      "save_as": "optional-output.json",
      "description": "Perform action"
    }
  ]
}

Generate the CDP script now. Output ONLY valid JSON. Do not include any text before or after the JSON."#
    }

    /// Build planning prompt (first phase - determine if clarification is needed)
    #[allow(dead_code)]
    pub fn build_planning_prompt(
        &self,
        user_request: &str,
        current_url: Option<&str>,
        page_title: Option<&str>,
        agent_instructions: &str,
    ) -> String {
        let context = if let (Some(url), Some(title)) = (current_url, page_title) {
            format!("\nCURRENT PAGE:\n- URL: {}\n- Title: {}\n", url, title)
        } else {
            String::new()
        };

        format!(
            r#"{agent_instructions}

{context}
USER REQUEST: {user_request}

TASK: Analyze if you can generate a CDP automation script for this request, or if you need clarification.

RESPONSE FORMATS:

If the request is CLEAR and you can proceed, respond with:
{{
  "response_type": "ready",
  "understanding": "Brief description of what you'll do",
  "next_step": "generate_script"
}}

If the request is AMBIGUOUS or you need clarification, respond with:
{{
  "response_type": "clarification_needed",
  "questions": [
    {{
      "question": "What specific action should I perform?",
      "options": ["option1", "option2", "option3"],
      "context": "Optional explanation of why this matters"
    }}
  ],
  "understanding": "What I understand so far"
}}

GUIDELINES:
- Only ask for clarification if the request is genuinely ambiguous
- Keep questions concise and actionable
- Provide multiple choice options when possible
- Aim to proceed without clarification when reasonable assumptions can be made

Respond with ONLY valid JSON in one of the above formats."#,
            agent_instructions = agent_instructions,
            context = context,
            user_request = user_request
        )
    }

    /// Build CDP generation prompt with clarification answers
    #[allow(dead_code)]
    pub fn build_cdp_prompt_with_clarification(
        &self,
        user_request: &str,
        clarification_answers: &str,
        current_url: Option<&str>,
        page_title: Option<&str>,
        agent_instructions: &str,
    ) -> String {
        let context = if let (Some(url), Some(title)) = (current_url, page_title) {
            format!("\nCURRENT PAGE:\n- URL: {}\n- Title: {}\n", url, title)
        } else {
            String::new()
        };

        format!(
            r#"{agent_instructions}

{context}
USER REQUEST: {user_request}

CLARIFICATION PROVIDED:
{clarification_answers}

Generate a JSON CDP script that accomplishes this task based on the original request and the clarification provided.

{cdp_reference}"#,
            agent_instructions = agent_instructions,
            context = context,
            user_request = user_request,
            clarification_answers = clarification_answers,
            cdp_reference = Self::cdp_commands_reference()
        )
    }

    /// Build CDP generation prompt (second phase - after clarification or if ready)
    pub fn build_cdp_prompt(
        &self,
        user_request: &str,
        current_url: Option<&str>,
        page_title: Option<&str>,
        agent_instructions: &str,
    ) -> String {
        let context = if let (Some(url), Some(title)) = (current_url, page_title) {
            format!("\nCURRENT PAGE:\n- URL: {}\n- Title: {}\n", url, title)
        } else {
            String::new()
        };

        format!(
            r#"{agent_instructions}

{context}
USER REQUEST: {user_request}

Generate a JSON CDP script that accomplishes this task.

{cdp_reference}"#,
            agent_instructions = agent_instructions,
            context = context,
            user_request = user_request,
            cdp_reference = Self::cdp_commands_reference()
        )
    }

    /// Build config update prompt
    pub fn build_config_update_prompt(
        &self,
        agent_name: &str,
        current_config: &str,
        user_feedback: &str,
        failure_context: Option<&str>,
    ) -> String {
        let failure_section = if let Some(context) = failure_context {
            format!("\nFAILURE CONTEXT:\n{}\n", context)
        } else {
            String::new()
        };

        format!(
            r#"You are updating the configuration for the "{agent_name}" agent based on user feedback.

CURRENT CONFIGURATION:
```toml
{current_config}
```

USER FEEDBACK:
{user_feedback}
{failure_section}
TASK:
1. Analyze what went wrong based on the feedback
2. Update the agent's instructions to fix the issue
3. Output the COMPLETE updated TOML configuration

RULES:
- Keep all existing fields
- Only modify the 'instructions' field and 'settings' if necessary
- Make minimal, targeted changes
- Ensure the TOML is valid
- Add specific guidance to prevent this error in the future

Output the updated TOML configuration now:"#,
            agent_name = agent_name,
            current_config = current_config,
            user_feedback = user_feedback,
            failure_section = failure_section
        )
    }

    /// Build a prompt based on the template type
    pub fn build(&self, context: PromptContext) -> String {
        match self.prompt_type {
            PromptType::CdpGeneration => self.build_cdp_prompt(
                &context.user_request,
                context.current_url.as_deref(),
                context.page_title.as_deref(),
                &context.agent_instructions,
            ),
            PromptType::ConfigUpdate => self.build_config_update_prompt(
                &context.agent_name,
                &context.current_config,
                &context.user_feedback,
                context.failure_context.as_deref(),
            ),
            PromptType::FeedbackImprovement => {
                self.build_feedback_improvement_prompt(&context.user_feedback)
            }
        }
    }

    /// Build feedback improvement prompt
    pub fn build_feedback_improvement_prompt(&self, user_feedback: &str) -> String {
        format!(
            r#"You are a helpful QA assistant. Your goal is to improve the quality of a bug report or feedback provided by a user.

USER INPUT:
{user_feedback}

INSTRUCTIONS:
1. Analyze the input for clarity, completeness, and tone.
2. If the input is vague (e.g., "it's broken"), ask 1-2 specific clarifying questions to get more details.
3. If the input is relatively clear but unstructured, rewrite it into a clear, professional format (e.g., describing the issue clearly).
4. If the input is a feature request, frame it as a user story or clear requirement.
5. Maintain a helpful, professional tone.

OUTPUT FORMAT:
Respond ONLY with a JSON object in the following format:

{{
  "response_type": "question" | "improvement",
  "message": "The text to show to the user (either your questions or the improved version explainer)",
  "refined_feedback": "The actual improved text of the feedback (null if asking questions)"
}}

Example - Asking Questions:
{{
  "response_type": "question",
  "message": "I can help submit this. Could you clarify what you mean by 'broken'? Does nothing happen when you click, or do you see an error message?",
  "refined_feedback": null
}}

Example - Improvement:
{{
  "response_type": "improvement",
  "message": "I've refined your feedback to be more actionable. How does this look?",
  "refined_feedback": "Bug Report: The login button is unresponsive. Clicking it produces no effect and no error message is displayed (Console checked)."
}}

Generate the JSON response now."#
        )
    }
}

/// Context for building prompts
#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    pub user_request: String,
    pub current_url: Option<String>,
    pub page_title: Option<String>,
    pub agent_instructions: String,
    pub agent_name: String,
    pub current_config: String,
    pub user_feedback: String,
    pub failure_context: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // CDP Generation Prompt Tests

    #[test]
    fn test_cdp_generation_prompt_with_full_context() {
        let template = PromptTemplate::new(PromptType::CdpGeneration);
        let prompt = template.build_cdp_prompt(
            "Click the login button",
            Some("https://example.com"),
            Some("Example Page"),
            "You are an automation expert",
        );

        // Verify all key components are present
        assert!(prompt.contains("Click the login button"));
        assert!(prompt.contains("example.com"));
        assert!(prompt.contains("Example Page"));
        assert!(prompt.contains("You are an automation expert"));
        assert!(prompt.contains("CURRENT PAGE"));
        assert!(prompt.contains("USER REQUEST"));
        assert!(prompt.contains("CDP COMMANDS"));

        // Verify CDP command reference includes key commands
        assert!(prompt.contains("Page.navigate"));
        assert!(prompt.contains("Page.captureScreenshot"));
        assert!(prompt.contains("Runtime.evaluate"));
        assert!(prompt.contains("Input.insertText"));
        assert!(prompt.contains("Input.dispatchMouseEvent"));
        assert!(prompt.contains("Input.dispatchKeyEvent"));

        // Verify output format instructions
        assert!(prompt.contains("OUTPUT FORMAT"));
        assert!(prompt.contains("JSON only"));
    }

    #[test]
    fn test_cdp_generation_prompt_without_page_context() {
        let template = PromptTemplate::new(PromptType::CdpGeneration);
        let prompt = template.build_cdp_prompt(
            "Navigate to google.com",
            None,
            None,
            "You are an automation expert",
        );

        // Should still contain core elements
        assert!(prompt.contains("Navigate to google.com"));
        assert!(prompt.contains("You are an automation expert"));
        assert!(prompt.contains("CDP COMMANDS"));

        // Should NOT contain page context section (empty)
        assert!(!prompt.contains("CURRENT PAGE:"));
        assert!(!prompt.contains("URL:"));
        assert!(!prompt.contains("Title:"));
    }

    #[test]
    fn test_cdp_generation_prompt_with_special_characters() {
        let template = PromptTemplate::new(PromptType::CdpGeneration);
        let prompt = template.build_cdp_prompt(
            "Type \"hello@example.com\" into the email field",
            Some("https://example.com/login?redirect=%2Fdashboard"),
            Some("Login | My App™"),
            "Handle special chars: <>&\"'",
        );

        // Verify special characters are preserved
        assert!(prompt.contains("hello@example.com"));
        assert!(prompt.contains("redirect=%2Fdashboard"));
        assert!(prompt.contains("My App™"));
        assert!(prompt.contains("<>&\"'"));
    }

    #[test]
    fn test_cdp_generation_prompt_with_empty_strings() {
        let template = PromptTemplate::new(PromptType::CdpGeneration);
        let prompt = template.build_cdp_prompt("", Some(""), Some(""), "");

        // Should still have structure
        assert!(prompt.contains("USER REQUEST:"));
        assert!(prompt.contains("CDP COMMANDS"));
        // Empty URL and title should create context section but with empty values
        assert!(prompt.contains("CURRENT PAGE"));
    }

    #[test]
    fn test_cdp_generation_prompt_with_multiline_request() {
        let template = PromptTemplate::new(PromptType::CdpGeneration);
        let prompt = template.build_cdp_prompt(
            "First, navigate to example.com\nThen, click the login button\nFinally, take a screenshot",
            Some("https://example.com"),
            Some("Example"),
            "You are an automation expert",
        );

        assert!(prompt.contains("First, navigate"));
        assert!(prompt.contains("Then, click"));
        assert!(prompt.contains("Finally, take"));
    }

    // Config Update Prompt Tests

    #[test]
    fn test_config_update_prompt_with_failure_context() {
        let template = PromptTemplate::new(PromptType::ConfigUpdate);
        let prompt = template.build_config_update_prompt(
            "test-agent",
            "name = \"test\"\ninstructions = \"old\"",
            "The agent failed to handle errors",
            Some("Error: element not found"),
        );

        assert!(prompt.contains("test-agent"));
        assert!(prompt.contains("failed to handle errors"));
        assert!(prompt.contains("element not found"));
        assert!(prompt.contains("CURRENT CONFIGURATION"));
        assert!(prompt.contains("USER FEEDBACK"));
        assert!(prompt.contains("FAILURE CONTEXT"));
        assert!(prompt.contains("```toml"));
    }

    #[test]
    fn test_config_update_prompt_without_failure_context() {
        let template = PromptTemplate::new(PromptType::ConfigUpdate);
        let prompt = template.build_config_update_prompt(
            "cdp-generator",
            "name = \"cdp-generator\"",
            "Please make the agent more careful",
            None,
        );

        assert!(prompt.contains("cdp-generator"));
        assert!(prompt.contains("more careful"));
        assert!(!prompt.contains("FAILURE CONTEXT"));
    }

    #[test]
    fn test_config_update_prompt_with_complex_toml() {
        let template = PromptTemplate::new(PromptType::ConfigUpdate);
        let config = r#"
name = "test-agent"
version = "1.0.0"

[settings]
model = "sonnet"
temperature = 0.7

instructions = """
Multi-line
instructions
here
"""

tags = ["tag1", "tag2"]
"#;
        let prompt = template.build_config_update_prompt(
            "test-agent",
            config,
            "Improve error handling",
            Some("Failed on timeout"),
        );

        assert!(prompt.contains("Multi-line"));
        assert!(prompt.contains("instructions"));
        assert!(prompt.contains("settings"));
        assert!(prompt.contains("Improve error handling"));
        assert!(prompt.contains("Failed on timeout"));
    }

    #[test]
    fn test_config_update_prompt_rules_present() {
        let template = PromptTemplate::new(PromptType::ConfigUpdate);
        let prompt = template.build_config_update_prompt(
            "test-agent",
            "name = \"test\"",
            "Update needed",
            None,
        );

        // Verify all rules are documented
        assert!(prompt.contains("RULES"));
        assert!(prompt.contains("Keep all existing fields"));
        assert!(prompt.contains("Only modify the 'instructions' field"));
        assert!(prompt.contains("Make minimal, targeted changes"));
        assert!(prompt.contains("Ensure the TOML is valid"));
        assert!(prompt.contains("prevent this error in the future"));
    }

    // PromptContext and build() Tests

    #[test]
    fn test_prompt_context_default() {
        let context = PromptContext::default();
        assert_eq!(context.user_request, "");
        assert_eq!(context.current_url, None);
        assert_eq!(context.page_title, None);
        assert_eq!(context.agent_instructions, "");
        assert_eq!(context.agent_name, "");
        assert_eq!(context.current_config, "");
        assert_eq!(context.user_feedback, "");
        assert_eq!(context.failure_context, None);
    }

    #[test]
    fn test_build_with_cdp_generation_context() {
        let template = PromptTemplate::new(PromptType::CdpGeneration);
        let context = PromptContext {
            user_request: "Click button".to_string(),
            current_url: Some("https://example.com".to_string()),
            page_title: Some("Test Page".to_string()),
            agent_instructions: "Be careful".to_string(),
            ..Default::default()
        };

        let prompt = template.build(context);
        assert!(prompt.contains("Click button"));
        assert!(prompt.contains("example.com"));
        assert!(prompt.contains("Test Page"));
        assert!(prompt.contains("Be careful"));
    }

    #[test]
    fn test_build_with_config_update_context() {
        let template = PromptTemplate::new(PromptType::ConfigUpdate);
        let context = PromptContext {
            agent_name: "test-agent".to_string(),
            current_config: "name = \"test\"".to_string(),
            user_feedback: "It failed".to_string(),
            failure_context: Some("Timeout error".to_string()),
            ..Default::default()
        };

        let prompt = template.build(context);
        assert!(prompt.contains("test-agent"));
        assert!(prompt.contains("It failed"));
        assert!(prompt.contains("Timeout error"));
    }

    // Edge Case Tests

    #[test]
    fn test_cdp_prompt_with_very_long_user_request() {
        let template = PromptTemplate::new(PromptType::CdpGeneration);
        let long_request = "a".repeat(10000);
        let prompt = template.build_cdp_prompt(
            &long_request,
            Some("https://example.com"),
            Some("Title"),
            "Instructions",
        );

        assert!(prompt.contains(&long_request));
        assert!(prompt.len() > 10000);
    }

    #[test]
    fn test_config_update_prompt_with_unicode() {
        let template = PromptTemplate::new(PromptType::ConfigUpdate);
        let prompt = template.build_config_update_prompt(
            "test-agent-日本語",
            "name = \"test\" # コメント",
            "エラーが発生しました 🔥",
            Some("失敗: タイムアウト ⏱️"),
        );

        assert!(prompt.contains("日本語"));
        assert!(prompt.contains("コメント"));
        assert!(prompt.contains("エラーが発生"));
        assert!(prompt.contains("🔥"));
        assert!(prompt.contains("⏱️"));
    }

    #[test]
    fn test_prompt_template_type_checking() {
        let cdp_template = PromptTemplate::new(PromptType::CdpGeneration);
        assert_eq!(cdp_template.prompt_type, PromptType::CdpGeneration);

        let config_template = PromptTemplate::new(PromptType::ConfigUpdate);
        assert_eq!(config_template.prompt_type, PromptType::ConfigUpdate);
    }

    #[test]
    fn test_cdp_prompt_includes_all_command_types() {
        let template = PromptTemplate::new(PromptType::CdpGeneration);
        let prompt = template.build_cdp_prompt("test", Some("url"), Some("title"), "instructions");

        // Verify comprehensive CDP command coverage
        let required_commands = [
            "Page.navigate",
            "Page.captureScreenshot",
            "Runtime.evaluate",
            "Input.insertText",
            "Input.dispatchMouseEvent",
            "Input.dispatchKeyEvent",
        ];

        for cmd in &required_commands {
            assert!(
                prompt.contains(cmd),
                "Prompt should contain CDP command: {}",
                cmd
            );
        }
    }

    #[test]
    fn test_config_update_task_steps_present() {
        let template = PromptTemplate::new(PromptType::ConfigUpdate);
        let prompt = template.build_config_update_prompt("agent", "config", "feedback", None);

        // Verify task breakdown is present
        assert!(prompt.contains("TASK:"));
        assert!(prompt.contains("1. Analyze what went wrong"));
        assert!(prompt.contains("2. Update the agent's instructions"));
        assert!(prompt.contains("3. Output the COMPLETE updated TOML"));
    }
}
