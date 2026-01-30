# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-01-30

### Added

#### Core Features
- Initial release of RepoLens CLI tool
- Command-line interface with subcommands: `init`, `plan`, `apply`, `report`
- Configuration system with TOML-based config files
- Three preset configurations: `opensource`, `enterprise`, `strict`

#### Audit Capabilities
- Rules engine for repository auditing
- Multiple rule categories:
  - **secrets**: Detection of exposed API keys, tokens, and passwords
  - **files**: Validation of required repository files (README, LICENSE, etc.)
  - **docs**: Documentation completeness and quality checks
  - **security**: Security best practices validation
  - **workflows**: CI/CD and GitHub Actions validation
  - **quality**: Code quality standards checks
  - **custom**: User-defined custom rules via `.repolens.toml`
  - **dependencies**: Dependency security checking via OSV API
  - **licenses**: License compliance checking with SPDX compatibility matrix

#### Action System
- Action planner to generate fix plans based on audit results
- Action executor to apply fixes automatically
- Support for creating missing files from templates, updating `.gitignore`, configuring branch protection, updating GitHub repository settings

#### Interactive Mode for Apply Command (#3)
- Interactive action selection with `--interactive` (`-i`) flag
- Visual action summary with categorized overview and icons
- Colored diff preview (before/after) using the `similar` crate
- Progress bar and spinner for real-time feedback
- Auto-accept mode with `--yes` (`-y`) flag

#### Audit Results Caching (#5)
- Caching system for audit results to avoid redundant scans
- Configurable cache expiration

#### Git Hooks Support (#6)
- `install-hooks` CLI command to install and remove Git hooks
- Pre-commit hook for secret detection, pre-push hook for full audit
- Configurable via `[hooks]` section in `.repolens.toml`
- Automatic backup and restore of existing hooks

#### License Compliance Checking (#9)
- New rule category `licenses` with rules LIC001-LIC004
- Project license detection from LICENSE files, Cargo.toml, package.json, setup.cfg, pyproject.toml
- Dependency license parsing from Cargo.toml, package.json, requirements.txt, go.mod
- License compatibility matrix for common SPDX licenses
- Configurable allowed/denied license lists

#### JSON Schema for Audit Report Output (#15)
- JSON Schema (draft-07) for audit report output (`schemas/audit-report.schema.json`)
- `schema` subcommand to display or export the JSON Schema
- `--schema` and `--validate` flags on `report` command
- Enhanced JSON report with metadata, summary, and optional schema reference

#### Compare Command (#18)
- `repolens compare` command to compare two audit report JSON files
- Score comparison with weighted severity diff
- New/resolved issues detection with category breakdown
- Multiple output formats: terminal, JSON, markdown
- `--fail-on-regression` flag for CI integration

#### Output Formats
- Terminal output with colored formatting
- JSON output for programmatic consumption
- SARIF output for security tooling integration
- Markdown and HTML reports

#### Templates
- LICENSE templates (MIT, Apache-2.0, GPL-3.0)
- CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md templates
- GitHub issue templates (bug report, feature request) and pull request template

#### GitHub Integration
- GitHub API provider for repository management
- Branch protection configuration and repository settings management

#### Distribution
- Publication on crates.io via `cargo install repolens`
- Pre-built binaries for Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), Windows (x86_64)
- SHA256 checksum generation and verification
- Official GitHub Action with 7 configurable inputs and 3 outputs

#### CI/CD
- Automated release workflow with multi-platform builds
- Nightly build pipeline with quality gates
- Changelog generation script for releases
- Automatic GitHub Discussion announcements on release

#### Documentation
- Comprehensive README with usage examples
- Wiki documentation
- Configuration and preset examples

[1.0.0]: https://github.com/delfour-co/repolens/releases/tag/v1.0.0
