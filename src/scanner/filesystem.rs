//! File system scanning utilities

use ignore::WalkBuilder;
use std::path::Path;

/// Information about a file in the repository
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Relative path from repository root
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Whether the file is a directory
    pub is_dir: bool,
}

/// Scan a directory and return information about all files
pub fn scan_directory(root: &Path) -> Vec<FileInfo> {
    let mut files = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .parents(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();

        // Skip the root directory itself
        if path == root {
            continue;
        }

        // Skip .git directory
        if path.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }

        // Get relative path
        let relative_path = path
            .strip_prefix(root)
            .ok()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        if relative_path.is_empty() {
            continue;
        }

        // Get file metadata
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        files.push(FileInfo {
            path: relative_path,
            size: metadata.len(),
            is_dir: metadata.is_dir(),
        });
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scan_directory() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create test files
        fs::write(root.join("test.txt"), "hello").unwrap();
        fs::create_dir(root.join("subdir")).unwrap();
        fs::write(root.join("subdir/nested.txt"), "world").unwrap();

        let files = scan_directory(root);

        assert!(files.iter().any(|f| f.path == "test.txt"));
        assert!(files.iter().any(|f| f.path == "subdir/nested.txt" || f.path == "subdir\\nested.txt"));
    }
}
