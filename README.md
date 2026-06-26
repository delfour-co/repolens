# RepoLens

A CLI auto-configurator for GitHub and GitLab repositories. It audits repository
hygiene and configuration, then turns every finding into a reviewable, applicable
action so you can bring a repository up to a consistent standard.

> RepoLens is not a secret or dependency scanner. Those concerns overlap with
> dedicated tooling; RepoLens focuses on repository structure, documentation, and
> hosting-platform configuration.

## Features

- Audit GitHub and GitLab repositories for hygiene and configuration best practices
- Check for required files (README, LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY)
- Validate CODEOWNERS presence and validity
- Recommend `.gitignore` and `.gitattributes` entries
- Audit repository settings and branch protection
- Verify repository metadata (description, topics/tags)
- Generate actionable fix plans (plan/apply split)
- Apply fixes automatically or with dry-run mode
- Multiple output formats: terminal, JSON, SARIF, Markdown, HTML

## Installation

### Docker (Recommended)

The easiest way to use RepoLens without local installation:

```bash
# Pull the official image
docker pull ghcr.io/systm-d/repolens:latest

# Audit current directory
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens plan

# Generate a report
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens report --format json
```

See [docs/docker.md](docs/docker.md) for detailed Docker usage.

### Package Managers

#### Homebrew (macOS/Linux)

```bash
brew tap systm-d/repolens
brew install repolens
```

#### Scoop (Windows)

```powershell
scoop bucket add systm-d https://github.com/systm-d/scoop-bucket
scoop install repolens
```

#### AUR (Arch Linux)

```bash
yay -S repolens
```

### From crates.io

```bash
cargo install repolens
```

### Pre-built Binaries

