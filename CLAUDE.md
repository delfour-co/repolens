# RepoLens - Claude Context

## Project Overview

RepoLens is a CLI tool written in Rust that audits GitHub repositories for best practices, security issues, and compliance with open-source standards.

## Architecture

```
src/
├── main.rs           - Entry point
├── lib.rs            - Library exports
├── cli/              - CLI commands (init, plan, apply, report)
├── config/           - Configuration loading and presets
├── rules/            - Audit rules engine and categories
├── actions/          - Action planning and execution
├── providers/        - GitHub API integration
├── scanner/          - File system and git scanning
└── utils/            - Shared utilities (prerequisites checks, etc.)
```

## Key Commands

```bash
cargo check           # Verify compilation
cargo build           # Build the project
cargo run -- --help   # Show help
cargo run -- init     # Initialize configuration
cargo run -- plan     # Generate action plan
cargo run -- apply    # Apply fixes
cargo run -- report   # Generate audit report
```

## Presets

- `opensource` - Standard open-source project requirements
- `enterprise` - Enterprise-grade security and compliance
- `strict` - Maximum security and documentation requirements

## Rules Categories

1. **secrets** - Detect exposed secrets and credentials
2. **files** - Check for required files (README, LICENSE, etc.)
3. **docs** - Documentation quality checks
4. **security** - Security best practices
5. **workflows** - CI/CD and GitHub Actions checks
6. **quality** - Code quality standards

## Configuration

Configuration is stored in `.repolens.toml` at the project root.

## Development Guidelines

- Use `cargo clippy` for linting
- Use `cargo fmt` for formatting
- Run `cargo test` before committing
