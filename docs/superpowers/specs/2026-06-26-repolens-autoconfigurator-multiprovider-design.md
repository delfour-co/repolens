# RepoLens — Auto-Configurator Recentering + Multi-Provider Design (v3.0.0)

**Status:** Proposed
**Date:** 2026-06-26
**Driver:** Kevin Delfour
**Supersedes:** the "Multi-provider support … out of scope" decision in
`2026-05-13-repolens-recentering-design.md`. That doc deferred multi-provider until "its own design
doc lands." This is that doc.

## Context

The v2.0.0 recentering cut the reporting layer but left the **rules engine untouched: 15 categories**.
In practice the product drifted into a generalist *compliance scanner*, and an audit of the codebase
(June 2026) showed the headline differentiator — the **plan/apply split** — is real but narrow:

- `ActionOperation` has **4 variants**; the planner emits **7 action types** total (`.gitignore`
  entries, four doc-file creations, branch protection, repo settings). See `src/actions/plan.rs:77`,
  `src/actions/planner.rs:65`.
- Across the ~15 categories and ~100 distinct `rule_id`s, **the large majority of findings are
  report-only** — they have no corresponding action and cannot be auto-fixed.
- The report-only mass is concentrated in **scanner** categories — `dependencies` (OSV / GitHub
  Advisory), `secrets`, dependency `licenses`, `history`, `issues`, `docker` linting — which:
  1. cannot be deterministically auto-fixed (a leaked secret must be rotated; a CVE needs a human
     upgrade decision), and
  2. duplicate entrenched, free tools: Dependabot / `osv-scanner`, gitleaks / GitHub secret
     scanning, `cargo-deny` / licensee, commitlint, stale-bot, hadolint.

The original intent was narrower and sharper: **a tool that audits a git repository and configures
it as well as possible, automatically.** The scanner surface works against that intent (it inflates
maintenance, forces a network dependency into CI, and competes where RepoLens cannot win).

Separately, the product is GitHub-only by design, but the operator base now spans **GitLab**. Repo
configuration (protected branches, merge-request approvals, project settings, topics) is exactly the
domain that benefits from a provider abstraction.

## Decision

Recenter RepoLens as an **auto-configurator for git repositories**: every kept check exists to drive
a reviewable, applicable `Action`. Drop the scanner categories that cannot be auto-fixed. Introduce
a **provider abstraction** and add **GitLab** as the second implementation behind it.

Ship as **v3.0.0**, single breaking-change release (consistent with the v2.0.0 no-deprecation-cycle
precedent).

### Scoping principle (load-bearing)

> **A category survives only if the majority of its findings map to an applicable `Action`** — a
> deterministic local-file write (**L**) or a provider configuration call (**P**). Detection-only
> findings are permitted **only** as the minority tail of an otherwise-fixable category, never as a
> standalone category.

This turns today's weakness (report-only findings) into the cut line, and makes "plan/apply" true
across the whole surface.

## Scope: what stays, what goes

### Kept — 6 categories, all auto-fix-dominant

| Category | Keeps (rule_ids) | Mechanism | Provider-portable? |
|---|---|---|---|
| `docs` | DOC001/004/005/006/007/008/010 (README, LICENSE, CONTRIBUTING, COC, SECURITY, CHANGELOG) | L | Trivial (local files) |
| `files` | FILE002/003 (`.gitignore` create + entries) | L | Trivial |
| `git` | GIT002/003 (`.gitattributes`, sensitive→`.gitignore`) | L | Trivial |
| `codeowners` | CODE001 (create CODEOWNERS) | L | Trivial |
| `metadata` | META001/002/003 (description, topics, homepage) | P | **Needs abstraction** |
| `settings` (was the auto-fixable half of `security`) | branch protection; SEC011-017 (vuln alerts, secret-scanning enable, Actions perms) | P | **Needs abstraction** |

Detection-only tails retained inside kept categories (minority, no action): DOC002/003/009 (README/
CHANGELOG content quality), CODE002 (CODEOWNERS syntax). These stay because their parent category is
fix-dominant.

### Removed

