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

```bash
# From source
git clone https://github.com/yourusername/repolens.git
cd repolens
cargo build --release

# The binary will be at target/release/repolens
```

## Usage

### Initialize Configuration

```bash
# Create default configuration
repolens init

# Use a preset
repolens init --preset opensource
repolens init --preset enterprise
repolens init --preset strict
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

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
