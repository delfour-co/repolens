//! Action plan structures

use serde::{Deserialize, Serialize};

/// A single action to be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action identifier
    id: String,
    /// Action category
    category: String,
    /// Human-readable description
    description: String,
    /// Additional details about what will be changed
    details: Vec<String>,
    /// The actual operation to perform
    operation: ActionOperation,
}

impl Action {
    /// Create a new action
    pub fn new(
        id: impl Into<String>,
        category: impl Into<String>,
        description: impl Into<String>,
        operation: ActionOperation,
    ) -> Self {
        Self {
            id: id.into(),
            category: category.into(),
            description: description.into(),
            details: Vec::new(),
            operation,
        }
    }

    /// Add a detail line
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    /// Add multiple details
    pub fn with_details(mut self, details: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.details.extend(details.into_iter().map(|d| d.into()));
        self
    }

    /// Get the action ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the category
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Get the description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the details
    pub fn details(&self) -> &[String] {
        &self.details
    }

    /// Get the operation
    pub fn operation(&self) -> &ActionOperation {
        &self.operation
    }
}

/// The type of operation to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionOperation {
    /// Update .gitignore file
    UpdateGitignore { entries: Vec<String> },

    /// Create a file from template
    CreateFile {
        path: String,
        template: String,
        variables: std::collections::HashMap<String, String>,
    },

    /// Configure protected branch (provider-agnostic)
    ConfigureProtectedBranch {
        branch: String,
        settings: BranchProtectionSettings,
    },

    /// Update repository settings (provider-agnostic)
    UpdateRepoSettings { settings: GitHubRepoSettings },

    /// Update repository metadata (description, topics, homepage)
    UpdateRepoMetadata {
        description: Option<String>,
        topics: Vec<String>,
        homepage: Option<String>,
    },

    /// Merge missing branch-protection sections into an EXISTING
    /// `.github/settings.yml` (review bug #12). This is a local file edit —
    /// not a forge call — so it applies regardless of which `RepoProvider` is
    /// configured. The executor parses the existing YAML and adds only the
    /// missing pieces, preserving any content the user already has.
    UpdateSettingsFile {
        /// Path to the settings file, relative to the repository root.
        path: String,
        /// Branch the protection block should target (falls back to the
        /// first existing entry if no entry named `branch` is found).
        branch: String,
        /// Required approving review count when adding
        /// `required_pull_request_reviews`.
        required_approvals: u32,
        /// Whether the `branches:` key itself is missing and must be added.
        ensure_branches_block: bool,
        /// Whether `required_pull_request_reviews` must be added (SEC009).
        ensure_pr_reviews: bool,
        /// Whether `required_status_checks` must be added (SEC010).
        ensure_status_checks: bool,
    },

    /// Update GitHub Actions & repository security settings: secret
    /// scanning, push protection, Actions permissions, default workflow
    /// permissions, and fork pull-request-workflow approval (review bug
    /// #11 -- SEC013-017). GitHub-only: `GitLabProvider`'s corresponding
    /// write methods all return `Err`, so the planner never plans this
    /// action for a non-GitHub provider.
    UpdateActionsSecuritySettings {
        settings: GitHubActionsSecuritySettings,
    },
}

/// Branch protection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchProtectionSettings {
    pub required_approvals: u32,
    pub require_status_checks: bool,
    pub require_conversation_resolution: bool,
    pub require_linear_history: bool,
    pub block_force_push: bool,
    pub block_deletions: bool,
    pub enforce_admins: bool,
    pub require_signed_commits: bool,
}

impl Default for BranchProtectionSettings {
    fn default() -> Self {
        Self {
            required_approvals: 1,
            require_status_checks: true,
            require_conversation_resolution: true,
            require_linear_history: true,
            block_force_push: true,
            block_deletions: true,
            enforce_admins: true,
            require_signed_commits: false,
        }
    }
}

