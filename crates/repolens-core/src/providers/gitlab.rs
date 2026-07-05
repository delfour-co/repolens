//! GitLab provider - Interactions with the GitLab API via the `glab` CLI.
//!
//! This module mirrors the GitHub provider's mechanism: it shells out to the
//! `glab` CLI (rather than embedding an HTTP client) and parses the JSON it
//! emits. `glab` honours the `GITLAB_TOKEN` environment variable itself, so no
//! token handling lives here.
//!
//! ## Capability gaps
//!
//! Several methods on [`RepoProvider`] describe GitHub-specific features
//! (Dependabot security updates, automated security fixes, GitHub Actions
//! permissions, fork-PR approval policy) with no faithful GitLab equivalent.
//! Those methods return `Err(...)`, which the kept `security`/`metadata` checks
//! already treat as "skip, emit no finding". Returning `Ok(false)`/`Ok(default)`
//! would emit a *false* finding (e.g. "vulnerability alerts disabled") on every
//! GitLab repository, so it is deliberately avoided.

use std::process::Command;

use serde::Deserialize;

use crate::error::{ProviderError, RepoLensError};
use crate::providers::{
    ActionsPermissions, BranchProtection, RepoInfo, RepoMetadata, RepoProvider,
    SecretScanningSettings,
};

/// GitLab provider for repository read operations.
///
/// Resolves the project slug (`namespace/path`) from the git `origin` remote and
/// reads project state through the `glab` CLI.
pub struct GitLabProvider {
    /// Namespace (group / user) — the part before the final `/`.
    namespace: String,
    /// Project path — the final path segment.
    project: String,
}

