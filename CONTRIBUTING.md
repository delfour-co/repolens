# Contributing to RepoLens

Thanks for your interest in improving RepoLens — a CLI that audits and prepares your
GitHub repositories for open-source and enterprise standards.

## Before you start

- Read the [conventions](CONVENTIONS.md): edition, formatting, lints, commit style.
- By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
- RepoLens is a 2-crate Cargo workspace (`repolens-core` + `repolens`). See
  [DEVELOPMENT.md](DEVELOPMENT.md) for architecture details.

## Development setup

```sh
git clone https://github.com/systm-d/repolens
cd repolens
cargo build --workspace
```

The toolchain is pinned by `rust-toolchain.toml` (stable + rustfmt + clippy).

## Running RepoLens locally

```sh
cargo run -p repolens -- plan --preset opensource
cargo run -p repolens -- report --format html
```

## Quality gate

Run this before opening a pull request — CI enforces the same:

```sh
cargo fmt --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
```

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
