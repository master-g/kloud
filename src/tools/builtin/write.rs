//! Write tool implementation

use crate::tools::path::resolve_writable_path;
use crate::tools::{Tool, ToolResult};

/// `WriteTool` - A tool for writing files from the filesystem
#[derive(Debug)]
pub struct WriteTool {
	root: std::path::PathBuf,
}

impl WriteTool {
	/// Create a new `WriteTool` with the given root directory
	pub fn new(root: std::path::PathBuf) -> Self {
		Self {
			root,
		}
	}
}

#[async_trait::async_trait]
impl Tool for WriteTool {
	fn name(&self) -> &str {
		"write"
	}

	/// Returns a description of the tool, which can be used for documentation or help messages.
	fn description(&self) -> &str {
		"Write file contents. Arguments: { path: string, content: string }"
	}

	/// Returns the input schema for the tool, which can be used for validation or documentation.
	fn input_schema(&self) -> serde_json::Value {
		serde_json::json!({
			"type": "object",
			"properties": {
				"path": { "type": "string" },
				"content": { "type": "string" },
			},
			"required": ["path", "content"],
			"additionalProperties": false,
		})
	}

	async fn execute(
		&self,
		call: &crate::tools::ToolCall,
	) -> crate::Result<crate::tools::ToolResult> {
		let Some(path_arg) = call.args.get("path").and_then(|v| v.as_str()) else {
			return Ok(ToolResult {
				name: self.name().to_string(),
				output: Err("'path' argument is required".to_string()),
			});
		};

		let Some(content_arg) = call.args.get("content").and_then(|v| v.as_str()) else {
			return Ok(ToolResult {
				name: self.name().to_string(),
				output: Err("'content' argument is required".to_string()),
			});
		};

		let resolved_path = match resolve_writable_path(&self.root, std::path::Path::new(path_arg))
		{
			Ok(path) => path,
			Err(e) => {
				return Ok(ToolResult {
					name: self.name().to_string(),
					output: Err(format!("Failed to resolve path '{}': {}", path_arg, e)),
				});
			}
		};

		// check if file already exists
		if tokio::fs::metadata(&resolved_path).await.is_ok() {
			return Ok(ToolResult {
				name: self.name().to_string(),
				output: Err(format!("File '{}' already exists", resolved_path.display())),
			});
		}

		// create parent directories if they don't exist
		if let Some(parent) = resolved_path.parent()
			&& let Err(e) = tokio::fs::create_dir_all(parent).await
		{
			return Ok(ToolResult {
				name: self.name().to_string(),
				output: Err(format!(
					"Failed to create parent directories for '{}': {}",
					resolved_path.display(),
					e
				)),
			});
		}

		// write content to the file
		if let Err(e) = tokio::fs::write(&resolved_path, content_arg).await {
			return Ok(ToolResult {
				name: self.name().to_string(),
				output: Err(format!(
					"Failed to write to file '{}': {}",
					resolved_path.display(),
					e
				)),
			});
		}

		Ok(crate::tools::ToolResult {
			name: self.name().to_string(),
			output: Ok(format!(
				"Successfully wrote {bytes} bytes to {path}",
				bytes = content_arg.len(),
				path = resolved_path.display()
			)),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;
	use tempfile::TempDir;

	#[tokio::test]
	async fn test_write_tool() {
		let tmp_dir = TempDir::new().unwrap();
		let tool = WriteTool::new(tmp_dir.path().to_path_buf());
		let call = crate::tools::ToolCall {
			name: "write".to_string(),
			args: serde_json::json!({
				"path": "test.txt",
				"content": "Hello, World!"
			}),
		};

		let path = resolve_writable_path(&tool.root, "test.txt").unwrap();
		let expected_output = format!("Successfully wrote 13 bytes to {}", path.display());

		let result = tool.execute(&call).await.unwrap();
		assert_eq!(result.name, "write");
		assert_eq!(result.output, Ok(expected_output));
	}

	#[tokio::test]
	async fn test_write_tool_missing_path() {
		let tmp_dir = TempDir::new().unwrap();
		let tool = WriteTool::new(tmp_dir.path().to_path_buf());
		let call = crate::tools::ToolCall {
			name: "write".to_string(),
			args: serde_json::json!({
				"content": "Hello, World!"
			}),
		};

		let result = tool.execute(&call).await.unwrap();
		assert_eq!(result.output, Err("'path' argument is required".to_string()));
	}

	#[tokio::test]
	async fn test_write_tool_rejects_path_escape() {
		let tmp_dir = TempDir::new().unwrap();
		let outside_file_path = tmp_dir.path().join("outside.txt");
		std::fs::File::create(&outside_file_path).unwrap().write_all(b"outside").unwrap();
		let root = tmp_dir.path().join("workspace");
		std::fs::create_dir_all(&root).unwrap();
		let tool = WriteTool::new(root);
		let call = crate::tools::ToolCall {
			name: "write".to_string(),
			args: serde_json::json!({
				"path": "../outside.txt",
				"content": "Hello, World!"
			}),
		};

		let result = tool.execute(&call).await.unwrap();
		match result.output {
			Err(msg) => assert!(
				msg.contains("path security violation")
					|| msg.contains("absolute paths are not allowed")
					|| msg.contains("path escapes workspace root")
			),
			Ok(output) => panic!("expected error output, got: {output}"),
		}
	}

	#[tokio::test]
	async fn test_write_tool_rejects_existing_file() {
		let tmp_dir = TempDir::new().unwrap();
		let file_path = tmp_dir.path().join("existing.txt");
		std::fs::write(&file_path, "original").unwrap();

		let tool = WriteTool::new(tmp_dir.path().to_path_buf());
		let call = crate::tools::ToolCall {
			name: "write".to_string(),
			args: serde_json::json!({
				"path": "existing.txt",
				"content": "new content"
			}),
		};

		let result = tool.execute(&call).await.unwrap();
		assert!(result.output.is_err());
		assert!(result.output.unwrap_err().contains("already exists"));
	}
}
