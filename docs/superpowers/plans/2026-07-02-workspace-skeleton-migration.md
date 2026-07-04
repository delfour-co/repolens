# Workspace Skeleton Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert RepoLens from a single flat crate into the shared `_templates/cli` workspace skeleton (`crates/repolens-core` + `crates/repolens`), matching josephine, with zero behavior change.

**Architecture:** Strategy C — generate a fresh scaffold with `cargo generate` for template-fidelity structural files, then port RepoLens's code into it. Porting is done **incrementally** (relocate the whole crate first, then carve `-core` out) so the test suite gates every task instead of one big-bang red window. Layout follows **josephine (thin core)**: `-core` is pure domain with no `clap`; the bin crate owns `cli`/`hooks`/`main`/`build.rs`.

**Tech Stack:** Rust edition 2024, MSRV 1.85, Cargo workspaces, `cargo generate` v0.23.12, `cargo test`/`clippy`/`fmt`, `cargo deny`, `cargo tarpaulin`.

## Global Constraints

- **Edition 2024, rust-version 1.85** — copied to `[workspace.package]`.
- **License:** `MIT OR Apache-2.0`. Dual `LICENSE-MIT` + `LICENSE-APACHE` kept.
- **Behavior-preserving:** 15 rule categories, 5 output formats, GitHub-only provider — unchanged.
- **All 1033 tests stay green** at the end of every task (`cargo test --workspace`).
- **`VALID_CATEGORIES` in `src/rules/constants.rs` mirrors `src/rules/engine.rs`** (moves to `crates/repolens-core/src/rules/`). The `build.rs` copy is separate pre-existing tech debt — do not "fix" it here.
- **Commit rules (repo hooks enforce these):** no `Co-authored-by:` trailer; no `feat!:`/`!` Conventional Commit shorthand (use `feat:` + `BREAKING CHANGE:` in body). The pre-commit hook runs RepoLens's own secret scan; the commit-msg hook validates format.
- **Branch:** `chore/workspace-skeleton-migration` (already created, spec already committed). **PR #236 is untouched.**
- **Decision record:** the raw template puts `cli`/`commands` in `-core` (fat core); this plan follows **josephine's thin core** (CLI in the bin), per the approved spec. If the maintainer prefers the literal template's fat-core layout, Tasks 2–3 change accordingly.

## File Structure (end state)

```
Cargo.toml                         # [workspace]: members, package, deps, lints, profile
rust-toolchain.toml  rustfmt.toml  # from scaffold
crates/
  repolens-core/
    Cargo.toml                     # pure-domain deps (no clap)
    src/lib.rs                     # re-exports domain modules
    src/{actions,cache,compare,config,providers,rules,scanner,utils}/  src/error.rs
    src/config/hooks_config.rs     # HooksConfig (moved from hooks/) — seam 1
    benches/{parse,scanner,rules}_benchmark.rs
  repolens/
    Cargo.toml                     # clap + CLI deps + repolens-core path dep
    build.rs                       # completion generation (unchanged internals)
    src/main.rs                    # mod cli; mod hooks; deps from repolens_core
    src/cli/  (commands + output + exit_codes)
    src/hooks/mod.rs               # install logic; imports HooksConfig from core
    tests/                         # integration (assert_cmd)
AGENTS.md CONVENTIONS.md CODE_OF_CONDUCT.md CONTRIBUTING.md SECURITY.md
.github/ISSUE_TEMPLATE/* .github/PULL_REQUEST_TEMPLATE.md .github/dependabot.yml
```

---

### Task 1: Become a workspace with one relocated member

Establish the Cargo workspace and relocate the *entire* existing crate to `crates/repolens/` with **zero code edits**. This isolates "become a workspace" from "split into core," so the risky import rewrite in Task 2 happens inside an already-green workspace.

**Files:**
- Generate (scratch): scaffold via `cargo generate`
- Create: `Cargo.toml` (workspace root), `rust-toolchain.toml`, `rustfmt.toml`
- Move: `src/` → `crates/repolens/src/`, `build.rs` → `crates/repolens/build.rs`, `benches/` → `crates/repolens/benches/`, `tests/` → `crates/repolens/tests/`, `Cargo.toml` → `crates/repolens/Cargo.toml`

