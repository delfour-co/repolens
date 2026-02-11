# RepoLens Audit Report

**Repository:** delfour-co/repolens
**Preset:** opensource
**Generated:** 2026-02-11 09:48:47 UTC
**RepoLens Version:** 1.0.0

## Summary

| Severity | Count |
|----------|-------|
| Critical | 22 |
| Warning | 90 |
| Info | 16 |

## Critical Issues

These issues must be resolved before proceeding.

### SEC001 - URL with Embedded Credentials detected

**Location:** `src/actions/templates.rs:91`

### SEC001 - URL with Embedded Credentials detected

**Location:** `.github/workflows/sync-wiki.yml:52`

### SEC001 - URL with Embedded Credentials detected

**Location:** `.github/workflows/release.yml:251`

### SEC001 - Private Key detected

**Location:** `src/rules/categories/secrets.rs:445`

### SEC001 - Generic API Key Assignment detected

**Location:** `src/rules/categories/secrets.rs:328`

### SEC001 - URL with Embedded Credentials detected

**Location:** `src/rules/categories/quality.rs:771`

### SEC001 - Generic Password Assignment detected

**Location:** `src/rules/categories/workflows.rs:682`

### SEC001 - Private Key detected

**Location:** `src/rules/categories/git.rs:459`

### SEC001 - GitHub Personal Access Token detected

**Location:** `src/rules/patterns/secrets.rs:207`

### SEC001 - AWS Access Key ID detected

**Location:** `src/rules/patterns/secrets.rs:214`

### SEC001 - Stripe Live Secret Key detected

**Location:** `src/rules/patterns/secrets.rs:221`

### SEC001 - MySQL Connection String detected

**Location:** `src/rules/patterns/secrets.rs:159`

### SEC001 - Redis Connection String detected

**Location:** `src/rules/patterns/secrets.rs:164`

### SEC001 - Generic API Key Assignment detected

**Location:** `src/rules/engine.rs:498`

### SEC001 - URL with Embedded Credentials detected

**Location:** `src/scanner/git.rs:40`

### SEC001 - AWS Access Key ID detected

**Location:** `tests/e2e_test.rs:429`

### SEC001 - URL with Embedded Credentials detected

**Location:** `tests/e2e_test.rs:791`

### SEC001 - URL with Embedded Credentials detected

**Location:** `action.yml:60`

### SEC001 - URL with Embedded Credentials detected

**Location:** `integrations/azure-devops/azure-pipelines.yml:190`

### SEC001 - URL with Embedded Credentials detected

**Location:** `integrations/github-actions/repolens.yml:145`

### WF001 - Potential hardcoded secret in workflow

**Location:** `.github/workflows/sync-wiki.yml:0`

### DOCKER003 - Base image 'alpine:latest' uses the 'latest' tag

**Location:** `Dockerfile:42`

## Warnings

These issues should be addressed.

### DOC005 - CONTRIBUTING file is missing

### DOC006 - CODE_OF_CONDUCT file is missing

### DOC007 - SECURITY policy file is missing

### WF002 - Workflow missing explicit permissions

**Location:** `.github/workflows/ci.yml`

### WF004 - Job 'check-ci-success' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job 'dependency-audit' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job 'security-audit' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job 'outdated-dependencies' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job 'clippy-analysis' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job 'code-metrics' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job 'semgrep-analysis' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job '# codeql-analysis' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job '#   permissions' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job '#   steps' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job '#       with' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job '# sonarcloud' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job '#   steps' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job '#       with' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job '#       env' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job 'quality-report' missing timeout-minutes

**Location:** `.github/workflows/code-quality.yml`

### WF004 - Job 'calculate-version' missing timeout-minutes

**Location:** `.github/workflows/create-release.yml`

### WF004 - Job 'check-ci' missing timeout-minutes

**Location:** `.github/workflows/create-release.yml`

### WF004 - Job 'check-code-quality' missing timeout-minutes

**Location:** `.github/workflows/create-release.yml`

### WF004 - Job 'prepare-release' missing timeout-minutes

**Location:** `.github/workflows/create-release.yml`

### WF004 - Job 'check-prerequisites' missing timeout-minutes

**Location:** `.github/workflows/nightly.yml`

### WF004 - Job 'quality-gates' missing timeout-minutes

**Location:** `.github/workflows/nightly.yml`

### WF004 - Job 'build' missing timeout-minutes

**Location:** `.github/workflows/nightly.yml`

### WF004 - Job 'skip-notification' missing timeout-minutes

**Location:** `.github/workflows/nightly.yml`

### WF004 - Job 'sync-wiki' missing timeout-minutes

**Location:** `.github/workflows/sync-wiki.yml`

### WF004 - Job 'check' missing timeout-minutes

**Location:** `.github/workflows/ci.yml`

### WF004 - Job 'fmt' missing timeout-minutes

**Location:** `.github/workflows/ci.yml`

### WF004 - Job 'clippy' missing timeout-minutes

**Location:** `.github/workflows/ci.yml`

### WF004 - Job 'test' missing timeout-minutes

**Location:** `.github/workflows/ci.yml`

### WF004 - Job 'package' missing timeout-minutes

**Location:** `.github/workflows/ci.yml`

### WF004 - Job 'coverage' missing timeout-minutes

**Location:** `.github/workflows/ci.yml`

### WF004 - Job 'security' missing timeout-minutes

**Location:** `.github/workflows/ci.yml`

### WF004 - Job 'build-and-push' missing timeout-minutes

**Location:** `.github/workflows/docker.yml`

### WF004 - Job 'verify' missing timeout-minutes

**Location:** `.github/workflows/docker.yml`

### WF004 - Job 'build' missing timeout-minutes

**Location:** `.github/workflows/release.yml`

### WF004 - Job 'changelog' missing timeout-minutes

**Location:** `.github/workflows/release.yml`

### WF004 - Job 'release' missing timeout-minutes

**Location:** `.github/workflows/release.yml`

### WF004 - Job 'publish-crate' missing timeout-minutes

**Location:** `.github/workflows/release.yml`

### WF004 - Job 'announce-discussion' missing timeout-minutes

**Location:** `.github/workflows/release.yml`

### LIC004 - Dependency 'anyhow' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'async-trait' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'chrono' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'clap' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'clap_mangen' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'colored' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'console' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'dialoguer' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'dirs' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'globset' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'ignore' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'indicatif' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'jsonschema' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'lazy_static' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'metrics' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'metrics-prometheus' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'minijinja' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'rayon' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'regex' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'reqwest' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'serde' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'serde-sarif' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'serde_json' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'serde_yaml' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'sha2' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'similar' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'tera' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'thiserror' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'tokio' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'toml' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'tracing' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'tracing-subscriber' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'url' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'walkdir' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'which' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'assert_cmd' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'criterion' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'futures' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'predicates' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'pretty_assertions' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'serial_test' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'tempfile' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'clap' has no license specified

**Location:** `Cargo.toml`

### LIC004 - Dependency 'clap_mangen' has no license specified

**Location:** `Cargo.toml`

### DOCKER005 - No HEALTHCHECK instruction in Dockerfile

**Location:** `Dockerfile`

### GIT003 - Sensitive file 'src/rules/categories/secrets.rs' may be tracked in repository

**Location:** `src/rules/categories/secrets.rs`

### GIT003 - Sensitive file 'src/rules/patterns/secrets.rs' may be tracked in repository

**Location:** `src/rules/patterns/secrets.rs`

---

*Report generated by [RepoLens](https://github.com/kdelfour/repolens)*
