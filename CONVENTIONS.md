# Conventions

The written source of truth for the standards shared across this project (and the
other CLIs built from the same `rust-cli-template` mood). Governance files link here
instead of restating these rules.

## Language & Edition

- Rust **edition 2024**, MSRV **1.85** (pinned via `rust-toolchain.toml`).
- Formatting: `rustfmt` with `max_width = 100`, edition 2024 (`rustfmt.toml`).
  `cargo fmt --check` must pass.
- Lints: `unsafe_code = "deny"` at the workspace level; clippy `all = { level = "warn", priority = -1 }`,
  inherited per crate via `[lints] workspace = true`. A small number of documented
  `#[allow(unsafe_code)]` sites exist in `repolens-core` for env-var access; see
  `Cargo.toml` and `.superpowers/sdd/task-3-report.md`.
  `cargo clippy --workspace --all-targets -- -D warnings` must pass.

## Project shape

- Workspace: `repolens-core` (pure logic library) + `repolens` (binary, thin CLI).
- Module discipline: business logic — rules engine, action planning, providers,
  scanner, cache, config — lives in `repolens-core`; argument parsing, subcommand
  dispatch, and output formatting live in the `repolens` binary (`cli/`, `hooks/`).
  The CLI dispatch is thin — no business logic in `cli/`.
- `rules/` is pure logic — no IO, no network. `providers/` is the sole boundary with
  GitHub. `cli/output/` is presentation only. See `CLAUDE.md` for the full module map.

## Language of text

- Documentation (README, this file, governance, future site) is in **English**.
- User-facing strings — CLI output, error messages — may be in **French** or English;
  French is allowed and intentional in places, but never required. Never
  `ERROR`/`FATAL`/`PANIC` in user-facing text; prefer clear, actionable messages.
- Code identifiers are in English.

## Git & releases

- **Conventional Commits** (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`,
  `test:`, `build:`, `style:`, `ci:`). No `feat!:` or similar `!` shorthand — use
  `feat: ...` plus a `BREAKING CHANGE: ...` trailer in the body instead.
- **Keep a Changelog** format in `CHANGELOG.md`; **Semantic Versioning**.
- Dual license: **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`).
- Single primary author per commit; no `Co-authored-by:` trailer.

## Caching

- Audit results are cached as JSON via `repolens-core/src/cache/` to speed up
  incremental runs. Cache entries are invalidated by content hash (SHA-256), not by
  a database — there is no SQL store in this project.

## Quality gate (run before every PR)

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
