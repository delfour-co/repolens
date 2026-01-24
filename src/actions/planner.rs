//! Action planner - Creates action plans based on audit results

use std::collections::HashMap;

use crate::config::Config;
use crate::rules::results::{AuditResults, Severity};

use super::plan::{Action, ActionOperation, ActionPlan, BranchProtectionSettings, GitHubRepoSettings};

/// Creates action plans based on audit results and configuration
pub struct ActionPlanner {
    config: Config,
}

impl ActionPlanner {
    /// Create a new action planner
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Create an action plan based on audit results
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

    fn plan_gitignore_update(&self, results: &AuditResults) -> Option<Action> {
        // Collect entries that should be added to .gitignore
        let mut entries = Vec::new();

        // Check findings for missing gitignore entries
        for finding in results.findings_by_category("files") {
            if finding.rule_id == "FILE003" {
                // Extract the pattern from the message
                if let Some(pattern) = finding.message.split("entry: ").nth(1) {
                    entries.push(pattern.to_string());
                }
            }
        }

        // Add standard entries if not present
        let standard_entries = [".env", "*.key", "*.pem", ".DS_Store"];
        for entry in standard_entries {
            if !entries.contains(&entry.to_string()) {
                entries.push(entry.to_string());
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
                ActionOperation::UpdateGitignore { entries: entries.clone() },
            )
            .with_details(entries)
        )
    }

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

        let year = self.config.actions.license.year.clone()
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
            .with_detail(format!("License type: {}", license_type))
        )
    }

    fn plan_contributing_creation(&self, results: &AuditResults) -> Option<Action> {
        let needs_contributing = results
            .findings_by_category("docs")
            .any(|f| f.rule_id == "DOC005");

        if !needs_contributing {
            return None;
        }

        Some(
            Action::new(
                "contributing-create",
                "file",
                "Create CONTRIBUTING.md",
                ActionOperation::CreateFile {
                    path: "CONTRIBUTING.md".to_string(),
                    template: "CONTRIBUTING.md".to_string(),
                    variables: HashMap::new(),
                },
            )
        )
    }

    fn plan_code_of_conduct_creation(&self, results: &AuditResults) -> Option<Action> {
        let needs_coc = results
            .findings_by_category("docs")
            .any(|f| f.rule_id == "DOC006");

        if !needs_coc {
            return None;
        }

        Some(
            Action::new(
                "coc-create",
                "file",
                "Create CODE_OF_CONDUCT.md",
                ActionOperation::CreateFile {
                    path: "CODE_OF_CONDUCT.md".to_string(),
                    template: "CODE_OF_CONDUCT.md".to_string(),
                    variables: HashMap::new(),
                },
            )
            .with_detail("Using Contributor Covenant template")
        )
    }

    fn plan_security_creation(&self, results: &AuditResults) -> Option<Action> {
        let needs_security = results
            .findings_by_category("docs")
            .any(|f| f.rule_id == "DOC007");

        if !needs_security {
            return None;
        }

        Some(
            Action::new(
                "security-create",
                "file",
                "Create SECURITY.md",
                ActionOperation::CreateFile {
                    path: "SECURITY.md".to_string(),
                    template: "SECURITY.md".to_string(),
                    variables: HashMap::new(),
                },
            )
        )
    }

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
