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
# Preview changes
repolens apply --dry-run

# Apply all fixes
repolens apply

# Apply specific fixes
repolens apply --only files,docs
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
```

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