Pre-built binaries are available for all major platforms. Download the latest release from the [Releases page](https://github.com/systm-d/repolens/releases).

#### Supported Platforms

| Platform | Architecture | Archive |
|----------|-------------|---------|
| Linux | x86_64 | `repolens-linux-x86_64.tar.gz` |
| Linux | ARM64 | `repolens-linux-arm64.tar.gz` |
| macOS | Intel x86_64 | `repolens-darwin-x86_64.tar.gz` |
| macOS | Apple Silicon ARM64 | `repolens-darwin-arm64.tar.gz` |
| Windows | x86_64 | `repolens-windows-x86_64.zip` |

#### Linux (x86_64)

```bash
curl -LO https://github.com/systm-d/repolens/releases/latest/download/repolens-linux-x86_64.tar.gz
tar xzf repolens-linux-x86_64.tar.gz
sudo mv repolens /usr/local/bin/
```

#### Linux (ARM64)

```bash
curl -LO https://github.com/systm-d/repolens/releases/latest/download/repolens-linux-arm64.tar.gz
tar xzf repolens-linux-arm64.tar.gz
sudo mv repolens /usr/local/bin/
```

#### macOS (Apple Silicon)

```bash
curl -LO https://github.com/systm-d/repolens/releases/latest/download/repolens-darwin-arm64.tar.gz
tar xzf repolens-darwin-arm64.tar.gz
sudo mv repolens /usr/local/bin/
```

#### macOS (Intel)

```bash
curl -LO https://github.com/systm-d/repolens/releases/latest/download/repolens-darwin-x86_64.tar.gz
tar xzf repolens-darwin-x86_64.tar.gz
sudo mv repolens /usr/local/bin/
```

#### Windows (x86_64)

```powershell
# Download the zip archive from the Releases page
Invoke-WebRequest -Uri https://github.com/systm-d/repolens/releases/latest/download/repolens-windows-x86_64.zip -OutFile repolens-windows-x86_64.zip
Expand-Archive repolens-windows-x86_64.zip -DestinationPath .
Move-Item repolens.exe C:\Users\$env:USERNAME\bin\
```

#### Verify Checksums

Each release includes a `checksums.sha256` file. After downloading your archive, verify its integrity:

```bash
# Download the checksums file
curl -LO https://github.com/systm-d/repolens/releases/latest/download/checksums.sha256

# Verify (Linux)
sha256sum -c checksums.sha256 --ignore-missing

# Verify (macOS)
shasum -a 256 -c checksums.sha256 --ignore-missing
```

#### Verify Installation

```bash
repolens --version
```

### From Source

```bash
# Clone repository
git clone https://github.com/systm-d/repolens.git
cd repolens

# Build
cargo build --release

# The binary will be at target/release/repolens
```

### Shell completions

Generate completions for your shell with `repolens completions <shell>`
(supports `bash`, `zsh`, `fish`, `powershell`, `elvish`, `nushell`):

```bash
# Bash (system-wide)
repolens completions bash | sudo tee /etc/bash_completion.d/repolens > /dev/null

# Zsh (drop into a directory in $fpath)
repolens completions zsh > "${fpath[1]}/_repolens"

# Fish
repolens completions fish > ~/.config/fish/completions/repolens.fish
```

See [docs/installation/completions.md](docs/installation/completions.md) for the full per-shell guide.

### Nightly Builds

Nightly builds are available for testing. See the [Releases page](https://github.com/systm-d/repolens/releases) for nightly builds (marked as pre-release).

**Warning**: Nightly builds may be unstable. Use at your own risk.

### Docker

RepoLens is available as a Docker image for easy deployment:

```bash
# Pull the latest image
docker pull ghcr.io/systm-d/repolens:latest

# Run on current directory
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens plan

# Generate a report
docker run --rm -v "$(pwd)":/repo ghcr.io/systm-d/repolens report
```

For GitHub API access, mount your GitHub CLI config:

```bash
docker run --rm \
  -v "$(pwd)":/repo \
  -v ~/.config/gh:/home/repolens/.config/gh:ro \
  ghcr.io/systm-d/repolens plan
```

See [docs/docker.md](docs/docker.md) for detailed Docker usage instructions.

## Prerequisites

RepoLens requires the following tools to be installed and configured:

| Tool | Required | Description |
|------|----------|-------------|
| Git | Yes | Must be installed and the directory must be a git repository |
| GitHub CLI (gh) | For GitHub | Used for GitHub provider operations; authenticate with `gh auth login` (or set `GITHUB_TOKEN`) |
| GitLab CLI (glab) | For GitLab | Used for GitLab provider operations; authenticate with `glab auth login` |

The provider is auto-detected from the `origin` remote (or set explicitly with
`--provider` / the `provider` config key). The relevant CLI is only needed for
checks and actions that talk to the hosting platform; if it is absent, those
platform-specific checks are skipped gracefully rather than failing the run.

When running `repolens init`, these prerequisites are automatically verified:

```
Checking prerequisites...

  ✓ Git installed
  ✓ Git repository
  ✓ GitHub CLI installed
  ✓ GitHub CLI authenticated
  ✓ Remote origin configured
  ✓ Remote is GitHub
```

If a required prerequisite fails, you'll see an error with a suggested fix:

```
  ✗ GitHub CLI installed
    GitHub CLI (gh) is not installed
    Fix: Install gh: https://cli.github.com/
```

Use `--skip-checks` to bypass prerequisite verification (not recommended).

## Usage

### Initialize Configuration

```bash
# Create default configuration
repolens init

# Use a preset
repolens init --preset opensource
repolens init --preset enterprise
repolens init --preset strict

# Skip prerequisite checks (not recommended)
repolens init --skip-checks
```

### Run Audit

```bash
# Generate audit plan
repolens plan

# Audit a different directory
repolens -C /path/to/project plan

# Select the hosting provider explicitly (otherwise auto-detected from origin)
repolens plan --provider github
repolens plan --provider gitlab

# Output in different formats
repolens plan --format json
repolens plan --format sarif
repolens plan --format markdown

# Verbose mode with timing information
repolens plan -v      # Basic timing
repolens plan -vv     # Detailed timing per category
repolens plan -vvv    # Debug level
```

### Apply Fixes

```bash
# Preview changes (shows diff without applying)
repolens apply --dry-run

# Apply all fixes with confirmation prompt
repolens apply

# Interactive mode: select actions individually with diff preview
repolens apply --interactive
repolens apply -i

# Auto-accept all actions without confirmation
repolens apply --yes
repolens apply -y

# Apply specific categories only
repolens apply --only files,docs

# Skip specific categories
repolens apply --skip security
```

#### Interactive Mode

The interactive mode (`-i` or `--interactive`) provides an enhanced user experience:

1. **Visual Summary**: Displays a categorized overview of all planned actions
2. **Action Selection**: Use `MultiSelect` to choose which actions to apply (Space to toggle, Enter to confirm)
3. **Diff Preview**: Shows a colored diff (green for additions, red for deletions) for each selected action
4. **Progress Bar**: Displays real-time progress during execution
5. **Execution Summary**: Shows detailed results with success/failure counts

Example output:
```
==============================================================================
                     ACTION SUMMARY
==============================================================================

[F] GITIGNORE (1 action)
    + Update .gitignore with recommended entries
      - .env
      - *.key

[F] FILES (2 actions)
    + Create CONTRIBUTING.md from template
    + Create SECURITY.md from template

==============================================================================
  Total: 3 actions to apply
==============================================================================
```

### Generate Report

```bash
# Terminal report
repolens report

# Export report
repolens report --format html --output report.html

# Select the hosting provider explicitly (also available on report/apply)
repolens report --provider gitlab

# JSON report with JSON Schema reference
repolens report --format json --schema

# JSON report with schema validation
repolens report --format json --schema --validate
```

### JSON Schema

RepoLens provides a JSON Schema (draft-07) that describes the structure of the JSON audit report output. This enables validation of report output and integration with tools that consume JSON Schema.

```bash
# Display the JSON Schema on stdout
repolens schema

# Save the JSON Schema to a file
repolens schema --output schemas/audit-report.schema.json
```

The schema defines the following structure:

- **repository_name**: Name of the audited repository
- **preset**: Audit preset used (opensource, enterprise, strict)
- **findings**: Array of audit findings, each with:
  - `rule_id`: Unique rule identifier (e.g., SEC001)
  - `category`: Finding category (files, docs, security, git, codeowners, metadata)
  - `severity`: Severity level (critical, warning, info)
  - `message`: Description of the finding
  - `location`: Optional file location
  - `description`: Optional detailed description
  - `remediation`: Optional suggested fix
- **metadata**: Report metadata (version, timestamp, schema_version)
- **summary**: Aggregated counts by severity and category

When using `--schema`, the JSON output includes a `$schema` field referencing the schema URI. When using `--validate`, the output is validated against the schema before being emitted.

### Comparing Audits

Compare two previously generated JSON audit reports to visualize improvements and regressions between runs.

```bash
# First, generate two JSON reports at different points in time
repolens report --format json --output report-before.json
# ... make changes ...
repolens report --format json --output report-after.json

# Compare the two reports (terminal output with colors)
repolens compare --base-file report-before.json --head-file report-after.json

# Output as JSON
repolens compare --base-file report-before.json --head-file report-after.json --format json

# Output as Markdown
repolens compare --base-file report-before.json --head-file report-after.json --format markdown

# Save comparison to a file
repolens compare --base-file report-before.json --head-file report-after.json --output comparison.md --format markdown

# Fail with exit code 1 if new issues are detected (useful in CI)
repolens compare --base-file baseline.json --head-file current.json --fail-on-regression
```

The comparison report includes:
- **Score summary**: Weighted score (Critical=10, Warning=3, Info=1) with diff
- **New issues**: Findings present in the head report but not in the base (regressions)
- **Resolved issues**: Findings present in the base report but not in the head (improvements)
- **Category breakdown**: Per-category count changes

## Configuration

Create a `.repolens.toml` file in your repository root:

```toml
# Repository hosting provider: "github" (default) or "gitlab".
# Omit to auto-detect from the origin remote.
provider = "github"

[general]
preset = "opensource"

[rules]
files = true
docs = true
security = true
git = true
codeowners = true
metadata = true

[files.required]
readme = true
license = true
contributing = true
code_of_conduct = true
security = true
```

### Cache

RepoLens includes a caching system to improve performance by avoiding re-auditing files that haven't changed. Cache entries are automatically invalidated when file content changes (detected via SHA256 hashing).

#### Cache Configuration

```toml
[cache]
# Enable/disable caching (default: true)
enabled = true
# Maximum age for cache entries in hours (default: 24)
max_age_hours = 24
# Cache directory (relative to project root or absolute path)
directory = ".repolens/cache"
```

#### Cache CLI Options

```bash
# Disable cache and force a complete re-audit
repolens plan --no-cache

# Clear the cache before running the audit
repolens plan --clear-cache

# Use a custom cache directory
repolens plan --cache-dir /tmp/repolens-cache
```

The same options are available for the `report` command.
```

### Environment Variables

RepoLens can be configured via environment variables. Priority order: CLI flags > Environment variables > Config file > Defaults.

| Variable | Description | Example |
|----------|-------------|---------|
| `REPOLENS_PRESET` | Default preset to use | `enterprise` |
| `REPOLENS_VERBOSE` | Verbosity level (0-3) | `2` |
| `REPOLENS_CONFIG` | Path to config file | `/path/to/.repolens.toml` |
| `REPOLENS_NO_CACHE` | Disable caching | `true` |
| `REPOLENS_GITHUB_TOKEN` | GitHub token for API calls | `ghp_xxx` |

```bash
# Example usage
export REPOLENS_PRESET=enterprise
export REPOLENS_VERBOSE=2
repolens plan
```

### Exit Codes

RepoLens uses standard exit codes for CI/CD integration:

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | Audit completed, no critical issues |
| 1 | Critical issues | Critical findings detected (e.g. missing required files) |
| 2 | Warnings | Missing files, non-critical findings |
| 3 | Runtime error | File not found, network error |
| 4 | Invalid arguments | Unknown category, invalid preset |

```bash
# Example usage in CI/CD
repolens plan
case $? in
  0) echo "All clear!" ;;
  1) echo "Critical issues found - blocking release" && exit 1 ;;
  2) echo "Warnings found - review recommended" ;;
  3) echo "Error running audit" && exit 1 ;;
  4) echo "Invalid arguments" && exit 1 ;;