/// GitHub repository settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubRepoSettings {
    pub enable_discussions: Option<bool>,
    pub enable_issues: Option<bool>,
    pub enable_wiki: Option<bool>,
    pub enable_vulnerability_alerts: Option<bool>,
    pub enable_automated_security_fixes: Option<bool>,
}

/// GitHub Actions & repository security settings (SEC013-017).
///
/// Each field is `Some(desired_value)` when that specific toggle needs to
/// change and `None` when it should be left untouched -- mirrors
/// [`GitHubRepoSettings`]'s "only touch what's needed" shape.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubActionsSecuritySettings {
    /// Enable/disable secret scanning (SEC013).
    pub secret_scanning: Option<bool>,
    /// Enable/disable push protection (SEC014).
    pub secret_scanning_push_protection: Option<bool>,
    /// Desired Actions permissions: `"all"`, `"local_only"`, or `"selected"` (SEC015).
    pub allowed_actions: Option<String>,
    /// Desired default `GITHUB_TOKEN` workflow permissions: `"read"` or `"write"` (SEC016).
    pub default_workflow_permissions: Option<String>,
    /// Whether fork pull request workflows must require approval (SEC017).
    pub require_fork_pr_approval: Option<bool>,
}

/// A collection of actions to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    actions: Vec<Action>,
}

impl ActionPlan {
    /// Create a new empty action plan
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Add an action to the plan
    pub fn add(&mut self, action: Action) {
        self.actions.push(action);
    }

    /// Get all actions
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Check if the plan is empty
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Get the number of actions.
    ///
    /// Part of the public API - provides the action count
    /// for reporting and display purposes.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Filter to only include specific action categories
    pub fn filter_only(&mut self, categories: &[String]) {
        self.actions
            .retain(|a| categories.contains(&a.category.to_string()));
    }

    /// Filter to skip specific action categories
    pub fn filter_skip(&mut self, categories: &[String]) {
        self.actions
            .retain(|a| !categories.contains(&a.category.to_string()));
    }
}

impl Default for ActionPlan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_new() {
        let action = Action::new(
            "action1",
            "files",
            "Test action",
            ActionOperation::UpdateGitignore {
                entries: vec!["*.log".to_string()],
            },
        );