**Interfaces:**
- Produces: a workspace whose single member `repolens` builds and tests identically to today.

- [ ] **Step 1: Generate the reference scaffold into scratch**

```bash
SCRATCH="$(mktemp -d)"
cargo generate --path /home/kdelfour/Workspace/Professionel/_templates/cli \
  --name repolens --destination "$SCRATCH" --vcs none --silent \
  -d project_description="A CLI tool to audit and prepare repositories for open source or enterprise standards" \
  -d tagline="Audit and prepare your repositories with confidence." \
  -d brand_color="#2D7DD2" \
  -d repo_url="https://github.com/systm-d/repolens" \
  -d topology=workspace -d ui=cli -d state=none -d privileges=single -d service=none
echo "$SCRATCH/repolens"
```
Expected: `✨ Done! New project created …/repolens`. Use this tree only as a reference for the manifests below; do **not** copy its placeholder `src/`.

- [ ] **Step 2: Copy the two toolchain files from the scaffold**

```bash
cp "$SCRATCH/repolens/rust-toolchain.toml" rust-toolchain.toml
cp "$SCRATCH/repolens/rustfmt.toml" rustfmt.toml
git add rust-toolchain.toml rustfmt.toml
```
`rust-toolchain.toml` = `channel="stable"`, `components=["rustfmt","clippy"]`. `rustfmt.toml` = `edition="2024"`, `max_width=100`.

- [ ] **Step 3: Relocate the existing crate wholesale**

```bash
mkdir -p crates/repolens
git mv Cargo.toml crates/repolens/Cargo.toml
git mv src crates/repolens/src
git mv build.rs crates/repolens/build.rs
git mv benches crates/repolens/benches
git mv tests crates/repolens/tests
```
Cargo.lock stays at the repo root (workspace lock).

- [ ] **Step 4: Write the minimal workspace root `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[profile.release]
lto = true
codegen-units = 1
strip = true
```
(Package metadata, `[workspace.package]`, `[workspace.dependencies]`, and `[workspace.lints]` are introduced in Task 3. For now `crates/repolens/Cargo.toml` remains the full self-contained manifest, minus the `[profile.release]` block, which now lives only at the root.)

- [ ] **Step 5: Remove the now-duplicated `[profile.release]` from the member manifest**

Delete the `[profile.release]` block from `crates/repolens/Cargo.toml` (Cargo forbids `profile` in a workspace member). Update its `exclude` paths that pointed at repo-root dirs (`docs/`, `.github/`, `coverage/`, `scripts/`, `wiki/`) — they are now outside the crate; drop the stale entries. Keep `readme = "README.md"` working by adding `readme = "../../README.md"`.

- [ ] **Step 6: Build and test the workspace**

Run: `cargo build --workspace`
Expected: PASS (clean build).
Run: `cargo test --workspace`
Expected: PASS — same 1033 tests, 0 failures.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "refactor: relocate crate into a cargo workspace"
```

---

### Task 2: Carve out `repolens-core`

Move the pure-domain modules into a new `repolens-core` library crate; make `repolens` depend on it; fix the two seams; keep tests green. This is the structural heart of the migration.

**Files:**
- Create: `crates/repolens-core/Cargo.toml`, `crates/repolens-core/src/lib.rs`, `crates/repolens-core/src/config/hooks_config.rs`
- Move: `crates/repolens/src/{actions,cache,compare,config,providers,rules,scanner,utils}` and `error.rs` → `crates/repolens-core/src/…`; `crates/repolens/benches` → `crates/repolens-core/benches`
- Modify: `crates/repolens/src/main.rs`, `crates/repolens/src/hooks/mod.rs`, `crates/repolens/Cargo.toml`, `crates/repolens/src/config/mod.rs` (during the move)

**Interfaces:**
- Produces: crate `repolens_core` exposing `actions`, `cache`, `compare`, `config` (incl. `config::HooksConfig`), `error` (`RepoLensError`, `ConfigError`), `providers`, `rules`, `scanner`, `utils`.
- Consumes (bin): `repolens_core::{config::get_env_verbosity, error::{RepoLensError, ConfigError}, …}`.

- [ ] **Step 1: Scaffold the core crate**

```bash
mkdir -p crates/repolens-core/src
```
Create `crates/repolens-core/Cargo.toml` (deps stay explicit here; hoisting is Task 3):

```toml
[package]
name = "repolens-core"
version = "2.0.2"
edition = "2024"
rust-version = "1.85"
authors = ["Kevin Delfour"]
description = "Core auditing library for RepoLens"
license = "MIT OR Apache-2.0"
repository = "https://github.com/systm-d/repolens"
homepage = "https://github.com/systm-d/repolens"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
toml = "0.8"
walkdir = "2"
ignore = "0.4"
globset = "0.4"
regex = "1.12"
lazy_static = "1"
url = "2"
sha2 = "0.10"
dirs = "5"
octocrab = "0.41"
which = "6"
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tera = "1"
minijinja = "2"
jsonschema = "0.29"
rayon = "1.8"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
metrics = "0.22"
metrics-prometheus = "0.3"
similar = "2"

