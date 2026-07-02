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

    /// Configure branch protection
    ConfigureBranchProtection {
        branch: String,
        settings: BranchProtectionSettings,
    },

    /// Update GitHub repository settings
    UpdateGitHubSettings { settings: GitHubRepoSettings },
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
            ActionOperation::ConfigureBranchProtection {
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
            ActionOperation::ConfigureBranchProtection {
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
    fn test_action_operation_update_github_settings() {
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
            "Update GitHub settings",
            ActionOperation::UpdateGitHubSettings {
                settings: settings.clone(),
            },
        );

        match action.operation() {
            ActionOperation::UpdateGitHubSettings { settings } => {
                assert_eq!(settings.enable_discussions, Some(true));
                assert_eq!(settings.enable_wiki, Some(false));
            }
            _ => panic!("Expected UpdateGitHubSettings operation"),
        }
    }
}