/// A single `glab api` call to issue: `(method, path, "-f key=value" fields)`.
type GlabApiCall = (&'static str, String, Vec<(String, String)>);

/// GitLab project metadata wire format (`GET /projects/:id`).
#[derive(Debug, Deserialize)]
struct GitLabProject {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    topics: Vec<String>,
    /// Configured project homepage, if any. Older GitLab versions omit it.
    #[serde(default)]
    homepage: Option<String>,
    /// Web URL of the project (always present). Used as a homepage fallback only
    /// when an explicit homepage is set; the project page itself is not treated
    /// as a "homepage" so the META003 contract matches GitHub's.
    #[serde(default)]
    #[allow(dead_code)]
    web_url: Option<String>,
    /// Whether issues are enabled. Newer GitLab exposes `issues_access_level`
    /// (`enabled` | `private` | `disabled`); older versions use `issues_enabled`.
    #[serde(default)]
    issues_enabled: Option<bool>,
    #[serde(default)]
    issues_access_level: Option<String>,
    /// Whether the wiki is enabled (`wiki_enabled` / `wiki_access_level`).
    #[serde(default)]
    wiki_enabled: Option<bool>,
    #[serde(default)]
    wiki_access_level: Option<String>,
}

impl GitLabProject {
    fn issues_on(&self) -> bool {
        if let Some(level) = &self.issues_access_level {
            return level != "disabled";
        }
        self.issues_enabled.unwrap_or(false)
    }

    fn wiki_on(&self) -> bool {
        if let Some(level) = &self.wiki_access_level {
            return level != "disabled";
        }
        self.wiki_enabled.unwrap_or(false)
    }
}

/// GitLab protected-branch wire format
/// (`GET /projects/:id/protected_branches/:name`).
#[derive(Debug, Deserialize)]
struct GitLabProtectedBranch {
    #[serde(default)]
    #[allow(dead_code)]
    name: Option<String>,
    /// Whether force pushes are allowed on the protected branch.
    #[serde(default)]
    allow_force_push: Option<bool>,
}

/// GitLab approval rule wire format (`GET /projects/:id/approval_rules`).
#[derive(Debug, Deserialize)]
struct GitLabApprovalRule {
    #[serde(default)]
    approvals_required: Option<u32>,
}

impl GitLabProvider {
    /// Create a new GitLab provider for the current repository.
    ///
    /// Resolves the project slug from the git `origin` remote. `glab` reads
    /// `GITLAB_TOKEN` from the environment itself, so no token handling is done
    /// here.
    pub fn new() -> Result<Self, RepoLensError> {
        let (namespace, project) = Self::get_project_info()?;
        Ok(Self { namespace, project })
    }

    /// Check whether the `glab` CLI is installed.
    ///
    /// Mirrors [`crate::providers::github::GitHubProvider::is_available`]'s
    /// shape. Note (documented gap): unlike GitHub's token-without-CLI mode,
    /// GitLab requires `glab` to be installed even when `GITLAB_TOKEN` is set —
    /// `glab` is the only transport. Absence is a graceful skip, not a hard
    /// failure.
    pub fn is_available() -> bool {
        Command::new("glab")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Resolve the project slug (namespace/path) from the git `origin` remote.
    fn get_project_info() -> Result<(String, String), RepoLensError> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: "git remote get-url origin".to_string(),
                })
            })?;

        if !output.status.success() {
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: "git remote get-url origin".to_string(),
            }));
        }

        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Self::parse_gitlab_url(&url)
    }

    /// Parse a GitLab remote URL into `(namespace, project)`.
    ///
    /// Handles both SSH (`git@gitlab.com:group/sub/project.git`) and HTTPS
    /// (`https://gitlab.com/group/sub/project.git`) forms. GitLab supports
    /// arbitrarily nested subgroups, so the namespace is everything before the
    /// final path segment.
    fn parse_gitlab_url(url: &str) -> Result<(String, String), RepoLensError> {
        // Any scheme-based URL (`https://`, `ssh://`, `git://`, …) is checked
        // FIRST: the path is everything after the first `/` following the
        // authority (`[user@]host[:port]`). Checking this before the classic
        // SCP-like heuristic below fixes review bug #6 — `ssh://git@host:PORT/…`
        // used to match the SCP-like branch (which just looks for a `:` after
        // the first `@`) and fold the port into the parsed namespace.
        let path = if let Some(idx) = url.find("://") {
            let after_scheme = &url[idx + 3..];
            let slash = after_scheme.find('/').ok_or_else(|| {
                RepoLensError::Provider(ProviderError::InvalidRepoName {
                    name: url.to_string(),
                })
            })?;
            after_scheme[slash + 1..].trim_end_matches(".git")
        } else if let Some(idx) = url.find('@').and_then(|at| {
            // Classic SCP-like form (`git@host:path`, no scheme). Only reached
            // when there is no `scheme://` prefix.
            url[at..].find(':').map(|c| at + c + 1)
        }) {
            url[idx..].trim_end_matches(".git")
        } else {
            return Err(RepoLensError::Provider(ProviderError::InvalidRepoName {
                name: url.to_string(),
            }));
        };

        Self::split_project_path(path)
    }

    /// Split a project path (`group/sub/project`) into `(namespace, project)`.
    fn split_project_path(path: &str) -> Result<(String, String), RepoLensError> {
        let path = path.trim_matches('/');
        let (namespace, project) = path.rsplit_once('/').ok_or_else(|| {
            RepoLensError::Provider(ProviderError::InvalidRepoName {
                name: path.to_string(),
            })
        })?;

        if namespace.is_empty() || project.is_empty() {
            return Err(RepoLensError::Provider(ProviderError::InvalidRepoName {
                name: path.to_string(),
            }));
        }

        Ok((namespace.to_string(), project.to_string()))
    }

    /// Full project slug (`namespace/project`).
    fn full_path(&self) -> String {
        format!("{}/{}", self.namespace, self.project)
    }

    /// URL-encoded project id for use in `glab api projects/:id` paths.
    ///
    /// GitLab accepts the URL-encoded `namespace/project` path as the `:id`.
    fn encoded_id(&self) -> String {
        urlencode(&self.full_path())
    }

    /// Run `glab api <path>` and return raw stdout on HTTP 2xx, or an error.
    fn glab_api(&self, path: &str) -> Result<Vec<u8>, RepoLensError> {
        let output = Command::new("glab")
            .args(["api", path])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!("glab api {path}"),
                })
            })?;

        if !output.status.success() {
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("glab api {path}"),
            }));
        }

        Ok(output.stdout)
    }

    /// GitLab access levels used for `push_access_level` / `merge_access_level`
    /// on the protected-branches API. See
    /// <https://docs.gitlab.com/ee/api/members.html#valid-access-levels>.
    const ACCESS_LEVEL_DEVELOPER: &'static str = "30";
    const ACCESS_LEVEL_MAINTAINER: &'static str = "40";

    /// Build the `-f key=value` fields to send when creating (`creating =
    /// true`, via `POST`) or updating (`creating = false`, via `PATCH`) a
    /// protected branch.
    ///
    /// Fixes review bug #3: the previous implementation sent only `name` +
    /// `allow_force_push`, yet returned `Ok`, silently claiming full
    /// protection. Every field GitLab's protected-branches API genuinely
    /// supports is now sent:
    /// - `allow_force_push` from `block_force_push`.
    /// - `push_access_level` fixed to Maintainer, so direct pushes require the
    ///   same trust level GitHub's `enforce_admins` intends (changes go
    ///   through a merge request).
    /// - `merge_access_level` fixed to Developer, so approved merge requests
    ///   can be merged.
    /// - `code_owner_approval_required` from `required_approvals > 0` (the
    ///   closest GitLab protected-branch concept to "require reviews before
    ///   merging").
    ///
    /// GitHub's `require_status_checks`, `require_linear_history`,
    /// `require_conversation_resolution`, `block_deletions`, and
    /// `require_signed_commits` have no GitLab protected-branch equivalent
    /// (see the module docs) and are intentionally not sent.
    fn protection_fields(
        branch: &str,
        settings: &crate::actions::plan::BranchProtectionSettings,
        creating: bool,
    ) -> Vec<(String, String)> {
        let mut fields = Vec::new();
        if creating {
            fields.push(("name".to_string(), branch.to_string()));
        }
        fields.push((
            "allow_force_push".to_string(),
            (!settings.block_force_push).to_string(),
        ));
        fields.push((
            "push_access_level".to_string(),
            Self::ACCESS_LEVEL_MAINTAINER.to_string(),
        ));
        fields.push((
            "merge_access_level".to_string(),
            Self::ACCESS_LEVEL_DEVELOPER.to_string(),
        ));
        fields.push((
            "code_owner_approval_required".to_string(),
            (settings.required_approvals > 0).to_string(),
        ));
        fields
    }

    /// Build the sequence of `glab api` calls needed to apply `settings` as
    /// protected-branch settings for `branch`, given whether the branch is
    /// already protected.
    ///
    /// Fixes review bug #4: the previous implementation always issued a
    /// best-effort DELETE before the POST, leaving the branch briefly — and,
    /// if the POST then failed, permanently — unprotected. This now issues
    /// exactly ONE call: `PATCH` (idempotent update) when the branch is
    /// already protected, or `POST` (create) when it is not. Nothing is ever
    /// deleted.
    fn protect_branch_calls(
        branch: &str,
        settings: &crate::actions::plan::BranchProtectionSettings,
        encoded_id: &str,
        already_protected: bool,
    ) -> Vec<GlabApiCall> {
        let fields = Self::protection_fields(branch, settings, !already_protected);
        if already_protected {
            let path = format!(
                "projects/{encoded_id}/protected_branches/{}",
                urlencode(branch)
            );
            vec![("PATCH", path, fields)]
        } else {
            let path = format!("projects/{encoded_id}/protected_branches");
            vec![("POST", path, fields)]
        }
    }

    /// Parse project metadata from raw `glab api projects/:id` JSON.
    fn parse_metadata(bytes: &[u8]) -> Result<RepoMetadata, RepoLensError> {
        let project: GitLabProject = serde_json::from_slice(bytes)?;
        // GitHub's RepoMetadata represents "missing" as None / empty Vec. Mirror
        // that: only treat a non-empty homepage as set. A configured homepage is
        // used as-is; the project `web_url` is NOT substituted, so META003's
        // semantics match GitHub (the project page is not a "homepage").
        let homepage = project.homepage.filter(|h| !h.trim().is_empty());

        Ok(RepoMetadata {
            description: project.description.filter(|d| !d.trim().is_empty()),
            topics: project.topics,
            homepage,
            has_pages: None,
        })
    }
}

