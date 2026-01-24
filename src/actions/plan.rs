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
    UpdateGitignore {
        entries: Vec<String>,
    },

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
    UpdateGitHubSettings {
        settings: GitHubRepoSettings,
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

    /// Get the number of actions
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Filter to only include specific action categories
    pub fn filter_only(&mut self, categories: &[String]) {
        self.actions.retain(|a| categories.contains(&a.category.to_string()));
    }

    /// Filter to skip specific action categories
    pub fn filter_skip(&mut self, categories: &[String]) {
        self.actions.retain(|a| !categories.contains(&a.category.to_string()));
    }
}

impl Default for ActionPlan {
    fn default() -> Self {
        Self::new()
    }
}
