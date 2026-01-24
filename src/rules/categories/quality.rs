//! Code quality rules

use anyhow::Result;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

pub struct QualityRules;

#[async_trait::async_trait]
impl RuleCategory for QualityRules {
    fn name(&self) -> &'static str {
        "quality"
    }

    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        // Check for tests
        if config.is_rule_enabled("quality/tests") {
            findings.extend(check_tests(scanner).await?);
        }

        // Check for linting configuration
        if config.is_rule_enabled("quality/linting") {
            findings.extend(check_linting(scanner).await?);
        }

        // Check for editor configuration
        if config.is_rule_enabled("files/editorconfig") {
            findings.extend(check_editorconfig(scanner).await?);
        }

        Ok(findings)
    }
}

async fn check_tests(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Check for test directories
    let test_dirs = ["test", "tests", "__tests__", "spec", "specs"];
    let has_test_dir = test_dirs.iter().any(|d| scanner.directory_exists(d));

    // Check for test files
    let test_file_patterns = ["*.test.*", "*.spec.*", "*_test.*", "*Test.*"];
    let has_test_files = test_file_patterns
        .iter()
        .any(|p| !scanner.files_matching_pattern(p).is_empty());

    if !has_test_dir && !has_test_files {
        findings.push(
            Finding::new(
                "QUALITY001",
                "quality",
                Severity::Info,
                "No tests detected",
            )
            .with_description(
                "Tests are important for ensuring code quality and catching regressions."
            )
            .with_remediation(
                "Add tests to your project. Consider using a testing framework appropriate for your language."
            )
        );
    }

    // Check if package.json has test script
    if scanner.file_exists("package.json") {
        if let Ok(content) = scanner.read_file("package.json") {
            if !content.contains(r#""test""#) || content.contains(r#""test": "echo"#) {
                findings.push(
                    Finding::new(
                        "QUALITY002",
                        "quality",
                        Severity::Info,
                        "No test script defined in package.json",
                    )
                    .with_description(
                        "A 'test' script in package.json enables running tests with 'npm test'."
                    )
                );
            }
        }
    }

    Ok(findings)
}

async fn check_linting(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Linting config files by language/tool
    let linting_configs = [
        // JavaScript/TypeScript
        (".eslintrc", "ESLint"),
        (".eslintrc.js", "ESLint"),
        (".eslintrc.json", "ESLint"),
        (".eslintrc.yml", "ESLint"),
        ("eslint.config.js", "ESLint"),
        ("biome.json", "Biome"),
        // Formatting
        (".prettierrc", "Prettier"),
        (".prettierrc.js", "Prettier"),
        (".prettierrc.json", "Prettier"),
        // Python
        ("pyproject.toml", "Python tooling"),
        (".flake8", "Flake8"),
        ("setup.cfg", "Python tooling"),
        (".pylintrc", "Pylint"),
        ("ruff.toml", "Ruff"),
        // Ruby
        (".rubocop.yml", "RuboCop"),
        // Go
        (".golangci.yml", "golangci-lint"),
        (".golangci.yaml", "golangci-lint"),
        // Rust
        ("rustfmt.toml", "rustfmt"),
        (".rustfmt.toml", "rustfmt"),
        ("clippy.toml", "Clippy"),
    ];

    // Detect project type
    let is_js_project = scanner.file_exists("package.json");
    let is_python_project = scanner.file_exists("pyproject.toml") || scanner.file_exists("requirements.txt");
    let is_ruby_project = scanner.file_exists("Gemfile");
    let is_go_project = scanner.file_exists("go.mod");
    let is_rust_project = scanner.file_exists("Cargo.toml");

    let has_linting = linting_configs.iter().any(|(f, _)| scanner.file_exists(f));

    if !has_linting && (is_js_project || is_python_project || is_ruby_project || is_go_project || is_rust_project) {
        let suggestion = if is_js_project {
            "ESLint for linting and Prettier for formatting"
        } else if is_python_project {
            "Ruff or Flake8 for linting"
        } else if is_ruby_project {
            "RuboCop for linting"
        } else if is_go_project {
            "golangci-lint for linting"
        } else {
            "Clippy for linting and rustfmt for formatting"
        };

        findings.push(
            Finding::new(
                "QUALITY003",
                "quality",
                Severity::Info,
                "No linting configuration detected",
            )
            .with_description(
                "Linting tools help maintain consistent code style and catch potential issues."
            )
            .with_remediation(format!(
                "Consider adding {} to your project.",
                suggestion
            ))
        );
    }

    Ok(findings)
}

async fn check_editorconfig(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    if !scanner.file_exists(".editorconfig") {
        findings.push(
            Finding::new(
                "QUALITY004",
                "quality",
                Severity::Info,
                ".editorconfig file is missing",
            )
            .with_description(
                "EditorConfig helps maintain consistent coding styles across different editors and IDEs."
            )
            .with_remediation(
                "Create a .editorconfig file to define coding style preferences."
            )
        );
    }

    Ok(findings)
}
