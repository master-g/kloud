//! An echo tool that simply echoes the input back to the user. For testing and debugging purposes only.

use ratatui::text::Line;

use crate::Result;
use crate::tools::{Tool, ToolCall, ToolResult, ToolResultKind};
use crate::ui::tui::theme::Theme;

/// An echo tool that simply echoes the input back to the user. For testing and debugging purposes only.
#[allow(dead_code)]
pub struct EchoTool;

#[async_trait::async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes the input back to the user."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Text to echo back to the caller."
                }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, call: &ToolCall) -> Result<ToolResult> {
        if let Some(msg) = call.args.get("message").and_then(|v| v.as_str()) {
            Ok(ToolResult {
                name: self.name().to_string(),
                kind: ToolResultKind::Success,
                output: msg.to_string(),
            })
        } else {
            Ok(ToolResult {
                name: self.name().to_string(),
                kind: ToolResultKind::Error,
                output: "missing or invalid message".to_string(),
            })
        }
    }

    fn user_facing_name(&self) -> String {
        "Echo".to_string()
    }

    fn render_tool_use_message(
        &self,
        input: &serde_json::Value,
        _theme: &Theme,
    ) -> Vec<Line<'static>> {
        let msg = input.get("message").and_then(|v| v.as_str()).unwrap_or("");
        vec![Line::from(msg.to_string())]
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::{Tool, ToolCall, ToolRegistry, ToolResultKind};

    use super::EchoTool;

    #[tokio::test]
    async fn test_register_and_get() {
        let mut registry = ToolRegistry::new();
        let tool = EchoTool;
        registry.register(tool);
        assert!(registry.get("echo").is_some());
    }

    #[tokio::test]
    async fn test_echo_tool_execute() {
        let tool = EchoTool;
        let call = ToolCall {
            name: "echo".to_string(),
            args: serde_json::json!({ "message": "hello" }),
        };
        let result = tool.execute(&call).await.unwrap();
        assert_eq!(result.name, "echo");
        assert_eq!(result.kind, ToolResultKind::Success);
        assert_eq!(result.output, "hello");
    }
}
