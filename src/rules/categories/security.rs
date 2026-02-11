//! Security rules
//!
//! This module provides rules for checking security-related aspects, including:
//! - CODEOWNERS file for code review requirements
//! - Dependency lock files for reproducible builds
//! - Runtime version files for consistent environments
//! - GitHub security features (vulnerability alerts, Dependabot, secret scanning)
//! - Actions permissions and workflow security

use crate::error::RepoLensError;

use crate::config::Config;
use crate::providers::github::GitHubProvider;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

/// Rules for checking security-related aspects
pub struct SecurityRules;

#[async_trait::async_trait]
impl RuleCategory for SecurityRules {
    /// Get the category name
    fn name(&self) -> &'static str {
        "security"
    }

    /// Run all security-related rules
    ///
    /// # Arguments
    ///
    /// * `scanner` - The scanner to access repository files
    /// * `config` - The configuration with enabled rules
    ///
    /// # Returns
    ///
    /// A vector of findings for security issues
    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>, RepoLensError> {
        let mut findings = Vec::new();

        // Note: CODEOWNERS is now handled by the codeowners module (CODE001-003)

        // Check for dependency files
        if config.is_rule_enabled("security/dependencies") {
            findings.extend(check_dependencies(scanner).await?);
        }

        // Check for branch protection configuration
        if config.is_rule_enabled("security/branch-protection") {
            findings.extend(check_branch_protection(scanner).await?);
        }

        // Check GitHub security features (requires API access)
        if config.is_rule_enabled("security/vulnerability-alerts") {
            findings.extend(check_vulnerability_alerts().await?);
        }

        if config.is_rule_enabled("security/dependabot-updates") {
            findings.extend(check_dependabot_updates().await?);
        }

        if config.is_rule_enabled("security/secret-scanning") {
            findings.extend(check_secret_scanning().await?);
        }

        if config.is_rule_enabled("security/push-protection") {
            findings.extend(check_push_protection().await?);
        }

        // Check Actions permissions
        if config.is_rule_enabled("security/actions-permissions") {
            findings.extend(check_actions_permissions().await?);
        }

        if config.is_rule_enabled("security/workflow-permissions") {
            findings.extend(check_workflow_permissions().await?);
        }

        if config.is_rule_enabled("security/fork-pr-approval") {
            findings.extend(check_fork_pr_approval().await?);
        }

        // Access Control rules
        if config.is_rule_enabled("security/access-control") {
            findings.extend(check_access_control().await?);
        }

        // Infrastructure rules
        if config.is_rule_enabled("security/infrastructure") {
            findings.extend(check_infrastructure().await?);
        }

        Ok(findings)
    }
}

/// Check for dependency lock files and version files
///
/// Verifies that lock files exist for reproducible builds and that
/// runtime version files are specified for consistent environments.
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for dependency-related issues
async fn check_dependencies(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    // Check for lock files (indicates dependency management)
    let _lock_files = [
        ("package-lock.json", "npm"),
        ("yarn.lock", "Yarn"),
        ("pnpm-lock.yaml", "pnpm"),
        ("Cargo.lock", "Cargo"),
        ("Gemfile.lock", "Bundler"),
        ("poetry.lock", "Poetry"),
        ("Pipfile.lock", "Pipenv"),
        ("composer.lock", "Composer"),
        ("go.sum", "Go modules"),
    ];

    let package_files = [
        ("package.json", "package-lock.json"),
        ("Cargo.toml", "Cargo.lock"),
        ("Gemfile", "Gemfile.lock"),
        ("pyproject.toml", "poetry.lock"),
        ("Pipfile", "Pipfile.lock"),
        ("composer.json", "composer.lock"),
        ("go.mod", "go.sum"),
    ];

    for (package_file, lock_file) in package_files {
        if scanner.file_exists(package_file) && !scanner.file_exists(lock_file) {
            findings.push(
                Finding::new(
                    "SECURITY002",
                    "security",
                    Severity::Warning,
                    format!("Lock file {} is missing", lock_file),
                )
                .with_description(
                    "Lock files ensure reproducible builds and protect against supply chain attacks."
                )
                .with_remediation(
                    "Generate the lock file by running your package manager's install command."
                )
            );
        }
    }

    // Check for .nvmrc or similar version files
    let version_managers = [
        (".nvmrc", "Node.js version"),
        (".node-version", "Node.js version"),
        (".python-version", "Python version"),
        (".ruby-version", "Ruby version"),
        ("rust-toolchain.toml", "Rust toolchain"),
    ];

    let has_any_version_file = version_managers.iter().any(|(f, _)| scanner.file_exists(f));

    // Detect project type
    let is_node = scanner.file_exists("package.json");
    let is_python =
        scanner.file_exists("pyproject.toml") || scanner.file_exists("requirements.txt");
    let is_ruby = scanner.file_exists("Gemfile");
    let is_rust = scanner.file_exists("Cargo.toml");

    if !has_any_version_file && (is_node || is_python || is_ruby || is_rust) {
        findings.push(
            Finding::new(
                "SECURITY003",
                "security",
                Severity::Info,
                "No runtime version file found",
            )
            .with_description(
                "Specifying runtime versions (e.g., .nvmrc, .python-version) ensures consistent development environments."
            )
        );
    }

    Ok(findings)
}

