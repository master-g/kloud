//! Read tool implementation

use futures::TryStreamExt;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, LinesCodec, LinesCodecError};

use crate::tools::path::resolve_existing_path;
use crate::tools::{Tool, ToolResult};

const MAX_LINES: usize = 2000;
const MAX_LINE_LENGTH: usize = 1024;

/// `ReadTool` - A tool for reading files from the filesystem
#[derive(Debug)]
pub struct ReadTool {
    root: std::path::PathBuf,
}

impl ReadTool {
    /// Create a new `ReadTool` with the given root directory
    pub fn new(root: std::path::PathBuf) -> Self {
        Self {
            root,
        }
    }
}

#[async_trait::async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    /// Returns a description of the tool, which can be used for documentation or help messages.
    fn description(&self) -> &str {
        "Reads file contents with optional offset and limit. Arguments: { path: string, offset?: integer, limit?: integer }"
    }

    /// Returns the input schema for the tool, which can be used for validation or documentation.
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "offset": { "type": "integer", "minimum": 0 },
                "limit":{ "type": "integer", "minimum": 0, "description": "omitted or 0 means default max of 2000 lines" }
            },
            "required": ["path"],
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

        let offset =
            call.args.get("offset").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
        let limit =
            call.args.get("limit").and_then(serde_json::Value::as_u64).unwrap_or(0) as usize;
        let limit = match limit {
            0 => MAX_LINES,
            c => c.min(MAX_LINES),
        };

        let resolved_path = match resolve_existing_path(&self.root, std::path::Path::new(path_arg))
        {
            Ok(path) => path,
            Err(e) => {
                return Ok(ToolResult {
                    name: self.name().to_string(),
                    output: Err(format!("Failed to resolve path '{}': {}", path_arg, e)),
                });
            }
        };
        let file = match tokio::fs::File::open(&resolved_path).await {
            Ok(file) => file,
            Err(e) => {
                return Ok(ToolResult {
                    name: self.name().to_string(),
                    output: Err(format!(
                        "Failed to open file '{}': {}",
                        resolved_path.display(),
                        e
                    )),
                });
            }
        };

        let lines_stream = FramedRead::new(file, LinesCodec::new_with_max_length(MAX_LINE_LENGTH));
        let collect_result: Result<Vec<String>, LinesCodecError> =
            lines_stream.skip(offset).take(limit).try_collect().await;

        let contents = match collect_result {
            Ok(lines) => lines
                .into_iter()
                .enumerate()
                .map(|(i, v)| format!("{}: {}", i + offset + 1, v))
                .collect::<Vec<String>>(),
            Err(LinesCodecError::MaxLineLengthExceeded) => {
                return Ok(ToolResult {
                    name: self.name().to_string(),
                    output: Err(format!(
                        "Line length exceeds maximum of {} bytes in file '{}'",
                        MAX_LINE_LENGTH,
                        resolved_path.display()
                    )),
                });
            }
            Err(LinesCodecError::Io(e)) => {
                return Ok(ToolResult {
                    name: self.name().to_string(),
                    output: Err(format!(
                        "Failed to read lines from file '{}': {}",
                        resolved_path.display(),
                        e
                    )),
                });
            }
        };

        Ok(crate::tools::ToolResult {
            name: self.name().to_string(),
            output: Ok(contents.join("\n")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_tool() {
        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("test.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"Line 1\nLine 2\nLine 3\nLine 4\nLine 5")
            .unwrap();

        let tool = ReadTool::new(tmp_dir.path().to_path_buf());

        let call = crate::tools::ToolCall {
            name: "read".to_string(),
            args: serde_json::json!({
                "path": "test.txt",
                "offset": 2,
                "limit": 2
            }),
        };

        let result = tool.execute(&call).await.unwrap();
        assert_eq!(result.name, "read");
        assert_eq!(result.output, Ok("3: Line 3\n4: Line 4".to_string()));

        let call = crate::tools::ToolCall {
            name: "read".to_string(),
            args: serde_json::json!({
                "path": "nonexistent.txt",
            }),
        };

        let result = tool.execute(&call).await.unwrap();
        assert_eq!(result.name, "read");
        match result.output {
            Err(msg) => assert!(
                msg.contains("Failed to resolve path") || msg.contains("Failed to open file")
            ),
            Ok(output) => panic!("expected error output, got: {output}"),
        }

        let call = crate::tools::ToolCall {
            name: "read".to_string(),
            args: serde_json::json!({
                "path": "test.txt",
                "offset": 0,
            }),
        };

        let result = tool.execute(&call).await.unwrap();
        assert_eq!(result.name, "read");
        assert_eq!(
            result.output,
            Ok("1: Line 1\n2: Line 2\n3: Line 3\n4: Line 4\n5: Line 5".to_string())
        );
    }

    #[tokio::test]
    async fn test_read_tool_missing_path() {
        let tmp_dir = TempDir::new().unwrap();
        let tool = ReadTool::new(tmp_dir.path().to_path_buf());
        let call = crate::tools::ToolCall {
            name: "read".to_string(),
            args: serde_json::json!({}),
        };

        let result = tool.execute(&call).await.unwrap();
        assert_eq!(result.output, Err("'path' argument is required".to_string()));
    }

    #[tokio::test]
    async fn test_read_tool_rejects_path_escape() {
        let tmp_dir = TempDir::new().unwrap();
        let outside_file_path = tmp_dir.path().join("outside.txt");
        std::fs::File::create(&outside_file_path).unwrap().write_all(b"outside").unwrap();
        let root = tmp_dir.path().join("workspace");
        std::fs::create_dir_all(&root).unwrap();
        let tool = ReadTool::new(root);
        let call = crate::tools::ToolCall {
            name: "read".to_string(),
            args: serde_json::json!({
                "path": "../outside.txt"
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
}
