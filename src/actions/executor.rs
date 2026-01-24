//! Action executor - Executes planned actions

use anyhow::Result;
use tracing::{debug, info, warn};

use crate::config::Config;

use super::plan::{Action, ActionOperation, ActionPlan};
use super::{gitignore, templates, branch_protection, github_settings};

/// Result of executing a single action
#[derive(Debug)]
pub struct ActionResult {
    pub action_name: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Executes actions from an action plan
pub struct ActionExecutor {
    config: Config,
}

impl ActionExecutor {
    /// Create a new action executor
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Execute all actions in the plan
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
    async fn execute_action(&self, action: &Action) -> Result<()> {
        match action.operation() {
            ActionOperation::UpdateGitignore { entries } => {
                debug!("Updating .gitignore with {} entries", entries.len());
                gitignore::update_gitignore(entries)?;
            }

            ActionOperation::CreateFile { path, template, variables } => {
                debug!("Creating file {} from template {}", path, template);
                templates::create_file_from_template(path, template, variables)?;
            }

            ActionOperation::ConfigureBranchProtection { branch, settings } => {
                debug!("Configuring branch protection for {}", branch);
                branch_protection::configure(branch, settings).await?;
            }

            ActionOperation::UpdateGitHubSettings { settings } => {
                debug!("Updating GitHub repository settings");
                github_settings::update(settings).await?;
            }
        }

        Ok(())
    }
}