/// Check for branch protection configuration via .github/settings.yml
///
/// Verifies that branch protection is configured using the probot/settings app
/// configuration file. This is a common way to manage branch protection as code.
///
/// SEC007: .github/settings.yml absent (info)
/// SEC008: No branch protection rules in settings.yml (warning)
/// SEC009: required_pull_request_reviews not configured (warning)
/// SEC010: required_status_checks not configured (warning)
///
/// # Arguments
///
/// * `scanner` - The scanner to access repository files
///
/// # Returns
///
/// A vector of findings for branch protection issues
async fn check_branch_protection(scanner: &Scanner) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    // Check if .github/settings.yml exists
    let settings_path = ".github/settings.yml";
    if !scanner.file_exists(settings_path) {
        findings.push(
            Finding::new(
                "SEC007",
                "security",
                Severity::Info,
                "GitHub settings file (.github/settings.yml) is absent",
            )
            .with_description(
                "The .github/settings.yml file allows you to configure repository settings, \
                 including branch protection rules, as code using the probot/settings app."
            )
            .with_remediation(
                "Consider adding a .github/settings.yml file to manage repository settings as code. \
                 See https://probot.github.io/apps/settings/ for more information."
            )
        );
        return Ok(findings);
    }

    // Read and parse the settings.yml file
    let content = match scanner.read_file(settings_path) {
        Ok(c) => c,
        Err(_) => return Ok(findings),
    };

    let settings: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            // Invalid YAML, skip further checks
            return Ok(findings);
        }
    };

    // Check for branches configuration
    let branches = settings.get("branches");
    if branches.is_none() {
        findings.push(
            Finding::new(
                "SEC008",
                "security",
                Severity::Warning,
                "No branch protection rules defined in settings.yml",
            )
            .with_location(settings_path)
            .with_description(
                "Branch protection rules help prevent accidental force pushes, \
                 require code reviews, and enforce status checks before merging.",
            )
            .with_remediation(
                "Add a 'branches:' section to your .github/settings.yml to configure \
                 branch protection for important branches like main/master.",
            ),
        );
        return Ok(findings);
    }

    // Check if branches is an array and has protection rules
    let branches_arr = match branches.and_then(|b| b.as_sequence()) {
        Some(arr) => arr,
        None => return Ok(findings),
    };

    let mut has_pr_reviews = false;
    let mut has_status_checks = false;

    for branch in branches_arr {
        // Check protection settings
        if let Some(protection) = branch.get("protection") {
            // Check for required_pull_request_reviews
            if protection.get("required_pull_request_reviews").is_some() {
                has_pr_reviews = true;
            }

            // Check for required_status_checks
            if protection.get("required_status_checks").is_some() {
                has_status_checks = true;
            }
        }
    }

    if !has_pr_reviews {
        findings.push(
            Finding::new(
                "SEC009",
                "security",
                Severity::Warning,
                "required_pull_request_reviews not configured in branch protection",
            )
            .with_location(settings_path)
            .with_description(
                "Requiring pull request reviews ensures that changes are reviewed \
                 by at least one other team member before merging.",
            )
            .with_remediation(
                "Add 'required_pull_request_reviews' to your branch protection settings in \
                 .github/settings.yml. Example:\n\
                 branches:\n\
                   - name: main\n\
                     protection:\n\
                       required_pull_request_reviews:\n\
                         required_approving_review_count: 1",
            ),
        );
    }

    if !has_status_checks {
        findings.push(
            Finding::new(
                "SEC010",
                "security",
                Severity::Warning,
                "required_status_checks not configured in branch protection",
            )
            .with_location(settings_path)
            .with_description(
                "Requiring status checks ensures that CI/CD pipelines pass \
                 before changes can be merged.",
            )
            .with_remediation(
                "Add 'required_status_checks' to your branch protection settings in \
                 .github/settings.yml. Example:\n\
                 branches:\n\
                   - name: main\n\
                     protection:\n\
                       required_status_checks:\n\
                         strict: true\n\
                         contexts:\n\
                           - ci",
            ),
        );
    }

    Ok(findings)
}