| Removed | rule_ids | Reason |
|---|---|---|
| `dependencies` | DEP000-004 | Not auto-fixable; overlaps Dependabot / `osv-scanner`. Removes the OSV/GitHub-Advisory network dependency from audits. |
| `secrets` | SEC001-003 | Irreversible remediation (rotation is human); overlaps gitleaks / GitHub secret scanning. |
| `licenses` (dependency analysis) | LIC002/003/004 | Overlaps `cargo-deny` / licensee. (LIC001 "no project license" is covered by DOC004.) |
| `history` | HIST001-004 | Auto-fix = rewriting history; out of bounds. Overlaps commitlint. |
| `issues` | ISSUE001-003, PR001-002 | Hygiene; "fixes" = closing issues/PRs (destructive). Overlaps stale-bot. |
| releases (in `codeowners.rs`) | REL001-003 | Human action. |
| `docker` | DOCKER001-008 | Lint overlap (hadolint); only `.dockerignore` survives (folded into the scaffolding decision below). |
| `workflows` | WF001-009 | **Decision 3** — cut now. |
| `quality` | QUALITY001-009 | **Decision 4** — reduced to essentials. |
| security posture "access" | TEAM001-003, KEY001-002, APP001, HOOK001-003, ENV001-003 | **Decision 2** — not auto-fixable without destructive action. |
| large-file checks | FILE001/004, GIT001 | Human choice (LFS / exclude). |
| `custom` | runtime | Detection-only escape hatch; no deterministic fix. Reconsider post-v3 as a power-user feature. |

**Net effect:** 15 → 6 categories; ~100 → ~20 active rule_ids; all kept categories fix-dominant.

## Decisions (explicit — flip any single line to re-scope)

1. **Scoping principle = strict, fix-first.** (See above.)
2. **Security posture "access" checks (TEAM/KEY/APP/HOOK/ENV): cut.** Keep only the API-settable
   settings (SEC011-017) under the new `settings` category. Revoking a collaborator / app / deploy
   key is destructive and judgment-heavy — out of "auto-configure."
3. **CI / `workflows` (WF002-007): cut now.** Rewriting a user's `.github/workflows` YAML is risky
   and overlaps actionlint / zizmor. Re-introducible later as a focused, opt-in "CI hardening"
   action set — its own mini-design.
4. **Tooling scaffolding: reduce to consensus-neutral files only.** Keep `.editorconfig`,
   `.dockerignore`, `.gitattributes` creation. Drop opinionated, ecosystem-specific lint/coverage
   config scaffolding (QUALITY002/003/005/007/008/009).

## Provider abstraction

Today `src/providers/` is GitHub-only (one concrete type, `GitHubProvider`, ~20 public methods over
`octocrab` + `gh` CLI). v3 introduces a trait and two implementations.

### Trait surface (post-recentering — only what kept categories + actions need)

```rust
// src/providers/mod.rs
#[async_trait]
pub trait RepoProvider {
    // identity
    fn slug(&self) -> &RepoSlug;                 // owner/name (GitHub) | namespace/project (GitLab)

    // ---- reads (drive `plan`) ----
    async fn repo_metadata(&self) -> Result<RepoMetadata>;        // description, topics, homepage  (META001/2/3)
    async fn protected_branch(&self, branch: &str) -> Result<Option<ProtectedBranch>>;  // branch protection
    async fn repo_settings(&self) -> Result<RepoSettings>;        // vuln alerts, secret scanning, CI perms (SEC011-017)

    // ---- writes (drive `apply`) ----
    async fn set_repo_metadata(&self, patch: RepoMetadataPatch) -> Result<()>;
    async fn set_protected_branch(&self, branch: &str, cfg: ProtectedBranchConfig) -> Result<()>;
    async fn set_repo_settings(&self, patch: RepoSettingsPatch) -> Result<()>;
    async fn open_change_request(&self, cr: ChangeRequest) -> Result<ChangeRequestRef>;  // PR / MR for file actions
}
```

`L`-mechanism actions (`CreateFile`, `UpdateGitignore`) do **not** go through the trait — they are
plain local git writes, then surfaced via `open_change_request`. Only `P`-mechanism actions touch
provider config.

### GitHub ↔ GitLab concept mapping

| Concept | GitHub | GitLab |
|---|---|---|
| Change request | Pull Request | Merge Request |
| Branch protection | Branch protection rule | Protected branch + Merge-request approval rules |
| Required reviews | `required_pull_request_reviews` | Approval rules (`approvals_before_merge`) |
| Repo description/topics/homepage | repo fields | project `description` / `topics` / `web_url` settings |
| Vuln alerts / secret scanning | security_and_analysis | Security & Compliance (secret detection, dependency scanning toggles) |
| Default CI permissions | Actions workflow permissions | CI/CD job token / pipeline settings |
| Auth | `GITHUB_TOKEN` / `gh` CLI | `GITLAB_TOKEN` / `glab` CLI |

