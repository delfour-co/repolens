# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Cross-platform pre-built binaries for Linux (x86_64, ARM64), macOS (Intel, Apple Silicon), and Windows (x86_64)
- Automated release workflow that builds, archives, and publishes binaries on version tags
- SHA256 checksum generation and consolidated `checksums.sha256` file for release verification
- Nightly build pipeline with quality gates (CI status, code quality, coverage)
- Installation documentation for downloading and verifying pre-built binaries

## [0.1.0] - 2026-01-24

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

#### Action System
- Action planner to generate fix plans based on audit results
- Action executor to apply fixes automatically
- Support for:
  - Creating missing files from templates
  - Updating `.gitignore` files
  - Configuring GitHub branch protection
  - Updating GitHub repository settings

#### Output Formats
- Terminal output with colored formatting
- JSON output for programmatic consumption
- SARIF output for security tooling integration
- Markdown reports
- HTML reports

#### Templates
- LICENSE templates (MIT, Apache-2.0, GPL-3.0)
- CONTRIBUTING.md template
- CODE_OF_CONDUCT.md template
- SECURITY.md template
- GitHub issue templates (bug report, feature request)
- Pull request template

#### GitHub Integration
- GitHub API provider for repository management
- Branch protection configuration
- Repository settings management

#### Documentation
- Comprehensive README with usage examples
- Configuration examples
- CLI help documentation

### Changed

- N/A (initial release)

### Deprecated

- N/A (initial release)

### Removed

- N/A (initial release)

### Fixed

- N/A (initial release)

### Security

- Initial security-focused auditing capabilities
- Secret detection patterns
- Security policy templates

[0.1.0]: https://github.com/kdelfour/repolens/releases/tag/v0.1.0
