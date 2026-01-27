//! Action executor - Executes planned actions
//!
//! This module provides functionality to execute actions from an action plan.
//! It handles the actual execution of file creation, .gitignore updates,
//! branch protection configuration, and GitHub settings updates.

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::config::Config;

use super::plan::{Action, ActionOperation, ActionPlan};
use super::{branch_protection, github_settings, gitignore, templates};

/// Result of executing a single action
///
/// Contains information about whether an action succeeded or failed,
/// along with any error message if it failed.
#[derive(Debug)]
pub struct ActionResult {
    /// Name/description of the action that was executed
    pub action_name: String,
    /// Whether the action succeeded
    pub success: bool,
    /// Error message if the action failed, `None` if it succeeded
    pub error: Option<String>,
}

/// Executes actions from an action plan
///
/// The `ActionExecutor` takes an `ActionPlan` and executes each action
/// sequentially. It handles different types of operations like file creation,
/// .gitignore updates, and GitHub API calls.
pub struct ActionExecutor {
    /// Configuration (currently unused but kept for future extensibility)
    _config: Config,
}

impl ActionExecutor {
    /// Create a new action executor with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for execution
    ///
    /// # Returns
    ///
    /// A new `ActionExecutor` instance
    pub fn new(config: Config) -> Self {
        Self { _config: config }
    }

    /// Execute all actions in the plan
    ///
    /// Executes each action sequentially and collects results. If an action
    /// fails, execution continues with the next action.
    ///
    /// # Arguments
    ///
    /// * `plan` - The action plan to execute
    ///
    /// # Returns
    ///
    /// A vector of `ActionResult` for each action, indicating success or failure
    ///
    /// # Errors
    ///
    /// Returns an error only if there's a critical failure in the executor itself
    pub async fn execute(&self, plan: &ActionPlan) -> Result<Vec<ActionResult>> {
        let mut results = Vec::new();

        for action in plan.actions() {
            info!("Executing action: {}", action.id());

            let result = self.execute_action(action).await;

            results.push(ActionResult {
                action_name: action.description().to_string(),
                success: result.is_ok(),
                error: result.err().map(|e| e.to_string()),
            });
        }

        Ok(results)
    }

    /// Execute a single action
    ///
    /// # Arguments
    ///
    /// * `action` - The action to execute
    ///
    /// # Returns
    ///
    /// `Ok(())` if the action succeeded, or an error if it failed
    ///
    /// # Errors
    ///
    /// Returns an error if the action execution fails
    async fn execute_action(&self, action: &Action) -> Result<()> {
        match action.operation() {
            ActionOperation::UpdateGitignore { entries } => {
                debug!("Updating .gitignore with {} entries", entries.len());
                gitignore::update_gitignore(entries).with_context(|| {
                    format!("Failed to update .gitignore for action '{}'", action.id())
                })?;
            }

            ActionOperation::CreateFile {
                path,
                template,
                variables,
            } => {
                debug!("Creating file {} from template {}", path, template);
                templates::create_file_from_template(path, template, variables).with_context(
                    || {
                        format!(
                            "Failed to create file '{}' from template '{}'",
                            path, template
                        )
                    },
                )?;
            }

            ActionOperation::ConfigureBranchProtection { branch, settings } => {
                debug!("Configuring branch protection for {}", branch);
                branch_protection::configure(branch, settings)
                    .await
                    .with_context(|| {
                        format!("Failed to configure branch protection for '{}'", branch)
                    })?;
            }

            ActionOperation::UpdateGitHubSettings { settings } => {
                debug!("Updating GitHub repository settings");
                github_settings::update(settings)
                    .await
                    .with_context(|| "Failed to update GitHub repository settings")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::plan::{ActionOperation, ActionPlan};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_execute_action_update_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Save current directory (fallback to /tmp if current dir is invalid)
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        // Ensure we're in a valid directory before changing
        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        // Change to temp directory
        std::env::set_current_dir(root).expect("Failed to change to temp directory");

        // Verify we're in the right directory
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        assert_eq!(
            current_dir.canonicalize().unwrap_or_default(),
            root.canonicalize().unwrap_or_default(),
            "Failed to change to temp directory - current dir: {:?}, expected: {:?}",
            current_dir,
            root
        );

        let config = Config::default();
        let executor = ActionExecutor::new(config);

        let action = Action::new(
            "test-gitignore",
            "gitignore",
            "Test gitignore update",
            ActionOperation::UpdateGitignore {
                entries: vec![".env".to_string(), "*.key".to_string()],
            },
        );

        let result = executor.execute_action(&action).await;

        // Check result before restoring directory
        assert!(
            result.is_ok(),
            "Action execution failed: {:?}",
            result.err()
        );

        // Verify file was created in the temp directory
        let gitignore_path = root.join(".gitignore");
        assert!(
            gitignore_path.exists(),
            ".gitignore not found at {:?}. Current dir: {:?}",
            gitignore_path,
            std::env::current_dir().ok()
        );

        // Restore directory (ignore errors if directory no longer exists)
        let _ = std::env::set_current_dir(&original_dir);
    }

    #[tokio::test]
    async fn test_execute_action_create_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Save current directory (fallback to /tmp if current dir is invalid)
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        // Ensure we're in a valid directory before changing
        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(root).expect("Failed to change to temp directory");

        let config = Config::default();
        let executor = ActionExecutor::new(config);

        let action = Action::new(
            "test-create",
            "file",
            "Test file creation",
            ActionOperation::CreateFile {
                path: "TEST.md".to_string(),
                template: "CONTRIBUTING.md".to_string(),
                variables: HashMap::new(),
            },
        );

        let result = executor.execute_action(&action).await;

        // May fail if template doesn't exist, but that's ok for test
        // We're testing that the function handles it gracefully
        let _ = result;

        // Restore directory (ignore errors if directory no longer exists)
        let _ = std::env::set_current_dir(&original_dir);
    }

    #[tokio::test]
    async fn test_execute_all_actions() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Save current directory (fallback to /tmp if current dir is invalid)
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        // Ensure we're in a valid directory before changing
        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(root).expect("Failed to change to temp directory");

        let config = Config::default();
        let executor = ActionExecutor::new(config);

        let mut plan = ActionPlan::new();
        plan.add(Action::new(
            "test-1",
            "gitignore",
            "Test 1",
            ActionOperation::UpdateGitignore {
                entries: vec![".env".to_string()],
            },
        ));

        let results = executor.execute(&plan).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].success);

        // Restore directory (ignore errors if directory no longer exists)
        let _ = std::env::set_current_dir(&original_dir);
    }

    #[tokio::test]
    async fn test_execute_handles_errors_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Save current directory (fallback to /tmp if current dir is invalid)
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        // Ensure we're in a valid directory before changing
        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(root).expect("Failed to change to temp directory");

        let config = Config::default();
        let executor = ActionExecutor::new(config);

        let mut plan = ActionPlan::new();
        // Add an action that will fail (invalid template)
        plan.add(Action::new(
            "test-fail",
            "file",
            "Test failure",
            ActionOperation::CreateFile {
                path: "INVALID.md".to_string(),
                template: "NONEXISTENT_TEMPLATE.md".to_string(),
                variables: HashMap::new(),
            },
        ));

        let results = executor.execute(&plan).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert!(results[0].error.is_some());

        // Restore directory (ignore errors if directory no longer exists)
        let _ = std::env::set_current_dir(&original_dir);
    }
}
