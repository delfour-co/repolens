# RepoLens v3.0.0 Recentering — Re-application on the Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Re-apply PR #236's v3.0.0 recentering (drop 9 rule categories → 6; add a `RepoProvider` trait + GitLab provider; generalize the action catalog) onto the new 2-crate workspace, **and** fix the 15 confirmed correctness bugs from the xhigh review before merge.

**Architecture:** PR #236 was implemented on the OLD flat `src/` layout (branch `claude/tool-capabilities-scope-2daznv`, head `073b71c`, based on pre-migration `main@1cd986e`). `main` is now the 2-crate workspace (`crates/repolens-core` domain + `crates/repolens` bin, `0dfc6af`). Its 61-file diff will **not** rebase cleanly, so we re-apply the *intent* file-by-file onto the migrated tree: domain changes land in `repolens-core`, CLI/hooks changes land in `repolens`, and the 15 reviewed bugs are fixed as an explicit checklist. The source of truth for ported code is the fetched #236 branch (`git show 073b71c:<old-path>`); the source of truth for the *design* is the two v3 docs carried in that PR; the source of truth for the *fixes* is the 15 review comments.

**Tech Stack:** Rust 2024, MSRV 1.85, `octocrab` + `gh` CLI (GitHub), `glab` CLI + `GITLAB_TOKEN` (GitLab), `wiremock`/fixtures for provider HTTP shapes, `insta` for new snapshot tests.

## Global Constraints

