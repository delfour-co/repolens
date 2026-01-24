//! Secrets detection rules

use anyhow::Result;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::patterns::SECRET_PATTERNS;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

pub struct SecretsRules;

#[async_trait::async_trait]
impl RuleCategory for SecretsRules {
    fn name(&self) -> &'static str {
        "secrets"
    }

    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        // Check for hardcoded secrets in source files
        if config.is_rule_enabled("secrets/hardcoded") {
            findings.extend(check_hardcoded_secrets(scanner, config).await?);
        }

        // Check for sensitive files
        if config.is_rule_enabled("secrets/files") {
            findings.extend(check_sensitive_files(scanner, config).await?);
        }

        // Check for .env files
        if config.is_rule_enabled("secrets/env") {
            findings.extend(check_env_files(scanner, config).await?);
        }

        Ok(findings)
    }
}

async fn check_hardcoded_secrets(scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // File extensions to scan
    let extensions = [
        "js", "ts", "jsx", "tsx", "py", "rb", "php", "java", "go", "rs",
        "cpp", "c", "yml", "yaml", "json", "toml", "env", "config", "conf",
        "sql", "sh", "bash",
    ];

    for file in scanner.files_with_extensions(&extensions) {
        // Skip ignored files
        if config.should_ignore_file(&file.path) {
            continue;
        }

        // Check file content against secret patterns
        if let Ok(content) = scanner.read_file(&file.path) {
            for pattern in SECRET_PATTERNS.iter() {
                if let Some(captures) = pattern.regex.captures(&content) {
                    // Skip if pattern should be ignored
                    if config.should_ignore_pattern(&file.path) {
                        continue;
                    }

                    // Find line number
                    let line_num = content[..captures.get(0).unwrap().start()]
                        .matches('\n')
                        .count() + 1;

                    findings.push(
                        Finding::new(
                            "SEC001",
                            "secrets",
                            Severity::Critical,
                            format!("{} detected", pattern.name),
                        )
                        .with_location(format!("{}:{}", file.path, line_num))
                        .with_description(pattern.description.to_string())
                        .with_remediation(
                            "Remove the secret and use environment variables or a secrets manager instead."
                        )
                    );
                }
            }
        }
    }

    Ok(findings)
}

async fn check_sensitive_files(scanner: &Scanner, _config: &Config) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // List of sensitive file patterns
    let sensitive_patterns = [
        ("*.pem", "Private key file"),
        ("*.key", "Private key file"),
        ("*.p12", "PKCS#12 certificate bundle"),
        ("*.pfx", "PKCS#12 certificate bundle"),
        ("*.jks", "Java keystore"),
        ("id_rsa", "SSH private key"),
        ("id_dsa", "SSH private key"),
        ("id_ecdsa", "SSH private key"),
        ("id_ed25519", "SSH private key"),
        (".htpasswd", "Apache password file"),
        ("credentials.json", "Credentials file"),
        ("service-account.json", "Service account credentials"),
        ("secrets.yml", "Secrets configuration"),
        ("secrets.yaml", "Secrets configuration"),
        ("secrets.json", "Secrets configuration"),
    ];

    for (pattern, description) in sensitive_patterns {
        for file in scanner.files_matching_pattern(pattern) {
            findings.push(
                Finding::new(
                    "SEC002",
                    "secrets",
                    Severity::Critical,
                    format!("{} found in repository", description),
                )
                .with_location(&file.path)
                .with_description(format!(
                    "The file '{}' appears to contain sensitive data and should not be committed to version control.",
                    file.path
                ))
                .with_remediation(
                    "Remove the file from the repository and add it to .gitignore. If the file was previously committed, consider rotating any contained credentials."
                )
            );
        }
    }

    Ok(findings)
}

async fn check_env_files(scanner: &Scanner, _config: &Config) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    // Check for .env files (but allow .env.example)
    let env_patterns = [".env", ".env.local", ".env.production", ".env.development", ".env.test"];

    for pattern in env_patterns {
        for file in scanner.files_matching_pattern(pattern) {
            // Allow example/template files
            if file.path.contains(".example") || file.path.contains(".template") || file.path.contains(".sample") {
                continue;
            }

            findings.push(
                Finding::new(
                    "SEC003",
                    "secrets",
                    Severity::Critical,
                    "Environment file found in repository",
                )
                .with_location(&file.path)
                .with_description(
                    "Environment files often contain sensitive configuration and secrets that should not be committed."
                )
                .with_remediation(
                    "Add the file to .gitignore and create a .env.example file as a template."
                )
            );
        }
    }

    Ok(findings)
}
