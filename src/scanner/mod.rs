//! Scanner module - File system and repository scanning

mod filesystem;
mod git;

use std::path::PathBuf;

pub use filesystem::FileInfo;

/// Main scanner for repository analysis
pub struct Scanner {
    root: PathBuf,
    file_cache: Vec<FileInfo>,
}

impl Scanner {
    /// Create a new scanner for the given root directory
    pub fn new(root: PathBuf) -> Self {
        let file_cache = filesystem::scan_directory(&root);

        Self { root, file_cache }
    }

    /// Get the repository name from the root path or git remote
    pub fn repository_name(&self) -> String {
        // Try to get from git remote first
        if let Some(name) = git::get_repository_name(&self.root) {
            return name;
        }

        // Fall back to directory name
        self.root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Check if a file exists
    pub fn file_exists(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }

    /// Check if a directory exists
    pub fn directory_exists(&self, path: &str) -> bool {
        let full_path = self.root.join(path);
        full_path.exists() && full_path.is_dir()
    }

    /// Read file content
    pub fn read_file(&self, path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(self.root.join(path))
    }

    /// Get files with specific extensions
    pub fn files_with_extensions(&self, extensions: &[&str]) -> Vec<&FileInfo> {
        self.file_cache
            .iter()
            .filter(|f| {
                extensions.iter().any(|ext| f.path.ends_with(&format!(".{}", ext)))
            })
            .collect()
    }

    /// Get files matching a glob pattern
    pub fn files_matching_pattern(&self, pattern: &str) -> Vec<&FileInfo> {
        self.file_cache
            .iter()
            .filter(|f| {
                if pattern.contains('*') {
                    glob_match(pattern, &f.path)
                } else {
                    f.path.ends_with(pattern) || f.path.contains(pattern)
                }
            })
            .collect()
    }

    /// Get files larger than a given size in bytes
    pub fn files_larger_than(&self, size: u64) -> Vec<&FileInfo> {
        self.file_cache.iter().filter(|f| f.size > size).collect()
    }

    /// Get files in a specific directory
    pub fn files_in_directory(&self, dir: &str) -> Vec<&FileInfo> {
        let dir_path = if dir.ends_with('/') {
            dir.to_string()
        } else {
            format!("{}/", dir)
        };

        self.file_cache
            .iter()
            .filter(|f| f.path.starts_with(&dir_path) || f.path.starts_with(dir))
            .collect()
    }

    /// Get all files
    pub fn all_files(&self) -> &[FileInfo] {
        &self.file_cache
    }
}

/// Simple glob matching
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with("*.") {
        let ext = &pattern[1..];
        return text.ends_with(ext);
    }

    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            if !prefix.is_empty() && !text.starts_with(prefix) {
                return false;
            }
            if !suffix.is_empty() {
                return text.ends_with(suffix) || text.contains(&format!("/{}", suffix));
            }
            return true;
        }
    }

    text.contains(pattern.trim_start_matches('*').trim_end_matches('*'))
}
