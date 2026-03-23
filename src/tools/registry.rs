//! This module contains a registry of available tools.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::ToolError;
use crate::tools::{Tool, ToolCall, ToolResult};

/// A registry of available tools.
#[derive(Default)]
pub struct ToolRegistry {
	tools: BTreeMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
	/// Creates a new, empty `ToolRegistry`.
	pub fn new() -> Self {
		Self::default()
	}

	/// Registers a tool with the registry.
	pub fn register<T>(&mut self, tool: T)
	where
		T: Tool + 'static,
	{
		let name = tool.name().to_string();
		let tool: Arc<dyn Tool> = Arc::new(tool);
		self.tools.insert(name, tool);
	}

	/// Returns a reference to the tool with the given name, if one exists.
	pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
		self.tools.get(name)
	}

	/// Returns a list of the names of all registered tools.
	pub fn list_names(&self) -> Vec<String> {
		self.tools.keys().cloned().collect()
	}

	/// Returns a list of the tool definitions, one per registered tool.
	pub fn definitions(&self) -> Vec<serde_json::Value> {
		self.tools
			.iter()
			.map(|(k, v)| {
				serde_json::json!({
					"name": k.to_string(),
					"description": v.description(),
					"input_schema": v.input_schema(),
				})
			})
			.collect()
	}

	/// Dispatches a tool call to the appropriate tool and returns the result.
	pub async fn dispatch(&self, call: &ToolCall) -> crate::Result<ToolResult> {
		let tool = self.get(&call.name).ok_or_else(|| ToolError::NotFound(call.name.clone()))?;
		tool.execute(call).await
	}
}

#[cfg(test)]
mod tests {
	use crate::{error::ToolError, tools::ToolRegistry};

	#[tokio::test]
	async fn test_tool_registry() {
		use crate::tools::ToolCall;
		use crate::tools::builtin::EchoTool;

		let mut registry = ToolRegistry::new();
		registry.register(EchoTool);

		let call = ToolCall {
			name: "echo".to_string(),
			args: serde_json::json!({"message": "Hello, world!"}),
		};

		let result = registry.dispatch(&call).await.unwrap();
		assert_eq!(result.output, Ok("Hello, world!".to_string()));

		let call = ToolCall {
			name: "nonexistent".to_string(),
			args: serde_json::json!({}),
		};

		let err = registry.dispatch(&call).await.unwrap_err();
		match err {
			crate::Error::Tool(ToolError::NotFound(name)) => {
				assert_eq!(name, "nonexistent");
			}
			_ => panic!("unexpected error type"),
		}
	}

	#[tokio::test]
	async fn test_tool_registry_definitions() {
		use crate::tools::builtin::EchoTool;

		let mut registry = ToolRegistry::new();
		registry.register(EchoTool);

		let definitions = registry.definitions();
		assert_eq!(definitions.len(), 1);
		assert_eq!(definitions[0]["name"], "echo");
	}
}
