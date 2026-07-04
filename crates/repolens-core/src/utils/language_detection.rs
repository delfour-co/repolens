//! Language detection and .gitignore entry mapping
//!
//! This module provides functionality to detect programming languages present
//! in a repository and generate appropriate .gitignore entries for those languages.

use crate::scanner::Scanner;
use std::collections::{HashMap, HashSet};

/// Programming languages that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// JavaScript/TypeScript projects
    JavaScript,
    /// Python projects
    Python,
    /// Rust projects
    Rust,
    /// Go projects
    Go,
    /// Ruby projects
    Ruby,
    /// PHP projects
    Php,
    /// Java projects
    Java,
    /// C#/.NET projects
    CSharp,
}

/// Detect programming languages present in the repository
///
/// Analyzes the repository structure to identify which programming languages
/// are being used based on characteristic files.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A set of detected languages
pub fn detect_languages(scanner: &Scanner) -> HashSet<Language> {
    let mut languages = HashSet::new();

    // JavaScript/TypeScript detection
    if scanner.file_exists("package.json")
        || scanner.file_exists("package-lock.json")
        || scanner.file_exists("yarn.lock")
        || scanner.file_exists("pnpm-lock.yaml")
        || scanner.file_exists("tsconfig.json")
        || scanner.file_exists("jsconfig.json")
    {
        languages.insert(Language::JavaScript);
    }

    // Python detection
    if scanner.file_exists("requirements.txt")
        || scanner.file_exists("pyproject.toml")
        || scanner.file_exists("Pipfile")
        || scanner.file_exists("poetry.lock")
        || scanner.file_exists("setup.py")
        || scanner.file_exists("setup.cfg")
    {
        languages.insert(Language::Python);
    }

    // Rust detection
    if scanner.file_exists("Cargo.toml") || scanner.file_exists("Cargo.lock") {
        languages.insert(Language::Rust);
    }

    // Go detection
    if scanner.file_exists("go.mod") || scanner.file_exists("go.sum") {
        languages.insert(Language::Go);
    }

    // Ruby detection
    if scanner.file_exists("Gemfile") || scanner.file_exists("Gemfile.lock") {
        languages.insert(Language::Ruby);
    }

    // PHP detection
    if scanner.file_exists("composer.json") || scanner.file_exists("composer.lock") {
        languages.insert(Language::Php);
    }

    // Java detection
    if scanner.file_exists("pom.xml")
        || scanner.file_exists("build.gradle")
        || scanner.file_exists("build.gradle.kts")
    {
        languages.insert(Language::Java);
    }

    // C#/.NET detection
    // Check for common .NET project files
    let dotnet_files = scanner.files_matching_pattern("*.csproj");
    let dotnet_sln_files = scanner.files_matching_pattern("*.sln");
    let dotnet_fsproj = scanner.files_matching_pattern("*.fsproj");

    if !dotnet_files.is_empty() || !dotnet_sln_files.is_empty() || !dotnet_fsproj.is_empty() {
        languages.insert(Language::CSharp);
    }

    languages
}

/// Get recommended .gitignore entries for detected languages
///
/// Returns a list of .gitignore patterns that should be added based on
/// the languages present in the repository. Includes universal entries
/// that should always be present.
///
/// # Arguments
///
/// * `languages` - Set of detected languages
///
/// # Returns
///
/// A vector of .gitignore entry strings
#[allow(dead_code)] // May be used in the future or by external code
pub fn get_gitignore_entries_for_languages(languages: &HashSet<Language>) -> Vec<String> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    // Universal entries (always added)
    let universal_entries = [".env", "*.key", "*.pem", ".DS_Store"];

    for entry in universal_entries {
        entries.push(entry.to_string());
        seen.insert(entry);
    }

    // Language-specific entries
    let language_entries: HashMap<Language, Vec<&str>> = [
        (
            Language::JavaScript,
            vec![
                "node_modules/",
                ".npm/",
                ".yarn/",
                ".pnpm-store/",
                "dist/",
                "build/",
                ".next/",
                ".nuxt/",
                ".cache/",
            ],
        ),
        (
            Language::Python,
            vec![
                "__pycache__/",
                "*.pyc",
                "*.pyo",
                "*.pyd",
                ".Python",
                "venv/",
                "env/",
                ".venv/",
                "*.egg-info/",
                ".pytest_cache/",
                ".mypy_cache/",
            ],
        ),
        (
            Language::Rust,
            vec![
                "target/",
                "Cargo.lock", // Only for binaries, but we'll suggest it
            ],
        ),
        (Language::Go, vec!["vendor/", "*.exe", "*.test", "*.out"]),
        (Language::Ruby, vec!["vendor/bundle/", ".bundle/", "*.gem"]),
        (Language::Php, vec!["vendor/"]),
        (
            Language::Java,
            vec!["target/", "*.class", "*.jar", "*.war", ".gradle/"],
        ),
        (
            Language::CSharp,
            vec!["bin/", "obj/", "*.dll", "*.exe", "*.pdb"],
        ),
    ]
    .iter()
    .cloned()
    .collect();

    // Add entries for each detected language
    for language in languages {
        if let Some(lang_entries) = language_entries.get(language) {
            for entry in lang_entries {
                if !seen.contains(entry) {
                    entries.push(entry.to_string());
                    seen.insert(entry);
                }
            }
        }
    }

    entries
}