esac
```

### Git Hooks

RepoLens can install Git hooks to automatically check your code before commits and pushes.

#### Install Hooks

```bash
# Install all configured hooks (pre-commit + pre-push)
repolens install-hooks

# Install only the pre-commit hook
repolens install-hooks --pre-commit

# Install only the pre-push hook
repolens install-hooks --pre-push

# Force overwrite existing hooks (backs up originals)
repolens install-hooks --force
```

#### Remove Hooks

```bash
# Remove all RepoLens hooks (restores backups if they exist)
repolens install-hooks --remove
```

#### Hook Behavior

- **pre-commit**: Runs a quick RepoLens check before each commit. If blocking findings are detected, the commit is aborted.
- **pre-push**: Runs a full audit before pushing. If issues are found, the push is aborted.

Both hooks can be bypassed with `--no-verify` (e.g., `git commit --no-verify`).

#### Configuration

Configure hooks in `.repolens.toml`:

```toml
[hooks]
# Install pre-commit hook (quick check before commit)
pre_commit = true
# Install pre-push hook (runs full audit)
pre_push = true
# Whether warnings should cause hook failure
fail_on_warnings = false
```

When `fail_on_warnings` is `true`, hooks will also fail on warning-level findings, not just critical issues.

## Presets

| Preset | Description |
|--------|-------------|
| `opensource` | Standard open-source requirements |
| `enterprise` | Enterprise security and compliance |
| `strict` | Maximum security and documentation |

## Rules Categories

RepoLens ships six rule categories. Each finding drives a reviewable action that
can be applied with `repolens apply` (the plan/apply split).

- **files**: Check for required repository files (CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, etc.)
- **docs**: README, LICENSE, and documentation quality checks
- **security**: Repository settings and branch protection best practices (SEC007-010)
- **git**: `.gitattributes` presence and sensitive files tracked in git (GIT002-003)
- **codeowners**: CODEOWNERS file presence and validity (CODE001-002)
- **metadata**: Repository description and topics/tags configuration (META001-003)

Use `--only` / `--skip` to restrict the run to a subset of these categories, e.g.
`repolens plan --only docs,files` or `repolens plan --skip metadata`.

### Git Hygiene Rules

| Rule | Severity | Description |
|------|----------|-------------|
| GIT002 | Info | `.gitattributes` file missing |
| GIT003 | Warning | Sensitive files tracked (.env, *.key, *.pem, credentials) |

### Branch Protection & Repository Settings Rules

| Rule | Severity | Description |
|------|----------|-------------|
| SEC007 | Info | `.github/settings.yml` missing |
| SEC008 | Warning | No branch protection rules in settings.yml |
| SEC009 | Warning | `required_pull_request_reviews` not configured |
| SEC010 | Warning | `required_status_checks` not configured |

### Actions

Every kept check produces actions from a small, provider-agnostic catalog that
`repolens apply` can execute against GitHub or GitLab:

| Action | Description |
|--------|-------------|
| `CreateFile` | Create a missing repository file from a template (README, LICENSE, CODEOWNERS, etc.) |
| `UpdateGitignore` | Add recommended entries to `.gitignore` |
| `ConfigureProtectedBranch` | Configure branch protection on the hosting platform |
| `UpdateRepoSettings` | Update repository settings via the provider |
| `UpdateRepoMetadata` | Update repository description and topics/tags |

## GitHub Action

RepoLens is available as a GitHub Action to integrate repository auditing directly into your CI/CD workflows.

### Basic Usage

```yaml
name: RepoLens Audit
on: [push, pull_request]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: systm-d/repolens@main
        with:
          preset: 'opensource'
          format: 'terminal'
          fail-on: 'critical'
