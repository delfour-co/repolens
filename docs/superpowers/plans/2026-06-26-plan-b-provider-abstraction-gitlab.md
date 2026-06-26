# Plan B — Provider Abstraction + GitLab (v3.0.0)

> **For agentic workers:** implement task-by-task; each task is a self-contained commit that must
> reach `cargo build` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib` green
> (modulo the ~6 known pre-existing environment failures) before moving on.

**Goal:** Turn the single concrete `GitHubProvider` into a `RepoProvider` trait with two
implementations (GitHub, GitLab), and make the kept rules + actions depend on the trait rather than
constructing `GitHubProvider` directly. Then generalize the action catalog so every kept rule_id maps
to a provider-agnostic action.

**Spec:** [2026-06-26-repolens-autoconfigurator-multiprovider-design.md](../specs/2026-06-26-repolens-autoconfigurator-multiprovider-design.md)
**Depends on:** Plan A (category cut + trim + config cleanup) — DONE.

**Tech stack:** Rust 2024, MSRV 1.85, `octocrab` + `gh` CLI (GitHub), `glab` CLI + `GITLAB_TOKEN`
(GitLab), `wiremock` for HTTP tests.

---

## Design decisions (load-bearing)

1. **Injection via factory, not signature change.** Add `providers::for_config(config: &Config) ->
   Option<Box<dyn RepoProvider>>`. Rules/actions call it instead of `GitHubProvider::new()`. The
   `RuleCategory::run(&self, scanner, config)` signature is unchanged; each category constructs the
   provider once at the top of `run()` and passes `&dyn RepoProvider` to its helper fns. `None` =
   no provider available/authenticated → graceful skip (same contract as today's `is_available()`).

2. **Shared types move to `providers/mod.rs`.** `BranchProtection`, repo-settings reads
   (`RepoInfo`/a new `RepoSettings`), `ActionsPermissions`, `SecretScanningSettings`, and
   `RepoMetadata` (currently local to `metadata.rs`) become provider-agnostic types in
   `providers/mod.rs`. GitHub-wire-format structs stay private in `github.rs`.

3. **Capability gaps are explicit.** Settings without a provider counterpart (e.g. GitHub
   `automated_security_fixes` / `dependabot_security_updates` on GitLab) return
   `Err(ProviderError::Unsupported)`; the caller skips that rule and emits no finding — never a false
   positive. Add `#[derive]` `ProviderError` with an `Unsupported { provider, capability }` variant.

4. **One trait, reads first.** B1 defines the read side of `RepoProvider` (drives `plan`). The write
   side (drives `apply`) lands in B3 alongside the action-catalog work, so the trait is not churned
   twice.

5. **Provider selection.** New `config.provider: Provider` (`github` | `gitlab`, default inferred
   from the `origin` remote host, fallback `github`). CLI `--provider {github|gitlab}` overrides.

---

## B1 — Define `RepoProvider`, move GitHub onto it (GitHub-only, no behaviour change)

Outcome: identical behaviour and findings, but all GitHub access goes through the trait. Removes the
dead access-posture methods left annotated in Plan A.

**Files:**
- Modify: `src/providers/mod.rs` (trait + shared types + `ProviderError` + `for_config` factory)
- Modify: `src/providers/github.rs` (impl `RepoProvider` for `GitHubProvider`; delete dead methods)
- Modify: `src/rules/categories/security.rs` (use `&dyn RepoProvider`)
- Modify: `src/rules/categories/metadata.rs` (use `&dyn RepoProvider`; move `RepoMetadata`)
- Modify: `src/actions/planner.rs` (read current state through the trait)
- Modify: `src/config/{mod.rs,loader.rs}` (add `Provider` enum + `provider` field, default + serde)

- [ ] **Step 1: trait + shared types + error + factory** in `providers/mod.rs`.
  ```rust
  pub enum Provider { GitHub, GitLab }
  pub enum ProviderError { Unsupported { provider: &'static str, capability: &'static str }, /* … */ }

  #[async_trait::async_trait]
  pub trait RepoProvider: Send + Sync {
      fn slug(&self) -> RepoSlug;                                  // owner/name | namespace/project
      fn repo_metadata(&self) -> Result<RepoMetadata, ProviderError>;
      fn protected_branch(&self, branch: &str) -> Result<Option<BranchProtection>, ProviderError>;
      fn vulnerability_alerts(&self) -> Result<bool, ProviderError>;
      fn dependabot_security_updates(&self) -> Result<bool, ProviderError>;   // GitLab: Unsupported
      fn automated_security_fixes(&self) -> Result<bool, ProviderError>;      // GitLab: Unsupported
      fn secret_scanning(&self) -> Result<SecretScanningSettings, ProviderError>;
      fn actions_permissions(&self) -> Result<ActionsPermissions, ProviderError>;
      fn workflow_permissions(&self) -> Result<ActionsPermissions, ProviderError>;
      fn fork_pr_approval(&self) -> Result<bool, ProviderError>;
  }
  pub fn for_config(config: &Config) -> Option<Box<dyn RepoProvider>>;
  ```
- [ ] **Step 2: impl for `GitHubProvider`** — wrap the existing methods (rename to match trait), keep
  the dual-auth (token/`gh`) internals. Delete the now-dead `list_collaborators`, `list_teams`,
  `list_deploy_keys`, `list_installations`, `list_webhooks`, `list_environments`,
  `get_environment_protection` and the structs only they used (`Collaborator`, `Team`, `DeployKey`,
  `Installation`, `Webhook`, `Environment`, `EnvironmentProtection`) — removing the Plan A
  `#[allow(dead_code)]` annotations.