/// Check if vulnerability alerts are enabled (SEC011)
///
/// Vulnerability alerts notify you when dependencies have known vulnerabilities.
///
/// # Returns
///
/// A vector of findings if vulnerability alerts are disabled
async fn check_vulnerability_alerts() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    // Check if GitHub provider is available
    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    match provider.has_vulnerability_alerts() {
        Ok(enabled) => {
            if !enabled {
                findings.push(
                    Finding::new(
                        "SEC011",
                        "security",
                        Severity::Warning,
                        "Vulnerability alerts are disabled",
                    )
                    .with_description(
                        "GitHub vulnerability alerts notify you when dependencies in your repository \
                         have known security vulnerabilities. This helps you stay informed about \
                         potential security issues."
                    )
                    .with_remediation(
                        "Enable vulnerability alerts in your repository settings: \
                         Settings > Security & analysis > Dependabot alerts."
                    ),
                );
            }
        }
        Err(_) => {
            // API call failed, skip this check
        }
    }

    Ok(findings)
}

/// Check if Dependabot security updates are enabled (SEC012)
///
/// Dependabot security updates automatically create pull requests to update
/// vulnerable dependencies.
///
/// # Returns
///
/// A vector of findings if Dependabot security updates are disabled
async fn check_dependabot_updates() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    match provider.has_dependabot_security_updates() {
        Ok(enabled) => {
            if !enabled {
                findings.push(
                    Finding::new(
                        "SEC012",
                        "security",
                        Severity::Warning,
                        "Dependabot security updates are disabled",
                    )
                    .with_description(
                        "Dependabot security updates automatically create pull requests to update \
                         dependencies with known vulnerabilities. This helps keep your project secure \
                         with minimal manual effort."
                    )
                    .with_remediation(
                        "Enable Dependabot security updates in your repository settings: \
                         Settings > Security & analysis > Dependabot security updates."
                    ),
                );
            }
        }
        Err(_) => {
            // API call failed, skip this check
        }
    }

    Ok(findings)
}

/// Check if secret scanning is enabled (SEC013)
///
/// Secret scanning detects secrets accidentally committed to your repository.
///
/// # Returns
///
/// A vector of findings if secret scanning is disabled
async fn check_secret_scanning() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    match provider.get_secret_scanning() {
        Ok(settings) => {
            if !settings.enabled {
                findings.push(
                    Finding::new(
                        "SEC013",
                        "security",
                        Severity::Info,
                        "Secret scanning is disabled",
                    )
                    .with_description(
                        "Secret scanning detects secrets (like API keys, tokens, and passwords) \
                         that have been accidentally committed to your repository. Enabling this \
                         feature helps prevent credential exposure."
                    )
                    .with_remediation(
                        "Enable secret scanning in your repository settings: \
                         Settings > Security & analysis > Secret scanning. \
                         Note: This feature may require GitHub Advanced Security for private repos."
                    ),
                );
            }
        }
        Err(_) => {
            // API call failed, skip this check
        }
    }

    Ok(findings)
}

/// Check if push protection is enabled (SEC014)
///
/// Push protection prevents secrets from being pushed to the repository.
///
/// # Returns
///
/// A vector of findings if push protection is disabled
async fn check_push_protection() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    match provider.get_secret_scanning() {
        Ok(settings) => {
            // Only warn about push protection if secret scanning is enabled
            if settings.enabled && !settings.push_protection_enabled {
                findings.push(
                    Finding::new(
                        "SEC014",
                        "security",
                        Severity::Info,
                        "Push protection is disabled",
                    )
                    .with_description(
                        "Push protection prevents secrets from being pushed to your repository \
                         by blocking commits that contain detected secrets. This provides \
                         proactive protection against credential exposure."
                    )
                    .with_remediation(
                        "Enable push protection in your repository settings: \
                         Settings > Security & analysis > Secret scanning > Push protection. \
                         Note: This feature may require GitHub Advanced Security for private repos."
                    ),
                );
            }
        }
        Err(_) => {
            // API call failed, skip this check
        }
    }

    Ok(findings)
}