impl RepoProvider for GitLabProvider {
    fn owner(&self) -> &str {
        &self.namespace
    }

    fn name(&self) -> &str {
        &self.project
    }

    /// Fetch project metadata (description, topics, homepage) via
    /// `glab api projects/:id`.
    fn repo_metadata(&self) -> Result<RepoMetadata, RepoLensError> {
        let bytes = self.glab_api(&format!("projects/{}", self.encoded_id()))?;
        Self::parse_metadata(&bytes)
    }

    /// Branch protection settings for `branch`, built from GitLab's protected
    /// branches (+ approval rules for the required-approval count when reachable).
    ///
    /// Returns `Ok(None)` when the branch is unprotected (GitLab returns 404),
    /// and `Err` only on a real failure.
    fn get_branch_protection(
        &self,
        branch: &str,
    ) -> Result<Option<BranchProtection>, RepoLensError> {
        let path = format!(
            "projects/{}/protected_branches/{}",
            self.encoded_id(),
            urlencode(branch)
        );
        let output = Command::new("glab")
            .args(["api", &path])
            .output()
            .map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!("glab api {path}"),
                })
            })?;

        // A non-success exit means the branch is not protected (404) — mirror the
        // GitHub provider, which treats that as "no protection".
        if !output.status.success() {
            return Ok(None);
        }

        let protected: GitLabProtectedBranch = serde_json::from_slice(&output.stdout)?;

        // Required-approval count comes from a separate endpoint; best-effort.
        let required_approvals = self.approval_rules_required().unwrap_or(0);

        Ok(Some(BranchProtection::from_gitlab(
            protected.allow_force_push.unwrap_or(true),
            required_approvals,
        )))
    }

    /// Repository settings (issues / wiki enablement) from `glab api projects/:id`.
    ///
    /// GitLab has no "discussions" concept, so that field is left `false`; the
    /// kept checks only read issues/wiki here.
    fn get_repo_settings(&self) -> Result<RepoInfo, RepoLensError> {
        let bytes = self.glab_api(&format!("projects/{}", self.encoded_id()))?;
        let project: GitLabProject = serde_json::from_slice(&bytes)?;
        Ok(RepoInfo::from_gitlab(
            &self.project,
            &self.namespace,
            project.issues_on(),
            project.wiki_on(),
        ))
    }

    /// GitHub-specific: GitLab's vulnerability/dependency surface (security
    /// dashboard) has no faithful 1:1 of GitHub's repo-level "vulnerability
    /// alerts" toggle. Return `Err` so the caller skips and emits no finding.
    fn has_vulnerability_alerts(&self) -> Result<bool, RepoLensError> {
        Err(RepoLensError::Provider(ProviderError::CommandFailed {
            command: "vulnerability-alerts: unsupported on GitLab".to_string(),
        }))
    }

    /// GitHub-specific (Dependabot). No GitLab equivalent — skip via `Err`.
    fn has_automated_security_fixes(&self) -> Result<bool, RepoLensError> {
        Err(RepoLensError::Provider(ProviderError::CommandFailed {
            command: "automated-security-fixes: unsupported on GitLab".to_string(),
        }))
    }

    /// GitHub-specific (Dependabot). No GitLab equivalent — skip via `Err`.
    fn has_dependabot_security_updates(&self) -> Result<bool, RepoLensError> {
        Err(RepoLensError::Provider(ProviderError::CommandFailed {
            command: "dependabot-security-updates: unsupported on GitLab".to_string(),
        }))
    }

    /// GitHub-specific (secret scanning toggle). GitLab Secret Detection is a
    /// CI-job concept, not a repo setting matching this shape — skip via `Err`
    /// so no SEC013/SEC014 false positive is emitted.
    fn get_secret_scanning(&self) -> Result<SecretScanningSettings, RepoLensError> {
        Err(RepoLensError::Provider(ProviderError::CommandFailed {
            command: "secret-scanning: unsupported on GitLab".to_string(),
        }))
    }

    /// GitHub Actions concept. GitLab CI has no equivalent permissions shape —
    /// skip via `Err`.
    fn get_actions_permissions(&self) -> Result<ActionsPermissions, RepoLensError> {
        Err(RepoLensError::Provider(ProviderError::CommandFailed {
            command: "actions-permissions: unsupported on GitLab".to_string(),
        }))
    }

    /// GitHub Actions workflow-token permissions. No GitLab equivalent — skip.
    fn get_actions_workflow_permissions(&self) -> Result<ActionsPermissions, RepoLensError> {
        Err(RepoLensError::Provider(ProviderError::CommandFailed {
            command: "workflow-permissions: unsupported on GitLab".to_string(),
        }))
    }

    /// GitHub Actions fork-PR approval policy. No GitLab equivalent — skip.
    fn get_fork_pr_workflows_policy(&self) -> Result<bool, RepoLensError> {
        Err(RepoLensError::Provider(ProviderError::CommandFailed {
            command: "fork-pr-approval: unsupported on GitLab".to_string(),
        }))
    }

    /// Protect `branch` via GitLab's protected-branches API.
    ///
    /// GitLab maps a subset of the GitHub branch-protection model — see
    /// [`Self::protection_fields`] for exactly which fields are sent and why.
    /// The branch is created via `POST /projects/:id/protected_branches` when
    /// unprotected, or updated in place via
    /// `PATCH /projects/:id/protected_branches/:name` when already protected
    /// (see [`Self::protect_branch_calls`]) — never deleted and recreated.
    fn set_protected_branch(
        &self,
        branch: &str,
        settings: &crate::actions::plan::BranchProtectionSettings,
    ) -> Result<(), RepoLensError> {
        // Determine whether the branch is already protected so a single
        // idempotent call (PATCH to update, POST to create) can be issued —
        // see `protect_branch_calls` (review bug #4: no DELETE is ever sent).
        let already_protected = matches!(self.get_branch_protection(branch), Ok(Some(_)));
        let calls =
            Self::protect_branch_calls(branch, settings, &self.encoded_id(), already_protected);

        for (method, path, fields) in calls {
            let mut args: Vec<String> = vec![
                "api".to_string(),
                path.clone(),
                "--method".to_string(),
                method.to_string(),
            ];
            for (key, value) in &fields {
                args.push("-f".to_string());
                args.push(format!("{key}={value}"));
            }

            let output = Command::new("glab").args(&args).output().map_err(|_| {
                RepoLensError::Provider(ProviderError::CommandFailed {
                    command: format!("glab api {path}"),
                })
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(RepoLensError::Action(
                    crate::error::ActionError::ExecutionFailed {
                        message: format!("Failed to protect branch on GitLab: {stderr}"),
                    },
                ));
            }
        }

        Ok(())
    }

    /// Apply repository settings via `PUT /projects/:id`.
    ///
    /// GitLab has no faithful equivalent of GitHub's `enable_discussions` /
    /// `enable_vulnerability_alerts` / `enable_automated_security_fixes`
    /// toggles (see the module docs) — requesting one of those returns an
    /// honest `Err` rather than silently doing nothing (review bug #9's
    /// counterpart to bug #10: never claim `Ok` on an unperformed request).
    /// `enable_issues` / `enable_wiki` ARE genuinely settable and are applied.
    fn set_repo_settings(
        &self,
        settings: &crate::actions::plan::GitHubRepoSettings,
    ) -> Result<(), RepoLensError> {
        let fields = Self::repo_settings_fields(settings);
        let unsupported_requested = Self::requests_unsupported_settings(settings);

        if fields.is_empty() {
            return if unsupported_requested {
                Err(RepoLensError::Provider(ProviderError::CommandFailed {
                    command: "repo-settings: discussions/vulnerability-alerts/automated-security-fixes are unsupported on GitLab".to_string(),
                }))
            } else {
                Ok(())
            };
        }

        let path = format!("projects/{}", self.encoded_id());
        let mut args: Vec<String> = vec![
            "api".to_string(),
            path.clone(),
            "--method".to_string(),
            "PUT".to_string(),
        ];
        for (key, value) in &fields {
            args.push("-f".to_string());
            args.push(format!("{key}={value}"));
        }

        let output = Command::new("glab").args(&args).output().map_err(|_| {
            RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("glab api {path}"),
            })
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RepoLensError::Action(
                crate::error::ActionError::ExecutionFailed {
                    message: format!("Failed to update GitLab project settings: {stderr}"),
                },
            ));
        }

        if unsupported_requested {
            // The supported fields (issues/wiki) were applied above, but the
            // request also asked for a GitHub-only toggle — report that
            // honestly instead of claiming full success.
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: "repo-settings: discussions/vulnerability-alerts/automated-security-fixes are unsupported on GitLab".to_string(),
            }));
        }

        Ok(())
    }

    /// Apply project metadata (description, topics, homepage) via
    /// `PUT /projects/:id` (`glab api`).
    fn set_repo_metadata(
        &self,
        description: Option<&str>,
        topics: &[String],
        homepage: Option<&str>,
    ) -> Result<(), RepoLensError> {
        let path = format!("projects/{}", self.encoded_id());
        let mut args: Vec<String> = vec![
            "api".to_string(),
            path.clone(),
            "--method".to_string(),
            "PUT".to_string(),
        ];

        if let Some(desc) = description {
            args.push("-f".to_string());
            args.push(format!("description={desc}"));
        }
        if let Some(home) = homepage {
            // GitLab stores the project website under `homepage`.
            args.push("-f".to_string());
            args.push(format!("homepage={home}"));
        }
        if !topics.is_empty() {
            // GitLab accepts a comma-separated `topics` field on project update.
            args.push("-f".to_string());
            args.push(format!("topics={}", topics.join(",")));
        }

        // Nothing to update.
        if args.len() == 4 {
            return Ok(());
        }

        let output = Command::new("glab").args(&args).output().map_err(|_| {
            RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("glab api {path}"),
            })
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RepoLensError::Action(
                crate::error::ActionError::ExecutionFailed {
                    message: format!("Failed to update GitLab project metadata: {stderr}"),
                },
            ));
        }

        Ok(())
    }

    /// Create an issue via `glab issue create`.
    ///
    /// Foundation for review bug #8 (`apply --create-pr` hardcoded
    /// `GitHubProvider`, orphaning the request on GitLab).
    fn create_issue(
        &self,
        title: &str,
        body: &str,
        labels: &[&str],
    ) -> Result<String, RepoLensError> {
        let args = Self::issue_create_args(title, body, labels);
        let output = Command::new("glab").args(&args).output().map_err(|_| {
            RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("glab {}", args.join(" ")),
            })
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("Failed to create GitLab issue: {stderr}"),
            }));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Open a merge request via `glab mr create`.
    ///
    /// Foundation for review bug #8: this is GitLab's counterpart to
    /// `GitHubProvider::create_pull_request`.
    fn open_change_request(
        &self,
        title: &str,
        body: &str,
        head: &str,
        base: Option<&str>,
    ) -> Result<String, RepoLensError> {
        let args = Self::mr_create_args(title, body, head, base);
        let output = Command::new("glab").args(&args).output().map_err(|_| {
            RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("glab {}", args.join(" ")),
            })
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RepoLensError::Provider(ProviderError::CommandFailed {
                command: format!("Failed to open GitLab merge request: {stderr}"),
            }));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl GitLabProvider {
    /// Build the `glab issue create` args for `(title, body, labels)`.
    fn issue_create_args(title: &str, body: &str, labels: &[&str]) -> Vec<String> {
        let mut args = vec![
            "issue".to_string(),
            "create".to_string(),
            "--title".to_string(),
            title.to_string(),
            "--description".to_string(),
            body.to_string(),
            "--yes".to_string(),
        ];
        if !labels.is_empty() {
            args.push("--label".to_string());
            args.push(labels.join(","));
        }
        args
    }

    /// Build the `glab mr create` args for `(title, body, head, base)`.
    fn mr_create_args(title: &str, body: &str, head: &str, base: Option<&str>) -> Vec<String> {
        let mut args = vec![
            "mr".to_string(),
            "create".to_string(),
            "--title".to_string(),
            title.to_string(),
            "--description".to_string(),
            body.to_string(),
            "--source-branch".to_string(),
            head.to_string(),
            "--yes".to_string(),
        ];
        if let Some(base_branch) = base {
            args.push("--target-branch".to_string());
            args.push(base_branch.to_string());
        }
        args
    }

    /// Best-effort read of the highest required-approval count from the
    /// project's approval rules. Returns `None` if the endpoint is unreachable
    /// (e.g. GitLab CE without merge-request approvals).
    fn approval_rules_required(&self) -> Option<u32> {
        let bytes = self
            .glab_api(&format!("projects/{}/approval_rules", self.encoded_id()))
            .ok()?;
        let rules: Vec<GitLabApprovalRule> = serde_json::from_slice(&bytes).ok()?;
        rules.iter().filter_map(|r| r.approvals_required).max()
    }

    /// Fields settable via `PUT /projects/:id` for the issues/wiki toggles.
    ///
    /// Fixes review bug #9: `set_repo_settings` used to return `Err`
    /// unconditionally, even for `enable_issues`/`enable_wiki`, which GitLab
    /// genuinely supports via the project's `issues_access_level` /
    /// `wiki_access_level` fields.
    fn repo_settings_fields(
        settings: &crate::actions::plan::GitHubRepoSettings,
    ) -> Vec<(String, String)> {
        let mut fields = Vec::new();
        if let Some(enable) = settings.enable_issues {
            let level = if enable { "enabled" } else { "disabled" };
            fields.push(("issues_access_level".to_string(), level.to_string()));
        }
        if let Some(enable) = settings.enable_wiki {
            let level = if enable { "enabled" } else { "disabled" };
            fields.push(("wiki_access_level".to_string(), level.to_string()));
        }
        fields
    }

    /// Whether `settings` requests a toggle GitLab has no equivalent for
    /// (discussions, vulnerability alerts, automated security fixes — see
    /// the module docs). `enable_issues`/`enable_wiki` are excluded: those
    /// ARE settable (see [`Self::repo_settings_fields`]).
    fn requests_unsupported_settings(settings: &crate::actions::plan::GitHubRepoSettings) -> bool {
        settings.enable_discussions.is_some()
            || settings.enable_vulnerability_alerts.is_some()
            || settings.enable_automated_security_fixes.is_some()
    }
}

