# RepoLens

A CLI tool to audit GitHub repositories for best practices, security, and compliance.

## Features

- Audit repositories for security issues and best practices
- Detect exposed secrets and credentials
- Check for required files (README, LICENSE, CONTRIBUTING, etc.)
- Validate GitHub workflows and Actions
- Generate actionable fix plans
- Apply fixes automatically or with dry-run mode
- Multiple output formats: terminal, JSON, SARIF, Markdown, HTML

## Installation

### From crates.io

```bash
cargo install repolens
```

### Pre-built Binaries

Download the latest release from the [Releases page](https://github.com/kdelfour/repolens/releases):

```bash
# Download and install
wget https://github.com/kdelfour/repolens/releases/download/v0.1.0/repolens-linux-x86_64.tar.gz
Pre-built binaries are available for all major platforms. Download the latest release from the [Releases page](https://github.com/delfour-co/cli--repolens/releases).

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
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-linux-x86_64.tar.gz
tar xzf repolens-linux-x86_64.tar.gz
sudo mv repolens /usr/local/bin/
```

#### Linux (ARM64)

```bash
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-linux-arm64.tar.gz
tar xzf repolens-linux-arm64.tar.gz
sudo mv repolens /usr/local/bin/
```

#### macOS (Apple Silicon)

```bash
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-darwin-arm64.tar.gz
tar xzf repolens-darwin-arm64.tar.gz
sudo mv repolens /usr/local/bin/
```

#### macOS (Intel)

```bash
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-darwin-x86_64.tar.gz
tar xzf repolens-darwin-x86_64.tar.gz
sudo mv repolens /usr/local/bin/
```

#### Windows (x86_64)

```powershell
# Download the zip archive from the Releases page
Invoke-WebRequest -Uri https://github.com/delfour-co/cli--repolens/releases/latest/download/repolens-windows-x86_64.zip -OutFile repolens-windows-x86_64.zip
Expand-Archive repolens-windows-x86_64.zip -DestinationPath .
Move-Item repolens.exe C:\Users\$env:USERNAME\bin\
```

#### Verify Checksums

Each release includes a `checksums.sha256` file. After downloading your archive, verify its integrity:

```bash
# Download the checksums file
curl -LO https://github.com/delfour-co/cli--repolens/releases/latest/download/checksums.sha256

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
git clone https://github.com/kdelfour/repolens.git
cd repolens

# Build
cargo build --release

# The binary will be at target/release/repolens
```

### Nightly Builds

Nightly builds are available for testing. See the [Releases page](https://github.com/delfour-co/cli--repolens/releases) for nightly builds (marked as pre-release).

⚠️ **Warning**: Nightly builds may be unstable. Use at your own risk.

## Prerequisites

RepoLens requires the following tools to be installed and configured:

| Tool | Required | Description |
|------|----------|-------------|
| Git | Yes | Must be installed and the directory must be a git repository |
| GitHub CLI (gh) | Yes | Must be installed and authenticated (`gh auth login`) |

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

# Output in different formats
repolens plan --format json
repolens plan --format sarif
repolens plan --format markdown
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
```

## Configuration

Create a `.repolens.toml` file in your repository root:

```toml
[general]
preset = "opensource"

[rules]
secrets = true
files = true
docs = true
security = true
workflows = true
quality = true

[files.required]
readme = true
license = true
contributing = true
code_of_conduct = true
security = true
```

### Custom Rules

Define your own audit rules using regex patterns or shell commands:

```toml
# Detect TODO comments
[rules.custom."no-todo"]
pattern = "TODO"
severity = "warning"
files = ["**/*.rs"]
message = "TODO comment found"

# Check git status (shell command)
[rules.custom."check-git-status"]
command = "git status --porcelain"
severity = "warning"
invert = true  # Fail if uncommitted changes
message = "Working directory is not clean"
```

See the [Custom Rules documentation](wiki/Custom-Rules.md) for more examples and details.

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

- **pre-commit**: Scans staged files for exposed secrets before each commit. If secrets are detected, the commit is aborted.
- **pre-push**: Runs a full audit before pushing. If issues are found, the push is aborted.

Both hooks can be bypassed with `--no-verify` (e.g., `git commit --no-verify`).

#### Configuration

Configure hooks in `.repolens.toml`:

```toml
[hooks]
# Install pre-commit hook (checks for exposed secrets)
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

- **secrets**: Detect exposed API keys, tokens, passwords
- **files**: Check for required repository files
- **docs**: Documentation completeness and quality
- **security**: Security best practices and policies
- **workflows**: CI/CD and GitHub Actions validation
- **quality**: Code quality standards

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
      - uses: kdelfour/repolens@main
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
| `version` | RepoLens version to install (e.g. `0.1.0` or `latest`) | `latest` |
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
- uses: kdelfour/repolens@main
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

### PR Comment

Post audit results as a comment on pull requests. See the full example in [`examples/github-action/pr-comment.yml`](examples/github-action/pr-comment.yml).

### More Examples

See the [`examples/github-action/`](examples/github-action/) directory for complete workflow examples:

- [`basic.yml`](examples/github-action/basic.yml) -- Basic usage on push and pull requests
- [`sarif-upload.yml`](examples/github-action/sarif-upload.yml) -- SARIF upload for GitHub Security
- [`pr-comment.yml`](examples/github-action/pr-comment.yml) -- Post results as a PR comment

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for development setup, architecture, and contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
