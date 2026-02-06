# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.3.0] - 2026-02-06

### Added

#### Docker Distribution
- Official Docker image published to GitHub Container Registry (`ghcr.io/delfour-co/repolens`)
- Multi-architecture support (linux/amd64, linux/arm64)
- Multi-stage build for optimized image size (Alpine-based)
- `docker-compose.yml` for easy local usage
- Image verification job in CI/CD pipeline

#### Package Managers
- **Homebrew**: Formula for macOS and Linux (`brew install repolens`)
- **Scoop**: Manifest for Windows (`scoop install repolens`)
- **AUR**: PKGBUILD for Arch Linux (`yay -S repolens`)
- **Debian**: Packaging files for .deb distribution

#### CI/CD Integration Templates
- GitHub Actions reusable workflow (`integrations/github-actions/repolens.yml`)
- GitLab CI configuration (`integrations/gitlab-ci/.gitlab-ci.yml`)
- CircleCI configuration (`integrations/circleci/config.yml`)
- Jenkins declarative pipeline (`integrations/jenkins/Jenkinsfile`)
- Azure DevOps pipeline (`integrations/azure-devops/azure-pipelines.yml`)

#### Documentation
- `docs/docker.md` - Docker usage guide
- `docs/installation.md` - Comprehensive installation guide
- `docs/ci-cd-integration.md` - CI/CD integration guide for all platforms

## [1.2.0] - 2026-02-06

### Added

#### CLI Improvements
- **-C/--directory option**: Audit a different directory without changing current working directory
- **Environment variables configuration**: Configure RepoLens via `REPOLENS_*` environment variables
  - `REPOLENS_PRESET` - Default preset to use
  - `REPOLENS_VERBOSE` - Verbosity level (0-3)
  - `REPOLENS_CONFIG` - Path to config file
  - `REPOLENS_NO_CACHE` - Disable caching
  - `REPOLENS_GITHUB_TOKEN` - GitHub token for API calls
- **Verbose mode with timing**: Display execution time per category (`-v`, `-vv`, `-vvv`)

#### Error Handling
- Improved error messages with context and suggestions
- Colored error output for better readability
- `RepolensError` enum with structured error types

### Fixed
- Flaky tests due to parallel execution changing current directory (using `serial_test` crate)
- E2E tests excluded from tarpaulin coverage runs to prevent orphan processes

## [1.1.0] - 2026-02-06

### Added

#### Git Hygiene Rules
- **GIT001** (warning): Large binary files detected (>1MB, should use Git LFS)
- **GIT002** (info): `.gitattributes` file missing
- **GIT003** (warning): Sensitive files tracked (.env, *.key, *.pem, credentials, *_rsa)

#### Branch Protection Rules
- **SEC007** (info): `.github/settings.yml` missing
- **SEC008** (warning): No branch protection rules in settings.yml
- **SEC009** (warning): `required_pull_request_reviews` not configured
- **SEC010** (warning): `required_status_checks` not configured

#### Lock File Rules
- **DEP003** (warning): Lock file missing for detected ecosystem

#### New Ecosystems Support
- **.NET (NuGet)**: `*.csproj` / `packages.lock.json` - OSV supported
- **Ruby (Bundler)**: `Gemfile` / `Gemfile.lock` - OSV supported
- **Dart/Flutter (Pub)**: `pubspec.yaml` / `pubspec.lock` - OSV supported
- **Swift (SPM)**: `Package.swift` / `Package.resolved` - No OSV support
- **iOS (CocoaPods)**: `Podfile` / `Podfile.lock` - No OSV support

#### Additional Rules
- Docker best practices rules
- CI/CD configuration rules
- CHANGELOG validation rules
- Java (Maven/Gradle) and PHP (Composer) ecosystem support

#### Testing
- Comprehensive E2E test suite (38 tests)
- Test coverage improvements

## [1.0.1] - 2026-01-30

### Fixed

- **apply**: Prevent parasitic branch creation for API-only actions (settings updates no longer trigger file commits) (#139)
- **providers**: Use process exit code instead of HTTP status code for GitHub API checks (`has_vulnerability_alerts`, `has_automated_security_fixes`) (#140)
- **apply**: Show pending warning issue creation in planned actions when no file actions are required (#144)
- **apply**: Execute warning issue creation even when action plan is empty (#146)
- **apply**: Auto-create `repolens-audit` label before issue creation to prevent "label not found" errors (#147)

### Added

- **apply**: Selective file staging — only files modified by actions are committed, excluding report files (#141)
- **apply**: Automatic GitHub issue creation per warning category with `repolens-audit` label (#142)
- **apply**: `--no-issues` flag to disable automatic issue creation
- **providers**: `ensure_label()` method for auto-creating missing labels
- **providers**: `create_issue()` method for creating GitHub issues via `gh`

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

[1.3.0]: https://github.com/delfour-co/repolens/releases/tag/v1.3.0
[1.2.0]: https://github.com/delfour-co/repolens/releases/tag/v1.2.0
[1.1.0]: https://github.com/delfour-co/repolens/releases/tag/v1.1.0
[1.0.1]: https://github.com/delfour-co/repolens/releases/tag/v1.0.1
[1.0.0]: https://github.com/delfour-co/repolens/releases/tag/v1.0.0
