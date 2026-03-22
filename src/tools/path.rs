//! Path utils for kloud tools

use std::path::Path;

use crate::error::ToolError;

/// Resolve a path and ensure it exists, returning an error if it does not
pub fn resolve_existing_path(
	root: impl AsRef<Path>,
	user_path: impl AsRef<Path>,
) -> crate::Result<std::path::PathBuf> {
	if user_path.as_ref().is_absolute() {
		return Err(ToolError::PathSecurity("absolute paths are not allowed".to_string()).into());
	}

	let root = root.as_ref().canonicalize()?;

	let candidate = root.join(user_path.as_ref());

	let resolved = candidate.canonicalize()?;

	if !resolved.starts_with(&root) {
		return Err(ToolError::PathSecurity(format!(
			"path escapes workspace root: {}",
			user_path.as_ref().display()
		))
		.into());
	}

	Ok(resolved)
}

#[cfg(test)]
mod tests {
	use std::io::Write;

	use tempfile::TempDir;

	use super::*;

	#[test]
	fn test_resolve_existing_path() {
		let tmp_dir = TempDir::new().unwrap();

		let outside_file_path = tmp_dir.path().join("outside.txt");
		std::fs::File::create(&outside_file_path)
			.unwrap()
			.write_all(b"This file is outside the root")
			.unwrap();

		let root = tmp_dir.path().join("workspace");
		std::fs::create_dir_all(root.join("subdir")).unwrap();
		std::fs::write(root.join("subdir/file.txt"), "hello").unwrap();

		let resolved = resolve_existing_path(&root, "subdir/file.txt").unwrap();
		assert_eq!(resolved, root.join("subdir/file.txt").canonicalize().unwrap());

		let err = resolve_existing_path(&root, root.join("subdir/file.txt")).unwrap_err();
		match err {
			crate::Error::Tool(ToolError::PathSecurity(msg)) => {
				assert_eq!(msg, "absolute paths are not allowed");
			}
			_ => panic!("unexpected error type"),
		}

		let err = resolve_existing_path(&root, "../outside.txt").unwrap_err();
		match err {
			crate::Error::Tool(ToolError::PathSecurity(msg)) => {
				assert!(msg.contains("path escapes workspace root"));
			}
			_ => panic!("unexpected error type"),
		}
	}
}