Settings without a clean counterpart (e.g. GitHub-only `automated_security_fixes`) degrade
gracefully: the provider returns `Unsupported`, the rule is skipped for that provider, the finding is
not emitted (same graceful-degradation contract as today's missing-`gh` path).

### Module discipline (unchanged invariant)

`providers/` stays the sole boundary with any forge. `rules/` and `actions/` must depend on the
`RepoProvider` trait, never on `GitHubProvider`/`GitLabProvider` concretes. The CLAUDE.md rule
"Provider is GitHub-only … exactly one trait implementation" is updated to "exactly one trait, N
implementations; no other module reaches for a concrete provider."

## Action catalog (target)

The 4 existing operations collapse cleanly to a provider-agnostic set:

| ActionOperation | Mechanism | Replaces / adds | Drives |
|---|---|---|---|
| `CreateFile { path, template, vars }` | L | existing | all DOC*, CODE001, GIT002, `.editorconfig`/`.dockerignore` |
| `UpdateGitignore { entries }` | L | existing | FILE002/003, GIT003 |
| `ConfigureProtectedBranch { branch, cfg }` | P | renamed from `ConfigureBranchProtection` | branch protection (both providers) |
| `UpdateRepoSettings { patch }` | P | renamed from `UpdateGitHubSettings`, generalized | SEC011-017 |
| `UpdateRepoMetadata { description?, topics?, homepage? }` | P | **new** | META001/002/003 |

Five operations, all provider-agnostic. Every kept rule_id now has a planner mapping → no report-only
gap.

## Implementation strategy

Three independent, separately-committed plans (mirrors the v2.0.0 A/B/C structure):

1. **Plan A — Rule & action recentering.** Remove the dropped categories (engine registration in
   `src/rules/engine.rs`, `VALID_CATEGORIES` in `src/rules/constants.rs` — keep the mirror test
   green), their modules, tests, fixtures, presets. Add `UpdateRepoMetadata`; rename the two `P`
   operations to provider-agnostic names; wire planner mappings for every kept rule_id. Drop the OSV
   client and its deps.
2. **Plan B — Provider abstraction + GitLab.** Extract `RepoProvider` trait; refactor
   `GitHubProvider` to implement it; add `GitLabProvider` (`glab` CLI + token, mirroring the dual-auth
   contract); add `--provider {github|gitlab}` (auto-detect from remote). `wiremock` for both
   providers' HTTP paths.
3. **Plan C — Docs & surface.** Rewrite CLAUDE.md (6 categories, provider list, updated invariants).
   Update README, presets, schemas, completions/man. Update `.repolens.toml` schema for the new
   category set + `provider` key.

## Out of scope (v3)

- **Bitbucket / Gitea / other forges.** The trait makes them cheap later, but v3 ships GitHub +
  GitLab only.
- **CI-workflow auto-hardening** (the cut `workflows` actions) — revisit as a focused follow-up.
- **Re-architecting `custom` rules** into an action-producing form.
- **Reviving any scanner category** — deps/secrets/licenses stay out; users compose dedicated tools.

## Validation criteria

v3.0.0 ships only if:

- `cargo check && cargo fmt --check && cargo clippy --all-targets -- -D warnings` clean.
- `cargo test --all` green; `VALID_CATEGORIES` count test asserts **6**.
- **No report-only rule_id remains** — a test asserts every emitted rule_id has a planner mapping (or
  is on an explicit detection-only allowlist: DOC002/003/009, CODE002).
- Both providers exercised in CI (GitHub + GitLab paths, token and CLI modes) via `wiremock`.
- `repolens plan --provider gitlab` and `--provider github` both produce an `ActionPlan`; `apply`
  round-trips on a fixture repo per provider.
- README + CLAUDE.md match reality (category count, provider list, action catalog).

## Non-obvious risks

- **Self-audit bootstrap.** RepoLens audits itself; dropping categories changes its own baseline
  report and any golden/snapshot files in `src/compare/` and `tests/`. Re-baseline in Plan A.
- **Preset drift.** `src/config/presets/` reference removed categories; an unknown category in a
  preset is an error by design (presets are static at build time) — every preset must be updated in
  lockstep with Plan A or the build breaks.
- **`VALID_CATEGORIES` mirror test.** Must move from 15 → 6 atomically with the engine registration,
  or the unit test in `constants.rs` fails the build.
- **GitLab auth parity.** The dual-mode contract (token preferred, CLI fallback, graceful skip)
  must hold for `glab` exactly as for `gh`, including the offline-degradation path — both must be
  tested, not just the token path.
- **Schema/versioning.** `schemas/*.schema.json` enumerate categories; bump and update them, and
  decide whether `--provider` belongs in the report schema.

## Appendix — full rule_id → disposition mapping

Legend: **L** local-file action · **P** provider-config action · **✗** not auto-fixable.

| rule_id | category | detects | méca | disposition |
|---|---|---|---|---|
| DOC001 | docs | README missing | L | keep (action) |
| DOC002 | docs | README too short | L | keep (detect tail) |
| DOC003 | docs | README missing sections | ✗ | keep (detect tail) |
| DOC004 | docs | LICENSE missing | L | keep (action ✅) |
| DOC005 | docs | CONTRIBUTING missing | L | keep (action ✅) |
| DOC006 | docs | CODE_OF_CONDUCT missing | L | keep (action ✅) |
| DOC007 | docs | SECURITY.md missing | L | keep (action ✅) |
| DOC008 | docs | CHANGELOG missing | L | keep (action) |
| DOC009 | docs | CHANGELOG malformed | ✗ | keep (detect tail) |
| DOC010 | docs | empty Unreleased | L | keep (action) |
| FILE001 | files | large file >10MB | ✗ | **drop** |
| FILE002 | files | .gitignore missing | L | keep (action) |
| FILE003 | files | .gitignore entry missing | L | keep (action ✅) |
| FILE004 | files | temp file tracked | ✗ | **drop** |
| GIT001 | git | large binary >1MB | ✗ | **drop** |
| GIT002 | git | .gitattributes missing | L | keep (action) |
| GIT003 | git | sensitive file tracked | L | keep (action → gitignore) |
| CODE001 | codeowners | CODEOWNERS missing | L | keep (action) |
| CODE002 | codeowners | CODEOWNERS syntax error | ✗ | keep (detect tail) |
| CODE003 | codeowners | invalid owner ref | P/✗ | **drop** (network validate) |
| REL001-003 | (codeowners.rs) | releases hygiene | ✗ | **drop** |
| META001 | metadata | description missing | P | keep (action new) |
| META002 | metadata | topics missing | P | keep (action new) |
| META003 | metadata | homepage missing | P | keep (action new) |
| META004 | metadata | social image missing | ✗ | **drop** |
| SEC007 | security→settings | .github/settings.yml missing | L | keep (action) |
| SEC008-010 | security→settings | branch rules absent in settings.yml | P | keep (action) |
| SEC011 | settings | vuln alerts off | P | keep (action ✅) |
| SEC012 | settings | dependabot updates off | P | keep (GitHub-only; skip on GitLab) |
| SEC013/014 | settings | secret scanning / push protection off | P | keep (action) |
| SEC015 | settings | Actions allow-all | P | keep (action) |
| SEC016 | settings | default workflow perms = write | P | keep (action) |
| SEC017 | settings | fork PR no approval | P | keep (action) |
| SECURITY002 | security | lock file missing | ✗ | **drop** (can't write deterministically) |
| SECURITY003 | security | runtime version file missing | L | keep (action) |
| TEAM001-003, KEY001-002, APP001, HOOK001-003, ENV001-003 | security (access) | access/posture audit | ✗ | **drop** (Decision 2) |
| SEC001-003 | secrets | hardcoded secrets / sensitive files / .env | ✗ | **drop** |
| WF001-009 | workflows | CI hardening | L/✗ | **drop** (Decision 3) |
| DEP000-004 | dependencies | CVEs / lock / ecosystem | ✗ | **drop** |
| LIC001 | licenses | no project license | L | fold into DOC004 |
| LIC002-004 | licenses | dependency license issues | ✗ | **drop** |
| HIST001-004 | history | commit hygiene | ✗ | **drop** |
| ISSUE001-003, PR001-002 | issues | stale issues/PRs | ✗ | **drop** |
| DOCKER001/004-007 | docker | Dockerfile lint | ✗ | **drop** |
| DOCKER002/008 | docker | .dockerignore missing | L | keep as scaffolding (Decision 4) |
| DOCKER003 | docker | base image unpinned | ✗ | **drop** |
| QUALITY004 | quality | .editorconfig missing | L | keep as scaffolding (Decision 4) |
| QUALITY001/002/003/005/006/007/008/009 | quality | tooling configs | L/✗ | **drop** (Decision 4) |
| custom/* | custom | user-defined | varies | **drop** (reconsider post-v3) |