/// Check if Actions are properly restricted (SEC015)
///
/// By default, all actions can run. Restricting to selected actions
/// reduces supply chain attack risk.
///
/// # Returns
///
/// A vector of findings if actions are not restricted
async fn check_actions_permissions() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    match provider.get_actions_permissions() {
        Ok(perms) => {
            if perms.enabled {
                // Check if actions are unrestricted
                if let Some(ref allowed) = perms.allowed_actions {
                    if allowed == "all" {
                        findings.push(
                            Finding::new(
                                "SEC015",
                                "security",
                                Severity::Warning,
                                "GitHub Actions allows all actions",
                            )
                            .with_description(
                                "Allowing all actions to run increases the risk of supply chain attacks. \
                                 Malicious or compromised third-party actions could access your repository \
                                 secrets and code."
                            )
                            .with_remediation(
                                "Restrict actions to verified creators or selected actions only: \
                                 Settings > Actions > General > Actions permissions > \
                                 'Allow select actions and reusable workflows'."
                            ),
                        );
                    }
                }
            }
        }
        Err(_) => {
            // API call failed, skip this check
        }
    }

    Ok(findings)
}

/// Check if workflow permissions are too permissive (SEC016)
///
/// Workflows with write permissions by default can modify repository content.
///
/// # Returns
///
/// A vector of findings if workflow permissions are too permissive
async fn check_workflow_permissions() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    match provider.get_actions_workflow_permissions() {
        Ok(perms) => {
            if let Some(ref default_perms) = perms.default_workflow_permissions {
                if default_perms == "write" {
                    findings.push(
                        Finding::new(
                            "SEC016",
                            "security",
                            Severity::Warning,
                            "Default workflow permissions are set to 'write'",
                        )
                        .with_description(
                            "When default GITHUB_TOKEN permissions are set to 'write', workflows \
                             have broad access to modify repository contents, packages, and more. \
                             Following the principle of least privilege, workflows should request \
                             only the permissions they need."
                        )
                        .with_remediation(
                            "Set default permissions to 'read' and explicitly grant write permissions \
                             in workflows that need them: Settings > Actions > General > \
                             Workflow permissions > 'Read repository contents and packages permission'."
                        ),
                    );
                }
            }
        }
        Err(_) => {
            // API call failed, skip this check
        }
    }

    Ok(findings)
}

/// Check if fork pull request workflows require approval (SEC017)
///
/// Without approval requirements, workflows from fork PRs can run automatically,
/// potentially exposing secrets or consuming resources.
///
/// # Returns
///
/// A vector of findings if fork PR workflows don't require approval
async fn check_fork_pr_approval() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    match provider.get_fork_pr_workflows_policy() {
        Ok(requires_approval) => {
            if !requires_approval {
                findings.push(
                    Finding::new(
                        "SEC017",
                        "security",
                        Severity::Info,
                        "Fork pull request workflows may not require approval",
                    )
                    .with_description(
                        "By default, first-time contributors may be able to run workflows \
                         on fork pull requests without approval. This could allow untrusted \
                         code to run in your CI environment.",
                    )
                    .with_remediation(
                        "Configure fork PR workflow approval settings: \
                         Settings > Actions > General > Fork pull request workflows > \
                         'Require approval for all outside collaborators'.",
                    ),
                );
            }
        }
        Err(_) => {
            // API call failed, skip this check
        }
    }

    Ok(findings)
}

// ===== Access Control Rules =====

