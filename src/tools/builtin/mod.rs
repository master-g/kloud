//! Built-in tools for the agent.

pub mod echo;
pub mod read;
pub mod write;

pub use echo::EchoTool;
pub use read::ReadTool;
pub use write::WriteTool;

use crate::tools::ToolRegistry;

/// Creates a tool registry with the built-in tools registered.
///
/// # Arguments
///
/// * `root` - The root directory for the `ReadTool`, which will be used to restrict file access.
///
/// # Returns
///
/// A `ToolRegistry` with the built-in tools registered.
pub fn create_builtin_tools_registry(root: impl AsRef<std::path::Path>) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(EchoTool);
    registry.register(ReadTool::new(root.as_ref().to_path_buf()));
    registry.register(WriteTool::new(root.as_ref().to_path_buf()));
    registry
}

#[cfg(test)]
mod tests {
    use super::create_builtin_tools_registry;

    #[tokio::test]
    async fn test_builtin_tools_registry() {
        let registry = create_builtin_tools_registry("/tmp");
        let tool_names = registry.list_names();
        assert!(tool_names.contains(&"echo".to_string()));
        assert!(tool_names.contains(&"read".to_string()));
    }
}
