//! Path utils for kloud tools

use std::{io::ErrorKind, path::Path};

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

/// Resolve a path and ensure it is writable, returning an error if it is not
///
/// # Arguments
///
/// * `root` - The root path to resolve against
/// * `user_path` - The user-provided path to resolve
///
/// # Returns
///
/// A `Result` containing the resolved path if successful, or an error if the path is not writable
pub fn resolve_writable_path(
    root: impl AsRef<Path>,
    user_path: impl AsRef<Path>,
) -> crate::Result<std::path::PathBuf> {
    if user_path.as_ref().is_absolute() {
        return Err(ToolError::PathSecurity("absolute paths are not allowed".to_string()).into());
    }

    let root = match root.as_ref().canonicalize() {
        Ok(root) => root,
        Err(e) if e.kind() == ErrorKind::NotFound => root.as_ref().to_path_buf(),
        Err(e) => return Err(e.into()),
    };

    let candidate = root.join(user_path.as_ref());

    let mut check_path = candidate.as_path();
    let mut resolved_parent = None;

    while let Some(parent) = check_path.parent() {
        match parent.canonicalize() {
            Ok(resolved) => {
                resolved_parent = Some(resolved);
                break;
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                check_path = parent;
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }

    if let Some(ref resolved_parent) = resolved_parent
        && !resolved_parent.starts_with(&root)
    {
        return Err(ToolError::PathSecurity(format!(
            "path escapes workspace root: {}",
            user_path.as_ref().display()
        ))
        .into());
    }

    Ok(candidate)
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

    #[test]
    fn resolve_writable_path_allows_new_file_under_root() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("workspace");
        std::fs::create_dir(&root).unwrap();

        let resolved = resolve_writable_path(&root, "new_file.txt").unwrap();
        assert_eq!(resolved, root.canonicalize().unwrap().join("new_file.txt"));
    }

    #[test]
    fn resolve_writable_path_rejects_absolute_path() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("workspace");
        std::fs::create_dir(&root).unwrap();

        let err = resolve_writable_path(&root, "/etc/passwd").unwrap_err();
        assert!(matches!(err, crate::Error::Tool(ToolError::PathSecurity(_))));
    }

    #[test]
    fn resolve_writable_path_rejects_escape() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("workspace");
        std::fs::create_dir(&root).unwrap();

        let err = resolve_writable_path(&root, "../outside.txt").unwrap_err();
        assert!(matches!(err, crate::Error::Tool(ToolError::PathSecurity(_))));
    }

    #[test]
    fn resolve_writable_path_creates_nested_dirs() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("workspace");
        std::fs::create_dir(&root).unwrap();

        let resolved = resolve_writable_path(&root, "deep/nested/file.txt").unwrap();
        assert_eq!(resolved, root.canonicalize().unwrap().join("deep/nested/file.txt"));
    }
}
