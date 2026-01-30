//! File system scanning utilities

use ignore::WalkBuilder;
use rayon::prelude::*;
use std::path::Path;

/// Information about a file in the repository
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Relative path from repository root
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Whether the file is a directory
    #[allow(dead_code)]
    pub is_dir: bool,
}

/// Scan a directory and return information about all files
///
/// Uses parallel processing for better performance on large repositories.
pub fn scan_directory(root: &Path) -> Vec<FileInfo> {
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .parents(true)
        .build();

    walker
        .into_iter()
        .par_bridge()
        .filter_map(|entry_result| {
            let entry = entry_result.ok()?;
            let path = entry.path();

            // Skip the root directory itself
            if path == root {
                return None;
            }

            // Skip .git directory
            if path.components().any(|c| c.as_os_str() == ".git") {
                return None;
            }

            // Get relative path - handle errors gracefully
            let relative_path = match path.strip_prefix(root) {
                Ok(stripped) => stripped
                    .to_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| stripped.to_string_lossy().to_string()),
                Err(_) => {
                    return None;
                }
            };

            if relative_path.is_empty() {
                return None;
            }

            // Get file metadata
            let metadata = entry.metadata().ok()?;

            Some(FileInfo {
                path: relative_path,
                size: metadata.len(),
                is_dir: metadata.is_dir(),
            })
        })
        .collect()
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
        assert!(files
            .iter()
            .any(|f| f.path == "subdir/nested.txt" || f.path == "subdir\\nested.txt"));
    }
}