/// Get recommended .gitignore entries with descriptions
///
/// Returns a list of tuples containing .gitignore patterns and their descriptions.
/// This is useful for generating findings with helpful descriptions.
///
/// # Arguments
///
/// * `languages` - Set of detected languages
///
/// # Returns
///
/// A vector of tuples (pattern, description)
pub fn get_gitignore_entries_with_descriptions(
    languages: &HashSet<Language>,
) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();

    // Universal entries
    let universal_entries = [
        (".env", "Environment files"),
        ("*.key", "Private keys"),
        ("*.pem", "Certificates"),
        (".DS_Store", "macOS metadata"),
    ];

    for (pattern, description) in universal_entries {
        entries.push((pattern.to_string(), description.to_string()));
        seen.insert(pattern);
    }

    // Language-specific entries with descriptions
    let language_entries: HashMap<Language, Vec<(&str, &str)>> = [
        (
            Language::JavaScript,
            vec![
                ("node_modules/", "Node.js dependencies"),
                (".npm/", "npm cache"),
                (".yarn/", "Yarn cache"),
                (".pnpm-store/", "pnpm store"),
                ("dist/", "Build output"),
                ("build/", "Build output"),
            ],
        ),
        (
            Language::Python,
            vec![
                ("__pycache__/", "Python cache"),
                ("*.pyc", "Python compiled files"),
                ("*.pyo", "Python optimized files"),
                ("venv/", "Python virtual environment"),
                ("env/", "Python virtual environment"),
                (".venv/", "Python virtual environment"),
            ],
        ),
        (Language::Rust, vec![("target/", "Rust build output")]),
        (
            Language::Go,
            vec![("vendor/", "Go dependencies"), ("*.exe", "Go executables")],
        ),
        (
            Language::Ruby,
            vec![
                ("vendor/bundle/", "Ruby dependencies"),
                (".bundle/", "Bundler cache"),
            ],
        ),
        (Language::Php, vec![("vendor/", "PHP dependencies")]),
        (
            Language::Java,
            vec![
                ("target/", "Java build output"),
                ("*.class", "Java compiled files"),
            ],
        ),
        (
            Language::CSharp,
            vec![
                ("bin/", ".NET build output"),
                ("obj/", ".NET build artifacts"),
            ],
        ),
    ]
    .iter()
    .cloned()
    .collect();

    // Add entries for each detected language
    for language in languages {
        if let Some(lang_entries) = language_entries.get(language) {
            for (pattern, description) in lang_entries {
                if !seen.contains(pattern) {
                    entries.push((pattern.to_string(), description.to_string()));
                    seen.insert(pattern);
                }
            }
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_javascript() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("package.json"), "{}").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::JavaScript));
        assert!(!languages.contains(&Language::Rust));
    }

    #[test]
    fn test_detect_rust() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("Cargo.toml"), "[package]").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::Rust));
        assert!(!languages.contains(&Language::JavaScript));
    }

    #[test]
    fn test_detect_python() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("requirements.txt"), "").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::Python));
    }

    #[test]
    fn test_detect_multiple_languages() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("package.json"), "{}").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::JavaScript));
        assert!(languages.contains(&Language::Rust));
    }

    #[test]
    fn test_get_gitignore_entries_javascript() {
        let mut languages = HashSet::new();
        languages.insert(Language::JavaScript);

        let entries = get_gitignore_entries_for_languages(&languages);

        assert!(entries.contains(&"node_modules/".to_string()));
        assert!(entries.contains(&".env".to_string())); // Universal
        assert!(!entries.contains(&"target/".to_string())); // Rust-specific
    }

    #[test]
    fn test_get_gitignore_entries_rust() {
        let mut languages = HashSet::new();
        languages.insert(Language::Rust);

        let entries = get_gitignore_entries_for_languages(&languages);

        assert!(entries.contains(&"target/".to_string()));
        assert!(entries.contains(&".env".to_string())); // Universal
        assert!(!entries.contains(&"node_modules/".to_string())); // JS-specific
    }

    #[test]
    fn test_get_gitignore_entries_multiple_languages() {
        let mut languages = HashSet::new();
        languages.insert(Language::JavaScript);
        languages.insert(Language::Rust);

        let entries = get_gitignore_entries_for_languages(&languages);

        assert!(entries.contains(&"node_modules/".to_string()));
        assert!(entries.contains(&"target/".to_string()));
        assert!(entries.contains(&".env".to_string())); // Universal
    }

    #[test]
    fn test_get_gitignore_entries_universal_always_present() {
        let languages = HashSet::new(); // No languages detected

        let entries = get_gitignore_entries_for_languages(&languages);

        assert!(entries.contains(&".env".to_string()));
        assert!(entries.contains(&"*.key".to_string()));
        assert!(entries.contains(&"*.pem".to_string()));
        assert!(entries.contains(&".DS_Store".to_string()));
    }

    #[test]
    fn test_get_gitignore_entries_with_descriptions() {
        let mut languages = HashSet::new();
        languages.insert(Language::Rust);

        let entries = get_gitignore_entries_with_descriptions(&languages);

        let rust_entry = entries.iter().find(|(p, _)| p == "target/");
        assert!(rust_entry.is_some());
        assert_eq!(rust_entry.unwrap().1, "Rust build output");

        let universal_entry = entries.iter().find(|(p, _)| p == ".env");
        assert!(universal_entry.is_some());
        assert_eq!(universal_entry.unwrap().1, "Environment files");
    }

    #[test]
    fn test_detect_go() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("go.mod"), "module example.com/app").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::Go));
    }

    #[test]
    fn test_detect_ruby() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("Gemfile"), "source 'https://rubygems.org'").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::Ruby));
    }

    #[test]
    fn test_detect_php() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("composer.json"), "{}").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::Php));
    }

    #[test]
    fn test_detect_java() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("pom.xml"), "<project></project>").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::Java));
    }

    #[test]
    fn test_detect_csharp() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("App.csproj"), "<Project></Project>").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::CSharp));
    }

    #[test]
    fn test_get_gitignore_entries_go() {
        let mut languages = HashSet::new();
        languages.insert(Language::Go);

        let entries = get_gitignore_entries_for_languages(&languages);
        assert!(entries.contains(&"vendor/".to_string()));
    }

    #[test]
    fn test_get_gitignore_entries_python() {
        let mut languages = HashSet::new();
        languages.insert(Language::Python);

        let entries = get_gitignore_entries_for_languages(&languages);
        assert!(entries.contains(&"__pycache__/".to_string()));
        assert!(entries.contains(&"venv/".to_string()));
    }

    #[test]
    fn test_get_gitignore_entries_ruby() {
        let mut languages = HashSet::new();
        languages.insert(Language::Ruby);

        let entries = get_gitignore_entries_for_languages(&languages);
        assert!(entries.contains(&"vendor/bundle/".to_string()));
    }

    #[test]
    fn test_get_gitignore_entries_php() {
        let mut languages = HashSet::new();
        languages.insert(Language::Php);

        let entries = get_gitignore_entries_for_languages(&languages);
        assert!(entries.contains(&"vendor/".to_string()));
    }

    #[test]
    fn test_get_gitignore_entries_java() {
        let mut languages = HashSet::new();
        languages.insert(Language::Java);

        let entries = get_gitignore_entries_for_languages(&languages);
        assert!(entries.contains(&"target/".to_string()));
        assert!(entries.contains(&"*.class".to_string()));
    }

    #[test]
    fn test_get_gitignore_entries_csharp() {
        let mut languages = HashSet::new();
        languages.insert(Language::CSharp);

        let entries = get_gitignore_entries_for_languages(&languages);
        assert!(entries.contains(&"bin/".to_string()));
        assert!(entries.contains(&"obj/".to_string()));
    }

    #[test]
    fn test_get_gitignore_entries_with_descriptions_multiple() {
        let mut languages = HashSet::new();
        languages.insert(Language::JavaScript);
        languages.insert(Language::Python);

        let entries = get_gitignore_entries_with_descriptions(&languages);

        // Should contain entries for both languages
        assert!(entries.iter().any(|(p, _)| p == "node_modules/"));
        assert!(entries.iter().any(|(p, _)| p == "__pycache__/"));
    }

    #[test]
    fn test_detect_no_languages() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        // Empty directory, no language markers

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.is_empty());
    }

    #[test]
    fn test_detect_typescript() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("tsconfig.json"), "{}").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::JavaScript));
    }

    #[test]
    fn test_detect_python_pipfile() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("Pipfile"), "[packages]").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::Python));
    }

    #[test]
    fn test_detect_java_gradle() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("build.gradle"), "apply plugin: 'java'").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let languages = detect_languages(&scanner);

        assert!(languages.contains(&Language::Java));
    }
}