/// Minimal percent-encoding for path segments used in `glab api` URLs.
///
/// Encodes `/` (so a nested project path is a single `:id` segment) and a few
/// other reserved characters. GitLab accepts the encoded path as the project id.
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_provider() -> GitLabProvider {
        GitLabProvider {
            namespace: "group".to_string(),
            project: "repo".to_string(),
        }
    }

    #[test]
    fn test_full_path_and_accessors() {
        let p = test_provider();
        assert_eq!(p.full_path(), "group/repo");
        assert_eq!(p.owner(), "group");
        assert_eq!(p.name(), "repo");
    }

    #[test]
    fn test_encoded_id_encodes_slash() {
        let p = test_provider();
        assert_eq!(p.encoded_id(), "group%2Frepo");
    }

    #[test]
    fn test_parse_gitlab_url_https() {
        let (ns, proj) =
            GitLabProvider::parse_gitlab_url("https://gitlab.com/group/repo.git").unwrap();
        assert_eq!(ns, "group");
        assert_eq!(proj, "repo");
    }

    #[test]
    fn test_parse_gitlab_url_https_no_git() {
        let (ns, proj) = GitLabProvider::parse_gitlab_url("https://gitlab.com/group/repo").unwrap();
        assert_eq!(ns, "group");
        assert_eq!(proj, "repo");
    }

    #[test]
    fn test_parse_gitlab_url_ssh() {
        let (ns, proj) = GitLabProvider::parse_gitlab_url("git@gitlab.com:group/repo.git").unwrap();
        assert_eq!(ns, "group");
        assert_eq!(proj, "repo");
    }

    #[test]
    fn test_parse_gitlab_url_nested_subgroups() {
        // Namespace is everything before the final path segment.
        let (ns, proj) =
            GitLabProvider::parse_gitlab_url("https://gitlab.com/group/sub/repo.git").unwrap();
        assert_eq!(ns, "group/sub");
        assert_eq!(proj, "repo");

        let (ns, proj) =
            GitLabProvider::parse_gitlab_url("git@gitlab.example.com:a/b/c/repo.git").unwrap();
        assert_eq!(ns, "a/b/c");
        assert_eq!(proj, "repo");
    }

    #[test]
    fn test_parse_gitlab_url_self_managed_host() {
        let (ns, proj) =
            GitLabProvider::parse_gitlab_url("https://gitlab.internal.corp/team/svc.git").unwrap();
        assert_eq!(ns, "team");
        assert_eq!(proj, "svc");
    }

    #[test]
    fn test_parse_gitlab_url_ssh_custom_port() {
        // Regression test for review bug #6: an `ssh://` URL with a custom
        // port must not have the port folded into the namespace.
        let (ns, proj) =
            GitLabProvider::parse_gitlab_url("ssh://git@gitlab.example.com:2222/group/repo.git")
                .unwrap();
        assert_eq!(ns, "group");
        assert_eq!(proj, "repo");
    }

    #[test]
    fn test_parse_gitlab_url_invalid() {
        assert!(GitLabProvider::parse_gitlab_url("not-a-url").is_err());
        assert!(GitLabProvider::parse_gitlab_url("https://gitlab.com/onlyone").is_err());
    }

    #[test]
    fn test_parse_metadata_full() {
        let json = br#"{
            "description": "A GitLab project",
            "topics": ["rust", "cli"],
            "homepage": "https://example.com",
            "web_url": "https://gitlab.com/group/repo"
        }"#;
        let md = GitLabProvider::parse_metadata(json).unwrap();
        assert_eq!(md.description.as_deref(), Some("A GitLab project"));
        assert_eq!(md.topics, vec!["rust".to_string(), "cli".to_string()]);
        assert_eq!(md.homepage.as_deref(), Some("https://example.com"));
        assert_eq!(md.has_pages, None);
    }

    #[test]
    fn test_parse_metadata_missing_fields() {
        // GitLab omits homepage on older versions; description may be null.
        let json = br#"{
            "description": null,
            "web_url": "https://gitlab.com/group/repo"
        }"#;
        let md = GitLabProvider::parse_metadata(json).unwrap();
        assert_eq!(md.description, None);
        assert!(md.topics.is_empty());
        // No explicit homepage -> missing (not the web_url).
        assert_eq!(md.homepage, None);
    }

    #[test]
    fn test_parse_metadata_empty_strings_are_missing() {
        let json = br#"{
            "description": "   ",
            "topics": [],
            "homepage": ""
        }"#;
        let md = GitLabProvider::parse_metadata(json).unwrap();
        assert_eq!(md.description, None);
        assert_eq!(md.homepage, None);
    }

    #[test]
    fn test_parse_protected_branch_allow_force_push() {
        let json = br#"{
            "name": "main",
            "push_access_levels": [{"access_level": 40}],
            "merge_access_levels": [{"access_level": 40}],
            "allow_force_push": false,
            "code_owner_approval_required": true
        }"#;
        let pb: GitLabProtectedBranch = serde_json::from_slice(json).unwrap();
        assert_eq!(pb.name.as_deref(), Some("main"));
        assert_eq!(pb.allow_force_push, Some(false));

        // And it maps into the shared BranchProtection shape.
        let bp = BranchProtection::from_gitlab(pb.allow_force_push.unwrap_or(true), 2);
        assert!(bp.allow_force_pushes.is_some());
        assert!(!bp.allow_force_pushes.as_ref().unwrap().enabled);
        assert_eq!(
            bp.required_pull_request_reviews
                .as_ref()
                .unwrap()
                .required_approving_review_count,
            2
        );
    }

    #[test]
    fn test_parse_approval_rules_picks_max() {
        let json = br#"[
            {"name": "rule-a", "approvals_required": 1},
            {"name": "rule-b", "approvals_required": 3}
        ]"#;
        let rules: Vec<GitLabApprovalRule> = serde_json::from_slice(json).unwrap();
        let max = rules.iter().filter_map(|r| r.approvals_required).max();
        assert_eq!(max, Some(3));
    }

    #[test]
    fn test_project_issues_and_wiki_access_levels() {
        let json = br#"{
            "issues_access_level": "enabled",
            "wiki_access_level": "disabled"
        }"#;
        let project: GitLabProject = serde_json::from_slice(json).unwrap();
        assert!(project.issues_on());
        assert!(!project.wiki_on());
    }

    #[test]
    fn test_project_issues_and_wiki_boolean_fallback() {
        let json = br#"{
            "issues_enabled": false,
            "wiki_enabled": true
        }"#;
        let project: GitLabProject = serde_json::from_slice(json).unwrap();
        assert!(!project.issues_on());
        assert!(project.wiki_on());
    }

    /// Regression test for review bug #3: `set_protected_branch` sent only
    /// `name` + `allow_force_push` (2 of the settable GitLab protected-branch
    /// fields) yet returned `Ok`, silently claiming full protection.
    #[test]
    fn test_gitlab_protect_branch_sends_full_settable_field_set() {
        let settings = crate::actions::plan::BranchProtectionSettings {
            required_approvals: 2,
            block_force_push: true,
            ..Default::default()
        };
        let calls = GitLabProvider::protect_branch_calls("main", &settings, "group%2Frepo", false);
        let (_, _, fields) = calls
            .iter()
            .find(|(method, _, _)| *method == "POST" || *method == "PATCH")
            .expect("expected a POST or PATCH call");
        let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"allow_force_push"));
        assert!(
            keys.contains(&"push_access_level"),
            "expected push_access_level, got {keys:?}"
        );
        assert!(
            keys.contains(&"merge_access_level"),
            "expected merge_access_level, got {keys:?}"
        );
        assert!(
            keys.contains(&"code_owner_approval_required"),
            "expected code_owner_approval_required, got {keys:?}"
        );
    }

    /// Regression test for review bug #4: a DELETE issued before the POST
    /// left the branch unprotected if the POST then failed. The fixed
    /// implementation must never construct a DELETE call — it uses PATCH
    /// (idempotent update) when the branch is already protected, or POST
    /// (create) when it is not.
    #[test]
    fn test_gitlab_protect_branch_never_issues_delete() {
        let settings = crate::actions::plan::BranchProtectionSettings::default();
        for already_protected in [true, false] {
            let calls = GitLabProvider::protect_branch_calls(
                "main",
                &settings,
                "group%2Frepo",
                already_protected,
            );
            assert!(
                calls.iter().all(|(method, _, _)| *method != "DELETE"),
                "protect_branch_calls must never include a DELETE (already_protected={already_protected}): {calls:?}"
            );
        }
    }

    /// Regression test for review bug #9: `set_repo_settings` returned `Err`
    /// unconditionally — even for `enable_issues`/`enable_wiki`, which GitLab
    /// genuinely supports via `PUT /projects/:id` — yet the planner always
    /// planned the action for GitLab, a permanent never-converges mismatch.
    #[test]
    fn test_gitlab_repo_settings_fields_supports_issues_and_wiki() {
        let settings = crate::actions::plan::GitHubRepoSettings {
            enable_issues: Some(true),
            enable_wiki: Some(false),
            ..Default::default()
        };
        let fields = GitLabProvider::repo_settings_fields(&settings);
        assert!(fields.contains(&("issues_access_level".to_string(), "enabled".to_string())));
        assert!(fields.contains(&("wiki_access_level".to_string(), "disabled".to_string())));
    }

    #[test]
    fn test_gitlab_repo_settings_unsupported_excludes_issues_and_wiki() {
        let issues_and_wiki = crate::actions::plan::GitHubRepoSettings {
            enable_issues: Some(true),
            enable_wiki: Some(true),
            ..Default::default()
        };
        assert!(!GitLabProvider::requests_unsupported_settings(
            &issues_and_wiki
        ));

        let discussions = crate::actions::plan::GitHubRepoSettings {
            enable_discussions: Some(true),
            ..Default::default()
        };
        assert!(GitLabProvider::requests_unsupported_settings(&discussions));
    }

    #[test]
    fn test_github_only_methods_return_err_not_false() {
        // The Critical correctness rule: GitHub-specific reads must Err (skip),
        // never Ok(false)/Ok(default), to avoid false findings on GitLab.
        let p = test_provider();
        assert!(p.has_vulnerability_alerts().is_err());
        assert!(p.has_automated_security_fixes().is_err());
        assert!(p.has_dependabot_security_updates().is_err());
        assert!(p.get_secret_scanning().is_err());
        assert!(p.get_actions_permissions().is_err());
        assert!(p.get_actions_workflow_permissions().is_err());
        assert!(p.get_fork_pr_workflows_policy().is_err());
    }

    #[test]
    fn test_is_available_returns_bool() {
        let _: bool = GitLabProvider::is_available();
    }

    #[test]
    fn test_gitlab_issue_create_args_includes_title_body_and_labels() {
        let args = GitLabProvider::issue_create_args("Bug title", "Bug body", &["bug", "audit"]);
        assert!(args.contains(&"--title".to_string()));
        assert!(args.contains(&"Bug title".to_string()));
        assert!(args.contains(&"--description".to_string()));
        assert!(args.contains(&"Bug body".to_string()));
        assert!(args.contains(&"--label".to_string()));
        assert!(args.contains(&"bug,audit".to_string()));
    }

    #[test]
    fn test_gitlab_issue_create_args_omits_label_flag_when_no_labels() {
        let args = GitLabProvider::issue_create_args("Title", "Body", &[]);
        assert!(!args.contains(&"--label".to_string()));
    }

    #[test]
    fn test_gitlab_mr_create_args_includes_source_and_target_branch() {
        let args = GitLabProvider::mr_create_args("Title", "Body", "feature-x", Some("main"));
        assert!(args.contains(&"--source-branch".to_string()));
        assert!(args.contains(&"feature-x".to_string()));
        assert!(args.contains(&"--target-branch".to_string()));
        assert!(args.contains(&"main".to_string()));
    }

    #[test]
    fn test_gitlab_mr_create_args_omits_target_branch_when_base_is_none() {
        let args = GitLabProvider::mr_create_args("Title", "Body", "feature-x", None);
        assert!(!args.contains(&"--target-branch".to_string()));
    }

    /// Compile-time proof that `GitLabProvider` (like `GitHubProvider`) is
    /// object-safe with the two new trait methods and can be used behind
    /// `&dyn RepoProvider` (foundation for review bug #8). Deliberately does
    /// NOT invoke `create_issue`/`open_change_request` here — both shell out
    /// to `glab`, which is not installed in this sandbox; the constructed
    /// commands are covered by the pure `issue_create_args`/`mr_create_args`
    /// tests above instead.
    #[test]
    fn test_gitlab_provider_exposes_change_request_methods_via_trait_object() {
        let provider: Box<dyn RepoProvider> = Box::new(test_provider());
        let _: &dyn RepoProvider = provider.as_ref();
    }

    #[test]
    fn test_urlencode() {
        assert_eq!(urlencode("group/repo"), "group%2Frepo");
        assert_eq!(urlencode("feature/x"), "feature%2Fx");
        assert_eq!(urlencode("main"), "main");
    }
}
