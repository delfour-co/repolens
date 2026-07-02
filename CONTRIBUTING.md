# Contributing to RepoLens

Thanks for your interest in improving RepoLens — a CLI that audits and prepares your
GitHub repositories for open-source and enterprise standards.

## Before you start

- Read the [conventions](CONVENTIONS.md): edition, formatting, lints, commit style.
- By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
- RepoLens is a 2-crate Cargo workspace (`repolens-core` + `repolens`). See
  [docs/architecture.md](docs/architecture.md) for the module layout and
  [docs/development.md](docs/development.md) for the full dev reference.

## Development setup

Prerequisites: Rust stable (pinned by `rust-toolchain.toml`, includes rustfmt + clippy), Git, and
either a `GITHUB_TOKEN` env var or the [GitHub CLI](https://cli.github.com/) (`gh auth login`) —
needed by commands and rule categories that talk to the GitHub API.

```sh
git clone https://github.com/systm-d/repolens
cd repolens
cargo build --workspace
./scripts/install-hooks.sh   # pre-commit (fmt + clippy) and commit-msg (Conventional Commits)
```

## Running RepoLens locally

```sh
cargo run -p repolens -- plan --preset opensource
cargo run -p repolens -- report --format html
```

Use `-v` / `-vv` / `-vvv` for increasing log verbosity, or `RUST_LOG=debug cargo run -p repolens -- plan`.

## Testing

```sh
cargo test --workspace              # unit + integration tests
cargo test -p repolens-core         # a single crate
cargo test --workspace -- --nocapture   # see println!/dbg! output
```

Integration tests live in `crates/repolens/tests/` (CLI end-to-end, security, regression,
providers). `repolens-core` has Criterion benchmarks under `crates/repolens-core/benches/`:

```sh
cargo bench -p repolens-core
```

## Quality gate

Run this before opening a pull request — CI enforces the same:

```sh
cargo fmt --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
```

## Releasing

Maintainers: see [docs/releasing.md](docs/releasing.md) for the tag-and-publish process.

## Commits & pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`, …). No `feat!:` or
  similar `!` shorthand — use `feat: ...` plus a `BREAKING CHANGE: ...` trailer in
  the body instead.
- Add a `CHANGELOG.md` entry under `[Unreleased]` for user-visible changes.
- Keep docs and code identifiers in English; user-facing CLI strings may be in
  French or English.
- One focused change per PR. Fill in the pull request template.

## Reporting bugs & ideas

Open an issue using the bug or feature template. For security issues, do **not**
open a public issue — see [SECURITY.md](SECURITY.md).
