//! Action executor - Executes planned actions
//!
//! This module provides functionality to execute actions from an action plan.
//! It handles the actual execution of file creation, .gitignore updates,
//! branch protection configuration, and GitHub settings updates.

use crate::error::RepoLensError;
use tracing::{debug, info};

use crate::config::Config;

use super::plan::{Action, ActionOperation, ActionPlan};
use super::settings_file::SettingsFileUpdate;
use super::{
    actions_security, branch_protection, github_settings, gitignore, metadata, settings_file,
    templates,
};

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
    /// Configuration, used to build the provider for write actions.
    config: Config,
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
        Self { config }
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
    pub async fn execute(&self, plan: &ActionPlan) -> Result<Vec<ActionResult>, RepoLensError> {
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
    async fn execute_action(&self, action: &Action) -> Result<(), RepoLensError> {
        match action.operation() {
            ActionOperation::UpdateGitignore { entries } => {
                debug!("Updating .gitignore with {} entries", entries.len());
                // Get current directory at the start to avoid race conditions in parallel tests
                let current_dir = std::env::current_dir().map_err(|e| {
                    RepoLensError::Action(crate::error::ActionError::ExecutionFailed {
                        message: format!("Failed to get current directory: {}", e),
                    })
                })?;
                gitignore::update_gitignore_at(&current_dir, entries)?;
            }

            ActionOperation::CreateFile {
                path,
                template,
                variables,
            } => {
                debug!("Creating file {} from template {}", path, template);
                templates::create_file_from_template(path, template, variables)?;
            }

            ActionOperation::ConfigureProtectedBranch { branch, settings } => {
                debug!("Configuring branch protection for {}", branch);
                branch_protection::configure(&self.config, branch, settings).await?;
            }

            ActionOperation::UpdateRepoSettings { settings } => {
                debug!("Updating repository settings");
                github_settings::update(&self.config, settings).await?;
            }

            ActionOperation::UpdateRepoMetadata {
                description,
                topics,
                homepage,
            } => {
                debug!("Updating repository metadata");
                metadata::update(
                    &self.config,
                    description.as_deref(),
                    topics,
                    homepage.as_deref(),
                )
                .await?;
            }

            ActionOperation::UpdateSettingsFile {
                path,
                branch,
                required_approvals,
                ensure_branches_block,
                ensure_pr_reviews,
                ensure_status_checks,
            } => {
                debug!("Merging branch-protection sections into {}", path);
                let current_dir = std::env::current_dir().map_err(|e| {
                    RepoLensError::Action(crate::error::ActionError::ExecutionFailed {
                        message: format!("Failed to get current directory: {}", e),
                    })
                })?;
                let update = SettingsFileUpdate {
                    branch: branch.clone(),
                    required_approvals: *required_approvals,
                    ensure_branches_block: *ensure_branches_block,
                    ensure_pr_reviews: *ensure_pr_reviews,
                    ensure_status_checks: *ensure_status_checks,
                };
                settings_file::update_settings_file_at(&current_dir, path, &update)?;
            }

            ActionOperation::UpdateActionsSecuritySettings { settings } => {
                debug!("Updating GitHub Actions & security settings");
                actions_security::update(&self.config, settings).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::plan::{Action, ActionOperation, ActionPlan};
    use serial_test::serial;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    #[serial]
    #[cfg_attr(tarpaulin, ignore)]
    async fn test_execute_action_update_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        // Save current directory (fallback to /tmp if current dir is invalid)
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        // Ensure we're in a valid directory before changing
        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        // Change to temp directory
        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

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

        // Execute action - it will get current_dir at start
        let result = executor.execute_action(&action).await;

        // Restore directory immediately after execution
        let _ = std::env::set_current_dir(&original_dir);

        // Check result after restoring directory
        assert!(
            result.is_ok(),
            "Action execution failed: {:?}",
            result.err()
        );

        // Verify file was created in the temp directory using absolute path
        // This works regardless of what the current directory is
        let gitignore_path = root_abs.join(".gitignore");
        assert!(
            gitignore_path.exists(),
            ".gitignore not found at {:?}. Root was: {:?}. Current dir: {:?}",
            gitignore_path,
            root_abs,
            std::env::current_dir()
        );
    }

    #[tokio::test]
    #[serial]
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
    #[serial]
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
    #[serial]
    async fn test_execute_dispatches_update_settings_file() {
        // Bug #12 regression: executing an UpdateSettingsFile action must
        // actually merge the missing sections into an EXISTING
        // .github/settings.yml on disk (a local file edit, no provider
        // needed -- unlike branch-protection/github-settings actions).
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));
        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }
        std::env::set_current_dir(root).expect("Failed to change to temp directory");

        std::fs::create_dir_all(root.join(".github")).unwrap();
        std::fs::write(
            root.join(".github/settings.yml"),
            "repository:\n  name: my-repo\n",
        )
        .unwrap();

        let config = Config::default();
        let executor = ActionExecutor::new(config);

        let mut plan = ActionPlan::new();
        plan.add(Action::new(
            "settings-file-update",
            "security",
            "Update .github/settings.yml",
            ActionOperation::UpdateSettingsFile {
                path: ".github/settings.yml".to_string(),
                branch: "main".to_string(),
                required_approvals: 1,
                ensure_branches_block: true,
                ensure_pr_reviews: true,
                ensure_status_checks: true,
            },
        ));

        let results = executor.execute(&plan).await.unwrap();

        let merged = std::fs::read_to_string(root.join(".github/settings.yml")).unwrap();

        // Restore directory before asserting so a failed assertion doesn't
        // leave the process in the (about to be dropped) temp directory.
        let _ = std::env::set_current_dir(&original_dir);

        assert_eq!(results.len(), 1);
        assert!(results[0].success, "{:?}", results[0].error);
        assert!(merged.contains("my-repo"), "original content must survive");
        assert!(merged.contains("branches"));
        assert!(merged.contains("required_pull_request_reviews"));
        assert!(merged.contains("required_status_checks"));
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_dispatches_update_repo_metadata() {
        // Dispatch the metadata operation through the executor and assert it
        // surfaces a (failed) result rather than panicking.
        //
        // Deliberately configured for GitLab (not `Config::default()`): this
        // sandbox has an authenticated `gh` CLI bound to a real GitHub repo,
        // and `Config::default()` would route through the LIVE GitHub provider
        // -- actually running `gh repo edit` and overwriting that real repo's
        // description/topics. `glab` is not installed here, so `for_config`
        // deterministically returns `None` and the call fails fast with no
        // network I/O, while still exercising the executor's dispatch arm.
        let config = Config {
            provider: Some(crate::providers::Provider::GitLab),
            ..Config::default()
        };
        let executor = ActionExecutor::new(config);

        let mut plan = ActionPlan::new();
        plan.add(Action::new(
            "repo-metadata",
            "metadata",
            "Update repository metadata",
            ActionOperation::UpdateRepoMetadata {
                description: Some("desc".to_string()),
                topics: vec!["rust".to_string()],
                homepage: None,
            },
        ));

        let results = executor.execute(&plan).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action_name, "Update repository metadata");
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_dispatches_update_actions_security_settings() {
        // Regression test for review bug #11 (SEC013-017): the executor must
        // dispatch UpdateActionsSecuritySettings to actions_security::update
        // rather than silently doing nothing.
        //
        // Deliberately configured for GitLab (not `Config::default()`):
        // this sandbox has an authenticated `gh` CLI bound to a real GitHub
        // repository, and `Config::default()` would route through the LIVE
        // GitHub provider -- actually restricting Actions permissions /
        // enabling secret scanning on that real repository. `glab` is not
        // installed here, so `for_config` deterministically returns `None`
        // and the call fails fast with no network I/O at all, while still
        // exercising the executor's dispatch arm end-to-end.
        let config = Config {
            provider: Some(crate::providers::Provider::GitLab),
            ..Config::default()
        };
        let executor = ActionExecutor::new(config);

        let mut plan = ActionPlan::new();
        plan.add(Action::new(
            "actions-security-settings",
            "github",
            "Update GitHub Actions & security settings",
            ActionOperation::UpdateActionsSecuritySettings {
                settings: crate::actions::plan::GitHubActionsSecuritySettings {
                    secret_scanning: Some(true),
                    secret_scanning_push_protection: Some(true),
                    allowed_actions: Some("selected".to_string()),
                    default_workflow_permissions: Some("read".to_string()),
                    require_fork_pr_approval: Some(true),
                },
            },
        ));

        let results = executor.execute(&plan).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].action_name,
            "Update GitHub Actions & security settings"
        );
        assert!(
            !results[0].success,
            "no glab CLI is installed in this sandbox, so this must fail fast, not panic"
        );
    }

    #[tokio::test]
    #[serial]
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