        assert_eq!(action.id(), "action1");
        assert_eq!(action.category(), "files");
        assert_eq!(action.description(), "Test action");
        assert!(action.details().is_empty());
    }

    #[test]
    fn test_action_with_detail() {
        let action = Action::new(
            "action1",
            "files",
            "Test action",
            ActionOperation::UpdateGitignore {
                entries: vec!["*.log".to_string()],
            },
        )
        .with_detail("Detail 1");

        assert_eq!(action.details().len(), 1);
        assert_eq!(action.details()[0], "Detail 1");
    }

    #[test]
    fn test_action_with_details() {
        let action = Action::new(
            "action1",
            "files",
            "Test action",
            ActionOperation::UpdateGitignore {
                entries: vec!["*.log".to_string()],
            },
        )
        .with_details(vec!["Detail 1", "Detail 2", "Detail 3"]);

        assert_eq!(action.details().len(), 3);
    }

    #[test]
    fn test_action_operation() {
        let action = Action::new(
            "action1",
            "files",
            "Test action",
            ActionOperation::UpdateGitignore {
                entries: vec!["*.log".to_string()],
            },
        );

        match action.operation() {
            ActionOperation::UpdateGitignore { entries } => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0], "*.log");
            }
            _ => panic!("Expected UpdateGitignore operation"),
        }
    }

    #[test]
    fn test_action_plan_new() {
        let plan = ActionPlan::new();
        assert!(plan.is_empty());
        assert_eq!(plan.len(), 0);
    }

    #[test]
    fn test_action_plan_default() {
        let plan = ActionPlan::default();
        assert!(plan.is_empty());
    }

    #[test]
    fn test_action_plan_add() {
        let mut plan = ActionPlan::new();
        plan.add(Action::new(
            "action1",
            "files",
            "Test action",
            ActionOperation::UpdateGitignore {
                entries: vec!["*.log".to_string()],
            },
        ));

        assert!(!plan.is_empty());
        assert_eq!(plan.len(), 1);
        assert_eq!(plan.actions().len(), 1);
    }

    #[test]
    fn test_action_plan_filter_only() {
        let mut plan = ActionPlan::new();
        plan.add(Action::new(
            "action1",
            "files",
            "Files action",
            ActionOperation::UpdateGitignore {
                entries: vec!["*.log".to_string()],
            },
        ));
        plan.add(Action::new(
            "action2",
            "security",
            "Security action",
            ActionOperation::ConfigureProtectedBranch {
                branch: "main".to_string(),
                settings: BranchProtectionSettings::default(),
            },
        ));
        plan.add(Action::new(
            "action3",
            "files",
            "Another files action",
            ActionOperation::UpdateGitignore {
                entries: vec!["*.tmp".to_string()],
            },
        ));

        plan.filter_only(&["files".to_string()]);

        assert_eq!(plan.len(), 2);
        for action in plan.actions() {
            assert_eq!(action.category(), "files");
        }
    }

    #[test]
    fn test_action_plan_filter_skip() {
        let mut plan = ActionPlan::new();
        plan.add(Action::new(
            "action1",
            "files",
            "Files action",
            ActionOperation::UpdateGitignore {
                entries: vec!["*.log".to_string()],
            },
        ));
        plan.add(Action::new(
            "action2",
            "security",
            "Security action",
            ActionOperation::ConfigureProtectedBranch {
                branch: "main".to_string(),
                settings: BranchProtectionSettings::default(),
            },
        ));
        plan.add(Action::new(
            "action3",
            "docs",
            "Docs action",
            ActionOperation::CreateFile {
                path: "README.md".to_string(),
                template: "readme".to_string(),
                variables: std::collections::HashMap::new(),
            },
        ));

        plan.filter_skip(&["security".to_string()]);

        assert_eq!(plan.len(), 2);
        for action in plan.actions() {
            assert_ne!(action.category(), "security");
        }
    }

    #[test]
    fn test_branch_protection_settings_default() {
        let settings = BranchProtectionSettings::default();

        assert_eq!(settings.required_approvals, 1);
        assert!(settings.require_status_checks);
        assert!(settings.require_conversation_resolution);
        assert!(settings.require_linear_history);
        assert!(settings.block_force_push);
        assert!(settings.block_deletions);
        assert!(settings.enforce_admins);
        assert!(!settings.require_signed_commits);
    }

    #[test]
    fn test_github_repo_settings_default() {
        let settings = GitHubRepoSettings::default();

        assert!(settings.enable_discussions.is_none());
        assert!(settings.enable_issues.is_none());
        assert!(settings.enable_wiki.is_none());
        assert!(settings.enable_vulnerability_alerts.is_none());
        assert!(settings.enable_automated_security_fixes.is_none());
    }

    #[test]
    fn test_action_operation_create_file() {
        let mut variables = std::collections::HashMap::new();
        variables.insert("name".to_string(), "Test Project".to_string());

        let action = Action::new(
            "action1",
            "docs",
            "Create README",
            ActionOperation::CreateFile {
                path: "README.md".to_string(),
                template: "readme".to_string(),
                variables,
            },
        );

        match action.operation() {
            ActionOperation::CreateFile {
                path,
                template,
                variables,
            } => {
                assert_eq!(path, "README.md");
                assert_eq!(template, "readme");
                assert_eq!(variables.get("name"), Some(&"Test Project".to_string()));
            }
            _ => panic!("Expected CreateFile operation"),
        }
    }

    #[test]
    fn test_action_operation_update_repo_settings() {
        let settings = GitHubRepoSettings {
            enable_discussions: Some(true),
            enable_issues: Some(true),
            enable_wiki: Some(false),
            enable_vulnerability_alerts: Some(true),
            enable_automated_security_fixes: Some(true),
        };

        let action = Action::new(
            "action1",
            "security",
            "Update repository settings",
            ActionOperation::UpdateRepoSettings {
                settings: settings.clone(),
            },
        );

        match action.operation() {
            ActionOperation::UpdateRepoSettings { settings } => {
                assert_eq!(settings.enable_discussions, Some(true));
                assert_eq!(settings.enable_wiki, Some(false));
            }
            _ => panic!("Expected UpdateRepoSettings operation"),
        }
    }

    #[test]
    fn test_action_operation_update_settings_file() {
        let action = Action::new(
            "settings-file-update",
            "security",
            "Update .github/settings.yml",
            ActionOperation::UpdateSettingsFile {
                path: ".github/settings.yml".to_string(),
                branch: "main".to_string(),
                required_approvals: 1,
                ensure_branches_block: false,
                ensure_pr_reviews: true,
                ensure_status_checks: false,
            },
        );

        match action.operation() {
            ActionOperation::UpdateSettingsFile {
                path,
                branch,
                ensure_pr_reviews,
                ensure_status_checks,
                ..
            } => {
                assert_eq!(path, ".github/settings.yml");
                assert_eq!(branch, "main");
                assert!(ensure_pr_reviews);
                assert!(!ensure_status_checks);
            }
            _ => panic!("Expected UpdateSettingsFile operation"),
        }
    }

    #[test]
    fn test_github_actions_security_settings_default() {
        let settings = GitHubActionsSecuritySettings::default();
        assert!(settings.secret_scanning.is_none());
        assert!(settings.secret_scanning_push_protection.is_none());
        assert!(settings.allowed_actions.is_none());
        assert!(settings.default_workflow_permissions.is_none());
        assert!(settings.require_fork_pr_approval.is_none());
    }

    #[test]
    fn test_action_operation_update_actions_security_settings() {
        let settings = GitHubActionsSecuritySettings {
            secret_scanning: Some(true),
            secret_scanning_push_protection: Some(true),
            allowed_actions: Some("selected".to_string()),
            default_workflow_permissions: Some("read".to_string()),
            require_fork_pr_approval: Some(true),
        };

        let action = Action::new(
            "actions-security-settings",
            "github",
            "Update GitHub Actions & security settings",
            ActionOperation::UpdateActionsSecuritySettings {
                settings: settings.clone(),
            },
        );

        match action.operation() {
            ActionOperation::UpdateActionsSecuritySettings { settings } => {
                assert_eq!(settings.secret_scanning, Some(true));
                assert_eq!(settings.allowed_actions.as_deref(), Some("selected"));
                assert_eq!(
                    settings.default_workflow_permissions.as_deref(),
                    Some("read")
                );
                assert_eq!(settings.require_fork_pr_approval, Some(true));
            }
            _ => panic!("Expected UpdateActionsSecuritySettings operation"),
        }
    }

    #[test]
    fn test_action_operation_update_repo_metadata() {
        let action = Action::new(
            "action1",
            "metadata",
            "Update repository metadata",
            ActionOperation::UpdateRepoMetadata {
                description: Some("A test repo".to_string()),
                topics: vec!["rust".to_string(), "cli".to_string()],
                homepage: Some("https://example.com".to_string()),
            },
        );

        match action.operation() {
            ActionOperation::UpdateRepoMetadata {
                description,
                topics,
                homepage,
            } => {
                assert_eq!(description.as_deref(), Some("A test repo"));
                assert_eq!(topics.len(), 2);
                assert_eq!(homepage.as_deref(), Some("https://example.com"));
            }
            _ => panic!("Expected UpdateRepoMetadata operation"),
        }
    }
}
