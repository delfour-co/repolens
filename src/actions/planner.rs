//! Action planner - Creates action plans based on audit results
//!
//! This module provides functionality to generate action plans from audit results.
//! It analyzes findings and creates appropriate actions to fix issues.

use std::collections::HashMap;

use crate::config::Config;
use crate::rules::results::AuditResults;

use super::plan::{
    Action, ActionOperation, ActionPlan, BranchProtectionSettings, GitHubRepoSettings,
};

/// Parameters for planning file creation
struct FileCreationParams<'a> {
    rule_id: &'a str,
    file_path: &'a str,
    template: &'a str,
    action_id: &'a str,
    action_description: &'a str,
    detail: Option<&'a str>,
}

/// Creates action plans based on audit results and configuration
///
/// The `ActionPlanner` analyzes audit findings and generates a plan of actions
/// to fix detected issues. Actions can include:
/// - Creating missing files (LICENSE, CONTRIBUTING.md, etc.)
/// - Updating .gitignore
/// - Configuring branch protection
/// - Updating GitHub repository settings
pub struct ActionPlanner {
    /// Configuration for action planning
    config: Config,
}

impl ActionPlanner {
    /// Create a new action planner with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration that determines which actions to plan
    ///
    /// # Returns
    ///
    /// A new `ActionPlanner` instance
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Create an action plan based on audit results
    ///
    /// Analyzes the audit results and generates actions to fix detected issues.
    /// Only actions enabled in the configuration will be included.
    ///
    /// # Arguments
    ///
    /// * `results` - The audit results to analyze
    ///
    /// # Returns
    ///
    /// An `ActionPlan` containing all planned actions
    pub fn create_plan(&self, results: &AuditResults) -> ActionPlan {
        let mut plan = ActionPlan::new();

        // Plan gitignore updates
        if self.config.actions.gitignore {
            if let Some(action) = self.plan_gitignore_update(results) {
                plan.add(action);
            }
        }

        // Plan license creation
        if self.config.actions.license.enabled {
            if let Some(action) = self.plan_license_creation(results) {
                plan.add(action);
            }
        }

        // Plan CONTRIBUTING creation
        if self.config.actions.contributing {
            if let Some(action) = self.plan_contributing_creation(results) {
                plan.add(action);
            }
        }

        // Plan CODE_OF_CONDUCT creation
        if self.config.actions.code_of_conduct {
            if let Some(action) = self.plan_code_of_conduct_creation(results) {
                plan.add(action);
            }
        }

        // Plan SECURITY.md creation
        if self.config.actions.security_policy {
            if let Some(action) = self.plan_security_creation(results) {
                plan.add(action);
            }
        }

        // Plan branch protection
        if self.config.actions.branch_protection.enabled {
            plan.add(self.plan_branch_protection());
        }

        // Plan GitHub settings
        plan.add(self.plan_github_settings());

        plan
    }

    /// Plan .gitignore updates based on findings
    ///
    /// Collects entries that should be added to .gitignore from audit findings.
    /// The findings already contain language-specific recommendations based on
    /// detected languages in the repository.
    ///
    /// # Arguments
    ///
    /// * `results` - The audit results
    ///
    /// # Returns
    ///
    /// An `Action` to update .gitignore, or `None` if no updates are needed
    fn plan_gitignore_update(&self, results: &AuditResults) -> Option<Action> {
        // Collect entries that should be added to .gitignore from findings
        // These findings are already language-aware thanks to check_gitignore
        let mut entries = Vec::new();

        // Extract patterns from FILE003 findings
        for finding in results.findings_by_category("files") {
            if finding.rule_id == "FILE003" {
                // Extract the pattern from the message
                // Format: ".gitignore missing recommended entry: <pattern>"
                if let Some(pattern) = finding.message.split("entry: ").nth(1) {
                    entries.push(pattern.trim().to_string());
                }
            }
        }

        if entries.is_empty() {
            return None;
        }

        Some(
            Action::new(
                "gitignore-update",
                "gitignore",
                "Add entries to .gitignore",
                ActionOperation::UpdateGitignore {
                    entries: entries.clone(),
                },
            )
            .with_details(entries),
        )
    }

    /// Plan LICENSE file creation
    ///
    /// Creates a LICENSE file if one is missing and license creation is enabled.
    ///
    /// # Arguments
    ///
    /// * `results` - The audit results
    ///
    /// # Returns
    ///
    /// An `Action` to create LICENSE, or `None` if not needed
    fn plan_license_creation(&self, results: &AuditResults) -> Option<Action> {
        // Check if LICENSE is missing
        let needs_license = results
            .findings_by_category("docs")
            .any(|f| f.rule_id == "DOC004");

        if !needs_license {
            return None;
        }

        let license_type = &self.config.actions.license.license_type;
        let mut variables = HashMap::new();

        if let Some(author) = &self.config.actions.license.author {
            variables.insert("author".to_string(), author.clone());
        }

        let year = self
            .config
            .actions
            .license
            .year
            .clone()
            .unwrap_or_else(|| chrono::Utc::now().format("%Y").to_string());
        variables.insert("year".to_string(), year);

        Some(
            Action::new(
                "license-create",
                "file",
                "Create LICENSE file",
                ActionOperation::CreateFile {
                    path: "LICENSE".to_string(),
                    template: format!("LICENSE/{}", license_type),
                    variables,
                },
            )
            .with_detail(format!("License type: {}", license_type)),
        )
    }

