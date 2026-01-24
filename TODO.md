# RepoLens - TODO

## Progress Summary

**Completed:** 11/12 tasks
**In Progress:** 0 tasks
**Pending:** 1 task (templates blocked by content filter)

---

## Completed Tasks

- [x] Initialize Rust project with Cargo
- [x] Create project structure (src/, templates/, presets/)
- [x] Setup Cargo.toml with dependencies
- [x] Create core module files
- [x] Create CLI commands structure
- [x] Create rules engine and categories
- [x] Create actions module
- [x] Create presets (opensource, enterprise, strict)
- [x] Create CLAUDE.md and .claude/commands/
- [x] Create README.md
- [x] Setup GitHub workflows (ci.yml, release.yml)

---

## Blocked by Content Filter

### Create file templates (LICENSE, CONTRIBUTING, etc.)

**Done:**
- [x] `templates/LICENSE/MIT.txt`
- [x] `templates/LICENSE/Apache-2.0.txt`
- [x] `templates/CONTRIBUTING.md`

**Blocked (content filter):**
- [ ] `templates/CODE_OF_CONDUCT.md`
- [ ] `templates/SECURITY.md`
- [ ] `templates/ISSUE_TEMPLATE/bug_report.md`
- [ ] `templates/ISSUE_TEMPLATE/feature_request.md`
- [ ] `templates/PULL_REQUEST_TEMPLATE/pull_request_template.md`

---

## Completed Tasks (New)

### Create CLAUDE.md and Claude context files

- [x] `CLAUDE.md` - Main context file for Claude Code
- [x] `.claude/commands/` - Custom slash commands
  - [x] `/audit` - Run audit
  - [x] `/fix` - Fix issues

### Create README.md and documentation

- [x] `README.md` - Main documentation

### Setup GitHub workflows

- [x] `.github/workflows/ci.yml` - CI pipeline
- [x] `.github/workflows/release.yml` - Release pipeline
- [x] `.github/PULL_REQUEST_TEMPLATE.md`

---

## Additional Tasks (Nice to have for v0.1.0)

- [x] Add `async-trait` crate to Cargo.toml (needed for RuleCategory trait)
- [x] Verify compilation with `cargo check`
- [x] Fix Cargo.toml (edition 2021, sarif dependency)
- [ ] Write basic integration tests
- [ ] Add example `.repolens.toml` configuration file
- [ ] Create `CHANGELOG.md`

---

## Architecture Summary

```
repolens/
├── src/
│   ├── main.rs              ✅ Entry point
│   ├── lib.rs               ✅ Library exports
│   ├── cli/
│   │   ├── mod.rs           ✅ CLI definition
│   │   ├── commands/        ✅ init, plan, apply, report
│   │   └── output/          ✅ terminal, json, sarif, markdown, html
│   ├── config/
│   │   ├── mod.rs           ✅ Config structures
│   │   ├── loader.rs        ✅ Config loading
│   │   └── presets/         ✅ Preset definitions
│   ├── rules/
│   │   ├── mod.rs           ✅ Rules module
│   │   ├── engine.rs        ✅ Rules engine
│   │   ├── results.rs       ✅ Audit results
│   │   ├── categories/      ✅ secrets, files, docs, security, workflows, quality
│   │   └── patterns/        ✅ Secret patterns
│   ├── actions/
│   │   ├── mod.rs           ✅ Actions module
│   │   ├── plan.rs          ✅ Action plan
│   │   ├── planner.rs       ✅ Action planner
│   │   ├── executor.rs      ✅ Action executor
│   │   ├── gitignore.rs     ✅ Gitignore management
│   │   ├── templates.rs     ✅ Template creation
│   │   ├── branch_protection.rs  ✅ Branch protection
│   │   └── github_settings.rs    ✅ GitHub settings
│   ├── providers/
│   │   ├── mod.rs           ✅ Providers module
│   │   └── github.rs        ✅ GitHub provider
│   └── scanner/
│       ├── mod.rs           ✅ Scanner module
│       ├── filesystem.rs    ✅ File system scanning
│       └── git.rs           ✅ Git utilities
├── presets/                 ✅ TOML preset files
├── templates/               ⚠️ Partially done (blocked by filter)
├── .github/                 ✅ Workflows created
├── .claude/                 ✅ Commands created
├── CLAUDE.md                ✅ Done
└── README.md                ✅ Done
```

---

## Commands Reference (for development)

```bash
# Check compilation
cargo check

# Build
cargo build

# Run
cargo run -- --help
cargo run -- init --preset opensource
cargo run -- plan
cargo run -- apply --dry-run

# Test
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```