- [ ] **Step 3: rewire `security.rs`** — construct the provider once in `run()` via
  `providers::for_config(config)`; pass `&dyn RepoProvider` to each `check_*`; map
  `Err(Unsupported)` to "skip, no finding".
- [ ] **Step 4: rewire `metadata.rs`** — same; move `RepoMetadata` to `providers/mod.rs`.
- [ ] **Step 5: rewire `actions/planner.rs`** reads to the trait.
- [ ] **Step 6: `config.provider`** — add `Provider` enum + field (serde `provider = "github"`),
  default = GitHub. Auto-detection from remote can wait for B2.
- [ ] **Verify** green; findings on a GitHub repo are unchanged (spot-check `plan` output).

## B2 — GitLab implementation + provider selection

**Transport decision (resolved):** `GitLabProvider` uses the **`glab` CLI** via `std::process`
(mirroring the GitHub provider's `gh`-CLI-primary mechanism), NOT a blocking HTTP client. Rationale:
keeps the trait methods synchronous with no tokio block-in-async hazard (a blocking `reqwest` call
inside the async rule runtime would panic), adds zero HTTP dependency (`reqwest` was removed in
Plan A), and `glab` already honours `GITLAB_TOKEN`. **Known gap (documented):** unlike GitHub's
token-without-CLI mode, GitLab requires `glab` installed even when a token is set; a token-only REST
path (sync client such as `ureq`) can be added later. Graceful-skip when `glab` is absent, exactly
like the `gh` path.

**Files:** new `src/providers/gitlab.rs`; modify `providers/mod.rs` (factory branch), CLI args, config
auto-detect, tests.

- [ ] **Step 1: `GitLabProvider`** implementing `RepoProvider` via `glab api` / `glab` subcommands
  (mirror the GitHub graceful-skip contract). Map concepts per the spec table (protected branches +
  approval rules, project description/topics, secret detection, CI settings). GitHub-only
  capabilities (`has_dependabot_security_updates`, `has_automated_security_fixes`) return the same
  "skip" signal the callers already handle — for B2, return `Ok(false)`/`Ok(None)` or an `Err` that
  the existing match treats as skip; do NOT introduce `ProviderError::Unsupported` yet unless it is
  needed (kept minimal, consistent with B1's no-new-error-type choice).
- [ ] **Step 2: `for_config` branch** on `config.provider`; build `GitLabProvider` when
  `Provider::GitLab` and `glab` is available, else `None`. Auto-detect the default from the `origin`
  host (`github.com` → GitHub, `gitlab.*`/self-managed → GitLab) when config doesn't pin one.
- [ ] **Step 3: `--provider {github|gitlab}`** flag on the relevant subcommands, overriding config.
- [ ] **Step 4: tests** — parse-from-fixture tests for the `glab` JSON shapes (mirroring how the
  GitHub provider is unit-tested), plus the availability/skip path and `for_config` selection. No
  network or `glab` process in CI.
- [ ] **Verify** green; `plan --provider gitlab` and `--provider github` both produce an `ActionPlan`.

## B3 — Provider-agnostic action catalog (close the report-only gap)

**Files:** `src/actions/plan.rs` (operations), `planner.rs` (mappings), executors
(`branch_protection.rs`, `github_settings.rs` → provider-agnostic), `apply.rs` (preview arms).

- [ ] **Step 1: rename operations** `ConfigureBranchProtection` → `ConfigureProtectedBranch`,
  `UpdateGitHubSettings` → `UpdateRepoSettings` (update planner, executor, apply previews, tests).
- [ ] **Step 2: add `UpdateRepoMetadata { description?, topics?, homepage? }`** + executor that calls
  the trait write `set_repo_metadata`; planner maps META001/002/003 → this action.
- [ ] **Step 3: route executors through trait writes** — add the write methods to `RepoProvider`
  (`set_protected_branch`, `set_repo_settings`, `set_repo_metadata`, `open_change_request`) with
  GitHub + GitLab impls; executors stop shelling raw `gh`.
- [ ] **Step 4: zero-report-only test** — assert every emitted kept rule_id has a planner mapping, or
  is on the explicit detection-only allowlist (DOC002/003/009, CODE002).
- [ ] **Verify** green; `apply` round-trips on a fixture repo per provider.

---

## Out of scope (this plan)
- Bitbucket/Gitea. - CI-workflow auto-hardening. - reviving `custom`. - the Plan C docs rewrite
  (CLAUDE.md/README/schemas) lands separately.

## Risks
- **Provider construction cost.** `for_config` may shell out / hit the network; construct once per
  category `run()`, not per check (today some checks each call `GitHubProvider::new()` — consolidate).
- **`gh`/`glab` absence in CI.** The graceful-skip path must be exercised; never hard-fail an audit
  because a CLI is missing.
- **Trait object + async.** Methods are sync today (they block on CLI). Keep them sync in the trait
  (or `async_trait`) consistently to avoid churn; prefer keeping the current sync shape in B1.
- **Self-audit baseline.** `security`/`metadata` findings must be byte-identical on GitHub after B1;
  diff `plan --format json` before/after.