[dev-dependencies]
tempfile = "3"
pretty_assertions = "1"
criterion = { version = "0.5", features = ["html_reports"] }
futures = "0.3"
serial_test = "3.3.1"
insta = "1"

[lints.rust]
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(tarpaulin)'] }

[[bench]]
name = "parse_benchmark"
harness = false
[[bench]]
name = "scanner_benchmark"
harness = false
[[bench]]
name = "rules_benchmark"
harness = false
```

- [ ] **Step 2: Move the domain modules and benches into core**

```bash
cd crates
git mv repolens/src/actions   repolens-core/src/actions
git mv repolens/src/cache     repolens-core/src/cache
git mv repolens/src/compare   repolens-core/src/compare
git mv repolens/src/config    repolens-core/src/config
git mv repolens/src/providers repolens-core/src/providers
git mv repolens/src/rules     repolens-core/src/rules
git mv repolens/src/scanner   repolens-core/src/scanner
git mv repolens/src/utils     repolens-core/src/utils
git mv repolens/src/error.rs  repolens-core/src/error.rs
git mv repolens/benches       repolens-core/benches
cd ..
```
Remove the three `[[bench]]` blocks from `crates/repolens/Cargo.toml`.

- [ ] **Step 3: Write core `lib.rs`**

Create `crates/repolens-core/src/lib.rs` (existing `src/lib.rs` doc comment, minus the bin-only modules `cli`/`hooks` and the `exit_codes` re-export):

```rust
//! # RepoLens core
//!
//! Pure auditing logic: scanning, rules, action planning, providers, caching,
//! comparison. No CLI, no `clap`. The `repolens` binary crate builds the CLI on
//! top of this library.

pub mod actions;
pub mod cache;
pub mod compare;
pub mod config;
pub mod error;
pub mod providers;
pub mod rules;
pub mod scanner;
pub mod utils;

pub use error::RepoLensError;
```
Delete the old `crates/repolens/src/lib.rs` if present (`git rm crates/repolens/src/lib.rs`) — the bin crate no longer needs a `lib.rs`.

- [ ] **Step 4: Seam 1 — move `HooksConfig` into core `config`**

Create `crates/repolens-core/src/config/hooks_config.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Configuration for RepoLens-managed Git hooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Whether to install the pre-commit hook
    #[serde(default = "default_true")]
    pub pre_commit: bool,
    /// Whether to install the pre-push hook
    #[serde(default = "default_true")]
    pub pre_push: bool,
    /// Whether warnings should cause hook failure
    #[serde(default)]
    pub fail_on_warnings: bool,
}

fn default_true() -> bool {
    true
}

impl Default for HooksConfig {
    fn default() -> Self {
        Self {
            pre_commit: true,
            pre_push: true,
            fail_on_warnings: false,
        }
    }
}
```
In `crates/repolens-core/src/config/mod.rs`: add `mod hooks_config;` and replace the line `pub use crate::hooks::HooksConfig;` with `pub use hooks_config::HooksConfig;`. (Copy the exact derives from the original `hooks/mod.rs:26` struct — match its `#[derive(...)]` attributes verbatim, including any `Deserialize`/`Serialize` and `Debug`.)

Delete the `HooksConfig` struct, `default_true`, and `impl Default for HooksConfig` (lines ~26–50) from `crates/repolens/src/hooks/mod.rs`, and add at its top: `use repolens_core::config::HooksConfig;`.