```

### Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `preset` | Audit preset (`opensource`, `enterprise`, `strict`) | `opensource` |
| `format` | Output format (`terminal`, `json`, `sarif`, `markdown`, `html`) | `terminal` |
| `fail-on` | Fail on severity level (`critical`, `high`, `medium`, `low`, `none`) | `critical` |
| `config` | Path to a custom `.repolens.toml` config file | |
| `version` | RepoLens version to install (e.g. `1.0.0` or `latest`) | `latest` |
| `upload-artifact` | Upload report as a GitHub Actions artifact | `true` |
| `artifact-name` | Name of the uploaded artifact | `repolens-report` |

### Outputs

| Output | Description |
|--------|-------------|
| `report-path` | Path to the generated report file |
| `findings-count` | Total number of findings detected |
| `exit-code` | Exit code (`0`=success, `1`=critical, `2`=warnings) |

### SARIF Integration

Upload results to GitHub Advanced Security for visibility in the Security tab:

```yaml
- uses: systm-d/repolens@main
  id: audit
  with:
    format: 'sarif'
    fail-on: 'none'

- uses: github/codeql-action/upload-sarif@v3
  if: always()
  with:
    sarif_file: ${{ steps.audit.outputs.report-path }}
    category: 'repolens'
```

## CI/CD Integration

RepoLens integrates with all major CI/CD platforms. See [docs/ci-cd-integration.md](docs/ci-cd-integration.md) for detailed integration guides and examples.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for development setup, architecture, and contribution guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
