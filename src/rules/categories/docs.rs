//! Documentation rules

use anyhow::Result;

use crate::config::Config;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

pub struct DocsRules;

#[async_trait::async_trait]
impl RuleCategory for DocsRules {
    fn name(&self) -> &'static str {
        "docs"
    }

    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        // Check README
        if config.is_rule_enabled("docs/readme") {
            findings.extend(check_readme(scanner).await?);
        }

        // Check LICENSE
        if config.is_rule_enabled("docs/license") {
            findings.extend(check_license(scanner, config).await?);
        }

        // Check CONTRIBUTING
        if config.is_rule_enabled("docs/contributing") {
            findings.extend(check_contributing(scanner).await?);
        }

        // Check CODE_OF_CONDUCT
        if config.is_rule_enabled("docs/code-of-conduct") {
            findings.extend(check_code_of_conduct(scanner).await?);
        }

        // Check SECURITY
        if config.is_rule_enabled("docs/security") {
            findings.extend(check_security(scanner).await?);
        }

        Ok(findings)
    }
}

async fn check_readme(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let readme_files = ["README.md", "README", "README.txt", "README.rst"];
    let has_readme = readme_files.iter().any(|f| scanner.file_exists(f));

    if !has_readme {
        findings.push(
            Finding::new(
                "DOC001",
                "docs",
                Severity::Warning,
                "README file is missing",
            )
            .with_description(
                "A README file is essential for explaining what the project does and how to use it."
            )
            .with_remediation(
                "Create a README.md file with project description, installation instructions, and usage examples."
            )
        );
        return Ok(findings);
    }

    // Check README quality
    if let Ok(content) = scanner.read_file("README.md") {
        let line_count = content.lines().count();

        if line_count < 10 {
            findings.push(
                Finding::new(
                    "DOC002",
                    "docs",
                    Severity::Warning,
                    format!("README is too short ({} lines)", line_count),
                )
                .with_description(
                    "A comprehensive README should include sections for description, installation, usage, and contribution guidelines."
                )
            );
        }

        // Check for recommended sections
        let sections = [
            ("installation", "Installation instructions"),
            ("usage", "Usage examples"),
            ("license", "License information"),
        ];

        for (keyword, description) in sections {
            if !content.to_lowercase().contains(keyword) {
                findings.push(
                    Finding::new(
                        "DOC003",
                        "docs",
                        Severity::Info,
                        format!("README missing section: {}", description),
                    )
                );
            }
        }
    }

    Ok(findings)
}

async fn check_license(scanner: &Scanner, config: &Config) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let license_files = ["LICENSE", "LICENSE.md", "LICENSE.txt", "COPYING"];
    let has_license = license_files.iter().any(|f| scanner.file_exists(f));

    // For enterprise preset, LICENSE is optional
    if config.preset == "enterprise" && !has_license {
        return Ok(findings);
    }

    if !has_license {
        findings.push(
            Finding::new(
                "DOC004",
                "docs",
                Severity::Critical,
                "LICENSE file is missing",
            )
            .with_description(
                "A LICENSE file is required for open source projects to define how others can use your code."
            )
            .with_remediation(
                "Add a LICENSE file with an appropriate open source license (MIT, Apache-2.0, GPL-3.0, etc.)."
            )
        );
    }

    Ok(findings)
}

async fn check_contributing(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let contributing_files = ["CONTRIBUTING.md", "CONTRIBUTING", ".github/CONTRIBUTING.md"];
    let has_contributing = contributing_files.iter().any(|f| scanner.file_exists(f));

    if !has_contributing {
        findings.push(
            Finding::new(
                "DOC005",
                "docs",
                Severity::Warning,
                "CONTRIBUTING file is missing",
            )
            .with_description(
                "A CONTRIBUTING file helps potential contributors understand how to participate in your project."
            )
            .with_remediation(
                "Create a CONTRIBUTING.md file with contribution guidelines, code style, and pull request process."
            )
        );
    }

    Ok(findings)
}

async fn check_code_of_conduct(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let coc_files = ["CODE_OF_CONDUCT.md", "CODE_OF_CONDUCT", ".github/CODE_OF_CONDUCT.md"];
    let has_coc = coc_files.iter().any(|f| scanner.file_exists(f));

    if !has_coc {
        findings.push(
            Finding::new(
                "DOC006",
                "docs",
                Severity::Warning,
                "CODE_OF_CONDUCT file is missing",
            )
            .with_description(
                "A Code of Conduct establishes expectations for behavior and helps create a welcoming community."
            )
            .with_remediation(
                "Add a CODE_OF_CONDUCT.md file. Consider using the Contributor Covenant as a starting point."
            )
        );
    }

    Ok(findings)
}

async fn check_security(scanner: &Scanner) -> Result<Vec<Finding>> {
    let mut findings = Vec::new();

    let security_files = ["SECURITY.md", ".github/SECURITY.md"];
    let has_security = security_files.iter().any(|f| scanner.file_exists(f));

    if !has_security {
        findings.push(
            Finding::new(
                "DOC007",
                "docs",
                Severity::Warning,
                "SECURITY policy file is missing",
            )
            .with_description(
                "A SECURITY.md file tells users how to report security vulnerabilities responsibly."
            )
            .with_remediation(
                "Create a SECURITY.md file with instructions for reporting security issues."
            )
        );
    }

    Ok(findings)
}