- [ ] **Step 5: Rewrite the bin entry `main.rs`**

In `crates/repolens/src/main.rs`, replace the module declarations block (`mod actions; … mod utils;`) with only the bin-owned modules and pull the rest from core:

```rust
mod cli;
mod hooks;

use repolens_core::config::get_env_verbosity;
use repolens_core::error::{self, RepoLensError};

use cli::exit_codes;
use cli::{Cli, Commands};
```
Leave the rest of `main.rs` unchanged (the `error::ConfigError::Serialize { … }` construction now resolves through `repolens_core::error`).

- [ ] **Step 6: Add the path dependency and trim the bin manifest**

In `crates/repolens/Cargo.toml`: add under `[dependencies]`:

```toml
repolens-core = { path = "../repolens-core", version = "2.0.2" }
```
Remove the dependencies now owned by core (serde_json, serde_yaml, toml, walkdir, ignore, globset, regex, lazy_static, url, sha2, octocrab, which, reqwest, tera, minijinja, jsonschema, rayon, metrics, metrics-prometheus, similar, async-trait). Keep in the bin: `clap`, `clap_mangen`, `clap_complete`, `clap_complete_nushell`, `serde`, `colored`, `indicatif`, `console`, `dialoguer`, `anyhow`, `thiserror`, `dirs`, `chrono`, `tokio`, `tracing`, `tracing-subscriber`. Keep `[build-dependencies]` as-is. Keep the dev-deps the bin's own tests use (`assert_cmd`, `predicates`, `tempfile`, `serial_test`, `pretty_assertions`, `insta` as needed).

- [ ] **Step 7: Compiler-driven import rewrite (bin)**

Run: `cargo build --workspace`
For each unresolved-import error of the form `crate::<m>` where `<m> ∈ {actions,cache,compare,config,providers,rules,scanner,utils,error}`, rewrite to `repolens_core::<m>` in the bin crate (`crates/repolens/src/cli/**` and `hooks/mod.rs`). Repeat until it builds. Intra-core references (`crate::…` inside `repolens-core`) are already correct and need no change.

Locate them:
```bash
grep -rn 'crate::\(actions\|cache\|compare\|config\|providers\|rules\|scanner\|utils\|error\)' crates/repolens/src
```

- [ ] **Step 8: Build, test, and verify green**

