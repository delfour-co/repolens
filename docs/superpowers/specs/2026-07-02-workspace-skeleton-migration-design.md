# RepoLens → workspace skeleton migration

- **Date:** 2026-07-02
- **Status:** Approved (design); implementation plan pending
- **Author:** Kevin Delfour
- **Scope:** Structural migration of RepoLens onto the shared `_templates/cli` skeleton,
  matching the topology already applied to `josephine`.

## Context

RepoLens is currently a **single crate** with a flat `src/` layout (`main.rs` plus ~11 top-level
modules). The sibling CLI `josephine` was migrated onto the shared `rust-cli-template`
(`_templates/cli`), which produces a **workspace** split into a `-core` library crate and a thin
CLI binary crate, plus a set of normalized governance files.

This migration brings RepoLens to the same skeleton. It is a prerequisite the maintainer set before
resuming PR #236 (the v3.0.0 recentering): the tool is normalized first, then #236 is re-applied on
top of the new structure from its review comments.

## Goals

1. Convert RepoLens from a single crate to the template's **workspace topology**: `crates/repolens-core`
   (pure logic) + `crates/repolens` (CLI binary depending on core).
2. Adopt the template's structural files: workspace `Cargo.toml`, `rust-toolchain.toml`,
   `rustfmt.toml`, `.gitignore`, README/CHANGELOG conventions.
3. Add the normalized governance files josephine carries on top of the template.
4. Replace RepoLens-specific loose docs with the normalized equivalents.
5. Keep behavior **identical**: same CLI surface, same 15 rule categories, GitHub-only provider,
   all 1033 tests green.

## Non-goals

- **No feature or rule changes.** This is a pure structural refactor. The 15 categories, 5 output
  formats, and GitHub-only provider are preserved exactly as they are on `main` (v2.0.2).
- **PR #236 is not touched.** The v3.0.0 recentering (15→6 categories, GitLab provider) is a
  separate effort that rebases onto this skeleton afterward, driven by its posted review comments.
- **No dependency upgrades** beyond what the workspace hoist requires. Versions stay as-is.
- **No SQLite / state layer.** RepoLens caching stays file-based (`state=none`).

## Chosen approach — strategy C: fresh generate + port

Rejected alternatives:

- **A. Big-bang** hand restructure — one enormous diff, long red-to-green window. Rejected: hard to
  review, no test gate mid-flight.
- **B. Incremental hand-scaffold + `git mv`** — safe, but the workspace `Cargo.toml`, toolchain, and
  crate shells are hand-written, so template fidelity depends on my transcription.

Chosen: **C. Fresh `cargo generate` + port.** Generate a clean scaffold from the template (guaranteed
fidelity to the same provenance as josephine), then port RepoLens's code into it using `git mv` to
preserve history. `cargo generate` v0.23.12 is installed and confirmed working.

## Design

### Template axes

The template (`_templates/cli/cargo-generate.toml`) is generated with:

| Axis | Value | Rationale |
|------|-------|-----------|
| `topology` | `workspace` | core + bin split, matching josephine |
| `ui` | `cli` | RepoLens has no TUI |
| `state` | `none` | caching is file-based JSON, not SQLite |
| `privileges` | `single` | no privileged helper process |
| `service` | `none` | no background daemon |

Plus string placeholders: `project-name=repolens`, `repo_url=https://github.com/systm-d/repolens`,
`project_description`, `tagline`, `brand_color`.

The template's `.genignore` excludes `docs/`, `.superpowers/`, and `.template-dev/` from generated
output, so the scaffold is a clean shell: workspace `Cargo.toml`, `crates/repolens-core/` +
`crates/repolens/` skeletons, `rust-toolchain.toml`, `rustfmt.toml`, `.gitignore`, and
README/CHANGELOG/LICENSE files. `state=none` drops `migrations/` and `db.rs`; `ui=cli` drops any TUI.

### Generation procedure