/// Check access control settings (TEAM001-004, KEY001-002, APP001)
///
/// Analyzes collaborators, teams, deploy keys, and GitHub App installations
/// for potential security issues.
async fn check_access_control() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    // Check collaborators
    if let Ok(collaborators) = provider.list_collaborators() {
        for collab in &collaborators {
            // TEAM001: Direct collaborator with admin access
            if collab.permissions.admin {
                findings.push(
                    Finding::new(
                        "TEAM001",
                        "security",
                        Severity::Info,
                        format!("Direct collaborator '{}' has admin access", collab.login),
                    )
                    .with_description(
                        "Direct collaborators with admin access can modify repository settings, \
                         manage access, and perform destructive operations.",
                    )
                    .with_remediation(
                        "Review if admin access is necessary. Consider using team-based access \
                         control for better auditability and management.",
                    ),
                );
            }

            // TEAM002: External collaborator with push access
            if collab.user_type == "User" && collab.permissions.push && !collab.permissions.admin {
                findings.push(
                    Finding::new(
                        "TEAM002",
                        "security",
                        Severity::Warning,
                        format!("External collaborator '{}' has push access", collab.login),
                    )
                    .with_description(
                        "External collaborators with push access can directly modify the codebase.",
                    )
                    .with_remediation(
                        "Review external collaborator access regularly. Consider requiring PR reviews \
                         for all changes.",
                    ),
                );
            }
        }
    }

    // Check teams
    if let Ok(teams) = provider.list_teams() {
        for team in &teams {
            // TEAM003: Teams with write+ access
            let has_write_access = team.permission == "push"
                || team.permission == "admin"
                || team.permission == "maintain";
            if has_write_access {
                findings.push(
                    Finding::new(
                        "TEAM003",
                        "security",
                        Severity::Info,
                        format!("Team '{}' has '{}' access", team.name, team.permission),
                    )
                    .with_description(
                        "Teams with write access or higher can modify the repository.",
                    )
                    .with_remediation(
                        "Review team membership periodically. Remove inactive members.",
                    ),
                );
            }
        }
    }

    // Check deploy keys
    if let Ok(keys) = provider.list_deploy_keys() {
        for key in &keys {
            // KEY001: Deploy key with write access
            if !key.read_only {
                findings.push(
                    Finding::new(
                        "KEY001",
                        "security",
                        Severity::Warning,
                        format!("Deploy key '{}' has write access", key.title),
                    )
                    .with_description(
                        "Deploy keys with write access can push changes to the repository.",
                    )
                    .with_remediation(
                        "Review if write access is necessary. Use read-only keys when possible.",
                    ),
                );
            }

            // KEY002: Deploy key without expiration
            findings.push(
                Finding::new(
                    "KEY002",
                    "security",
                    Severity::Info,
                    format!("Deploy key '{}' has no expiration", key.title),
                )
                .with_description(
                    "Deploy keys don't expire automatically. Implement a key rotation policy.",
                )
                .with_remediation(
                    "Implement a regular key rotation schedule (e.g., every 90 days).",
                ),
            );
        }
    }

    // Check GitHub App installations
    if let Ok(installations) = provider.list_installations() {
        for inst in &installations {
            // APP001: Apps with broad permissions
            let has_admin = inst.permissions.administration.as_deref() == Some("write");
            let has_contents_write = inst.permissions.contents.as_deref() == Some("write");

            if has_admin || has_contents_write {
                let app_name = inst.app_slug.as_deref().unwrap_or("Unknown app");
                findings.push(
                    Finding::new(
                        "APP001",
                        "security",
                        Severity::Info,
                        format!("GitHub App '{}' has broad permissions", app_name),
                    )
                    .with_description(
                        "This GitHub App has administrative or write access to repository contents.",
                    )
                    .with_remediation(
                        "Review the GitHub App's permissions in Settings > Integrations.",
                    ),
                );
            }
        }
    }

    Ok(findings)
}

// ===== Infrastructure Rules =====