Run: `cargo build --workspace`
Expected: PASS.
Run: `cargo test --workspace`
Expected: PASS — 1033 tests, 0 failures.
Run: `cargo clippy --all-targets --workspace -- -D warnings`
Expected: clean.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "refactor: split domain logic into repolens-core crate"
```

---

### Task 3: Hoist shared deps and metadata into the workspace

Deduplicate versions into `[workspace.dependencies]` / `[workspace.package]` / `[workspace.lints]`, mirroring josephine.

**Files:**
- Modify: `Cargo.toml` (root), `crates/repolens-core/Cargo.toml`, `crates/repolens/Cargo.toml`

- [ ] **Step 1: Add workspace tables to the root manifest**

Extend the root `Cargo.toml` with `[workspace.package]` (version `2.0.2`, edition `2024`, rust-version `1.85`, authors, license, repository, homepage), `[workspace.dependencies]` (every third-party crate used by either member, pinned once), and `[workspace.lints]` (`rust.unsafe_code = "forbid"` per template + the existing `unexpected_cfgs` allowance under `[workspace.lints.rust]`, `clippy.all = { level = "warn", priority = -1 }`).

- [ ] **Step 2: Point both members at the workspace**

In each member `Cargo.toml`: replace `version`/`edition`/`rust-version`/`authors`/`license`/`repository`/`homepage` with `.workspace = true`, replace each third-party `dep = "x"` with `dep.workspace = true`, and replace the per-crate `[lints.rust]` block with `[lints]\nworkspace = true`. The bin keeps its crate-specific `[package]` fields (`description`, `keywords`, `categories`, `documentation`, `readme`, `[[bin]]`) and the `repolens-core` path dep.

- [ ] **Step 3: Verify and commit**

Run: `cargo build --workspace && cargo test --workspace`
Expected: PASS, 1033 tests.
```bash
git add -A
git commit -m "chore: hoist dependencies into workspace tables"
```

---

### Task 4: Add normalized governance files

Add the meta files josephine carries on top of the template, adapted to RepoLens. No build impact.

**Files:**
- Create: `AGENTS.md`, `CONVENTIONS.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.github/ISSUE_TEMPLATE/bug.yml`, `.github/ISSUE_TEMPLATE/feature.yml`, `.github/ISSUE_TEMPLATE/config.yml`, `.github/PULL_REQUEST_TEMPLATE.md`, `.github/dependabot.yml`

- [ ] **Step 1: Copy josephine's meta files as the base**

```bash
J=/home/kdelfour/Workspace/Professionel/systm-D/josephine
for f in CONVENTIONS.md CODE_OF_CONDUCT.md CONTRIBUTING.md SECURITY.md AGENTS.md; do cp "$J/$f" "$f"; done
mkdir -p .github/ISSUE_TEMPLATE
cp "$J/.github/PULL_REQUEST_TEMPLATE.md" .github/PULL_REQUEST_TEMPLATE.md
cp "$J/.github/dependabot.yml" .github/dependabot.yml
cp "$J"/.github/ISSUE_TEMPLATE/*.yml .github/ISSUE_TEMPLATE/
```

- [ ] **Step 2: Rewrite Joséphine-specific content for RepoLens**

Edit each copied file: replace "Joséphine"/"josephine" with "RepoLens"/"repolens", the tagline, the crate names (`josephine-core`→`repolens-core`), repo URLs, and the `AGENTS.md` pointer so it references RepoLens's existing `CLAUDE.md`. Verify `dependabot.yml` `directory`/ecosystem entries match the workspace (`/` for cargo, `.github/workflows` for actions). Keep `CONVENTIONS.md`'s shared-standards framing.

- [ ] **Step 3: Verify and commit**

Run: `cargo build --workspace`
Expected: PASS (docs don't affect the build).
```bash
git add -A
git commit -m "docs: add normalized governance files (AGENTS, CONVENTIONS, SECURITY, templates)"
```

---

### Task 5: Replace RepoLens-specific loose docs

Fold `DEVELOPMENT.md`, `RELEASE.md`, `TEST.md`, `TODO.md` into `CONTRIBUTING.md` + `docs/`, then remove the originals and stray report artifacts.

**Files:**
- Modify: `CONTRIBUTING.md`, `docs/…`
- Remove: `DEVELOPMENT.md`, `RELEASE.md`, `TEST.md`, `TODO.md`, and (after a git check) `repolens-report.json`, `repolens-report.md`

- [ ] **Step 1: Merge content by topic**

Move build/test/local-dev instructions from `DEVELOPMENT.md` and `TEST.md` into `CONTRIBUTING.md` (under "Development"/"Testing" sections). Move release steps from `RELEASE.md` into `docs/` (e.g. `docs/releasing.md`) and link it from `CONTRIBUTING.md`. Migrate any still-relevant `TODO.md` items into GitHub issues or `docs/`; discard done items.

- [ ] **Step 2: Remove the originals and stray artifacts**

```bash
git rm DEVELOPMENT.md RELEASE.md TEST.md TODO.md
git ls-files --error-unmatch repolens-report.json repolens-report.md 2>/dev/null \
  && git rm repolens-report.json repolens-report.md || echo "report artifacts not tracked — skip"
```

- [ ] **Step 3: Update cross-references and commit**

Grep for links to the removed files and repoint them:
```bash
grep -rn 'DEVELOPMENT.md\|RELEASE.md\|TEST.md\|TODO.md' --include='*.md' . || echo "no dangling links"
```
```bash
git add -A
git commit -m "docs: fold DEVELOPMENT/RELEASE/TEST/TODO into CONTRIBUTING and docs"
```

---

### Task 6: Update CI, release, and packaging for the workspace

Adapt RepoLens's existing workflows and packaging to the two-crate layout.

**Files:**
- Modify: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `tarpaulin.toml`, `packaging/**`, `Dockerfile`

- [ ] **Step 1: `ci.yml` — workspace flags**

Add `--workspace` to build/test/clippy invocations; ensure `cargo fmt --all --check`; point coverage at the workspace. Keep the `cargo deny check` and packaging-dry-run jobs.

- [ ] **Step 2: `release.yml` — binary path + publish order**

Update the built-binary path to the workspace target (`target/release/repolens`, produced by `crates/repolens`). Publish crates.io in order: `cargo publish -p repolens-core` then `cargo publish -p repolens`. Update the Docker build context/binary path (`ghcr.io/systm-d/repolens`, linux/amd64).

- [ ] **Step 3: `tarpaulin.toml` and packaging paths**

Update `tarpaulin.toml` for the workspace. Update `packaging/**` (Homebrew/AUR/deb/Scoop) and any `[package.metadata.deb]`/`generate-rpm` asset paths to `../../target/release/repolens` and `../../{README,LICENSE-*}`, mirroring josephine's bin manifest.

- [ ] **Step 4: Verify locally where possible, then commit**

Run: `cargo build --release --workspace`
Expected: PASS; `target/release/repolens` exists.
Run: `cargo publish -p repolens-core --dry-run` then `cargo publish -p repolens --dry-run --allow-dirty`
Expected: both package successfully (bin resolves the path dep via its `version`).
Run: `cargo deny check`
Expected: PASS.
```bash
git add -A
git commit -m "ci: adapt CI, release, and packaging to the workspace layout"
```

---

### Task 7: Final verification and doc-path refresh

Confirm the full acceptance criteria and update path references in `CLAUDE.md` and specs.

**Files:**
- Modify: `CLAUDE.md`, `docs/superpowers/specs/2026-05-13-repolens-recentering-design.md` (path refs only if broken)

- [ ] **Step 1: Refresh `CLAUDE.md` architecture section**

Rewrite the `src/` tree in `CLAUDE.md` to the `crates/repolens{,-core}/src/…` layout. Update the module-discipline notes (`rules/` etc. now under `repolens-core`), the "Common commands" (`cargo run -p repolens -- …`), and the constants note (`crates/repolens-core/src/rules/constants.rs`).

- [ ] **Step 2: Full acceptance gate**

Run: `cargo build --workspace`  → PASS
Run: `cargo test --workspace`   → PASS, **1033 tests**, 0 failures
Run: `cargo clippy --all-targets --workspace -- -D warnings` → clean
Run: `cargo fmt --all --check`  → clean
Run: `cargo deny check`         → PASS

- [ ] **Step 3: CLI surface + generator smoke test**

```bash
cargo run -p repolens -- --help
for c in init plan apply report compare install-hooks schema completions generate-man; do
  cargo run -p repolens -- "$c" --help >/dev/null && echo "ok: $c"
done
GENERATE_COMPLETIONS=1 cargo build -p repolens && ls target/completions
cargo run -p repolens -- generate-man --output /tmp/repolens-man && ls /tmp/repolens-man
```
Expected: every subcommand help renders; completions and man pages generate. If completions land somewhere other than the expected path due to the workspace target dir, adjust `build.rs` to write under `OUT_DIR`/workspace `target`.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "docs: update CLAUDE.md for the workspace layout"
```

---

## Self-Review

**Spec coverage:**
- Goal 1 (workspace split) → Tasks 1–2. Goal 2 (structural files) → Tasks 1, 3. Goal 3 (governance files) → Task 4. Goal 4 (replace docs) → Task 5. Goal 5 (behavior-preserving, 1033 tests) → gate at the end of Tasks 1–3, 7.
- Spec §CI/release → Task 6. Spec §Two seams → Task 2 Steps 4–5. Spec §Dependency reconciliation → Task 2 Step 6 + Task 3. Spec §Docs replacement → Task 5. Spec §Guardrails/acceptance → Task 7.
- No spec section left without a task.

**Placeholder scan:** No "TBD"/"handle edge cases"/"write tests for the above". Refactor steps give exact `git mv`/`grep`/`cargo` commands; code-changing steps (seam, lib.rs, main.rs, manifests) show the actual content.

**Type consistency:** `HooksConfig` fields (`pre_commit`, `pre_push`, `fail_on_warnings`) and `repolens_core::config::HooksConfig` path are consistent across Task 2 Steps 4–6. `repolens_core::error::{RepoLensError, ConfigError}` matches `main.rs` usage. Module list moved to core is identical in Task 2 Steps 2, 3, 7 and the file-structure map.