    /// Generic helper to plan file creation from template
    ///
    /// # Arguments
    ///
    /// * `results` - The audit results
    /// * `params` - Parameters for file creation
    ///
    /// # Returns
    ///
    /// An `Action` if the file needs to be created, `None` otherwise
    fn plan_file_creation(
        &self,
        results: &AuditResults,
        params: FileCreationParams<'_>,
    ) -> Option<Action> {
        let needs_file = results
            .findings_by_category("docs")
            .any(|f| f.rule_id == params.rule_id);

        if !needs_file {
            return None;
        }

        let mut action = Action::new(
            params.action_id,
            "file",
            params.action_description,
            ActionOperation::CreateFile {
                path: params.file_path.to_string(),
                template: params.template.to_string(),
                variables: HashMap::new(),
            },
        );

        if let Some(detail) = params.detail {
            action = action.with_detail(detail);
        }

        Some(action)
    }

    fn plan_contributing_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "DOC005",
                file_path: "CONTRIBUTING.md",
                template: "CONTRIBUTING.md",
                action_id: "contributing-create",
                action_description: "Create CONTRIBUTING.md",
                detail: None,
            },
        )
    }

    fn plan_code_of_conduct_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "DOC006",
                file_path: "CODE_OF_CONDUCT.md",
                template: "CODE_OF_CONDUCT.md",
                action_id: "coc-create",
                action_description: "Create CODE_OF_CONDUCT.md",
                detail: Some("Using Contributor Covenant template"),
            },
        )
    }

    fn plan_security_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "DOC007",
                file_path: "SECURITY.md",
                template: "SECURITY.md",
                action_id: "security-create",
                action_description: "Create SECURITY.md",
                detail: None,
            },
        )
    }

    /// Plan branch protection configuration
    ///
    /// Creates an action to configure branch protection settings based on
    /// the configuration.
    ///
    /// # Returns
    ///
    /// An `Action` to configure branch protection
    fn plan_branch_protection(&self) -> Action {
        let bp = &self.config.actions.branch_protection;

        let settings = BranchProtectionSettings {
            required_approvals: bp.required_approvals,
            require_status_checks: bp.require_status_checks,
            require_conversation_resolution: true,
            require_linear_history: true,
            block_force_push: bp.block_force_push,
            block_deletions: true,
            enforce_admins: true,
            require_signed_commits: bp.require_signed_commits,
        };

        let mut details = vec![
            format!("Require PR reviews: {}", bp.required_approvals),
            format!("Require status checks: {}", bp.require_status_checks),
            format!("Block force push: {}", bp.block_force_push),
        ];

        if bp.require_signed_commits {
            details.push("Require signed commits".to_string());
        }

        Action::new(
            "branch-protection",
            "github",
            format!("Enable branch protection on '{}'", bp.branch),
            ActionOperation::ConfigureBranchProtection {
                branch: bp.branch.clone(),
                settings,
            },
        )
        .with_details(details)
    }

    /// Plan GitHub repository settings updates
    ///
    /// Creates an action to update GitHub repository settings like discussions,
    /// vulnerability alerts, etc.
    ///
    /// # Returns
    ///
    /// An `Action` to update GitHub settings
    fn plan_github_settings(&self) -> Action {
        let gs = &self.config.actions.github_settings;

        let settings = GitHubRepoSettings {
            enable_discussions: Some(gs.discussions),
            enable_issues: Some(gs.issues),
            enable_wiki: Some(gs.wiki),
            enable_vulnerability_alerts: Some(gs.vulnerability_alerts),
            enable_automated_security_fixes: Some(gs.automated_security_fixes),
        };

        let mut details = Vec::new();

        if gs.discussions {
            details.push("Enable discussions".to_string());
        }
        if gs.vulnerability_alerts {
            details.push("Enable vulnerability alerts".to_string());
        }
        if gs.automated_security_fixes {
            details.push("Enable automated security fixes".to_string());
        }

        Action::new(
            "github-settings",
            "github",
            "Update repository settings",
            ActionOperation::UpdateGitHubSettings { settings },
        )
        .with_details(details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::rules::results::{AuditResults, Finding, Severity};

    #[test]
    fn test_create_plan_includes_gitignore() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "FILE003",
            "files",
            Severity::Info,
            ".gitignore missing recommended entry: .env",
        ));

        let plan = planner.create_plan(&results);

        assert!(!plan.is_empty());
        assert!(plan.actions().iter().any(|a| a.id() == "gitignore-update"));
    }

    #[test]
    fn test_create_plan_includes_license() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC004",
            "docs",
            Severity::Critical,
            "LICENSE file is missing",
        ));

        let plan = planner.create_plan(&results);

        assert!(plan.actions().iter().any(|a| a.id() == "license-create"));
    }

    #[test]
    fn test_create_plan_includes_contributing() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC005",
            "docs",
            Severity::Warning,
            "CONTRIBUTING file is missing",
        ));

        let plan = planner.create_plan(&results);

        assert!(plan
            .actions()
            .iter()
            .any(|a| a.id() == "contributing-create"));
    }

    #[test]
    fn test_create_plan_filters_by_config() {
        let mut config = Config::default();
        config.actions.contributing = false;

        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC005",
            "docs",
            Severity::Warning,
            "CONTRIBUTING file is missing",
        ));

        let plan = planner.create_plan(&results);

        // Should not include contributing because it's disabled in config
        assert!(!plan
            .actions()
            .iter()
            .any(|a| a.id() == "contributing-create"));
    }
}
