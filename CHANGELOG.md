# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.2] - 2026-02-11

### Fixed

- Improved OSV API error handling by filtering packages with empty names or versions
- Added debug logging for failed OSV API requests to help diagnose issues

## [1.1.1] - 2026-02-11

### Fixed

- Fixed scanner including directories in file list, causing "Is a directory" errors when scanning repositories with directories named like files (e.g., `openapi.json/`)

## [1.1.0] - 2026-02-11

### Added

#### Documentation
- Comprehensive module-level documentation with examples for all public APIs
- Enhanced contribution guide with detailed workflow and code standards
- Rustdoc documentation for all public types and functions

#### Testing
- 932 unit tests (up from 850)
- Test coverage at 86.63% for unit-testable code
- Tarpaulin configuration for accurate coverage reporting
- New tests for CLI commands, parsers, and utilities

### Changed

- Updated `regex` dependency to 1.12 for improved performance
- Refactored code organization for better maintainability
- Improved error messages and documentation strings

### Fixed

- Minor code quality improvements identified by enhanced test coverage

## [1.0.0] - 2026-02-08

### Added

#### Core Audit Engine
- Multi-category rule engine with parallel execution
- 50+ audit rules across 11 categories: secrets, files, docs, security, workflows, quality, dependencies, licenses, docker, git, custom
- Configurable presets: `opensource`, `enterprise`, `strict`
- Custom rules support via regex patterns or shell commands
- JSON Schema validation for audit reports

#### Security Features
- Secret detection (AWS, GitHub tokens, private keys, passwords, etc.)
- Dependency vulnerability scanning via OSV API (9 ecosystems supported)
- License compliance checking with allow/deny lists
- Branch protection validation
- Secure file permissions for configuration files (chmod 600 on Unix)

#### Supported Ecosystems
- **Rust** (Cargo): `Cargo.toml` / `Cargo.lock`
- **Node.js** (npm/yarn): `package.json` / `package-lock.json` / `yarn.lock`
- **Python** (pip/poetry): `requirements.txt` / `Pipfile` / `pyproject.toml`
- **Go**: `go.mod` / `go.sum`
- **Java** (Maven/Gradle): `pom.xml` / `build.gradle`
- **PHP** (Composer): `composer.json` / `composer.lock`
- **.NET** (NuGet): `*.csproj` / `packages.lock.json`
- **Ruby** (Bundler): `Gemfile` / `Gemfile.lock`
- **Dart/Flutter** (Pub): `pubspec.yaml` / `pubspec.lock`

#### CLI Features
- Commands: `init`, `plan`, `apply`, `report`, `compare`, `schema`, `install-hooks`, `generate-man`
- Output formats: Terminal (colored), JSON, SARIF, Markdown, HTML
- Verbose mode with timing breakdown (`-v`, `-vv`, `-vvv`)
- Directory option (`-C/--directory`) to audit different directories
- Environment variables configuration (`REPOLENS_*`)
- Standardized exit codes (0-4) for CI/CD integration
- Man page generation

#### Caching & Performance
- SHA256-based cache invalidation for faster audits
- Configurable cache TTL and directory
- `--no-cache` and `--clear-cache` options

#### Git Integration
- Pre-commit hook (secrets detection)
- Pre-push hook (full audit)
- Git hygiene rules (large binaries, gitattributes, sensitive files)

#### GitHub Integration
- Branch protection validation
- Automatic issue creation for warnings
- Repository settings validation
- GitHub Actions workflow validation

#### Compare & Diff
- Compare two audit reports
- Detect regressions and improvements
- Score-based comparison (Critical=10, Warning=3, Info=1)
- Multiple output formats (terminal, JSON, Markdown)

#### Distribution
- Docker image (multi-arch: amd64, arm64) on GitHub Container Registry
- Homebrew formula (macOS/Linux)
- Scoop manifest (Windows)
- AUR package (Arch Linux)
- Debian packaging

#### CI/CD Integration Templates
- GitHub Actions reusable workflow
- GitLab CI configuration
- CircleCI configuration
- Jenkins declarative pipeline
- Azure DevOps pipeline

#### Documentation
- Comprehensive wiki documentation
- Installation guide for all platforms
- Custom rules documentation with security considerations
- CI/CD integration guide

### Security

- CVE-2026-0007: Fixed integer overflow in `bytes` crate (updated to 1.11.1)
- Secure file permissions for `.repolens.toml`
- Shell command injection warnings for custom rules

### Testing

- 850+ unit tests
- 56 E2E integration tests
- Comprehensive error handling tests

---

[1.0.0]: https://github.com/delfour-co/repolens/releases/tag/v1.0.0
