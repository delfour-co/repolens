# Releasing RepoLens

This describes how to cut a RepoLens release. It targets maintainers with push and tag access to
`systm-d/repolens`.

## Overview

RepoLens is a 2-crate Cargo workspace (`repolens-core` + `repolens`), versioned together via
`[workspace.package].version` in the root `Cargo.toml`. A release is triggered by pushing a
`v*.*.*` tag; `.github/workflows/release.yml` then builds cross-platform binaries, generates the
changelog, creates the GitHub release, publishes to crates.io, and builds/pushes the Docker image.

## Cutting a release

1. **Bump the workspace version** in the root `Cargo.toml`:

   ```toml
   [workspace.package]
   version = "2.0.3"
   ```

   Both `crates/repolens-core` and `crates/repolens` inherit this via `version.workspace = true`.

2. **Run the quality gate** (see [CONTRIBUTING.md](../CONTRIBUTING.md#quality-gate)), then commit:

   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 2.0.3"
   ```

3. **Tag and push**:

   ```bash
   git tag -a v2.0.3 -m "Release 2.0.3"
   git push origin main
   git push origin v2.0.3
   ```

Pushing the tag triggers `release.yml`, which:

- Builds release binaries for Linux x86_64/ARM64, macOS Intel/Apple Silicon, and Windows x86_64,
  each with a checksum and (on Linux x86_64) a generated man page.
- Generates a CHANGELOG entry from Conventional Commits (`scripts/generate-changelog.sh`) and
  commits it back to `main`.
- Creates the GitHub release with all binaries, checksums, and the man page attached.
- Publishes to crates.io (see below).
- Builds and pushes the Docker image to `ghcr.io/systm-d/repolens` — `linux/amd64` only; see the
  note in `release.yml` on why ARM64 was dropped.

To re-run a release from an existing tag (e.g. after a partial failure), use the workflow's
`workflow_dispatch` trigger: Actions → Release → Run workflow, with `ref` set to the tag.

## Publishing to crates.io

`repolens` depends on `repolens-core` via a path + version dependency, so **`repolens-core` must be
published first** — crates.io rejects a package whose path dependency isn't already on the
registry:

```bash
cargo publish -p repolens-core
# wait for the crate to become available on crates.io (usually a few seconds)
cargo publish -p repolens
```

Both crates always share the workspace version, so they ship in lockstep.

> The `publish-crate` job in `release.yml` automates this. Its primary path is
> `cargo publish --workspace --allow-dirty` (stable since Cargo 1.90): it topologically sorts the
> two crates, builds `repolens` against a local registry overlay of the freshly-packaged
> `repolens-core`, then uploads in batches, automatically waiting for each batch to be indexed
> before publishing the next. Because `cargo publish` is explicitly non-atomic — a mid-workspace
> server error can leave `repolens-core` published and `repolens` not, with no built-in resume —
> the job falls back to the ordered, per-crate form above: it re-checks each crate's published
> status via the crates.io API, publishes only what's missing (`repolens-core` first, waiting up to
> 5 minutes for it to be indexed, then `repolens`), so a re-run after a partial failure won't hit
> "crate version already exists" on `repolens-core`.

## Version format

Follow [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`, optionally with a
pre-release suffix (`2.0.3-beta.1`). Bump MAJOR for breaking changes, MINOR for new features, PATCH
for fixes.

## Verifying release artifacts

Each release includes a consolidated `checksums.sha256`:

```bash
sha256sum -c checksums.sha256 --ignore-missing
```

## Generating the changelog manually

```bash
./scripts/generate-changelog.sh <from-tag> <to-tag>
```

`release.yml` runs this automatically between the previous tag and the new one; run it by hand to
preview an entry or to regenerate one after a failed release.

## Troubleshooting

- **Workflow doesn't trigger**: confirm the tag matches `v*.*.*` and was pushed
  (`git push origin v2.0.3`).
- **Changelog entry is empty**: check that there are commits between the previous and new tags.
- **A platform's artifacts are missing**: check the `build` job logs for that matrix entry in
  GitHub Actions — `fail-fast: false` means one platform failing doesn't block the others.
- **crates.io publish fails**: confirm `repolens-core` published successfully first (see above);
  `repolens`'s path dependency on it must resolve against the published version.
