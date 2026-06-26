# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository status

**v3.0.0.** RepoLens is recentered as an **auto-configurator for git repositories**: every kept check
exists to drive a reviewable, applicable `Action`. Two providers (**GitHub** + **GitLab**), **6 rule
categories**, 5 output formats. The detection-only "scanner" surface (dependency CVEs, secrets,
dependency licenses, history, stale issues, docker linting, CI workflow linting) was removed — it is
not deterministically auto-fixable and overlaps dedicated tools (Dependabot/osv-scanner, gitleaks,
cargo-deny, hadolint, actionlint). Rust edition 2024, MSRV 1.85, dual license MIT OR Apache-2.0. Two
CI workflows: `ci.yml` (test matrix + clippy + fmt + coverage + security audit + packaging dry-run)
and `release.yml` (multi-platform binary build + crates.io publish + Docker image push to
`ghcr.io/systm-d/repolens`). Canonical decisions:
`docs/superpowers/specs/2026-06-26-repolens-autoconfigurator-multiprovider-design.md` (v3 recentering
+ multi-provider) and `docs/superpowers/plans/2026-06-26-plan-b-provider-abstraction-gitlab.md`.

What's in the codebase right now:

- **CLI surface:** `repolens { init | plan | apply | report | compare | install-hooks | schema | completions | generate-man }`, with `--provider {github|gitlab}` on `plan`/`report`/`apply` (auto-detected from the origin remote otherwise).
- **6 rule categories** (all registered in `src/rules/engine.rs`):
  - `files` — `.gitignore` presence and recommended entries
  - `docs` — README, LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CHANGELOG presence/quality
  - `security` — repo security **settings**: `.github/settings.yml`, branch protection, vulnerability
    alerts, secret scanning, Actions/workflow permissions, fork-PR approval (the auto-fixable half of
    the former `security` category; access-posture audit was dropped)
  - `git` — `.gitattributes` presence, sensitive files that should be gitignored
  - `codeowners` — CODEOWNERS file presence and syntax
  - `metadata` — repository description, topics/tags, homepage
- **5 output formats:** terminal (colored), JSON, Markdown, SARIF, HTML
- **Edition 2024, MSRV 1.85**, ~721 unit tests passing
- **2 CI workflows:** `ci.yml` and `release.yml`

## Product in one line

RepoLens audits a GitHub or GitLab repository and **configures it as well as possible, automatically**;
the differentiator is the **plan/apply split** — every audit produces a typed `ActionPlan` that an
operator can review and selectively apply, rather than executing fixes immediately. Every kept rule
maps to an applicable `Action` (enforced by the `test_no_kept_rule_is_report_only` contract test) or
sits on a documented detection-only allowlist.

## Architecture

```
src/
├── main.rs            — entry point; dispatches Commands enum
├── lib.rs             — public re-exports
├── cli/               — subcommand dispatch + output formatters
│   ├── commands/       — one file per top-level subcommand
│   └── output/         — terminal, json, markdown, sarif, html
├── config/            — config loading + preset selection
│   └── presets/        — baked-in preset definitions
├── rules/             — rules engine
│   ├── engine.rs       — parallel rule execution, category dispatch
│   ├── categories/     — 6 category modules
│   └── results.rs      — Finding + AuditResults types
├── actions/           — ActionPlan + per-action executors (CreateFile, UpdateGitignore,
│                          ConfigureProtectedBranch, UpdateRepoSettings, UpdateRepoMetadata)
├── cache/             — incremental audit result caching
├── compare/           — diff two audit reports
├── hooks/             — git hook installation and management
├── providers/         — RepoProvider trait + GitHub (octocrab + gh) and GitLab (glab) impls
├── scanner/           — filesystem + git tree iteration
└── utils/             — prerequisites checks, exit codes
```

## Module discipline

`rules/` is pure logic — no IO, no network. Each category module receives scanner output and emits
`Vec<Finding>`. New rules must not import from `providers/` or `actions/`.

`actions/` is pure planning: it turns `AuditResults` into a `Vec<Action>`. No network calls, no file
writes happen inside this module. Execution is the job of each action's executor, invoked by the
`apply` command.

`providers/` is the sole boundary with any forge. All GitHub/GitLab access originates here, behind the
`RepoProvider` trait. No other module reaches for `octocrab`, `gh`, or `glab`; rules and actions obtain
a provider via `providers::for_config(config)` and depend only on the trait, never on a concrete impl.

`cli/output/` is presentation only: it renders an `AuditResults` or `ActionPlan` to a string. No
side effects. Each format (terminal, json, markdown, sarif, html) is an independent file.

## Non-obvious constraints

These are load-bearing rules — violating them changes what the product is:

- **Plan/apply split is non-negotiable.** Every fix is computed first as an `Action`, then
  optionally applied. `repolens plan` produces the `ActionPlan`; `repolens apply` executes it.
  They are not the same command with a dry-run flag.
- **Presets are static at build time.** `.repolens.toml` overrides preset values; the preset
  definitions themselves are baked in under `src/config/presets/`. Do not accept arbitrary preset
  names from config — an unknown name is an error, not an implicit rule list.
- **Provider access goes through the `RepoProvider` trait.** `src/providers/` has the trait plus two
  implementations (`github.rs`, `gitlab.rs`); no other module references a concrete provider. Adding a
  forge means a new trait impl, not provider-specific branches scattered across `rules/`/`actions/`.
  Capabilities with no faithful counterpart on a provider (e.g. GitHub-only Dependabot/Actions
  settings on GitLab) must return `Err` so the caller skips — never a fabricated `Ok` value, which
  would emit a false finding.
- **No `Co-authored-by` in commits or PRs.** Single primary author per commit. Mentioning
  contributors in the commit body without the `Co-authored-by:` trailer is allowed.
- **No `feat!:` or similar `!` Conventional Commit shorthand.** The commit-msg hook rejects it.
  Use `feat: ...` + `BREAKING CHANGE: ...` in the body instead.
- **GitHub auth is dual-mode; GitLab uses `glab`.** GitHub prefers the `GITHUB_TOKEN` env var (no
  `gh` CLI required), falling back to `gh auth token`. GitLab goes through the `glab` CLI (which reads
  `GITLAB_TOKEN` itself); known gap — GitLab currently requires `glab` installed even with a token. A
  missing CLI is a graceful skip, never a hard audit failure; these paths must be exercised in CI.
- **Every kept rule maps to an `Action` or is on the detection-only allowlist.** The
  `test_no_kept_rule_is_report_only` contract test in `src/actions/planner.rs` asserts every emitted
  rule_id is either remediable (an action) or on the explicit allowlist, with the two sets disjoint
  and exhaustive. A new rule with no remediation breaks the build until it is mapped or allowlisted.
- **`VALID_CATEGORIES` in `src/rules/constants.rs` must mirror the categories registered in
  `src/rules/engine.rs`** (currently 6). Adding a category requires updating both files; the unit test
  in `constants.rs` asserts the count. The CLI `--only` / `--skip` flags rely on this constant.

## Working conventions

- Tests live in `tests/` (integration) and inline `#[cfg(test)] mod tests` (unit). Per-rule unit
  tests are inside `src/rules/categories/<category>.rs`.
- `cargo test --all` runs everything. `cargo test --lib` skips integration tests; useful while
  iterating on rule logic.
- Snapshot tests with `insta` are not yet required for existing tests; new tests that produce stable
  structured output should prefer snapshots.
- Use `tracing` for operator-facing diagnostic logs. The audit report (terminal / JSON / Markdown /
  SARIF / HTML) is the user-facing artifact.
- French is allowed in user-facing strings, error messages, and documentation. Code identifiers stay
  English.
- Configuration lives in `.repolens.toml` at the repository root.

## Distribution

Primary distribution is via **GitHub Releases** on `systm-d/repolens`: `release.yml` builds
multi-platform binaries (Linux x86\_64, Linux aarch64, macOS Apple Silicon, Windows x86\_64) on
`v*.*.*` tags. Packaging metadata under `packaging/` covers Homebrew (macOS), AUR (Arch Linux),
Debian `.deb`, and Scoop (Windows). Docker image published to `ghcr.io/systm-d/repolens`. The
crates.io path (`cargo install repolens`) is also supported.

## Common commands

```bash
cargo build                              # build the binary
cargo run -- init                        # write a default .repolens.toml
cargo run -- plan --preset opensource    # smoke the CLI
cargo run -- report --format html        # generate an HTML report
cargo test --all                         # all tests (unit + integration)
cargo test --lib                         # unit tests only (faster iteration)
cargo clippy --all-targets -- -D warnings
cargo fmt
cargo deny check                         # license + advisory audit
```

## Reference

- `docs/superpowers/specs/2026-06-26-repolens-autoconfigurator-multiprovider-design.md` — v3.0.0
  recentering (auto-configurator) + multi-provider decision (current).
- `docs/superpowers/plans/2026-06-26-plan-b-provider-abstraction-gitlab.md` — provider-abstraction +
  GitLab implementation plan (B1/B2/B3).
- `docs/superpowers/specs/2026-05-13-repolens-recentering-design.md` — v2.0.0 recentering (historical).
- `docs/superpowers/specs/` — per-version design specs.
- `docs/superpowers/plans/` — implementation plans.
- `../../system/guardians/` — sibling Rust project. Edition, MSRV, license, and CI workflow
  structure are deliberately aligned with guardians; when in doubt about engineering conventions,
  check there first.
