//! # Scanner Module
//!
//! This module provides file system and git repository scanning capabilities
//! for RepoLens audit operations.
//!
//! ## Overview
//!
//! The scanner is responsible for:
//! - Walking the repository file tree
//! - Caching file information for efficient access
//! - Reading file contents for rule evaluation
//! - Extracting git repository metadata
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use repolens::scanner::Scanner;
//! use std::path::PathBuf;
//!
//! let scanner = Scanner::new(PathBuf::from("."));
//!
//! // Get repository name
//! println!("Repository: {}", scanner.repository_name());
//!
//! // Check if files exist
//! if scanner.file_exists("README.md") {
//!     println!("README.md found");
//! }
//! ```
//!
//! ### Finding Files
//!
//! ```rust,no_run
//! use repolens::scanner::Scanner;
//! use std::path::PathBuf;
//!
//! let scanner = Scanner::new(PathBuf::from("."));
//!
//! // Find all Rust files
//! let rust_files = scanner.files_with_extensions(&["rs"]);
//! println!("Found {} Rust files", rust_files.len());
//!
//! // Find files matching a pattern
//! let test_files = scanner.files_matching_pattern("**/test/**");
//! println!("Found {} test files", test_files.len());
//!
//! // Find large files
//! let large_files = scanner.files_larger_than(1_000_000); // > 1MB
//! println!("Found {} files larger than 1MB", large_files.len());
//! ```
//!
//! ### Reading File Contents
//!
//! ```rust,no_run
//! use repolens::scanner::Scanner;
//! use std::path::PathBuf;
//!
//! let scanner = Scanner::new(PathBuf::from("."));
//!
//! if let Ok(content) = scanner.read_file("Cargo.toml") {
//!     println!("Cargo.toml has {} bytes", content.len());
//! }
//! ```

mod filesystem;
mod git;

use std::path::PathBuf;

pub use filesystem::FileInfo;

/// Main scanner for repository analysis
///
/// The `Scanner` provides access to repository files and metadata.
/// It caches file information for efficient access during rule execution.
pub struct Scanner {
    /// Root directory of the repository
    root: PathBuf,
    /// Cached file information
    file_cache: Vec<FileInfo>,
}

impl Scanner {
    /// Create a new scanner for the given root directory
    ///
    /// Scans the directory and caches file information for later use.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory of the repository to scan
    ///
    /// # Returns
    ///
    /// A new `Scanner` instance with cached file information
    pub fn new(root: PathBuf) -> Self {
        let file_cache = filesystem::scan_directory(&root);

        Self { root, file_cache }
    }