- **Workspace boundary is load-bearing.** Domain logic → `crates/repolens-core` (no `clap`, no CLI concerns). CLI/output/exit-codes/hooks → `crates/repolens`. `repolens-core` must not depend on `clap`. New provider/action/rule types added by #236 live in `repolens-core`; the bin references them as `repolens_core::…`.
- **Provider is the sole forge boundary.** All `octocrab`/`gh`/`glab` access originates in `crates/repolens-core/src/providers/`. After v3 the invariant becomes "exactly one trait, N implementations; no other module constructs a concrete provider" — `rules/` and `actions/` depend on `&dyn RepoProvider`, never on `GitHubProvider`/`GitLabProvider`.
- **Plan/apply split is non-negotiable.** Every kept rule_id maps to an applicable `Action` or is on the explicit detection-only allowlist (DOC002/003/009, CODE002). No report-only rule_id may claim to be remediable — the zero-report-only test must confront the *real* action catalog, not a hand-maintained list.
- **`VALID_CATEGORIES` mirrors the engine.** `crates/repolens-core/src/rules/constants.rs` must list exactly the 6 kept categories and its count test must assert **6**; it must match registration in `crates/repolens-core/src/rules/engine.rs`. Move 15→6 atomically or the build breaks.
- **Presets are static at build time.** Every preset under `crates/repolens-core/src/config/presets/` must be updated in lockstep with the category cut — an unknown category in a preset is a build error by design.
- **Dual-mode auth, both providers.** GitHub: `GITHUB_TOKEN` preferred, `gh` fallback. GitLab: `GITLAB_TOKEN` + `glab` CLI, with the same graceful-skip-when-CLI-absent contract. Both paths tested; never hard-fail an audit because a CLI is missing.
- **`unsafe_code = "deny"`** workspace-wide (not `forbid`); keep the documented `#[allow(unsafe_code)]` + `// SAFETY:` sites intact.
- **Commit hygiene:** no `Co-authored-by:` trailer; no `feat!:`/`!` shorthand (use `feat:` + `BREAKING CHANGE:` body); never `--no-verify` (the pre-commit hook runs RepoLens's own secret scan + commit-msg format check).
- **6 kept categories:** `docs`, `files`, `git`, `codeowners`, `metadata`, `security` (trimmed to API-settable settings). **9 removed:** `secrets`, `dependencies`, `licenses`, `history`, `issues`, `docker`, `workflows`, `quality`, `custom`.

---

## Porting method (applies to every "port" step)

For each file #236 changed:
1. Read #236's version: `git show 073b71c:<old-flat-path>` (e.g. `git show 073b71c:src/providers/gitlab.rs`).
2. Read the current migrated file it maps to (may already differ from pre-migration `main` — the migration edited imports, split `HooksConfig` into `crates/repolens-core/src/config/hooks_config.rs`, moved `build.rs`, etc.). **Re-apply #236's semantic intent on top of the migrated file — do not clobber migration edits.**
3. Path map: `src/actions/*`, `src/providers/*`, `src/rules/*`, `src/config/*`, `src/scanner/*`, `src/utils/*`, `src/lib.rs`, `benches/*` → `crates/repolens-core/…`. `src/cli/*`, `src/hooks/*`, `src/main.rs`, `build.rs`, `schemas/*`, `tests/*` → `crates/repolens/…`.
4. Import rewrites: intra-`repolens-core` references stay `crate::…`. Any **bin** reference to a domain type (including the new `Provider`, `RepoProvider`, `ProviderError`, renamed `ConfigureProtectedBranch`/`UpdateRepoSettings`, new `UpdateRepoMetadata`) uses `repolens_core::…`.
5. **Faithful port first, then fix.** Port reproduces #236's behavior (including its 15 bugs); Phase D fixes them with regression tests. This keeps each phase's diff reviewable against #236 and isolates the fixes.

Per-phase Definition of Done: `cargo build --workspace`, `cargo clippy --all-targets --workspace -- -D warnings`, `cargo test --lib` green, `cargo fmt --all`.

---

## Task 0: Branch, fetch, and design docs

**Files:**
- Create: `docs/superpowers/specs/2026-06-26-repolens-autoconfigurator-multiprovider-design.md`
- Create: `docs/superpowers/plans/2026-06-26-plan-b-provider-abstraction-gitlab.md`

- [ ] **Step 1:** Confirm on branch `feat/v3-recentering-reapply` off `main@0dfc6af`; confirm `git rev-parse 073b71c` resolves (PR #236 fetched).
- [ ] **Step 2:** Port the two v3 design docs verbatim from `073b71c` into `docs/superpowers/`. (They are layout-agnostic; carry them so the workspace records the v3 decision.)
- [ ] **Step 3:** Commit `docs: carry v3 auto-configurator design + Plan B into the workspace`.

---

## Phase A — Rule & action recentering (drop 9 categories → 6)

Outcome: builds on 6 categories; `VALID_CATEGORIES` count test asserts 6; presets/config/scanner consistent; self-audit re-baselined. Faithful to #236's Plan A.

### Task A1: Remove the 9 dropped category modules + registration

**Files:**
- Delete: `crates/repolens-core/src/rules/categories/{secrets,dependencies,licenses,history,issues,docker,workflows,quality,custom}.rs`
- Delete: `crates/repolens-core/src/rules/patterns/secrets.rs` (+ prune `patterns/mod.rs`)
- Modify: `crates/repolens-core/src/rules/categories/mod.rs` (drop `mod`/`pub use` for removed)
- Modify: `crates/repolens-core/src/rules/engine.rs` (drop registration of removed categories)
- Modify: `crates/repolens-core/src/rules/constants.rs` (`VALID_CATEGORIES` → the 6; count test → 6)
- Modify: `crates/repolens-core/src/rules/mod.rs`, `crates/repolens-core/src/lib.rs` (re-exports)

- [ ] **Step 1:** Port #236's `constants.rs` change; update the mirror count test to assert `6`. Run it → expect FAIL until engine registration matches.
- [ ] **Step 2:** Delete the 9 modules + secrets patterns; prune `mod.rs`/`engine.rs` registration to the 6.
- [ ] **Step 3:** `cargo build --workspace` → resolve dangling refs (OSV client + its deps in `dependencies.rs` are gone; drop the deps from `crates/repolens-core/Cargo.toml` per #236's `Cargo.toml`/`Cargo.lock` deletions).
- [ ] **Step 4:** `cargo test --lib` green (removed-category unit tests are gone with their modules). Commit `feat: drop the 9 non-auto-fixable rule categories`.

### Task A2: Trim the 6 kept categories to their auto-fix-dominant rule set

**Files:** `crates/repolens-core/src/rules/categories/{docs,files,git,codeowners,metadata,security}.rs`

- [ ] **Step 1:** Port #236's trims: `security.rs` (+76/-657 — drop TEAM/KEY/APP/HOOK/ENV access checks, SEC001-003, keep branch-protection + SEC011-017 + SECURITY003; fold SEC007 settings.yml), `metadata.rs` (+5/-100 — META001/002/003 keep, META004 drop), `codeowners.rs` (+2/-351 — CODE001/002 keep, CODE003/REL001-003 drop), `files.rs` (-118 — FILE002/003 keep, FILE001/004 drop), `git.rs` (-135 — GIT002/003 keep, GIT001 drop). Use the spec's Appendix rule_id→disposition table as the checklist.
- [ ] **Step 2:** `cargo test --lib` green; adjust/port each kept category's unit tests to the trimmed rule set.
- [ ] **Step 3:** Commit `feat: trim kept categories to auto-fixable rule_ids`.

### Task A3: Config, presets, scanner, benches, self-audit baseline

**Files:** `crates/repolens-core/src/config/{mod.rs,loader.rs,presets/mod.rs}`, `crates/repolens-core/src/scanner/{mod.rs,filesystem.rs}`, `crates/repolens-core/benches/{parse_benchmark.rs,rules_benchmark.rs,scanner_benchmark.rs}`, `.repolens.toml`, `crates/repolens/schemas/*`

- [ ] **Step 1:** Port config trims (`mod.rs` +77/-257, `loader.rs` +27/-109, `presets/mod.rs` +30/-115) — remove config knobs for dropped categories; **every preset lists only kept categories** (build-error guard).
- [ ] **Step 2:** Port scanner simplifications (`scanner/mod.rs` -86: drop scanning paths only the removed categories needed) and bench updates (`parse_benchmark.rs` removed; `rules_benchmark.rs`/`scanner_benchmark.rs` trimmed).
- [ ] **Step 3:** Re-baseline the self-audit: RepoLens audits itself; update `.repolens.toml`, any golden/snapshot fixtures in `crates/repolens-core/src/compare/` and `crates/repolens/tests/`. `cargo test --workspace` green.
- [ ] **Step 4:** Commit `feat: reconcile config, presets, scanner, and self-audit baseline for 6 categories`.

---

## Phase B — Provider abstraction + GitLab (faithful port)

Outcome: `RepoProvider` trait with GitHub + GitLab impls; kept rules/actions depend on the trait; `--provider {github|gitlab}` + auto-detect. **Sync trait** (methods block on CLI — per #236 Plan B decision, no `async_trait` churn).

### Task B1: `RepoProvider` trait, shared types, `ProviderError`, `for_config` factory

**Files:** `crates/repolens-core/src/providers/mod.rs` (+206 in #236)

**Interfaces (Produces — later tasks rely on these exact names):**
- `enum Provider { GitHub, GitLab }`
- `enum ProviderError { Unsupported { provider: &'static str, capability: &'static str }, /* … */ }`
- `trait RepoProvider: Send + Sync` — reads: `slug`, `repo_metadata`, `protected_branch`, `vulnerability_alerts`, `dependabot_security_updates`, `automated_security_fixes`, `secret_scanning`, `actions_permissions`, `workflow_permissions`, `fork_pr_approval` (all sync, `Result<_, ProviderError>`).
- `fn for_config(config: &Config) -> Option<Box<dyn RepoProvider>>`
- Shared provider-agnostic types moved here: `RepoSlug`, `RepoMetadata` (moved from `actions/metadata.rs`), `BranchProtection`, `RepoSettings`, `ActionsPermissions`, `SecretScanningSettings`.

- [ ] Port #236's `providers/mod.rs`; keep GitHub wire-format structs private in `github.rs`. `cargo build` green. Commit `feat: introduce RepoProvider trait + provider-agnostic types`.

### Task B2: `GitHubProvider` implements the trait; delete dead access-posture methods

**Files:** `crates/repolens-core/src/providers/github.rs` (+434/-636 in #236)

- [ ] Port the rewrite: `impl RepoProvider for GitHubProvider`, wrap existing methods, keep dual-auth internals; delete `list_collaborators/list_teams/list_deploy_keys/list_installations/list_webhooks/list_environments/get_environment_protection` and their now-unused structs. `cargo test --lib` green. Commit `feat: GitHubProvider implements RepoProvider`.

### Task B3: `GitLabProvider` (new) via `glab` CLI

**Files:** Create `crates/repolens-core/src/providers/gitlab.rs` (+731 in #236)

- [ ] Port the full GitLab impl (protected branches + approval rules, project description/topics, secret detection, CI settings) via `glab api`/subcommands, mirroring the `gh` graceful-skip contract. Port #236's fixture-parse unit tests (no network/`glab` in CI). **Note:** bugs #3/#4/#6/#9 live here and are fixed in Phase D — port faithfully now. Commit `feat: add GitLabProvider (glab CLI)`.

### Task B4: Rewire kept rules + planner reads onto the trait

**Files:** `crates/repolens-core/src/rules/categories/{security,metadata}.rs`, `crates/repolens-core/src/actions/planner.rs` (read side)

- [ ] Port: each category constructs the provider once in `run()` via `for_config(config)`, passes `&dyn RepoProvider` to `check_*`; `Err(Unsupported)` → skip/no finding. Planner reads current state through the trait. `cargo test --lib` green. Commit `feat: route kept rules + planner reads through RepoProvider`.

### Task B5: `config.provider` + `--provider` flag + host auto-detect

**Files:** `crates/repolens-core/src/config/{mod.rs,loader.rs}` (Provider enum/field/serde), `crates/repolens-core/src/utils/prerequisites.rs` (remote-host parse + `provider_for_host`), `crates/repolens/src/cli/commands/{mod.rs,plan.rs,report.rs,apply.rs}` (`--provider {github|gitlab}`), `crates/repolens/build.rs` (completions: add `--provider`)

- [ ] Port: `Provider` enum + `provider` config field (serde `provider = "github"`); host auto-detect (`github.com`→GitHub, `gitlab.*`/self-managed→GitLab) when config doesn't pin one; CLI `--provider` overrides. Update `build.rs`'s duplicated CLI mirror so completions include `--provider`. **Bugs #5/#7 live here — fixed in Phase D.** `cargo test --workspace` green; `plan --provider gitlab` and `--provider github` both produce an `ActionPlan`. Commit `feat: provider selection (config + --provider + host auto-detect)`.

---

## Phase C — Provider-agnostic action catalog (close the report-only gap)

**Files:** `crates/repolens-core/src/actions/{plan.rs,planner.rs,executor.rs,branch_protection.rs,github_settings.rs,metadata.rs,templates.rs,mod.rs}`, `crates/repolens/src/cli/commands/apply.rs`, `crates/repolens/src/cli/output/json.rs`

- [ ] **Step 1:** Port operation renames: `ConfigureBranchProtection`→`ConfigureProtectedBranch`, `UpdateGitHubSettings`→`UpdateRepoSettings` (planner, executor, apply previews, json output, tests).
- [ ] **Step 2:** Port `UpdateRepoMetadata { description?, topics?, homepage? }` op + executor calling trait write `set_repo_metadata`; planner maps META001/002/003 → this action.
- [ ] **Step 3:** Add the trait **write** side (`set_protected_branch`, `set_repo_settings`, `set_repo_metadata`, `open_change_request`) with GitHub + GitLab impls; executors stop shelling raw `gh`. **Bugs #3/#9/#10 touch these writers — fixed in Phase D.**
- [ ] **Step 4:** Port the zero-report-only test (`test_no_kept_rule_is_report_only`). **It currently lies (bugs #11/#12) — port as-is, then Phase D makes it confront the real catalog.**
- [ ] **Step 5:** `cargo test --workspace` green. Commit `feat: provider-agnostic action catalog (rename ops + UpdateRepoMetadata + trait writes)`.

---

## Phase D — Fix the 15 confirmed review bugs (TDD, one regression test each)

**Every fix follows: (1) write a failing test that reproduces the bug, (2) run → FAIL, (3) fix, (4) run → PASS, (5) commit.** Group commits by subsystem. New tests prefer `insta` snapshots where output is structured.

### D-bin — CLI & hooks (`crates/repolens/`)

- [ ] **Bug #1 — `hooks/mod.rs`:** pre-commit hook must scan content-secrets. Since `secrets` is removed, the hook can no longer rely on `--only secrets`. Fix: the hook's fast pre-commit check must run a **content-secret scan** independent of the removed category (either retain a minimal secret-scan pass for the hook, or re-scope the hook's purpose and document it). **Decision needed at execution (see Open Questions):** simplest safe fix is to keep a dedicated lightweight secret-scan in the hook path rather than routing through the deleted category. Test: committing a file containing `sk_live_…` is blocked by the installed pre-commit hook.
- [ ] **Bug #2 — `cli/commands/plan.rs`:** `--only <unknown-or-removed>` filtering to an empty set must **not** silently fall back to a full audit. Fix: an explicit `--only`/`--skip` that resolves to zero valid categories is an error (or runs zero categories), never "all". Test: `plan --only secrets` errors/no-ops, does **not** run the 6 categories.
- [ ] **Bug #8 — `cli/commands/apply.rs`:** `--create-pr` must go through the trait, not hardcode `GitHubProvider`. Fix: route change-request creation via `RepoProvider::open_change_request`; do **not** push commits before confirming a provider can open the CR (avoid orphan branch on GitLab). Test: on a GitLab-detected repo, `--create-pr` uses the GitLab path (or fails loudly) and does not emit a GitHub-specific success message.

### D-provider — providers & host parsing (`crates/repolens-core/src/providers/`, `…/utils/prerequisites.rs`)

- [ ] **Bug #3 — `gitlab.rs` `set_protected_branch`:** send **all 8** `BranchProtectionSettings` fields (required_approvals, require_status_checks, enforce_admins, require_signed_commits, require_linear_history, block_deletions, require_conversation_resolution, allow_force_push), or return `Err` for any that can't be applied — never `Ok` after applying one. Test: asserts every field is included in the `glab` request payload.
- [ ] **Bug #4 — `gitlab.rs` `set_protected_branch`:** the DELETE-then-POST sequence must not leave the branch unprotected. Fix: if POST fails after a successful DELETE, restore/roll back or use an idempotent update that never has an unprotected window; surface the error. Test: simulated POST failure leaves protection intact (or the op is atomic).
- [ ] **Bug #6 — `gitlab.rs` `parse_gitlab_url`:** SSH URLs with a custom port (`ssh://git@host:2222/group/repo.git`) must parse namespace `group/repo`, not `2222/group`. Fix: strip the port before the scp-path split. Test: table of URL forms → expected `encoded_id`.
- [ ] **Bug #9 — `gitlab.rs` `set_repo_settings`:** must actually write via `PUT /projects/:id` (the metadata writer already does), not unconditionally `Err`. Fix: implement the write; only `Err` on real failure. Test: settings write succeeds against a fixture; plan converges (no re-proposal).
- [ ] **Bug #10 — `github.rs` `set_repo_settings`:** apply `enable_issues` and `enable_wiki == Some(true)` (not only `enable_discussions==Some(true)`/`enable_wiki==Some(false)`). Fix: handle all toggles. Test: each toggle produces the corresponding API field; plan converges.
- [ ] **Bug #5 — `providers/mod.rs` `for_config`:** an explicit `Provider::GitHub` (from config or `--provider`) must be honored; auto-detect only when the user did **not** pin a provider. Fix: distinguish "unset" from "explicitly GitHub" (e.g. `Option<Provider>` in config, or a `provider_explicit` flag). Test: `--provider github` on a gitlab.com remote builds the GitHub provider.
- [ ] **Bug #7 — `utils/prerequisites.rs` `parse_remote_host`:** HTTPS URLs with embedded credentials (`https://user:tok@host/…`) must parse `host`, not `user`. Fix: strip `userinfo@` before taking the host. Test: credential-bearing URLs → correct host → correct provider.
- [ ] Commit `fix: correct GitLab/GitHub provider bugs + remote-host parsing (review #3-#7,#9,#10)`.

### D-actions — plan/apply contract (`crates/repolens-core/src/actions/`)

- [ ] **Bug #13 — `planner.rs` FILE002:** `check_gitignore` returns early emitting FILE002 when `.gitignore` is absent, but `plan_gitignore_update` only consumes FILE003 → no remediation. Fix: plan a `.gitignore` **creation** action for FILE002 (and entries for FILE003). Test: repo without `.gitignore` yields a create-file action; FILE002 not report-only.
- [ ] **Bug #14 — `planner.rs:566/577`:** on a provider read error, default to "needs enabling" (safer), not `current=true`. Fix: on `Err`, treat state as unknown/disabled so an enable action is planned (consistent with branch-protection/settings planners). Test: simulated read error → enable action is planned.
- [ ] **Bug #15 — `templates.rs` `SETTINGS_YML_TEMPLATE`:** the SEC007 auto-fix template must not immediately trigger SEC008. Fix: emit a `branches:` block with actual (uncommented) protection defaults so the next audit sees defined rules; keep it idempotent. Test: apply SEC007 fix → re-audit emits neither SEC007 nor SEC008.
- [ ] **Bugs #11 & #12 — `planner.rs` `test_no_kept_rule_is_report_only`:** the `remediable` set must be **derived from the real action catalog**, not hand-maintained. Fix: either (a) add the missing actions so SEC008-010 (edit existing settings.yml) and SEC013-017 (`UpdateRepoSettings` covering those toggles) are truly remediable, or (b) move genuinely-detection-only rules to the explicit allowlist and update the spec. Prefer (a) for SEC013-017 (extend `UpdateRepoSettings`/writers — ties to bugs #9/#10); for SEC008-010, add an "edit existing settings.yml" action or reclassify. The test must fail if any emitted rule_id lacks a real mapping. Test: the contract test cross-checks emitted rule_ids against the actual planner output, not a literal list.
- [ ] Commit `fix: make plan/apply contract true — real remediation for FILE002, SEC00x/01x, safe error defaults (review #11-#15)`.

---

## Phase E — Docs & surface (reconcile Plan C with the migrated docs)

**Files:** `CLAUDE.md`, `README.md`, `CHANGELOG.md`, `crates/repolens/schemas/*.schema.json`, `crates/repolens/schemas/README.md`, presets, completions/man (via `build.rs`), `docs/architecture.md`

- [ ] **Step 1:** Update `CLAUDE.md` — 6 categories (not 15), provider list (GitHub + GitLab), the updated provider invariant ("one trait, N impls"), action catalog. **Reconcile with the migration's workspace rewrite — edit, don't revert to #236's flat-layout CLAUDE.md.**
- [ ] **Step 2:** `README.md` (+62/-97 in #236), `CHANGELOG.md` (v3.0.0 entry, `BREAKING CHANGE`), schemas (category enum → 6; decide whether `provider` belongs in the report schema), `docs/architecture.md` (provider trait).
- [ ] **Step 3:** Presets doc + `.repolens.toml` example include the `provider` key and 6-category set. Regenerate completions/man (`build.rs`) for `--provider`.
- [ ] **Step 4:** `cargo test --workspace` green. Commit `docs: v3.0.0 surface — 6 categories, GitLab provider, action catalog`.

---

## Phase F — Final acceptance gate + PR

- [ ] **Step 1:** Full gate: `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --all-targets --workspace -- -D warnings`, `cargo fmt --all --check`, `cargo audit` (expect only the allowed `number_prefix` warning), `cargo deny check`.
- [ ] **Step 2:** v3 validation criteria: `VALID_CATEGORIES` count test asserts **6**; zero-report-only test passes against the real catalog; both provider paths exercised (GitHub + GitLab, token + CLI modes) via `wiremock`/fixtures; `plan --provider {github,gitlab}` both produce an `ActionPlan`; self-audit baseline re-committed.
- [ ] **Step 3:** Re-run the 15-bug checklist: each has a regression test that fails on the pre-fix code.
- [ ] **Step 4:** Version bump to `3.0.0` in `[workspace.package]`; push branch; open PR titled `feat: v3.0.0 recenter as GitHub/GitLab auto-configurator` with `BREAKING CHANGE:` body. **Close/supersede PR #236** (its branch is on the old layout).

---

## Risks

- **Self-audit bootstrap.** RepoLens audits itself; dropping categories changes its own baseline and any golden/snapshot files — re-baseline in Phase A (spec risk #1).
- **`VALID_CATEGORIES` mirror + preset drift.** Both must move 15→6 atomically with engine registration or the build breaks (spec risks #2/#3).
- **Hook regression is the highest-severity fix.** Bugs #1+#2 mean an upgraded user's pre-commit hook silently stops catching content-secrets *and* starts blocking on unrelated findings. Land both fixes and document the hook-reinstall requirement in CHANGELOG.
- **Docs reconciliation, not overwrite.** The migration already rewrote CLAUDE.md/README/architecture for the workspace; #236's Plan C edits must merge onto those, not replace them.
- **GitLab auth parity.** Token-preferred + `glab` fallback + graceful skip must be tested, including the offline-degradation path (spec risk on GitLab parity).
- **Trait sync vs async.** Keep `RepoProvider` methods **sync** (they block on CLI) — matches #236 Plan B; do not introduce `async_trait` and a block-in-async hazard.

## Open questions (resolve at execution start)

1. **Bug #1 hook fix shape:** keep a dedicated minimal content-secret scan inside the pre-commit hook path (recommended — preserves the security property), *or* re-scope the hook to config-only and document the loss? Recommend the former.
2. **Bugs #11/#12 for SEC008-010:** add a real "edit existing `.github/settings.yml`" action, or reclassify SEC008-010 as detection-only tails and update the spec's zero-report-only allowlist? Recommend adding the edit action to keep the fix-first principle honest.
3. **`provider` in report schema:** include the selected provider in the JSON/SARIF report schema, or keep it CLI-only? Recommend including it (auditor wants to know which forge was queried).

## Self-review notes

- Spec coverage: Phases A (Plan A) / B+C (Plan B B1-B3) / E (Plan C) cover the three #236 plans; Phase D covers all 15 reviewed bugs; validation criteria mapped in Phase F.
- Type consistency: `Provider`, `RepoProvider`, `ProviderError`, `for_config`, `ConfigureProtectedBranch`, `UpdateRepoSettings`, `UpdateRepoMetadata` used identically across B1→C→D.
- The three Open Questions are genuine design forks flagged for the execution kickoff, not placeholders in code steps.
