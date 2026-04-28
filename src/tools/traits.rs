//! Trait definition for tools. This defines the common interface that all tools must implement, and allows us to execute them in a consistent way.

use ratatui::text::Line;

use crate::error::Result;
use crate::tools::{ToolCall, ToolResult, ToolResultKind};
use crate::ui::tui::theme::Theme;

/// Tool trait that all tools must implement. This allows us to have a common interface for all tools, and to execute them in a consistent way.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Returns the name of the tool, e.g. "read", "write", "bash", etc.
    fn name(&self) -> &str;

    /// Returns a description of the tool, which can be used for documentation or help messages.
    fn description(&self) -> &str;

    /// Returns the input schema for the tool, which can be used for validation or documentation.
    fn input_schema(&self) -> serde_json::Value;

    /// Executes the tool with the given arguments and returns the output as a string.
    /// The arguments are provided as a JSON value for flexibility.
    async fn execute(&self, call: &ToolCall) -> Result<ToolResult>;

    /// Execute the tool with progress callback support.
    /// Default implementation delegates to `execute()` ignoring progress.
    async fn execute_with_progress(
        &self,
        call: &ToolCall,
        _on_progress: Option<&(dyn Fn(&str) + Send + Sync)>,
    ) -> Result<ToolResult> {
        self.execute(call).await
    }

    /// Human-readable display name shown in the TUI. Defaults to `self.name()`.
    fn user_facing_name(&self) -> String {
        self.name().to_string()
    }

    /// Render the tool-use header lines. Called at store layer.
    fn render_tool_use_message(
        &self,
        input: &serde_json::Value,
        theme: &Theme,
    ) -> Vec<Line<'static>> {
        let truncated: String =
            serde_json::to_string(input).unwrap_or_default().chars().take(80).collect();
        vec![Line::from(vec![
            ratatui::text::Span::styled(
                self.user_facing_name(),
                theme.tool.add_modifier(ratatui::style::Modifier::BOLD),
            ),
            ratatui::text::Span::raw(" "),
            ratatui::text::Span::styled(truncated, theme.inactive),
        ])]
    }

    /// Render the tool-result lines. Called at store layer.
    fn render_tool_result_message(
        &self,
        output: &str,
        kind: ToolResultKind,
        theme: &Theme,
    ) -> Vec<Line<'static>> {
        let style = match kind {
            ToolResultKind::Error => theme.error,
            _ => theme.inactive,
        };
        output
            .lines()
            .map(|line| Line::from(ratatui::text::Span::styled(line.to_string(), style)))
            .collect()
    }
}
