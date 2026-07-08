//! Action planner - Creates action plans based on audit results
//!
//! This module provides functionality to generate action plans from audit results.
//! It analyzes findings and creates appropriate actions to fix issues.

use std::collections::HashMap;

use crate::config::{ActionsSecurityConfig, Config, GitHubSettingsConfig};
use crate::providers::{ActionsPermissions, Provider, SecretScanningSettings};
use crate::rules::results::AuditResults;

use super::plan::{
    Action, ActionOperation, ActionPlan, BranchProtectionSettings, GitHubActionsSecuritySettings,
    GitHubRepoSettings,
};

/// Parameters for planning file creation
struct FileCreationParams<'a> {
    rule_id: &'a str,
    category: &'a str,
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
    pub async fn create_plan(
        &self,
        results: &AuditResults,
    ) -> Result<ActionPlan, crate::error::RepoLensError> {
        let mut plan = ActionPlan::new();

        // Plan gitignore updates
        if self.config.actions.gitignore {
            // FILE002 (.gitignore entirely absent) and FILE003 (missing
            // recommended entries) are mutually exclusive per audit run --
            // check_gitignore returns early after FILE002 -- but both plan
            // paths are kept independent for clarity and defensiveness.
            if let Some(action) = self.plan_gitignore_creation(results) {
                plan.add(action);
            }
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

        // Plan README.md creation
        if self.config.actions.readme {
            if let Some(action) = self.plan_readme_creation(results) {
                plan.add(action);
            }
        }

        // Plan CHANGELOG.md creation
        if self.config.actions.changelog {
            if let Some(action) = self.plan_changelog_creation(results) {
                plan.add(action);
            }
        }

        // Plan .gitattributes creation
        if self.config.actions.gitattributes {
            if let Some(action) = self.plan_gitattributes_creation(results) {
                plan.add(action);
            }
        }

        // Plan CODEOWNERS creation
        if self.config.actions.codeowners {
            if let Some(action) = self.plan_codeowners_creation(results) {
                plan.add(action);
            }
        }

        // Plan .github/settings.yml creation
        if self.config.actions.settings_file {
            if let Some(action) = self.plan_settings_file_creation(results) {
                plan.add(action);
            }
        }

        // Plan .github/settings.yml UPDATES for a file that already exists
        // but is missing branch-protection sections (SEC008/009/010 -- see
        // review bug #12). Distinct from the CREATE path above (SEC007).
        if self.config.actions.settings_file {
            if let Some(action) = self.plan_settings_file_update_if_needed(results) {
                plan.add(action);
            }
        }

        // Plan branch protection (only if not already configured)
        if self.config.actions.branch_protection.enabled {
            if let Some(action) = self.plan_branch_protection_if_needed().await? {
                plan.add(action);
            }
        }

        // Plan GitHub settings (only if not already configured)
        if let Some(action) = self.plan_github_settings_if_needed().await? {
            plan.add(action);
        }

        // Plan GitHub Actions & security settings -- secret scanning, push
        // protection, Actions permissions, workflow permissions, fork-PR
        // approval (review bug #11: SEC013-017 genuine remediation).
        if self.config.actions.actions_security.enabled {
            if let Some(action) = self.plan_actions_security_if_needed().await? {
                plan.add(action);
            }
        }

        // Plan repository metadata (only if configured and a finding exists)
        if self.config.actions.metadata.enabled {
            if let Some(action) = self.plan_metadata_if_needed(results) {
                plan.add(action);
            }
        }

        Ok(plan)
    }

    /// Plan .gitignore creation when the file is entirely absent (FILE002).
    ///
    /// Review bug #13: `plan_gitignore_update` only reads FILE003 findings
    /// (missing recommended entries in an EXISTING .gitignore), so a repo
    /// with NO .gitignore at all (FILE002 -- `check_gitignore` returns early
    /// after emitting it, so no FILE003 findings accompany it) previously got
    /// no remediation whatsoever. This creates a baseline .gitignore from a
    /// generic template so `apply` has something to act on.
    ///
    /// # Arguments
    ///
    /// * `results` - The audit results
    ///
    /// # Returns
    ///
    /// An `Action` to create .gitignore, or `None` if it already exists
    fn plan_gitignore_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "FILE002",
                category: "files",
                file_path: ".gitignore",
                template: ".gitignore",
                action_id: "gitignore-create",
                action_description: "Create .gitignore",
                detail: None,
            },
        )
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
            .findings_by_category(params.category)
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
                category: "docs",
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
                category: "docs",
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
                category: "docs",
                file_path: "SECURITY.md",
                template: "SECURITY.md",
                action_id: "security-create",
                action_description: "Create SECURITY.md",
                detail: None,
            },
        )
    }

    fn plan_readme_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "DOC001",
                category: "docs",
                file_path: "README.md",
                template: "README.md",
                action_id: "readme-create",
                action_description: "Create README.md",
                detail: None,
            },
        )
    }

    fn plan_changelog_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "DOC008",
                category: "docs",
                file_path: "CHANGELOG.md",
                template: "CHANGELOG.md",
                action_id: "changelog-create",
                action_description: "Create CHANGELOG.md",
                detail: Some("Using Keep a Changelog format"),
            },
        )
    }

    fn plan_gitattributes_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "GIT002",
                category: "git",
                file_path: ".gitattributes",
                template: ".gitattributes",
                action_id: "gitattributes-create",
                action_description: "Create .gitattributes",
                detail: None,
            },
        )
    }

    fn plan_codeowners_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "CODE001",
                category: "codeowners",
                file_path: "CODEOWNERS",
                template: "CODEOWNERS",
                action_id: "codeowners-create",
                action_description: "Create CODEOWNERS",
                detail: None,
            },
        )
    }

    fn plan_settings_file_creation(&self, results: &AuditResults) -> Option<Action> {
        self.plan_file_creation(
            results,
            FileCreationParams {
                rule_id: "SEC007",
                category: "security",
                file_path: ".github/settings.yml",
                template: ".github/settings.yml",
                action_id: "settings-file-create",
                action_description: "Create .github/settings.yml",
                detail: None,
            },
        )
    }

    /// Plan a real remediation for SEC008/009/010 on an EXISTING
    /// `.github/settings.yml` (review bug #12).
    ///
    /// `plan_settings_file_creation` (SEC007) only fires when the file is
    /// entirely absent; before this fix, a file that already existed but was
    /// missing branch-protection sections had NO remediation path at all,
    /// even though the zero-report-only contract test claimed SEC008-010
    /// were remediable via `ConfigureProtectedBranch` (which actually
    /// configures live branch protection via the provider API, not the
    /// `.github/settings.yml` content the rule itself parses). This plans an
    /// `UpdateSettingsFile` action that merges in only the missing sections.
    ///
    /// # Arguments
    ///
    /// * `results` - The audit results
    ///
    /// # Returns
    ///
    /// An `Action` to update `.github/settings.yml`, or `None` if none of
    /// SEC008/009/010 fired.
    fn plan_settings_file_update_if_needed(&self, results: &AuditResults) -> Option<Action> {
        let mut has_sec008 = false;
        let mut has_sec009 = false;
        let mut has_sec010 = false;

        for finding in results.findings_by_category("security") {
            match finding.rule_id.as_str() {
                "SEC008" => has_sec008 = true,
                "SEC009" => has_sec009 = true,
                "SEC010" => has_sec010 = true,
                _ => {}
            }
        }

        if !has_sec008 && !has_sec009 && !has_sec010 {
            return None;
        }

        let bp = &self.config.actions.branch_protection;

        let mut details = Vec::new();
        if has_sec008 {
            details.push("Add 'branches:' section to .github/settings.yml".to_string());
        }
        if has_sec009 || has_sec008 {
            details.push("Add required_pull_request_reviews".to_string());
        }
        if has_sec010 || has_sec008 {
            details.push("Add required_status_checks".to_string());
        }

        Some(
            Action::new(
                "settings-file-update",
                "security",
                "Update .github/settings.yml with missing branch-protection sections",
                ActionOperation::UpdateSettingsFile {
                    path: ".github/settings.yml".to_string(),
                    branch: bp.branch.clone(),
                    required_approvals: bp.required_approvals,
                    ensure_branches_block: has_sec008,
                    ensure_pr_reviews: has_sec009,
                    ensure_status_checks: has_sec010,
                },
            )
            .with_details(details),
        )
    }

    /// Plan branch protection configuration if needed
    ///
    /// Checks the current branch protection status and creates an action only if
    /// the current settings don't match the desired configuration.
    ///
    /// # Returns
    ///
    /// An `Action` to configure branch protection, or `None` if already configured correctly
    async fn plan_branch_protection_if_needed(
        &self,
    ) -> Result<Option<Action>, crate::error::RepoLensError> {
        let bp = &self.config.actions.branch_protection;

        // Try to get current branch protection status
        let provider = match crate::providers::for_config(&self.config) {
            Some(p) => p,
            None => {
                // If no provider is available, still plan the action
                // (it will fail gracefully during apply)
                return Ok(Some(self.create_branch_protection_action()));
            }
        };

        let current_protection = match provider.get_branch_protection(&bp.branch) {
            Ok(Some(protection)) => protection,
            Ok(None) => {
                // No protection exists, plan the action
                return Ok(Some(self.create_branch_protection_action()));
            }
            Err(_) => {
                // Error fetching protection, plan the action to be safe
                return Ok(Some(self.create_branch_protection_action()));
            }
        };

        // Check if current protection matches desired settings
        let needs_update = {
            // Check required approvals
            let current_approvals = current_protection
                .required_pull_request_reviews
                .as_ref()
                .map(|r| r.required_approving_review_count)
                .unwrap_or(0);
            let needs_approvals = current_approvals != bp.required_approvals;

            // Check status checks
            let has_status_checks = current_protection.required_status_checks.is_some();
            let needs_status_checks = has_status_checks != bp.require_status_checks;

            // Check force push blocking
            // If block_force_push is true, we need allow_force_pushes.enabled to be false
            // If block_force_push is false, we need allow_force_pushes.enabled to be true
            let allows_force_push = current_protection
                .allow_force_pushes
                .as_ref()
                .map(|a| a.enabled)
                .unwrap_or(true);
            // We need to update if: (block_force_push && allows_force_push) || (!block_force_push && !allows_force_push)
            // Which simplifies to: allows_force_push == block_force_push
            let needs_force_push_block = allows_force_push == bp.block_force_push;

            needs_approvals || needs_status_checks || needs_force_push_block
        };

        if needs_update {
            Ok(Some(self.create_branch_protection_action()))
        } else {
            Ok(None)
        }
    }

    /// Create a branch protection action
    fn create_branch_protection_action(&self) -> Action {
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
            ActionOperation::ConfigureProtectedBranch {
                branch: bp.branch.clone(),
                settings,
            },
        )
        .with_details(details)
    }

    /// The effective provider for this run: an explicit `provider` in config
    /// (`.repolens.toml`'s `provider` key / `--provider` CLI flag) always
    /// wins; otherwise fall back to auto-detecting from the git remote, and
    /// finally to [`Provider::GitHub`]. Mirrors `providers::for_config`'s own
    /// resolution so the two never disagree about which provider is active.
    fn effective_provider(&self) -> Provider {
        self.config
            .provider
            .unwrap_or_else(|| crate::providers::detect_provider_from_remote().unwrap_or_default())
    }

    /// Whether the effective provider supports the GitHub-only repository
    /// settings toggles (discussions / vulnerability alerts / automated
    /// security fixes).
    ///
    /// Review bug #9 coordination: `GitLabProvider::set_repo_settings`
    /// returns `Err` when any of these three fields are requested (GitLab has
    /// no equivalent), so planning them for a GitLab repository would create
    /// an action that can never succeed. `enable_issues`/`enable_wiki` ARE
    /// settable on GitLab and are not gated by this.
    fn supports_github_only_settings(&self) -> bool {
        self.effective_provider() == Provider::GitHub
    }

    /// Pure computation of which repository-settings fields need to change,
    /// given already-fetched provider state.
    ///
    /// Extracted out of `plan_github_settings_if_needed` so the two review-bug
    /// fixes it encodes -- #14 (fail-safe error fallback) and #9 (don't plan
    /// GitHub-only toggles for GitLab) -- can be pinned with a plain unit test
    /// that doesn't require a live, authenticated provider.
    ///
    /// Returns `(needs_discussions, needs_issues, needs_wiki,
    /// needs_vuln_alerts, needs_auto_fixes)`.
    fn compute_settings_needs(
        gs: &GitHubSettingsConfig,
        current_issues_enabled: bool,
        current_wiki_enabled: bool,
        current_discussions_enabled: bool,
        vuln_alerts: Result<bool, crate::error::RepoLensError>,
        auto_fixes: Result<bool, crate::error::RepoLensError>,
        github_only_toggles_supported: bool,
    ) -> (bool, bool, bool, bool, bool) {
        let needs_vuln_alerts = github_only_toggles_supported && {
            let current = vuln_alerts.unwrap_or_else(|e| {
                tracing::debug!("Could not check vulnerability alerts status: {:?}", e);
                // Bug #14: fail SAFE. An error means we don't know the
                // current state; assuming it's already enabled makes
                // `needs = current != desired` silently `false` when desired
                // is `true`, under-protecting the repo. Assume NOT protected
                // instead, so the fix gets planned.
                false
            });
            current != gs.vulnerability_alerts
        };

        let needs_auto_fixes = github_only_toggles_supported && {
            let current = auto_fixes.unwrap_or_else(|e| {
                tracing::debug!("Could not check automated security fixes status: {:?}", e);
                false
            });
            current != gs.automated_security_fixes
        };

        let needs_discussions =
            github_only_toggles_supported && current_discussions_enabled != gs.discussions;

        // Issues/wiki ARE settable on every provider -- never gated.
        let needs_issues = current_issues_enabled != gs.issues;
        let needs_wiki = current_wiki_enabled != gs.wiki;

        (
            needs_discussions,
            needs_issues,
            needs_wiki,
            needs_vuln_alerts,
            needs_auto_fixes,
        )
    }

    /// Plan GitHub repository settings updates if needed
    ///
    /// Checks the current repository settings and creates an action only if
    /// the current settings don't match the desired configuration.
    ///
    /// # Returns
    ///
    /// An `Action` to update GitHub settings, or `None` if already configured correctly
    async fn plan_github_settings_if_needed(
        &self,
    ) -> Result<Option<Action>, crate::error::RepoLensError> {
        let gs = &self.config.actions.github_settings;
        let github_only_toggles_supported = self.supports_github_only_settings();

        // Try to get current repository settings
        let provider = match crate::providers::for_config(&self.config) {
            Some(p) => p,
            None => {
                // If no provider is available, still plan the action (it
                // will fail gracefully during apply), but never for
                // GitHub-only toggles when the effective provider is GitLab
                // (review bug #9).
                return Ok(Some(self.create_github_settings_action_filtered(
                    github_only_toggles_supported,
                    true,
                    true,
                    github_only_toggles_supported,
                    github_only_toggles_supported,
                )));
            }
        };

        let current_settings = match provider.get_repo_settings() {
            Ok(settings) => settings,
            Err(_) => {
                // Error fetching settings, plan the action to be safe (same
                // GitHub-only gating as above).
                return Ok(Some(self.create_github_settings_action_filtered(
                    github_only_toggles_supported,
                    true,
                    true,
                    github_only_toggles_supported,
                    github_only_toggles_supported,
                )));
            }
        };

        let (needs_discussions, needs_issues, needs_wiki, needs_vuln_alerts, needs_auto_fixes) =
            Self::compute_settings_needs(
                gs,
                current_settings.has_issues_enabled,
                current_settings.has_wiki_enabled,
                current_settings.has_discussions_enabled,
                provider.has_vulnerability_alerts(),
                provider.has_automated_security_fixes(),
                github_only_toggles_supported,
            );

        tracing::debug!(
            "GitHub settings check: discussions={} (current={}, desired={}), vuln_alerts={} (desired={}), auto_fixes={} (desired={})",
            needs_discussions,
            current_settings.has_discussions_enabled,
            gs.discussions,
            needs_vuln_alerts,
            gs.vulnerability_alerts,
            needs_auto_fixes,
            gs.automated_security_fixes
        );

        // Only create action if something needs to be changed
        if needs_discussions || needs_issues || needs_wiki || needs_vuln_alerts || needs_auto_fixes
        {
            Ok(Some(self.create_github_settings_action_filtered(
                needs_discussions,
                needs_issues,
                needs_wiki,
                needs_vuln_alerts,
                needs_auto_fixes,
            )))
        } else {
            Ok(None)
        }
    }

    /// Create a GitHub settings action with only settings that need to be changed
    fn create_github_settings_action_filtered(
        &self,
        needs_discussions: bool,
        needs_issues: bool,
        needs_wiki: bool,
        needs_vuln_alerts: bool,
        needs_auto_fixes: bool,
    ) -> Action {
        let gs = &self.config.actions.github_settings;

        let settings = GitHubRepoSettings {
            enable_discussions: if needs_discussions {
                Some(gs.discussions)
            } else {
                None
            },
            enable_issues: if needs_issues { Some(gs.issues) } else { None },
            enable_wiki: if needs_wiki { Some(gs.wiki) } else { None },
            enable_vulnerability_alerts: if needs_vuln_alerts {
                Some(gs.vulnerability_alerts)
            } else {
                None
            },
            enable_automated_security_fixes: if needs_auto_fixes {
                Some(gs.automated_security_fixes)
            } else {
                None
            },
        };

        let mut details = Vec::new();

        if needs_discussions && gs.discussions {
            details.push("Enable discussions".to_string());
        }
        if needs_vuln_alerts && gs.vulnerability_alerts {
            details.push("Enable vulnerability alerts".to_string());
        }
        if needs_auto_fixes && gs.automated_security_fixes {
            details.push("Enable automated security fixes".to_string());
        }

        Action::new(
            "github-settings",
            "github",
            "Update repository settings",
            ActionOperation::UpdateRepoSettings { settings },
        )
        .with_details(details)
    }

    /// Pure computation of which Actions/security-settings fields need to
    /// change, given already-fetched provider state (review bug #11:
    /// SEC013-017). Extracted out of [`Self::plan_actions_security_if_needed`]
    /// so it can be pinned with a plain unit test that doesn't require a
    /// live, authenticated provider.
    ///
    /// On a read error, each field fails SAFE by assuming the *least*
    /// secure current state (scanning/protection/approval off, actions
    /// unrestricted, workflow permissions `"write"`) -- mirroring bug #14's
    /// fail-safe fallback -- so a mismatch against the (more secure) desired
    /// default always triggers remediation rather than silently skipping it.
    ///
    /// Returns `(needs_secret_scanning, needs_push_protection,
    /// needs_restrict_actions, needs_workflow_permissions,
    /// needs_fork_pr_approval)`.
    fn compute_actions_security_needs(
        cfg: &ActionsSecurityConfig,
        secret_scanning: Result<SecretScanningSettings, crate::error::RepoLensError>,
        actions_permissions: Result<ActionsPermissions, crate::error::RepoLensError>,
        workflow_permissions: Result<ActionsPermissions, crate::error::RepoLensError>,
        fork_pr_requires_approval: Result<bool, crate::error::RepoLensError>,
    ) -> (bool, bool, bool, bool, bool) {
        let (current_secret_scanning, current_push_protection) = match secret_scanning {
            Ok(s) => (s.enabled, s.push_protection_enabled),
            Err(e) => {
                tracing::debug!("Could not check secret scanning status: {:?}", e);
                (false, false)
            }
        };
        let needs_secret_scanning = current_secret_scanning != cfg.secret_scanning;
        let needs_push_protection = current_push_protection != cfg.push_protection;

        let current_allowed_actions = actions_permissions
            .map(|p| p.allowed_actions.unwrap_or_else(|| "all".to_string()))
            .unwrap_or_else(|e| {
                tracing::debug!("Could not check Actions permissions: {:?}", e);
                "all".to_string()
            });
        let needs_restrict_actions = current_allowed_actions != cfg.allowed_actions;

        let current_workflow_permissions = workflow_permissions
            .ok()
            .and_then(|p| p.default_workflow_permissions)
            .unwrap_or_else(|| "write".to_string());
        let needs_workflow_permissions =
            current_workflow_permissions != cfg.default_workflow_permissions;

        let current_requires_approval = fork_pr_requires_approval.unwrap_or_else(|e| {
            tracing::debug!("Could not check fork-PR workflow approval policy: {:?}", e);
            false
        });
        let needs_fork_pr_approval = current_requires_approval != cfg.require_fork_pr_approval;

        (
            needs_secret_scanning,
            needs_push_protection,
            needs_restrict_actions,
            needs_workflow_permissions,
            needs_fork_pr_approval,
        )
    }

    /// Plan GitHub Actions & security-settings updates if needed (review bug
    /// #11: SEC013-017).
    ///
    /// GitHub-only: every one of these five toggles has no GitLab
    /// equivalent (`GitLabProvider`'s write methods all return `Err`), so
    /// this never plans anything unless the effective provider is GitHub --
    /// consistent with [`Self::supports_github_only_settings`]'s gating of
    /// the sibling repo-settings toggles (review bug #9).
    ///
    /// # Returns
    ///
    /// An `Action` to update Actions/security settings, or `None` if
    /// already configured correctly (or the provider isn't GitHub).
    async fn plan_actions_security_if_needed(
        &self,
    ) -> Result<Option<Action>, crate::error::RepoLensError> {
        if !self.supports_github_only_settings() {
            return Ok(None);
        }

        let cfg = &self.config.actions.actions_security;

        let provider = match crate::providers::for_config(&self.config) {
            Some(p) => p,
            None => {
                // No authenticated provider yet: plan the full fix so
                // `apply` can retry once one is available (same fail-safe
                // shape as branch protection / repo settings).
                return Ok(Some(self.create_actions_security_action_filtered(
                    true, true, true, true, true,
                )));
            }
        };

        let (
            needs_secret_scanning,
            needs_push_protection,
            needs_restrict_actions,
            needs_workflow_permissions,
            needs_fork_pr_approval,
        ) = Self::compute_actions_security_needs(
            cfg,
            provider.get_secret_scanning(),
            provider.get_actions_permissions(),
            provider.get_actions_workflow_permissions(),
            provider.get_fork_pr_workflows_policy(),
        );

        if needs_secret_scanning
            || needs_push_protection
            || needs_restrict_actions
            || needs_workflow_permissions
            || needs_fork_pr_approval
        {
            Ok(Some(self.create_actions_security_action_filtered(
                needs_secret_scanning,
                needs_push_protection,
                needs_restrict_actions,
                needs_workflow_permissions,
                needs_fork_pr_approval,
            )))
        } else {
            Ok(None)
        }
    }

    /// Create an Actions/security-settings action with only the fields that
    /// need to change (review bug #11: SEC013-017).
    fn create_actions_security_action_filtered(
        &self,
        needs_secret_scanning: bool,
        needs_push_protection: bool,
        needs_restrict_actions: bool,
        needs_workflow_permissions: bool,
        needs_fork_pr_approval: bool,
    ) -> Action {
        let cfg = &self.config.actions.actions_security;

        let settings = GitHubActionsSecuritySettings {
            secret_scanning: needs_secret_scanning.then_some(cfg.secret_scanning),
            secret_scanning_push_protection: needs_push_protection.then_some(cfg.push_protection),
            allowed_actions: needs_restrict_actions.then(|| cfg.allowed_actions.clone()),
            default_workflow_permissions: needs_workflow_permissions
                .then(|| cfg.default_workflow_permissions.clone()),
            require_fork_pr_approval: needs_fork_pr_approval
                .then_some(cfg.require_fork_pr_approval),
        };

        let mut details = Vec::new();
        if needs_secret_scanning {
            details.push(format!("Enable secret scanning: {}", cfg.secret_scanning)); // SEC013
        }
        if needs_push_protection {
            details.push(format!("Enable push protection: {}", cfg.push_protection)); // SEC014
        }
        if needs_restrict_actions {
            details.push(format!(
                "Restrict allowed Actions to: {}",
                cfg.allowed_actions
            )); // SEC015
        }
        if needs_workflow_permissions {
            details.push(format!(
                "Set default workflow permissions: {}",
                cfg.default_workflow_permissions
            )); // SEC016
        }
        if needs_fork_pr_approval {
            details.push("Require approval for fork pull request workflows".to_string());
            // SEC017
        }

        Action::new(
            "actions-security-settings",
            "github",
            "Update GitHub Actions & security settings",
            ActionOperation::UpdateActionsSecuritySettings { settings },
        )
        .with_details(details)
    }

    /// Plan a repository-metadata update if needed.
    ///
    /// Metadata (description / topics / homepage) cannot be auto-filled from
    /// nothing, so — like branch protection and repository settings — this
    /// action applies the values the user configured. It is only emitted when:
    /// the metadata action is enabled, at least one configured value is present,
    /// and the audit reports a missing-metadata finding (META001/002/003 in the
    /// `metadata` category).
    fn plan_metadata_if_needed(&self, results: &AuditResults) -> Option<Action> {
        let meta = &self.config.actions.metadata;

        // Need at least one configured value to apply.
        let has_value =
            meta.description.is_some() || !meta.topics.is_empty() || meta.homepage.is_some();
        if !has_value {
            return None;
        }

        // Need a missing-metadata finding to act on.
        let has_finding = results
            .findings_by_category("metadata")
            .any(|f| matches!(f.rule_id.as_str(), "META001" | "META002" | "META003"));
        if !has_finding {
            return None;
        }

        let mut details = Vec::new();
        if let Some(desc) = &meta.description {
            details.push(format!("Set description: {desc}"));
        }
        if !meta.topics.is_empty() {
            details.push(format!("Set topics: {}", meta.topics.join(", ")));
        }
        if let Some(home) = &meta.homepage {
            details.push(format!("Set homepage: {home}"));
        }

        Some(
            Action::new(
                "repo-metadata",
                "metadata",
                "Update repository metadata",
                ActionOperation::UpdateRepoMetadata {
                    description: meta.description.clone(),
                    topics: meta.topics.clone(),
                    homepage: meta.homepage.clone(),
                },
            )
            .with_details(details),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::rules::results::{AuditResults, Finding, Severity};

    #[tokio::test]
    async fn test_create_plan_includes_gitignore() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "FILE003",
            "files",
            Severity::Info,
            ".gitignore missing recommended entry: .env",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(!plan.is_empty());
        assert!(plan.actions().iter().any(|a| a.id() == "gitignore-update"));
    }

    #[tokio::test]
    async fn test_create_plan_includes_license() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC004",
            "docs",
            Severity::Critical,
            "LICENSE file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(plan.actions().iter().any(|a| a.id() == "license-create"));
    }

    #[tokio::test]
    async fn test_create_plan_includes_contributing() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC005",
            "docs",
            Severity::Warning,
            "CONTRIBUTING file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(
            plan.actions()
                .iter()
                .any(|a| a.id() == "contributing-create")
        );
    }

    #[tokio::test]
    async fn test_create_plan_filters_by_config() {
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

        let plan = planner.create_plan(&results).await.unwrap();

        // Should not include contributing because it's disabled in config
        assert!(
            !plan
                .actions()
                .iter()
                .any(|a| a.id() == "contributing-create")
        );
    }

    #[tokio::test]
    async fn test_create_plan_includes_code_of_conduct() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC006",
            "docs",
            Severity::Warning,
            "CODE_OF_CONDUCT file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(plan.actions().iter().any(|a| a.id() == "coc-create"));
    }

    #[tokio::test]
    async fn test_create_plan_includes_security_policy() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC007",
            "docs",
            Severity::Warning,
            "SECURITY.md is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(plan.actions().iter().any(|a| a.id() == "security-create"));
    }

    #[tokio::test]
    async fn test_create_plan_includes_branch_protection() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);
        let results = AuditResults::new("test-repo", "opensource");

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(plan.actions().iter().any(|a| a.id() == "branch-protection"));
    }

    #[tokio::test]
    async fn test_create_plan_includes_github_settings() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);
        let results = AuditResults::new("test-repo", "opensource");

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(plan.actions().iter().any(|a| a.id() == "github-settings"));
    }

    #[tokio::test]
    async fn test_create_plan_no_gitignore_needed() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);
        let results = AuditResults::new("test-repo", "opensource");

        let plan = planner.create_plan(&results).await.unwrap();

        // No FILE003 findings, so no gitignore update
        assert!(!plan.actions().iter().any(|a| a.id() == "gitignore-update"));
    }

    #[tokio::test]
    async fn test_create_plan_license_with_author_and_year() {
        let mut config = Config::default();
        config.actions.license.author = Some("Test Author".to_string());
        config.actions.license.year = Some("2024".to_string());

        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC004",
            "docs",
            Severity::Critical,
            "LICENSE file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(plan.actions().iter().any(|a| a.id() == "license-create"));
    }

    #[tokio::test]
    async fn test_branch_protection_with_signed_commits() {
        let mut config = Config::default();
        config.actions.branch_protection.require_signed_commits = true;

        let planner = ActionPlanner::new(config);
        let results = AuditResults::new("test-repo", "opensource");

        let plan = planner.create_plan(&results).await.unwrap();

        let bp_action = plan
            .actions()
            .iter()
            .find(|a| a.id() == "branch-protection");
        assert!(bp_action.is_some());
    }

    #[tokio::test]
    async fn test_create_plan_multiple_gitignore_entries() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "FILE003",
            "files",
            Severity::Info,
            ".gitignore missing recommended entry: .env",
        ));
        results.add_finding(Finding::new(
            "FILE003",
            "files",
            Severity::Info,
            ".gitignore missing recommended entry: *.log",
        ));
        results.add_finding(Finding::new(
            "FILE003",
            "files",
            Severity::Info,
            ".gitignore missing recommended entry: node_modules/",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        let gitignore_action = plan
            .actions()
            .iter()
            .find(|a| a.id() == "gitignore-update")
            .expect("Should have gitignore action");

        // Should collect all entries
        match gitignore_action.operation() {
            ActionOperation::UpdateGitignore { entries } => {
                assert_eq!(entries.len(), 3);
                assert!(entries.contains(&".env".to_string()));
                assert!(entries.contains(&"*.log".to_string()));
                assert!(entries.contains(&"node_modules/".to_string()));
            }
            _ => panic!("Expected UpdateGitignore operation"),
        }
    }

    #[tokio::test]
    async fn test_plan_license_uses_default_year() {
        let mut config = Config::default();
        config.actions.license.year = None; // No year specified, should use current year

        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC004",
            "docs",
            Severity::Critical,
            "LICENSE file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        let license_action = plan
            .actions()
            .iter()
            .find(|a| a.id() == "license-create")
            .expect("Should have license action");

        match license_action.operation() {
            ActionOperation::CreateFile { variables, .. } => {
                assert!(variables.contains_key("year"));
                // Year should be the current year (4 digits)
                assert_eq!(variables.get("year").unwrap().len(), 4);
            }
            _ => panic!("Expected CreateFile operation"),
        }
    }

    #[tokio::test]
    async fn test_plan_all_docs_disabled() {
        let mut config = Config::default();
        config.actions.contributing = false;
        config.actions.code_of_conduct = false;
        config.actions.security_policy = false;
        config.actions.license.enabled = false;
        config.actions.gitignore = false;
        config.actions.branch_protection.enabled = false;

        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC004",
            "docs",
            Severity::Critical,
            "LICENSE file is missing",
        ));
        results.add_finding(Finding::new(
            "DOC005",
            "docs",
            Severity::Warning,
            "CONTRIBUTING file is missing",
        ));
        results.add_finding(Finding::new(
            "DOC006",
            "docs",
            Severity::Warning,
            "CODE_OF_CONDUCT file is missing",
        ));
        results.add_finding(Finding::new(
            "DOC007",
            "docs",
            Severity::Warning,
            "SECURITY.md is missing",
        ));
        results.add_finding(Finding::new(
            "FILE003",
            "files",
            Severity::Info,
            ".gitignore missing recommended entry: .env",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        // Only github-settings should be present (always planned)
        assert!(!plan.actions().iter().any(|a| a.id() == "license-create"));
        assert!(
            !plan
                .actions()
                .iter()
                .any(|a| a.id() == "contributing-create")
        );
        assert!(!plan.actions().iter().any(|a| a.id() == "coc-create"));
        assert!(!plan.actions().iter().any(|a| a.id() == "security-create"));
        assert!(!plan.actions().iter().any(|a| a.id() == "gitignore-update"));
        assert!(!plan.actions().iter().any(|a| a.id() == "branch-protection"));
    }

    #[test]
    fn test_action_planner_new() {
        let config = Config::default();
        let planner = ActionPlanner::new(config.clone());
        // Verify planner was created (indirect test via create_plan)
        assert!(planner.config.actions.gitignore);
    }

    #[tokio::test]
    async fn test_create_branch_protection_action_directly() {
        let mut config = Config::default();
        config.actions.branch_protection.branch = "develop".to_string();
        config.actions.branch_protection.required_approvals = 2;
        config.actions.branch_protection.require_status_checks = false;
        config.actions.branch_protection.block_force_push = true;

        let planner = ActionPlanner::new(config);
        let action = planner.create_branch_protection_action();

        assert_eq!(action.id(), "branch-protection");
        assert!(action.description().contains("develop"));

        match action.operation() {
            ActionOperation::ConfigureProtectedBranch { branch, settings } => {
                assert_eq!(branch, "develop");
                assert_eq!(settings.required_approvals, 2);
                assert!(!settings.require_status_checks);
                assert!(settings.block_force_push);
            }
            _ => panic!("Expected ConfigureProtectedBranch operation"),
        }
    }

    #[test]
    fn test_create_github_settings_action_directly() {
        let mut config = Config::default();
        config.actions.github_settings.discussions = true;
        config.actions.github_settings.vulnerability_alerts = true;
        config.actions.github_settings.automated_security_fixes = true;

        let planner = ActionPlanner::new(config);
        // All fields flagged as "needing" a change is equivalent to the
        // old unconditional `create_github_settings_action` helper (removed
        // as dead code once every call site started routing through the
        // filtered variant for the #9/#14 fixes).
        let action = planner.create_github_settings_action_filtered(true, true, true, true, true);

        assert_eq!(action.id(), "github-settings");

        match action.operation() {
            ActionOperation::UpdateRepoSettings { settings } => {
                assert_eq!(settings.enable_discussions, Some(true));
                assert_eq!(settings.enable_vulnerability_alerts, Some(true));
                assert_eq!(settings.enable_automated_security_fixes, Some(true));
            }
            _ => panic!("Expected UpdateRepoSettings operation"),
        }
    }

    #[test]
    fn test_create_github_settings_action_filtered() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        // Only discussions needs update
        let action = planner.create_github_settings_action_filtered(
            true,  // needs_discussions
            false, // needs_issues
            false, // needs_wiki
            false, // needs_vuln_alerts
            false, // needs_auto_fixes
        );

        match action.operation() {
            ActionOperation::UpdateRepoSettings { settings } => {
                assert!(settings.enable_discussions.is_some());
                assert!(settings.enable_issues.is_none());
                assert!(settings.enable_wiki.is_none());
                assert!(settings.enable_vulnerability_alerts.is_none());
                assert!(settings.enable_automated_security_fixes.is_none());
            }
            _ => panic!("Expected UpdateRepoSettings operation"),
        }
    }

    #[tokio::test]
    async fn test_create_plan_includes_metadata_when_configured() {
        let mut config = Config::default();
        config.actions.metadata.description = Some("My project".to_string());
        config.actions.metadata.topics = vec!["rust".to_string()];

        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "META001",
            "metadata",
            Severity::Info,
            "Repository description is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        let action = plan
            .actions()
            .iter()
            .find(|a| a.id() == "repo-metadata")
            .expect("Should have repo-metadata action");

        match action.operation() {
            ActionOperation::UpdateRepoMetadata {
                description,
                topics,
                homepage,
            } => {
                assert_eq!(description.as_deref(), Some("My project"));
                assert_eq!(topics, &vec!["rust".to_string()]);
                assert!(homepage.is_none());
            }
            _ => panic!("Expected UpdateRepoMetadata operation"),
        }
    }

    #[tokio::test]
    async fn test_create_plan_no_metadata_when_unconfigured() {
        // Finding present but no configured values -> no action.
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "META001",
            "metadata",
            Severity::Info,
            "Repository description is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(!plan.actions().iter().any(|a| a.id() == "repo-metadata"));
    }

    #[tokio::test]
    async fn test_create_plan_no_metadata_when_no_finding() {
        // Configured values but no META finding -> no action.
        let mut config = Config::default();
        config.actions.metadata.description = Some("My project".to_string());

        let planner = ActionPlanner::new(config);
        let results = AuditResults::new("test-repo", "opensource");

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(!plan.actions().iter().any(|a| a.id() == "repo-metadata"));
    }

    #[tokio::test]
    async fn test_create_plan_no_metadata_when_disabled() {
        // Configured + finding present, but action disabled -> no action.
        let mut config = Config::default();
        config.actions.metadata.enabled = false;
        config.actions.metadata.description = Some("My project".to_string());

        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "META001",
            "metadata",
            Severity::Info,
            "Repository description is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(!plan.actions().iter().any(|a| a.id() == "repo-metadata"));
    }

    // ===== New file-creation actions (B3 part 2) =====

    #[tokio::test]
    async fn test_create_plan_includes_readme() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC001",
            "docs",
            Severity::Warning,
            "README file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(plan.actions().iter().any(|a| a.id() == "readme-create"));
    }

    #[tokio::test]
    async fn test_create_plan_no_readme_when_disabled() {
        let mut config = Config::default();
        config.actions.readme = false;
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC001",
            "docs",
            Severity::Warning,
            "README file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(!plan.actions().iter().any(|a| a.id() == "readme-create"));
    }

    #[tokio::test]
    async fn test_create_plan_includes_changelog() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC008",
            "docs",
            Severity::Warning,
            "CHANGELOG file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(plan.actions().iter().any(|a| a.id() == "changelog-create"));
    }

    #[tokio::test]
    async fn test_create_plan_no_changelog_when_disabled() {
        let mut config = Config::default();
        config.actions.changelog = false;
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "DOC008",
            "docs",
            Severity::Warning,
            "CHANGELOG file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(!plan.actions().iter().any(|a| a.id() == "changelog-create"));
    }

    #[tokio::test]
    async fn test_create_plan_includes_gitattributes() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "GIT002",
            "git",
            Severity::Info,
            ".gitattributes file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(
            plan.actions()
                .iter()
                .any(|a| a.id() == "gitattributes-create")
        );
    }

    #[tokio::test]
    async fn test_create_plan_no_gitattributes_when_disabled() {
        let mut config = Config::default();
        config.actions.gitattributes = false;
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "GIT002",
            "git",
            Severity::Info,
            ".gitattributes file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(
            !plan
                .actions()
                .iter()
                .any(|a| a.id() == "gitattributes-create")
        );
    }

    #[tokio::test]
    async fn test_create_plan_includes_codeowners() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "CODE001",
            "codeowners",
            Severity::Info,
            "CODEOWNERS file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(plan.actions().iter().any(|a| a.id() == "codeowners-create"));
    }

    #[tokio::test]
    async fn test_create_plan_no_codeowners_when_disabled() {
        let mut config = Config::default();
        config.actions.codeowners = false;
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "CODE001",
            "codeowners",
            Severity::Info,
            "CODEOWNERS file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(!plan.actions().iter().any(|a| a.id() == "codeowners-create"));
    }

    #[tokio::test]
    async fn test_create_plan_includes_settings_file() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "SEC007",
            "security",
            Severity::Info,
            "GitHub settings file (.github/settings.yml) is absent",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(
            plan.actions()
                .iter()
                .any(|a| a.id() == "settings-file-create")
        );
    }

    #[tokio::test]
    async fn test_create_plan_no_settings_file_when_disabled() {
        let mut config = Config::default();
        config.actions.settings_file = false;
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "SEC007",
            "security",
            Severity::Info,
            "GitHub settings file (.github/settings.yml) is absent",
        ));

        let plan = planner.create_plan(&results).await.unwrap();
        assert!(
            !plan
                .actions()
                .iter()
                .any(|a| a.id() == "settings-file-create")
        );
    }

    // ===== T10b regression tests: bugs #9, #12, #13, #14 =====

    /// Regression test for review bug #13: FILE002 (.gitignore entirely
    /// absent) was never wired to any remediation -- `plan_gitignore_update`
    /// only reads FILE003 (missing recommended entries in an EXISTING file).
    /// Fails before the fix (no "gitignore-create" action exists at all),
    /// passes after `plan_gitignore_creation` is wired in.
    #[tokio::test]
    async fn test_file002_missing_gitignore_plans_creation() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "FILE002",
            "files",
            Severity::Warning,
            ".gitignore file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(
            plan.actions().iter().any(|a| a.id() == "gitignore-create"),
            "FILE002 (.gitignore entirely absent) must plan a remediation"
        );
    }

    #[tokio::test]
    async fn test_file002_gitignore_create_disabled_by_config() {
        let mut config = Config::default();
        config.actions.gitignore = false;
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "FILE002",
            "files",
            Severity::Warning,
            ".gitignore file is missing",
        ));

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(!plan.actions().iter().any(|a| a.id() == "gitignore-create"));
    }

    /// Regression test for review bug #14: on a provider error, the OLD
    /// fallback assumed the setting was ALREADY enabled (`Err(_) => true`),
    /// which under-protects when `desired == true` (needs = current != desired
    /// silently becomes `false`). Fails before the fix (needs_vuln_alerts is
    /// `false` despite the error), passes after the fail-safe fallback
    /// (`Err(_) => false`) is applied.
    #[test]
    fn test_vulnerability_alerts_error_fails_safe_plans_fix() {
        let gs = GitHubSettingsConfig {
            vulnerability_alerts: true,
            ..GitHubSettingsConfig::default()
        };
        let err =
            crate::error::RepoLensError::Provider(crate::error::ProviderError::CommandFailed {
                command: "gh api ...".to_string(),
            });

        let (_, _, _, needs_vuln_alerts, _) = ActionPlanner::compute_settings_needs(
            &gs,
            gs.issues,
            gs.wiki,
            gs.discussions,
            Err(err),
            Ok(gs.automated_security_fixes),
            true,
        );

        assert!(
            needs_vuln_alerts,
            "a provider error must fail SAFE: plan the fix rather than assume it's already enabled"
        );
    }

    /// Same fail-safe requirement for automated security fixes.
    #[test]
    fn test_automated_security_fixes_error_fails_safe_plans_fix() {
        let gs = GitHubSettingsConfig {
            automated_security_fixes: true,
            ..GitHubSettingsConfig::default()
        };
        let err =
            crate::error::RepoLensError::Provider(crate::error::ProviderError::CommandFailed {
                command: "gh api ...".to_string(),
            });

        let (_, _, _, _, needs_auto_fixes) = ActionPlanner::compute_settings_needs(
            &gs,
            gs.issues,
            gs.wiki,
            gs.discussions,
            Ok(gs.vulnerability_alerts),
            Err(err),
            true,
        );

        assert!(
            needs_auto_fixes,
            "a provider error must fail SAFE: plan the fix rather than assume it's already enabled"
        );
    }

    #[test]
    fn test_settings_needs_ignores_github_only_toggles_when_unsupported() {
        // When the effective provider doesn't support the GitHub-only
        // toggles (github_only_toggles_supported = false), none of
        // discussions/vuln-alerts/auto-fixes should ever be flagged as
        // "needing" a change, even when their current/desired values
        // clearly differ.
        let gs = GitHubSettingsConfig {
            discussions: true,
            vulnerability_alerts: true,
            automated_security_fixes: true,
            ..GitHubSettingsConfig::default()
        };

        let (needs_discussions, _, _, needs_vuln_alerts, needs_auto_fixes) =
            ActionPlanner::compute_settings_needs(
                &gs,
                gs.issues,
                gs.wiki,
                false, // current discussions disabled, desired true -> would need update if supported
                Ok(false),
                Ok(false),
                false,
            );

        assert!(!needs_discussions);
        assert!(!needs_vuln_alerts);
        assert!(!needs_auto_fixes);
    }

    #[test]
    fn test_settings_needs_flags_github_only_toggles_when_supported() {
        // Mirror of the test above with `github_only_toggles_supported =
        // true` (e.g. the effective provider is GitHub): the same current/
        // desired mismatch DOES need an update.
        let gs = GitHubSettingsConfig {
            discussions: true,
            vulnerability_alerts: true,
            automated_security_fixes: true,
            ..GitHubSettingsConfig::default()
        };

        let (needs_discussions, _, _, needs_vuln_alerts, needs_auto_fixes) =
            ActionPlanner::compute_settings_needs(
                &gs,
                gs.issues,
                gs.wiki,
                false,
                Ok(false),
                Ok(false),
                true,
            );

        assert!(needs_discussions);
        assert!(needs_vuln_alerts);
        assert!(needs_auto_fixes);
    }

    /// Regression test for review bug #9 (planner-side coordination with
    /// T10a): `GitLabProvider::set_repo_settings` returns `Err` when
    /// discussions/vulnerability-alerts/automated-security-fixes are
    /// requested (GitLab has no equivalent). The planner must never emit an
    /// action that sets those fields for a GitLab repository -- otherwise
    /// `apply` plans an action that can never succeed ("never converges").
    /// Fails before the fix (the no-provider fallback unconditionally set all
    /// five fields via `create_github_settings_action`), passes after the
    /// GitHub-only gating is applied to both fallback branches.
    #[tokio::test]
    async fn test_gitlab_provider_does_not_plan_github_only_settings_fields() {
        let config = Config {
            provider: Some(crate::providers::Provider::GitLab),
            ..Config::default()
        };
        let planner = ActionPlanner::new(config);
        let results = AuditResults::new("test-repo", "opensource");

        let plan = planner.create_plan(&results).await.unwrap();

        let action = plan
            .actions()
            .iter()
            .find(|a| a.id() == "github-settings")
            .expect("issues/wiki are still settable on GitLab, so the action is still planned");

        match action.operation() {
            ActionOperation::UpdateRepoSettings { settings } => {
                assert!(
                    settings.enable_discussions.is_none(),
                    "discussions has no GitLab equivalent -- must not be planned"
                );
                assert!(
                    settings.enable_vulnerability_alerts.is_none(),
                    "vulnerability alerts have no GitLab equivalent -- must not be planned"
                );
                assert!(
                    settings.enable_automated_security_fixes.is_none(),
                    "automated security fixes have no GitLab equivalent -- must not be planned"
                );
            }
            _ => panic!("Expected UpdateRepoSettings operation"),
        }
    }

    // Note: no "GitHub provider still plans all fields" control test here --
    // this environment has an authenticated `gh` CLI with access to a real
    // repository, so `for_config`/`get_repo_settings` reach the LIVE
    // GitHub API and the "needs" outcome legitimately depends on that
    // repository's actual current settings (not a fixed expectation). The
    // pure-function tests above (`test_settings_needs_ignores_github_only_toggles_when_unsupported`
    // and its sibling with `github_only_toggles_supported: true`, implied by
    // `test_vulnerability_alerts_error_fails_safe_plans_fix`) already cover
    // the GitHub-only-toggles-supported branch deterministically.

    // ===== Review bug #11: real remediation for SEC013-017 =====
    //
    // Before this feature, SEC013-017 had no wiring in the planner at all --
    // `compute_actions_security_needs` and `plan_actions_security_if_needed`
    // did not exist, so these tests fail to even compile against pre-T10c
    // code (the strongest possible "fails before" signal). Each test below
    // pins one rule's "needs" computation deterministically, without a live
    // provider (see the no-control-test note above for why: this sandbox's
    // `gh` is authenticated against a real repository).

    fn default_actions_security_config() -> ActionsSecurityConfig {
        ActionsSecurityConfig::default()
    }

    /// SEC013: secret scanning disabled must be flagged as needing a fix.
    #[test]
    fn test_sec013_secret_scanning_disabled_needs_fix() {
        let cfg = default_actions_security_config();
        let (needs_secret_scanning, _, _, _, _) = ActionPlanner::compute_actions_security_needs(
            &cfg,
            Ok(SecretScanningSettings {
                enabled: false,
                push_protection_enabled: false,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: Some(cfg.allowed_actions.clone()),
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(cfg.require_fork_pr_approval),
        );

        assert!(
            needs_secret_scanning,
            "secret scanning disabled must plan SEC013's remediation"
        );
    }

    /// SEC013 (error path): a provider error must fail SAFE (assume secret
    /// scanning is NOT enabled), not silently skip the fix.
    #[test]
    fn test_sec013_secret_scanning_error_fails_safe_plans_fix() {
        let cfg = default_actions_security_config();
        let err =
            crate::error::RepoLensError::Provider(crate::error::ProviderError::CommandFailed {
                command: "gh api ...".to_string(),
            });

        let (needs_secret_scanning, _, _, _, _) = ActionPlanner::compute_actions_security_needs(
            &cfg,
            Err(err),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: Some(cfg.allowed_actions.clone()),
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(cfg.require_fork_pr_approval),
        );

        assert!(
            needs_secret_scanning,
            "a provider error must fail SAFE: plan SEC013's fix rather than assume it's already enabled"
        );
    }

    /// SEC014: push protection disabled (while secret scanning IS enabled)
    /// must be flagged as needing a fix.
    #[test]
    fn test_sec014_push_protection_disabled_needs_fix() {
        let cfg = default_actions_security_config();
        let (_, needs_push_protection, _, _, _) = ActionPlanner::compute_actions_security_needs(
            &cfg,
            Ok(SecretScanningSettings {
                enabled: true,
                push_protection_enabled: false,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: Some(cfg.allowed_actions.clone()),
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(cfg.require_fork_pr_approval),
        );

        assert!(
            needs_push_protection,
            "push protection disabled must plan SEC014's remediation"
        );
    }

    /// SEC015: `allowed_actions == "all"` (unrestricted) must be flagged as
    /// needing a fix when the desired value is more restrictive.
    #[test]
    fn test_sec015_unrestricted_actions_needs_fix() {
        let cfg = default_actions_security_config();
        let (_, _, needs_restrict_actions, _, _) = ActionPlanner::compute_actions_security_needs(
            &cfg,
            Ok(SecretScanningSettings {
                enabled: cfg.secret_scanning,
                push_protection_enabled: cfg.push_protection,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: Some("all".to_string()),
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(cfg.require_fork_pr_approval),
        );

        assert!(
            needs_restrict_actions,
            "allowed_actions == \"all\" must plan SEC015's remediation"
        );
    }

    /// SEC015 (error path): a provider error must fail SAFE (assume "all",
    /// the least restrictive value), not silently skip the fix.
    #[test]
    fn test_sec015_actions_permissions_error_fails_safe_plans_fix() {
        let cfg = default_actions_security_config();
        let err =
            crate::error::RepoLensError::Provider(crate::error::ProviderError::CommandFailed {
                command: "gh api ...".to_string(),
            });

        let (_, _, needs_restrict_actions, _, _) = ActionPlanner::compute_actions_security_needs(
            &cfg,
            Ok(SecretScanningSettings {
                enabled: cfg.secret_scanning,
                push_protection_enabled: cfg.push_protection,
            }),
            Err(err),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(cfg.require_fork_pr_approval),
        );

        assert!(
            needs_restrict_actions,
            "a provider error must fail SAFE: plan SEC015's fix rather than assume actions are already restricted"
        );
    }

    /// SEC016: `default_workflow_permissions == "write"` must be flagged as
    /// needing a fix when the desired value is `"read"`.
    #[test]
    fn test_sec016_write_workflow_permissions_needs_fix() {
        let cfg = default_actions_security_config();
        let (_, _, _, needs_workflow_permissions, _) =
            ActionPlanner::compute_actions_security_needs(
                &cfg,
                Ok(SecretScanningSettings {
                    enabled: cfg.secret_scanning,
                    push_protection_enabled: cfg.push_protection,
                }),
                Ok(ActionsPermissions {
                    enabled: true,
                    allowed_actions: Some(cfg.allowed_actions.clone()),
                    default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                    can_approve_pull_request_reviews: None,
                }),
                Ok(ActionsPermissions {
                    enabled: true,
                    allowed_actions: None,
                    default_workflow_permissions: Some("write".to_string()),
                    can_approve_pull_request_reviews: None,
                }),
                Ok(cfg.require_fork_pr_approval),
            );

        assert!(
            needs_workflow_permissions,
            "default_workflow_permissions == \"write\" must plan SEC016's remediation"
        );
    }

    /// SEC017: fork-PR workflows not requiring approval must be flagged as
    /// needing a fix.
    #[test]
    fn test_sec017_fork_pr_approval_not_required_needs_fix() {
        let cfg = default_actions_security_config();
        let (_, _, _, _, needs_fork_pr_approval) = ActionPlanner::compute_actions_security_needs(
            &cfg,
            Ok(SecretScanningSettings {
                enabled: cfg.secret_scanning,
                push_protection_enabled: cfg.push_protection,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: Some(cfg.allowed_actions.clone()),
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(false),
        );

        assert!(
            needs_fork_pr_approval,
            "fork-PR workflows not requiring approval must plan SEC017's remediation"
        );
    }

    /// SEC017 (error path): a provider error must fail SAFE (assume
    /// approval is NOT required), not silently skip the fix.
    #[test]
    fn test_sec017_fork_pr_approval_error_fails_safe_plans_fix() {
        let cfg = default_actions_security_config();
        let err =
            crate::error::RepoLensError::Provider(crate::error::ProviderError::CommandFailed {
                command: "gh api ...".to_string(),
            });

        let (_, _, _, _, needs_fork_pr_approval) = ActionPlanner::compute_actions_security_needs(
            &cfg,
            Ok(SecretScanningSettings {
                enabled: cfg.secret_scanning,
                push_protection_enabled: cfg.push_protection,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: Some(cfg.allowed_actions.clone()),
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Err(err),
        );

        assert!(
            needs_fork_pr_approval,
            "a provider error must fail SAFE: plan SEC017's fix rather than assume approval is already required"
        );
    }

    /// All fields already matching the desired configuration -> nothing to
    /// plan (proves the "needs" computation doesn't spuriously fire).
    #[test]
    fn test_all_actions_security_fields_already_correct_needs_nothing() {
        let cfg = default_actions_security_config();
        let needs = ActionPlanner::compute_actions_security_needs(
            &cfg,
            Ok(SecretScanningSettings {
                enabled: cfg.secret_scanning,
                push_protection_enabled: cfg.push_protection,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: Some(cfg.allowed_actions.clone()),
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(ActionsPermissions {
                enabled: true,
                allowed_actions: None,
                default_workflow_permissions: Some(cfg.default_workflow_permissions.clone()),
                can_approve_pull_request_reviews: None,
            }),
            Ok(cfg.require_fork_pr_approval),
        );

        assert_eq!(needs, (false, false, false, false, false));
    }

    /// Review bug #11's GitLab counterpart to bug #9: `GitLabProvider`'s
    /// four new write methods all return `Err` (no GitLab equivalent for
    /// any of SEC013-017), so the planner must never plan
    /// "actions-security-settings" for a GitLab repository -- otherwise
    /// `apply` would plan an action that can never succeed.
    ///
    /// Fails before this feature (no gating existed because the action
    /// itself didn't exist); passes after `supports_github_only_settings()`
    /// gates `plan_actions_security_if_needed`.
    #[tokio::test]
    async fn test_gitlab_provider_never_plans_actions_security_settings() {
        let config = Config {
            provider: Some(crate::providers::Provider::GitLab),
            ..Config::default()
        };
        let planner = ActionPlanner::new(config);
        let results = AuditResults::new("test-repo", "opensource");

        let plan = planner.create_plan(&results).await.unwrap();

        assert!(
            !plan
                .actions()
                .iter()
                .any(|a| a.id() == "actions-security-settings"),
            "SEC013-017 have no GitLab equivalent -- must never be planned for GitLab"
        );
    }

    /// GitHub-only gating: when the effective provider is NOT GitHub (and
    /// there is no live provider to consult), `plan_actions_security_if_needed`
    /// must return `None` outright rather than reaching the fail-safe
    /// "no provider" branch (which is reserved for an unauthenticated GitHub
    /// provider, not a fundamentally unsupported one).
    #[tokio::test]
    async fn test_actions_security_not_planned_when_provider_unsupported() {
        let config = Config {
            provider: Some(crate::providers::Provider::GitLab),
            ..Config::default()
        };
        let planner = ActionPlanner::new(config);

        let action = planner.plan_actions_security_if_needed().await.unwrap();
        assert!(action.is_none());
    }

    // ===== Review bug #12: real remediation for SEC008/009/010 =====

    #[tokio::test]
    async fn test_plan_settings_file_update_for_sec008() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "SEC008",
            "security",
            Severity::Warning,
            "No branch protection rules defined in settings.yml",
        ));

        let action = planner
            .plan_settings_file_update_if_needed(&results)
            .expect("SEC008 must plan an UpdateSettingsFile action");

        assert_eq!(action.id(), "settings-file-update");
        match action.operation() {
            ActionOperation::UpdateSettingsFile {
                ensure_branches_block,
                ensure_pr_reviews,
                ensure_status_checks,
                ..
            } => {
                assert!(*ensure_branches_block);
                // SEC009/SEC010 findings weren't present, but the merge
                // widens both when the branches block itself is missing.
                assert!(!*ensure_pr_reviews);
                assert!(!*ensure_status_checks);
            }
            _ => panic!("Expected UpdateSettingsFile operation"),
        }
    }

    #[tokio::test]
    async fn test_plan_settings_file_update_for_sec009_and_sec010() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "SEC009",
            "security",
            Severity::Warning,
            "required_pull_request_reviews not configured in branch protection",
        ));
        results.add_finding(Finding::new(
            "SEC010",
            "security",
            Severity::Warning,
            "required_status_checks not configured in branch protection",
        ));

        let action = planner
            .plan_settings_file_update_if_needed(&results)
            .expect("SEC009/SEC010 must plan an UpdateSettingsFile action");

        match action.operation() {
            ActionOperation::UpdateSettingsFile {
                ensure_branches_block,
                ensure_pr_reviews,
                ensure_status_checks,
                ..
            } => {
                assert!(!*ensure_branches_block);
                assert!(*ensure_pr_reviews);
                assert!(*ensure_status_checks);
            }
            _ => panic!("Expected UpdateSettingsFile operation"),
        }
    }

    #[tokio::test]
    async fn test_plan_settings_file_update_none_when_no_findings() {
        let config = Config::default();
        let planner = ActionPlanner::new(config);
        let results = AuditResults::new("test-repo", "opensource");

        assert!(
            planner
                .plan_settings_file_update_if_needed(&results)
                .is_none()
        );
    }

    /// End-to-end regression test for review bug #12: an EXISTING
    /// `.github/settings.yml` missing the `branches:` block gets a REAL
    /// remediation planned and, once executed, the security rules stop
    /// reporting SEC008/009/010 -- and the user's original content survives
    /// the merge (no clobbering). Fails before the fix (no "settings-file-
    /// update" action/`UpdateSettingsFile` operation existed at all).
    #[tokio::test]
    #[serial_test::serial]
    async fn test_sec008_existing_settings_file_gets_real_remediation() {
        use crate::rules::categories::security::SecurityRules;
        use crate::rules::engine::RuleCategory;
        use crate::scanner::Scanner;
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let github_dir = root.join(".github");
        fs::create_dir_all(&github_dir).unwrap();
        fs::write(
            github_dir.join("settings.yml"),
            "repository:\n  name: my-repo\n  description: A test repo\n",
        )
        .unwrap();

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));
        std::env::set_current_dir(root).expect("failed to switch to temp dir");

        let config = Config::default();
        let planner = ActionPlanner::new(config.clone());

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "SEC008",
            "security",
            Severity::Warning,
            "No branch protection rules defined in settings.yml",
        ));

        let action = planner
            .plan_settings_file_update_if_needed(&results)
            .expect("SEC008 on an EXISTING settings.yml must plan a real edit action");

        let mut plan = ActionPlan::new();
        plan.add(action);

        let executor = super::super::executor::ActionExecutor::new(config);
        let exec_results = executor.execute(&plan).await.unwrap();

        // Re-run the security rules against the now-merged file.
        let scanner = Scanner::new(root.to_path_buf());
        let rule_config = Config::default();
        let findings = SecurityRules.run(&scanner, &rule_config).await.unwrap();
        let merged = fs::read_to_string(github_dir.join("settings.yml")).unwrap();

        std::env::set_current_dir(&original_dir).ok();

        assert!(
            exec_results.iter().all(|r| r.success),
            "settings-file-update action must succeed: {:?}",
            exec_results
        );
        assert!(!findings.iter().any(|f| f.rule_id == "SEC008"));
        assert!(!findings.iter().any(|f| f.rule_id == "SEC009"));
        assert!(!findings.iter().any(|f| f.rule_id == "SEC010"));
        assert!(
            merged.contains("my-repo"),
            "original user content must survive the merge"
        );
    }

    /// Contract test: every rule_id the six kept categories (docs, files, git,
    /// codeowners, security, metadata) can emit must either have a remediation
    /// path (an `Action` operation that addresses it) or appear on an explicit,
    /// documented detection-only allowlist. This fails loudly if a future
    /// rule_id is added without wiring up a remediation or consciously marking
    /// it detection-only — i.e. "zero report-only by accident".
    #[test]
    fn test_no_kept_rule_is_report_only() {
        use std::collections::BTreeSet;

        // Every rule_id the kept categories actually emit today. Verified
        // against src/rules/categories/{docs,files,git,codeowners,security,
        // metadata}.rs (Finding::new call sites).
        let kept_rule_ids: BTreeSet<&str> = [
            // docs.rs
            "DOC001",
            "DOC002",
            "DOC003",
            "DOC004",
            "DOC005",
            "DOC006",
            "DOC007",
            "DOC008",
            "DOC009",
            "DOC010",
            // files.rs
            "FILE002",
            "FILE003",
            // git.rs
            "GIT002",
            "GIT003",
            // codeowners.rs (CODE003 is defined but never emitted)
            "CODE001",
            "CODE002",
            // metadata.rs
            "META001",
            "META002",
            "META003",
            // security.rs
            "SECURITY003",
            "SEC007",
            "SEC008",
            "SEC009",
            "SEC010",
            "SEC011",
            "SEC012",
            "SEC013",
            "SEC014",
            "SEC015",
            "SEC016",
            "SEC017",
        ]
        .into_iter()
        .collect();

        // Rule ids that have a concrete remediation path wired into the planner /
        // executor (operation type noted in the comment).
        let remediable: BTreeSet<&str> = [
            // CreateFile from template
            "DOC001",  // README.md
            "DOC004",  // LICENSE
            "DOC005",  // CONTRIBUTING.md
            "DOC006",  // CODE_OF_CONDUCT.md
            "DOC007",  // SECURITY.md
            "DOC008",  // CHANGELOG.md
            "GIT002",  // .gitattributes
            "CODE001", // CODEOWNERS
            "SEC007",  // .github/settings.yml (freshly created, already satisfies SEC008-010)
            "FILE002", // .gitignore missing entirely -> create it (review bug #13)
            // UpdateGitignore (executor creates or appends to .gitignore)
            "FILE003", // .gitignore missing recommended entry -> append
            "GIT003",  // sensitive file untracked -> add to .gitignore
            // UpdateSettingsFile (merges missing sections into an EXISTING
            // .github/settings.yml -- review bug #12; NOT
            // ConfigureProtectedBranch, which configures live branch
            // protection via the provider API and does not touch this file)
            "SEC008", "SEC009", "SEC010",
            // UpdateRepoSettings (GitHub repo-settings toggles: discussions
            // / issues / wiki / vulnerability alerts / automated security
            // fixes)
            "SEC011", "SEC012",
            // UpdateActionsSecuritySettings (GitHub `security_and_analysis`
            // + Actions-permissions API writes -- review bug #11; SEC013-017
            // were previously listed here as "remediable via UpdateRepoSettings"
            // but NO planner mapping actually existed for them until this
            // action was added: see providers::RepoProvider::set_secret_scanning
            // (SEC013/SEC014), ::set_actions_permissions (SEC015),
            // ::set_actions_workflow_permissions (SEC016), and
            // ::set_fork_pr_workflows_policy (SEC017), wired via
            // ActionPlanner::plan_actions_security_if_needed)
            "SEC013", "SEC014", "SEC015", "SEC016", "SEC017", // UpdateRepoMetadata
            "META001", "META002", "META003",
        ]
        .into_iter()
        .collect();

        // Detection-only: no deterministic auto-fix exists, so these are reported
        // for a human to act on. Each entry carries a justification.
        let detection_only_allowlist: BTreeSet<&str> = [
            "DOC002",      // README too short — quality judgement, no canonical fix.
            "DOC003",      // README missing a section — content is author-specific.
            "DOC009",      // CHANGELOG not Keep-a-Changelog — reformatting existing prose.
            "DOC010",      // CHANGELOG empty Unreleased section — needs real change notes.
            "CODE002",     // CODEOWNERS syntax error — fix depends on intended owners.
            "SECURITY003", // No runtime version file — the version is a project decision.
        ]
        .into_iter()
        .collect();

        // 1. The two dispositions must be mutually exclusive.
        let overlap: Vec<&&str> = remediable.intersection(&detection_only_allowlist).collect();
        assert!(
            overlap.is_empty(),
            "rule_ids classified as both remediable and detection-only: {:?}",
            overlap
        );

        // 2. No stale ids: everything we classify must still be emitted.
        let union: BTreeSet<&str> = remediable
            .union(&detection_only_allowlist)
            .copied()
            .collect();
        let stale: Vec<&&str> = union.difference(&kept_rule_ids).collect();
        assert!(
            stale.is_empty(),
            "rule_ids classified but no longer emitted by kept categories: {:?}",
            stale
        );

        // 3. Full coverage: every emitted rule_id has a disposition.
        let uncovered: Vec<&&str> = kept_rule_ids.difference(&union).collect();
        assert!(
            uncovered.is_empty(),
            "kept rule_ids with neither a remediation nor an allowlist entry: {:?}",
            uncovered
        );
    }
}
