//! This module contains a registry of available tools.

use std::{collections::HashMap, sync::Arc};

use crate::tools::Tool;

/// A registry of available tools.
#[derive(Default)]
pub struct ToolRegistry {
	tools: HashMap<String, Arc<dyn Tool>>,
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
}