The scaffold is generated into a **scratch directory** (never over RepoLens's `.git`). Its structural
files are then brought into a new branch of the existing RepoLens repository. This keeps the repo's
history, remote, and the open PR #236 intact.

### Crate cut

Verified against the current module graph. The only core→shell coupling is
`src/config/mod.rs` re-exporting `crate::hooks::HooksConfig`; `exit_codes` lives under `cli/` and is
consumed only by CLI commands. Both are resolved by the two seams below.

| Crate | Modules moved in | ~Rust files |
|-------|------------------|-------------|
| `repolens-core` | `actions/`, `cache/`, `compare/`, `config/`, `error.rs`, `providers/`, `rules/`, `scanner/`, `utils/` | ~48 |
| `repolens` (bin) | `main.rs`, `cli/` (commands + output + `exit_codes`), `hooks/`, `build.rs` | ~20 |

`git mv` preserves history for every ported file. The generated placeholder `lib.rs` / `main.rs` /
`cli.rs` are overwritten by RepoLens's real modules.

Benches (`parse`, `scanner`, `rules`) move to `crates/repolens-core/benches/`. Integration tests
(`assert_cmd`, they invoke the binary) move to `crates/repolens/tests/`.

### Two refactor seams

1. **`HooksConfig` → core.** The `HooksConfig` *type* (a serde config struct, part of the config
   schema) moves into `config` (core). The hook *installation logic* (script generation, file
   writes) stays in the bin crate's `hooks/`. This removes `config`'s dependency on `hooks`.
2. **`exit_codes` stays in bin.** Already only referenced by CLI commands; it lives in
   `crates/repolens/src/cli/exit_codes.rs`. The `lib.rs` re-export moves to the bin crate.

### Dependency reconciliation

The current flat dependency list splits into `[workspace.dependencies]` (shared, version-pinned once)
plus per-crate `[dependencies]` selecting from the workspace set.

- **`repolens-core`:** serde, serde_json, serde_yaml, toml, walkdir, ignore, globset, regex,
  lazy_static, url, sha2, dirs, octocrab, which, reqwest, tera, minijinja, jsonschema, rayon, chrono,
  tokio, async-trait, anyhow, thiserror, tracing, tracing-subscriber, metrics, metrics-prometheus,
  similar.
- **`repolens` (bin):** clap, clap_mangen, clap_complete, clap_complete_nushell, colored, indicatif,
  console, dialoguer, `repolens-core` (path dep), plus anyhow/tracing shared from workspace.
- **build-dependencies** (`clap_mangen`, `clap`, `clap_complete`, `clap_complete_nushell`) move with
  `build.rs` into the bin crate.
- **dev-dependencies** (tempfile, assert_cmd, predicates, pretty_assertions, criterion, futures,
  serial_test, insta) distribute to the crate whose tests use them.

The `[lints.rust]` `unexpected_cfgs` / `cfg(tarpaulin)` allowance is preserved (at the workspace or
per-crate level as the template dictates).

### Normalized governance files

Sourced from josephine (the template itself does **not** ship these) and adapted to RepoLens:

- `AGENTS.md` — points at the existing `CLAUDE.md`.
- `CONVENTIONS.md` — shared CLI standards (kept consistent with josephine's).
- `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`.
- `.github/ISSUE_TEMPLATE/*`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/dependabot.yml`.

Kept as-is: `CLAUDE.md`, `packaging/`, `deny.toml`, `tarpaulin.toml`, existing `LICENSE-*`.

### CI / release for the workspace

RepoLens's existing `.github/workflows/ci.yml` and `release.yml` are adapted (not regenerated — the
template ships no workflows):

- Build/test/clippy/fmt run with `--workspace`.
- Coverage (tarpaulin) covers the workspace; `tarpaulin.toml` updated for crate paths.
- Release binary path becomes `crates/repolens` output; `.deb` / rpm asset paths updated
  (mirroring josephine's `[package.metadata.deb]` / `generate-rpm` with `../../target/...`).
- crates.io publish order: `repolens-core` first, then `repolens` (path dep needs `version =`).
- Docker build (`ghcr.io/systm-d/repolens`, linux/amd64) updated for the new binary path.

### Docs replacement

`DEVELOPMENT.md`, `RELEASE.md`, `TEST.md`, `TODO.md` are folded into `CONTRIBUTING.md` and `docs/`
(build/test/release instructions → CONTRIBUTING; anything still live → docs/), then the originals are
removed. Loose report artifacts at the repo root (`repolens-report.json`, `repolens-report.md`) are
removed if not intentionally tracked.

## Guardrails / acceptance criteria

The migration is complete when, on the new branch:

1. `cargo build --workspace` and `cargo test --workspace` pass, with the **same 1033 tests** green.
2. `cargo clippy --all-targets --workspace -- -D warnings` is clean.
3. `cargo fmt --check` passes.
4. `cargo run -p repolens -- <subcommand>` reproduces the current CLI surface
   (`init | plan | apply | report | compare | install-hooks | schema | completions | generate-man`)
   byte-for-byte where output is deterministic.
5. `cargo deny check` passes.
6. No behavior change: 15 categories registered, `VALID_CATEGORIES` still mirrors
   `rules/engine.rs`, GitHub-only provider.
7. PR #236 branch is untouched.

## Risks

- **Import churn.** Moving modules across the crate boundary rewrites `crate::` paths to
  `repolens_core::` in the bin crate. Mitigation: mechanical, compiler-guided, gated by the test suite.
- **`build.rs` in a workspace.** Man/completion generation needs the clap `Command`; it moves with
  the bin crate and its `OUT_DIR` usage is unchanged. Verify generated man/completions still emit.
- **Publish ordering.** `repolens-core` must be published before `repolens`; the path dep carries a
  `version =` so crates.io resolves it. Verify with `cargo publish --dry-run` per crate.
- **Doc/spec cross-links.** `CLAUDE.md` and specs reference `src/...` paths that become
  `crates/repolens{,-core}/src/...`. Update references as part of the move.

## Out of scope / follow-ups

- Re-applying PR #236 (v3.0.0 recentering) on the new skeleton — separate effort, driven by the
  posted review comments.
- Any landing-page `site/` generator josephine carries — added later if desired, not part of this
  migration.