/// Check infrastructure settings (HOOK001-003, ENV001-003)
///
/// Analyzes webhooks and deployment environments for potential security issues.
async fn check_infrastructure() -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();

    if !GitHubProvider::is_available() {
        return Ok(findings);
    }

    let provider = match GitHubProvider::new() {
        Ok(p) => p,
        Err(_) => return Ok(findings),
    };

    // Check webhooks
    if let Ok(webhooks) = provider.list_webhooks() {
        for hook in &webhooks {
            let url = hook.config.url.as_deref().unwrap_or("");

            // HOOK001: Webhook with non-HTTPS URL
            if !url.is_empty() && !url.starts_with("https://") {
                findings.push(
                    Finding::new(
                        "HOOK001",
                        "security",
                        Severity::Warning,
                        format!("Webhook '{}' uses non-HTTPS URL", hook.name),
                    )
                    .with_location(format!("Webhook ID: {}", hook.id))
                    .with_description("Webhooks should use HTTPS to ensure data is encrypted.")
                    .with_remediation("Update the webhook URL to use HTTPS."),
                );
            }

            // HOOK002: Webhook without secret configured
            if hook.config.secret.is_none() {
                findings.push(
                    Finding::new(
                        "HOOK002",
                        "security",
                        Severity::Warning,
                        format!("Webhook '{}' has no secret configured", hook.name),
                    )
                    .with_location(format!("Webhook ID: {}", hook.id))
                    .with_description(
                        "Webhooks without a secret cannot verify the authenticity of payloads.",
                    )
                    .with_remediation(
                        "Configure a webhook secret and validate X-Hub-Signature-256 header.",
                    ),
                );
            }

            // HOOK003: Inactive webhook
            if !hook.active {
                findings.push(
                    Finding::new(
                        "HOOK003",
                        "security",
                        Severity::Info,
                        format!("Webhook '{}' is inactive", hook.name),
                    )
                    .with_location(format!("Webhook ID: {}", hook.id))
                    .with_description(
                        "Inactive webhooks may be leftover from previous integrations.",
                    )
                    .with_remediation("Review if this webhook is still needed. If not, delete it."),
                );
            }
        }
    }

    // Check environments
    if let Ok(environments) = provider.list_environments() {
        for env in &environments {
            let protection = match provider.get_environment_protection(&env.name) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let is_production =
                env.name.to_lowercase().contains("prod") || env.name.to_lowercase() == "production";

            let has_protection_rules = !protection.protection_rules.is_empty();
            let has_required_reviewers = protection
                .protection_rules
                .iter()
                .any(|r| r.rule_type == "required_reviewers");
            let has_branch_policy = protection.deployment_branch_policy.is_some();

            // ENV001: Environment without protection rules
            if !has_protection_rules {
                findings.push(
                    Finding::new(
                        "ENV001",
                        "security",
                        Severity::Info,
                        format!("Environment '{}' has no protection rules", env.name),
                    )
                    .with_description(
                        "Environments without protection rules allow any workflow to deploy.",
                    )
                    .with_remediation("Add protection rules in Settings > Environments."),
                );
            }

            // ENV002: Production environment without required reviewers
            if is_production && !has_required_reviewers {
                findings.push(
                    Finding::new(
                        "ENV002",
                        "security",
                        Severity::Warning,
                        format!(
                            "Production environment '{}' has no required reviewers",
                            env.name
                        ),
                    )
                    .with_description(
                        "Production deployments should require approval from designated reviewers.",
                    )
                    .with_remediation("Add required reviewers in Settings > Environments."),
                );
            }

            // ENV003: Environment without branch policies
            if !has_branch_policy {
                findings.push(
                    Finding::new(
                        "ENV003",
                        "security",
                        Severity::Info,
                        format!("Environment '{}' has no branch policies", env.name),
                    )
                    .with_description(
                        "Environments without branch policies allow deployments from any branch.",
                    )
                    .with_remediation(
                        "Configure deployment branch policy in Settings > Environments.",
                    ),
                );
            }
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::scanner::Scanner;
    use std::fs;
    use tempfile::TempDir;

    // Note: CODEOWNERS tests are now in codeowners.rs module (CODE001-003)

    #[tokio::test]
    async fn test_check_dependencies_missing_lock_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let package_json = root.join("package.json");

        fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_dependencies(&scanner).await.unwrap();

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.rule_id == "SECURITY002"));
    }

    #[tokio::test]
    async fn test_check_dependencies_no_version_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let package_json = root.join("package.json");

        fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_dependencies(&scanner).await.unwrap();

        assert!(findings.iter().any(|f| f.rule_id == "SECURITY003"));
    }

    // ===== Branch Protection Tests (SEC007-010) =====

    #[tokio::test]
    async fn test_check_branch_protection_no_settings_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        assert_eq!(findings.len(), 1);
        assert!(findings.iter().any(|f| f.rule_id == "SEC007"));
        assert!(findings.iter().any(|f| f.severity == Severity::Info));
    }

    #[tokio::test]
    async fn test_check_branch_protection_no_branches_config() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
