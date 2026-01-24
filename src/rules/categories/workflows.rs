//! GitHub Actions workflow rules

use anyhow::Result;
use regex::Regex;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

pub struct WorkflowsRules;

#[async_trait::async_trait]
impl RuleCategory for WorkflowsRules {
    fn name(&self) -> &'static str {
        "workflows"
    }

    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        // Check for workflows directory
        if !scanner.directory_exists(".github/workflows") {
            return Ok(findings);
        }

        // Check workflow security
        if config.is_rule_enabled("workflows/secrets") {
            findings.extend(check_workflow_secrets(scanner).await?);
        }

        // Check permissions
        if config.is_rule_enabled("workflows/permissions") {
            findings.extend(check_workflow_permissions(scanner).await?);
        }

        // Check pinned actions
        if config.is_rule_enabled("workflows/pinned-actions") {
            findings.extend(check_pinned_actions(scanner, config).await?);
        }

        Ok(findings)
    }
}

async fn check_workflow_secrets(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Patterns that suggest hardcoded secrets in workflows
    let secret_patterns = [
        (r#"password\s*:\s*['"][^'"]+['"]"#, "hardcoded password"),
        (r#"token\s*:\s*['"][^'"]+['"]"#, "hardcoded token"),
        (r#"api[_-]?key\s*:\s*['"][^'"]+['"]"#, "hardcoded API key"),
        (r#"secret\s*:\s*['"][^'"]+['"]"#, "hardcoded secret"),
    ];

    for file in scanner.files_in_directory(".github/workflows") {
        if !file.path.ends_with(".yml") && !file.path.ends_with(".yaml") {
            continue;
        }

        if let Ok(content) = scanner.read_file(&file.path) {
            for (pattern, description) in &secret_patterns {
                let regex = Regex::new(pattern).unwrap();
                if regex.is_match(&content) {
                    // Find line number
                    let line_num = content
                        .lines()
                        .enumerate()
                        .find(|(_, line)| regex.is_match(line))
                        .map(|(i, _)| i + 1)
                        .unwrap_or(0);

                    findings.push(
                        Finding::new(
                            "WF001",
                            "workflows",
                            Severity::Critical,
                            format!("Potential {} in workflow", description),
                        )
                        .with_location(format!("{}:{}", file.path, line_num))
                        .with_description(
                            "Secrets should never be hardcoded in workflow files."
                        )
                        .with_remediation(
                            "Use GitHub Secrets (secrets.SECRET_NAME) instead of hardcoded values."
                        )
                    );
                }
            }
        }
    }

    Ok(findings)
}

async fn check_workflow_permissions(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    for file in scanner.files_in_directory(".github/workflows") {
        if !file.path.ends_with(".yml") && !file.path.ends_with(".yaml") {
            continue;
        }

        if let Ok(content) = scanner.read_file(&file.path) {
            // Check if permissions are defined
            if !content.contains("permissions:") {
                findings.push(
                    Finding::new(
                        "WF002",
                        "workflows",
                        Severity::Warning,
                        "Workflow missing explicit permissions",
                    )
                    .with_location(&file.path)
                    .with_description(
                        "Workflows without explicit permissions use the default permissions, which may be more permissive than necessary."
                    )
                    .with_remediation(
                        "Add a 'permissions:' block to explicitly define the minimum required permissions."
                    )
                );
            }
        }
    }

    Ok(findings)
}

async fn check_pinned_actions(scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Only check in strict mode
    if config.preset != "strict" {
        return Ok(findings);
    }

    let unpinned_patterns = [
        r"uses:\s+\S+@main\b",
        r"uses:\s+\S+@master\b",
        r"uses:\s+\S+@latest\b",
    ];

    for file in scanner.files_in_directory(".github/workflows") {
        if !file.path.ends_with(".yml") && !file.path.ends_with(".yaml") {
            continue;
        }

        if let Ok(content) = scanner.read_file(&file.path) {
            for pattern in &unpinned_patterns {
                let regex = Regex::new(pattern).unwrap();
                for (line_num, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        findings.push(
                            Finding::new(
                                "WF003",
                                "workflows",
                                Severity::Warning,
                                "Workflow uses unpinned action reference",
                            )
                            .with_location(format!("{}:{}", file.path, line_num + 1))
                            .with_description(
                                "Using @main, @master, or @latest for actions can introduce breaking changes or security vulnerabilities."
                            )
                            .with_remediation(
                                "Pin actions to a specific version tag (e.g., @v4) or commit SHA for maximum security."
                            )
                        );
                    }
                }
            }
        }
    }

    Ok(findings)
}
