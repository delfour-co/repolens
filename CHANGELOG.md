# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Git Hooks Support (#6)
- New `install-hooks` CLI command to install and remove Git hooks
- **Pre-commit hook**: Checks for exposed secrets before each commit
- **Pre-push hook**: Runs a full audit before pushing to a remote
- Configurable via `[hooks]` section in `.repolens.toml`
- Automatic backup of existing hooks before overwriting (`--force`)
- Restore original hooks on removal
- Support for standard repositories and git worktrees
- 50+ unit tests covering hooks functionality

### Changed

- Increased CI coverage threshold from 90% to 95%
- Added `src/actions/git.rs` to tarpaulin exclusion list (external command execution module)

### Added

- Additional unit tests for improved coverage across multiple modules:
  - Cache module: save/load roundtrip, expired entries, edge cases
  - Config loader: glob matching edge cases, double-star patterns
  - Workflows rules: full `run` dispatcher, disabled rules, non-YAML files
  - Quality rules: Python, Ruby, Go, Rust project detection, linting configs
  - Secrets rules: template/sample env files, sensitive file detection, ignore patterns
  - Language detection: Go, Ruby, PHP, Java, C#, Gradle, Pipfile detection
  - Action planner: code of conduct, security policy, branch protection, GitHub settings
  - Templates: all template types, nested directories, variable replacement
  - Scanner: glob matching, file filtering, extension matching
  - Error types: additional display/conversion tests

#### Previous Code Coverage Improvement
- Improved test coverage to 90% minimum for core modules
- Fixed `test_has_changes` test race condition in `src/actions/git.rs` by using `std::sync::Mutex` instead of `tokio::sync::Mutex` for better cross-runtime compatibility
- Added comprehensive tests for:
  - CLI output modules (terminal, json, markdown, html, sarif)
  - Configuration modules (loader, presets)
  - Rules engine and results
  - Scanner modules (filesystem, git)
  - Error types
  - Action plan and templates
  - Cache module
- Updated CI coverage threshold from 50% to 90% with appropriate file exclusions for untestable modules (CLI commands, providers, external API dependencies)

### Added

#### Interactive Mode for Apply Command (#3)
- **Interactive action selection**: New `--interactive` (`-i`) flag enables users to select which actions to apply using a multi-select interface
- **Visual action summary**: Displays a categorized overview of all planned actions with icons per category
- **Diff preview**: Shows colored before/after diff for each action (green for additions, red for deletions) using the `similar` crate
- **Progress bar**: Real-time progress indicator during action execution with `indicatif`
- **Spinner for individual actions**: Visual feedback for each action being executed
- **Enhanced execution summary**: Detailed results display with success/failure counts
- **Auto-accept mode**: New `--yes` (`-y`) flag to skip confirmation prompts and apply all actions automatically

### Changed
- Improved terminal output formatting with better visual hierarchy
- Updated README documentation with interactive mode examples

## [0.2.0] - 2026-01-28

### Added

#### Distribution et Installation (#34, #35)
- **Publication sur crates.io** : RepoLens est maintenant disponible via `cargo install repolens` (#34)
- **Binaires pre-compiles multi-plateformes** : Binaires disponibles pour 5 plateformes (#35) :
  - Linux x86_64
  - Linux ARM64
  - macOS Intel (x86_64)
  - macOS Apple Silicon (ARM64)
  - Windows x86_64

#### GitHub Action officielle (#38)
- **Action GitHub officielle** pour integrer RepoLens dans vos workflows CI/CD (#38)
  - 7 inputs configurables : `preset`, `format`, `output`, `categories`, `exclude`, `verbose`, `fail-on-error`
  - 3 outputs exploitables : `score`, `report-path`, `issues-count`
  - 3 exemples d'utilisation fournis : audit basique, audit avec publication SARIF, audit multi-presets

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

[0.2.0]: https://github.com/kdelfour/repolens/releases/tag/v0.2.0
[0.1.0]: https://github.com/kdelfour/repolens/releases/tag/v0.1.0