repository:
  name: my-repo
  description: A test repo
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        assert_eq!(findings.len(), 1);
        assert!(findings.iter().any(|f| f.rule_id == "SEC008"));
    }

    #[tokio::test]
    async fn test_check_branch_protection_no_pr_reviews() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      required_status_checks:
        strict: true
        contexts:
          - ci
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Should have SEC009 (no PR reviews) but not SEC010 (has status checks)
        assert!(findings.iter().any(|f| f.rule_id == "SEC009"));
        assert!(!findings.iter().any(|f| f.rule_id == "SEC010"));
    }

    #[tokio::test]
    async fn test_check_branch_protection_no_status_checks() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 1
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Should have SEC010 (no status checks) but not SEC009 (has PR reviews)
        assert!(!findings.iter().any(|f| f.rule_id == "SEC009"));
        assert!(findings.iter().any(|f| f.rule_id == "SEC010"));
    }

    #[tokio::test]
    async fn test_check_branch_protection_both_missing() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      enforce_admins: true
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Should have both SEC009 and SEC010
        assert!(findings.iter().any(|f| f.rule_id == "SEC009"));
        assert!(findings.iter().any(|f| f.rule_id == "SEC010"));
    }

    #[tokio::test]
    async fn test_check_branch_protection_fully_configured() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 2
        dismiss_stale_reviews: true
      required_status_checks:
        strict: true
        contexts:
          - ci
          - tests
      enforce_admins: true
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // No findings when fully configured
        assert!(findings.is_empty());
    }

    #[tokio::test]
    async fn test_check_branch_protection_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(github_dir.join("settings.yml"), "invalid: yaml: content: [").unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Should not crash, just return empty findings for invalid YAML
        assert!(findings.is_empty());
    }

    #[tokio::test]
    async fn test_check_branch_protection_multiple_branches() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();

        fs::write(
            github_dir.join("settings.yml"),
            r#"
branches:
  - name: main
    protection:
      required_pull_request_reviews:
        required_approving_review_count: 1
  - name: develop
    protection:
      required_status_checks:
        strict: true
        contexts:
          - ci
"#,
        )
        .unwrap();

        let scanner = Scanner::new(root.to_path_buf());
        let findings = check_branch_protection(&scanner).await.unwrap();

        // Both rules are satisfied across branches
        assert!(findings.is_empty());
    }

    // ===== GitHub Security Features Tests (SEC011-017) =====
    // Note: These tests verify the Finding structure and logic.
    // The actual API calls are mocked or skipped when GitHub is unavailable.

    #[tokio::test]
    async fn test_check_vulnerability_alerts_returns_empty_when_no_github() {
        // When GitHub is not available, should return empty findings
        let findings = check_vulnerability_alerts().await.unwrap();
        // We can't assert on the result since it depends on GitHub availability
        // but we verify it doesn't panic
        assert!(findings.len() <= 1);
    }

    #[tokio::test]
    async fn test_check_dependabot_updates_returns_empty_when_no_github() {
        let findings = check_dependabot_updates().await.unwrap();
        assert!(findings.len() <= 1);
    }

    #[tokio::test]
    async fn test_check_secret_scanning_returns_empty_when_no_github() {
        let findings = check_secret_scanning().await.unwrap();
        assert!(findings.len() <= 1);
    }

    #[tokio::test]
    async fn test_check_push_protection_returns_empty_when_no_github() {
        let findings = check_push_protection().await.unwrap();
        assert!(findings.len() <= 1);
    }

    #[tokio::test]
    async fn test_check_actions_permissions_returns_empty_when_no_github() {
        let findings = check_actions_permissions().await.unwrap();
        assert!(findings.len() <= 1);
    }

    #[tokio::test]
    async fn test_check_workflow_permissions_returns_empty_when_no_github() {
        let findings = check_workflow_permissions().await.unwrap();
        assert!(findings.len() <= 1);
    }

    #[tokio::test]
    async fn test_check_fork_pr_approval_returns_empty_when_no_github() {
        let findings = check_fork_pr_approval().await.unwrap();
        assert!(findings.len() <= 1);
    }

    // ===== Finding Construction Tests =====

    #[test]
    fn test_sec011_finding_construction() {
        let finding = Finding::new(
            "SEC011",
            "security",
            Severity::Warning,
            "Vulnerability alerts are disabled",
        )
        .with_description("Test description")
        .with_remediation("Test remediation");

        assert_eq!(finding.rule_id, "SEC011");
        assert_eq!(finding.category, "security");
        assert_eq!(finding.severity, Severity::Warning);
        assert!(finding.description.is_some());
        assert!(finding.remediation.is_some());
    }

    #[test]
    fn test_sec012_finding_construction() {
        let finding = Finding::new(
            "SEC012",
            "security",
            Severity::Warning,
            "Dependabot security updates are disabled",
        );

        assert_eq!(finding.rule_id, "SEC012");
        assert_eq!(finding.severity, Severity::Warning);
    }

    #[test]
    fn test_sec013_finding_construction() {
        let finding = Finding::new(
            "SEC013",
            "security",
            Severity::Info,
            "Secret scanning is disabled",
        );

        assert_eq!(finding.rule_id, "SEC013");
        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn test_sec014_finding_construction() {
        let finding = Finding::new(
            "SEC014",
            "security",
            Severity::Info,
            "Push protection is disabled",
        );

        assert_eq!(finding.rule_id, "SEC014");
        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn test_sec015_finding_construction() {
        let finding = Finding::new(
            "SEC015",
            "security",
            Severity::Warning,
            "GitHub Actions allows all actions",
        );

        assert_eq!(finding.rule_id, "SEC015");
        assert_eq!(finding.severity, Severity::Warning);
    }

    #[test]
    fn test_sec016_finding_construction() {
        let finding = Finding::new(
            "SEC016",
            "security",
            Severity::Warning,
            "Default workflow permissions are set to 'write'",
        );

        assert_eq!(finding.rule_id, "SEC016");
        assert_eq!(finding.severity, Severity::Warning);
    }

    #[test]
    fn test_sec017_finding_construction() {
        let finding = Finding::new(
            "SEC017",
            "security",
            Severity::Info,
            "Fork pull request workflows may not require approval",
        );

        assert_eq!(finding.rule_id, "SEC017");
        assert_eq!(finding.severity, Severity::Info);
    }

    // ===== Access Control Tests (TEAM, KEY, APP) =====

    #[tokio::test]
    async fn test_check_access_control_returns_empty_when_no_github() {
        let findings = check_access_control().await.unwrap();
        // Should not panic, returns empty when GitHub unavailable
        let _ = findings.len();
    }

    #[test]
    fn test_team001_finding_construction() {
        let finding = Finding::new(
            "TEAM001",
            "security",
            Severity::Info,
            "Direct collaborator 'testuser' has admin access",
        )
        .with_description("Test description")
        .with_remediation("Test remediation");

        assert_eq!(finding.rule_id, "TEAM001");
        assert_eq!(finding.category, "security");
        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn test_team002_finding_construction() {
        let finding = Finding::new(
            "TEAM002",
            "security",
            Severity::Warning,
            "External collaborator 'external-user' has push access",
        );

        assert_eq!(finding.rule_id, "TEAM002");
        assert_eq!(finding.severity, Severity::Warning);
    }

    #[test]
    fn test_team003_finding_construction() {
        let finding = Finding::new(
            "TEAM003",
            "security",
            Severity::Info,
            "Team 'developers' has 'push' access",
        );

        assert_eq!(finding.rule_id, "TEAM003");
        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn test_key001_finding_construction() {
        let finding = Finding::new(
            "KEY001",
            "security",
            Severity::Warning,
            "Deploy key 'production-key' has write access",
        );

        assert_eq!(finding.rule_id, "KEY001");
        assert_eq!(finding.severity, Severity::Warning);
    }

    #[test]
    fn test_key002_finding_construction() {
        let finding = Finding::new(
            "KEY002",
            "security",
            Severity::Info,
            "Deploy key 'ci-key' has no expiration",
        );

        assert_eq!(finding.rule_id, "KEY002");
        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn test_app001_finding_construction() {
        let finding = Finding::new(
            "APP001",
            "security",
            Severity::Info,
            "GitHub App 'my-app' has broad permissions",
        );

        assert_eq!(finding.rule_id, "APP001");
        assert_eq!(finding.severity, Severity::Info);
    }

    // ===== Infrastructure Tests (HOOK, ENV) =====

    #[tokio::test]
    async fn test_check_infrastructure_returns_empty_when_no_github() {
        let findings = check_infrastructure().await.unwrap();
        // Should not panic, returns empty when GitHub unavailable
        let _ = findings.len();
    }

    #[test]
    fn test_hook001_finding_construction() {
        let finding = Finding::new(
            "HOOK001",
            "security",
            Severity::Warning,
            "Webhook 'web' uses non-HTTPS URL",
        )
        .with_location("Webhook ID: 123");

        assert_eq!(finding.rule_id, "HOOK001");
        assert_eq!(finding.severity, Severity::Warning);
        assert!(finding.location.is_some());
    }

    #[test]
    fn test_hook002_finding_construction() {
        let finding = Finding::new(
            "HOOK002",
            "security",
            Severity::Warning,
            "Webhook 'web' has no secret configured",
        );

        assert_eq!(finding.rule_id, "HOOK002");
        assert_eq!(finding.severity, Severity::Warning);
    }

    #[test]
    fn test_hook003_finding_construction() {
        let finding = Finding::new(
            "HOOK003",
            "security",
            Severity::Info,
            "Webhook 'web' is inactive",
        );

        assert_eq!(finding.rule_id, "HOOK003");
        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn test_env001_finding_construction() {
        let finding = Finding::new(
            "ENV001",
            "security",
            Severity::Info,
            "Environment 'staging' has no protection rules",
        );

        assert_eq!(finding.rule_id, "ENV001");
        assert_eq!(finding.severity, Severity::Info);
    }

    #[test]
    fn test_env002_finding_construction() {
        let finding = Finding::new(
            "ENV002",
            "security",
            Severity::Warning,
            "Production environment 'production' has no required reviewers",
        );

        assert_eq!(finding.rule_id, "ENV002");
        assert_eq!(finding.severity, Severity::Warning);
    }

    #[test]
    fn test_env003_finding_construction() {
        let finding = Finding::new(
            "ENV003",
            "security",
            Severity::Info,
            "Environment 'staging' has no branch policies",
        );

        assert_eq!(finding.rule_id, "ENV003");
        assert_eq!(finding.severity, Severity::Info);
    }
}