    /// Get the repository name from the root path or git remote
    ///
    /// First attempts to get the repository name from git remote.
    /// Falls back to the directory name if git information is unavailable.
    ///
    /// # Returns
    ///
    /// The repository name as a string
    pub fn repository_name(&self) -> String {
        // Try to get from git remote first
        if let Some(name) = git::get_repository_name(&self.root) {
            return name;
        }

        // Fall back to directory name
        self.root
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // If directory name contains non-UTF8, use lossy conversion
                self.root.to_string_lossy().to_string()
            })
    }

    /// Check if a file exists in the repository
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to the file from repository root
    ///
    /// # Returns
    ///
    /// `true` if the file exists, `false` otherwise
    pub fn file_exists(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }

    /// Check if a directory exists in the repository
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to the directory from repository root
    ///
    /// # Returns
    ///
    /// `true` if the directory exists, `false` otherwise
    pub fn directory_exists(&self, path: &str) -> bool {
        let full_path = self.root.join(path);
        full_path.exists() && full_path.is_dir()
    }

    /// Read file content as a string
    ///
    /// # Arguments
    ///
    /// * `path` - Relative path to the file from repository root
    ///
    /// # Returns
    ///
    /// The file content as a string, or an I/O error if reading fails
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or is not valid UTF-8
    pub fn read_file(&self, path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(self.root.join(path))
    }

    /// Get files with specific extensions
    ///
    /// # Arguments
    ///
    /// * `extensions` - Slice of file extensions (without the dot), e.g., `["rs", "toml"]`
    ///
    /// # Returns
    ///
    /// A vector of references to `FileInfo` for matching files
    pub fn files_with_extensions(&self, extensions: &[&str]) -> Vec<&FileInfo> {
        self.file_cache
            .iter()
            .filter(|f| {
                extensions
                    .iter()
                    .any(|ext| f.path.ends_with(&format!(".{}", ext)))
            })
            .collect()
    }

    /// Get files matching a glob pattern
    ///
    /// Supports simple glob patterns like `*.rs`, `**/test/**`, etc.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Glob pattern to match against file paths
    ///
    /// # Returns
    ///
    /// A vector of references to `FileInfo` for matching files
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
    ///
    /// # Arguments
    ///
    /// * `size` - Minimum file size in bytes
    ///
    /// # Returns
    ///
    /// A vector of references to `FileInfo` for files larger than the specified size
    pub fn files_larger_than(&self, size: u64) -> Vec<&FileInfo> {
        self.file_cache.iter().filter(|f| f.size > size).collect()
    }

    /// Get files in a specific directory
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory path (with or without trailing slash)
    ///
    /// # Returns
    ///
    /// A vector of references to `FileInfo` for files in the specified directory
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

    /// Get all files in the repository
    ///
    /// # Returns
    ///
    /// A slice of all `FileInfo` entries
    #[allow(dead_code)]
    pub fn all_files(&self) -> &[FileInfo] {
        &self.file_cache
    }

    /// Get the root directory of the repository
    ///
    /// # Returns
    ///
    /// A reference to the root path
    pub fn root(&self) -> &std::path::Path {
        &self.root
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create directory structure
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("tests")).unwrap();
        fs::create_dir_all(root.join(".github/workflows")).unwrap();

        // Create files
        fs::write(root.join("README.md"), "# Test Project").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn lib() {}").unwrap();
        fs::write(root.join("tests/test.rs"), "#[test] fn test() {}").unwrap();
        fs::write(root.join(".github/workflows/ci.yml"), "name: CI\non: push").unwrap();

        // Create a large file (for testing files_larger_than)
        let large_content = "x".repeat(10000);
        fs::write(root.join("large_file.bin"), large_content).unwrap();

        temp_dir
    }

    #[test]
    fn test_scanner_new() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        assert!(!scanner.all_files().is_empty());
    }

    #[test]
    fn test_repository_name_fallback() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        // Without git, should fall back to directory name
        let name = scanner.repository_name();
        assert!(!name.is_empty());
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        assert!(scanner.file_exists("README.md"));
        assert!(scanner.file_exists("src/main.rs"));
        assert!(!scanner.file_exists("nonexistent.txt"));
    }

    #[test]
    fn test_directory_exists() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        assert!(scanner.directory_exists("src"));
        assert!(scanner.directory_exists(".github/workflows"));
        assert!(!scanner.directory_exists("nonexistent"));
    }

    #[test]
    fn test_read_file() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let content = scanner.read_file("README.md").unwrap();
        assert_eq!(content, "# Test Project");

        let result = scanner.read_file("nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_files_with_extensions() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let rs_files = scanner.files_with_extensions(&["rs"]);
        assert!(rs_files.len() >= 2); // main.rs, lib.rs, test.rs

        let md_files = scanner.files_with_extensions(&["md"]);
        assert!(!md_files.is_empty());

        let multi_ext = scanner.files_with_extensions(&["rs", "toml"]);
        assert!(multi_ext.len() >= 3);
    }

    #[test]
    fn test_files_matching_pattern() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let rs_pattern = scanner.files_matching_pattern("*.rs");
        assert!(!rs_pattern.is_empty());

        let src_pattern = scanner.files_matching_pattern("src/");
        assert!(!src_pattern.is_empty());
    }

    #[test]
    fn test_files_larger_than() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let large_files = scanner.files_larger_than(5000);
        assert!(!large_files.is_empty());

        let very_large = scanner.files_larger_than(1_000_000);
        assert!(very_large.is_empty());
    }

    #[test]
    fn test_files_in_directory() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let src_files = scanner.files_in_directory("src");
        assert!(src_files.len() >= 2);

        let github_files = scanner.files_in_directory(".github");
        assert!(!github_files.is_empty());
    }

    #[test]
    fn test_glob_match_star() {
        assert!(glob_match("*", "anything"));
        assert!(glob_match("*", ""));
    }

    #[test]
    fn test_glob_match_extension() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("*.rs", "src/lib.rs"));
        assert!(!glob_match("*.rs", "main.txt"));
    }

    #[test]
    fn test_glob_match_double_star() {
        // The glob_match function has specific behavior for **
        assert!(glob_match("src/**", "src/lib.rs"));
        assert!(glob_match("src/**", "src/sub/file.rs"));
        // Test pattern with ** at end
        assert!(glob_match("tests/**", "tests/unit/test.rs"));
    }

    #[test]
    fn test_glob_match_partial() {
        assert!(glob_match("main", "src/main.rs"));
        assert!(glob_match("*main*", "src/main.rs"));
    }

    #[test]
    fn test_all_files() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let files = scanner.all_files();
        assert!(files.len() >= 6); // We created at least 6 files
    }

    #[test]
    fn test_files_matching_pattern_no_wildcard() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        // Pattern without wildcard should match via contains
        let pattern = scanner.files_matching_pattern("README.md");
        assert!(!pattern.is_empty());
    }

    #[test]
    fn test_glob_match_no_match() {
        assert!(!glob_match("*.rs", "main.txt"));
        assert!(!glob_match("src/**", "other/file.txt"));
    }

    #[test]
    fn test_files_in_directory_consistency() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let files = scanner.files_in_directory("src");
        // Should have at least main.rs and lib.rs
        assert!(files.len() >= 2);
    }

    #[test]
    fn test_files_larger_than_zero() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let files = scanner.files_larger_than(0);
        // Most files have content, so should have results
        assert!(!files.is_empty());
    }

    #[test]
    fn test_files_with_multiple_extensions() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let files = scanner.files_with_extensions(&["yml", "yaml"]);
        assert!(!files.is_empty()); // We have ci.yml
    }

    #[test]
    fn test_glob_match_empty_prefix() {
        // Test with empty prefix in double star pattern
        assert!(glob_match("**/workflows", ".github/workflows"));
    }

    #[test]
    fn test_glob_match_double_star_empty_suffix() {
        // Pattern "src/**" should match files inside src
        assert!(glob_match("src/**", "src/main.rs"));
        assert!(glob_match("src/**", "src/sub/file.rs"));
    }

    #[test]
    fn test_glob_match_double_star_all() {
        assert!(glob_match("**", "any/path/file.txt"));
    }

    #[test]
    fn test_files_matching_pattern_with_contains() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        // Pattern "main" should match via contains
        let result = scanner.files_matching_pattern("main");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_files_matching_exact_extension() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let md_files = scanner.files_matching_pattern("*.md");
        assert!(!md_files.is_empty()); // README.md
    }

    #[test]
    fn test_files_in_directory_with_trailing_slash() {
        let temp_dir = create_test_repo();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());

        let files = scanner.files_in_directory("src/");
        assert!(files.len() >= 2);
    }

    #[test]
    fn test_repository_name_without_git_remote() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
        let scanner = Scanner::new(temp_dir.path().to_path_buf());
        let name = scanner.repository_name();
        assert!(!name.is_empty());
    }
}
