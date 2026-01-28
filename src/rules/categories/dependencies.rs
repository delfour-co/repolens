//! Dependency security rules for checking vulnerabilities using OSV API

use crate::config::Config;
use crate::error::RepoLensError;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct DependencyRules;

#[async_trait::async_trait]
impl RuleCategory for DependencyRules {
    fn name(&self) -> &'static str {
        "dependencies"
    }

    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<Finding>, RepoLensError> {
        let mut findings = Vec::new();
        if config.is_rule_enabled("dependencies/vulnerabilities") {
            findings.extend(check_vulnerabilities(scanner, config).await?);
        }
        Ok(findings)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecosystem {
    Cargo,
    Npm,
    PyPI,
    Go,
}

impl Ecosystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            Ecosystem::Cargo => "crates.io",
            Ecosystem::Npm => "npm",
            Ecosystem::PyPI => "PyPI",
            Ecosystem::Go => "Go",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub ecosystem: Ecosystem,
}

#[derive(Debug, Serialize)]
struct OsvQuery {
    package: OsvPackage,
    version: String,
}
#[derive(Debug, Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}
#[derive(Debug, Serialize)]
struct OsvBatchQuery {
    queries: Vec<OsvQuery>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OsvVulnerability {
    pub id: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub severity: Vec<OsvSeverity>,
    #[serde(default)]
    pub affected: Vec<OsvAffected>,
    #[serde(default)]
    #[allow(dead_code)]
    pub references: Vec<OsvReference>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OsvSeverity {
    #[serde(rename = "type")]
    pub severity_type: String,
    pub score: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct OsvAffected {
    pub package: Option<OsvAffectedPackage>,
    #[serde(default)]
    pub ranges: Vec<OsvRange>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct OsvAffectedPackage {
    pub name: String,
    #[allow(dead_code)]
    pub ecosystem: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct OsvRange {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub range_type: String,
    #[serde(default)]
    pub events: Vec<OsvEvent>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct OsvEvent {
    #[serde(default)]
    #[allow(dead_code)]
    pub introduced: Option<String>,
    #[serde(default)]
    pub fixed: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct OsvReference {
    #[serde(rename = "type")]
    pub ref_type: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
struct OsvBatchResponse {
    results: Vec<OsvBatchResult>,
}
#[derive(Debug, Deserialize)]
struct OsvBatchResult {
    #[serde(default)]
    vulns: Vec<OsvVulnerability>,
}

async fn check_vulnerabilities(
    scanner: &Scanner,
    _config: &Config,
) -> Result<Vec<Finding>, RepoLensError> {
    let mut findings = Vec::new();
    let mut all_deps: Vec<Dependency> = Vec::new();
    if let Ok(deps) = parse_cargo_lock(scanner) {
        all_deps.extend(deps);
    }
    if let Ok(deps) = parse_package_lock(scanner) {
        all_deps.extend(deps);
    }
    if let Ok(deps) = parse_requirements_txt(scanner) {
        all_deps.extend(deps);
    }
    if let Ok(deps) = parse_go_sum(scanner) {
        all_deps.extend(deps);
    }
    if all_deps.is_empty() {
        return Ok(findings);
    }

    match query_osv_batch(&all_deps).await {
        Ok(vulns) => {
            for (dep, vuln_list) in vulns {
                for vuln in vuln_list {
                    let sev = determine_severity(&vuln);
                    let fixed = get_fixed_version(&vuln, &dep);
                    let mut f = Finding::new(
                        format!("DEP001-{}", vuln.id),
                        "dependencies",
                        sev,
                        format!(
                            "Vulnerability {} found in {} {}",
                            vuln.id, dep.name, dep.version
                        ),
                    );
                    if let Some(s) = &vuln.summary {
                        f = f.with_description(s.clone());
                    } else if let Some(d) = &vuln.details {
                        f = f.with_description(d.chars().take(500).collect::<String>());
                    }
                    let rem = if let Some(fix) = fixed {
                        format!(
                            "Upgrade {} to version {} or later. Aliases: {}",
                            dep.name,
                            fix,
                            vuln.aliases.join(", ")
                        )
                    } else {
                        format!(
                            "Check for updates to {}. Vulnerability: {}. Aliases: {}",
                            dep.name,
                            vuln.id,
                            vuln.aliases.join(", ")
                        )
                    };
                    f = f.with_remediation(rem);
                    let loc = match dep.ecosystem {
                        Ecosystem::Cargo => "Cargo.lock",
                        Ecosystem::Npm => "package-lock.json",
                        Ecosystem::PyPI => "requirements.txt",
                        Ecosystem::Go => "go.sum",
                    };
                    f = f.with_location(loc);
                    findings.push(f);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to query OSV API: {}", e);
            findings.push(
                Finding::new(
                    "DEP000",
                    "dependencies",
                    Severity::Warning,
                    "Could not check dependencies for vulnerabilities",
                )
                .with_description(format!(
                    "Failed to query OSV API. Error: {}. Please check your network connection.",
                    e
                )),
            );
        }
    }
    Ok(findings)
}

pub fn parse_cargo_lock(scanner: &Scanner) -> Result<Vec<Dependency>, RepoLensError> {
    let mut deps = Vec::new();
    if !scanner.file_exists("Cargo.lock") {
        return Ok(deps);
    }
    let content = scanner.read_file("Cargo.lock").map_err(|e| {
        RepoLensError::Scan(crate::error::ScanError::FileRead {
            path: "Cargo.lock".to_string(),
            source: e,
        })
    })?;
    let lock: toml::Value = toml::from_str(&content)?;
    if let Some(packages) = lock.get("package").and_then(|p| p.as_array()) {
        for pkg in packages {
            if let (Some(n), Some(v)) = (
                pkg.get("name").and_then(|n| n.as_str()),
                pkg.get("version").and_then(|v| v.as_str()),
            ) {
                deps.push(Dependency {
                    name: n.to_string(),
                    version: v.to_string(),
                    ecosystem: Ecosystem::Cargo,
                });
            }
        }
    }
    Ok(deps)
}

pub fn parse_package_lock(scanner: &Scanner) -> Result<Vec<Dependency>, RepoLensError> {
    let mut deps = Vec::new();
    if !scanner.file_exists("package-lock.json") {
        return Ok(deps);
    }
    let content = scanner.read_file("package-lock.json").map_err(|e| {
        RepoLensError::Scan(crate::error::ScanError::FileRead {
            path: "package-lock.json".to_string(),
            source: e,
        })
    })?;
    let lock: serde_json::Value = serde_json::from_str(&content)?;
    if let Some(packages) = lock.get("packages").and_then(|p| p.as_object()) {
        for (path, info) in packages {
            if path.is_empty() {
                continue;
            }
            let name = path.strip_prefix("node_modules/").unwrap_or(path);
            let name = if name.contains("/node_modules/") {
                name.split("/node_modules/").last().unwrap_or(name)
            } else {
                name
            };
            if let Some(v) = info.get("version").and_then(|v| v.as_str()) {
                deps.push(Dependency {
                    name: name.to_string(),
                    version: v.to_string(),
                    ecosystem: Ecosystem::Npm,
                });
            }
        }
    } else if let Some(d) = lock.get("dependencies").and_then(|d| d.as_object()) {
        parse_npm_deps(d, &mut deps);
    }
    Ok(deps)
}

fn parse_npm_deps(d: &serde_json::Map<String, serde_json::Value>, deps: &mut Vec<Dependency>) {
    for (n, i) in d {
        if let Some(v) = i.get("version").and_then(|v| v.as_str()) {
            deps.push(Dependency {
                name: n.clone(),
                version: v.to_string(),
                ecosystem: Ecosystem::Npm,
            });
        }
        if let Some(nested) = i.get("dependencies").and_then(|d| d.as_object()) {
            parse_npm_deps(nested, deps);
        }
    }
}

pub fn parse_requirements_txt(scanner: &Scanner) -> Result<Vec<Dependency>, RepoLensError> {
    let mut deps = Vec::new();
    for f in [
        "requirements.txt",
        "requirements-dev.txt",
        "requirements/base.txt",
        "requirements/production.txt",
    ] {
        if !scanner.file_exists(f) {
            continue;
        }
        let content = scanner.read_file(f).map_err(|e| {
            RepoLensError::Scan(crate::error::ScanError::FileRead {
                path: f.to_string(),
                source: e,
            })
        })?;
        for line in content.lines() {
            let l = line.trim();
            if l.is_empty() || l.starts_with('#') || l.starts_with('-') {
                continue;
            }
            if let Some((n, v)) = parse_pip_req(l) {
                deps.push(Dependency {
                    name: n,
                    version: v,
                    ecosystem: Ecosystem::PyPI,
                });
            }
        }
    }
    Ok(deps)
}

fn parse_pip_req(line: &str) -> Option<(String, String)> {
    let l = line.split(';').next()?.trim().split('#').next()?.trim();
    let l = if let Some(p) = l.find('[') {
        let e = l.find(']')?;
        format!("{}{}", &l[..p], &l[e + 1..])
    } else {
        l.to_string()
    };
    for sep in ["==", ">=", "<=", "~=", "!=", ">", "<"] {
        if let Some(p) = l.find(sep) {
            let n = l[..p].trim().to_lowercase();
            let v = l[p + sep.len()..]
                .trim()
                .split(',')
                .next()?
                .trim()
                .to_string();
            if !n.is_empty() && !v.is_empty() {
                return Some((n, v));
            }
        }
    }
    None
}

pub fn parse_go_sum(scanner: &Scanner) -> Result<Vec<Dependency>, RepoLensError> {
    let mut deps = Vec::new();
    let mut seen: HashMap<String, bool> = HashMap::new();
    if !scanner.file_exists("go.sum") {
        return Ok(deps);
    }
    let content = scanner.read_file("go.sum").map_err(|e| {
        RepoLensError::Scan(crate::error::ScanError::FileRead {
            path: "go.sum".to_string(),
            source: e,
        })
    })?;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let module = parts[0];
        let version = parts[1].trim_end_matches("/go.mod");
        let key = format!("{}@{}", module, version);
        if seen.contains_key(&key) {
            continue;
        }
        seen.insert(key, true);
        let v = version.strip_prefix('v').unwrap_or(version);
        let v = if v.contains('-') {
            v.split('-').next().unwrap_or(v)
        } else {
            v
        };
        deps.push(Dependency {
            name: module.to_string(),
            version: v.to_string(),
            ecosystem: Ecosystem::Go,
        });
    }
    Ok(deps)
}

async fn query_osv_batch(
    deps: &[Dependency],
) -> Result<Vec<(Dependency, Vec<OsvVulnerability>)>, String> {
    if deps.is_empty() {
        return Ok(Vec::new());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let queries: Vec<OsvQuery> = deps
        .iter()
        .map(|d| OsvQuery {
            package: OsvPackage {
                name: d.name.clone(),
                ecosystem: d.ecosystem.as_str().to_string(),
            },
            version: d.version.clone(),
        })
        .collect();
    let resp = client
        .post("https://api.osv.dev/v1/querybatch")
        .json(&OsvBatchQuery { queries })
        .send()
        .await
        .map_err(|e| format!("HTTP failed: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("OSV API status: {}", resp.status()));
    }
    let batch: OsvBatchResponse = resp
        .json()
        .await
        .map_err(|e| format!("Parse failed: {}", e))?;
    let mut results = Vec::new();
    for (i, r) in batch.results.into_iter().enumerate() {
        if !r.vulns.is_empty() {
            if let Some(d) = deps.get(i) {
                results.push((d.clone(), r.vulns));
            }
        }
    }
    Ok(results)
}

fn determine_severity(vuln: &OsvVulnerability) -> Severity {
    for s in &vuln.severity {
        if s.severity_type == "CVSS_V3" || s.severity_type == "CVSS_V2" {
            if let Ok(score) = s.score.parse::<f64>() {
                if score >= 7.0 {
                    return Severity::Critical;
                } else if score >= 4.0 {
                    return Severity::Warning;
                } else {
                    return Severity::Info;
                }
            }
            if s.score.starts_with("CVSS:") {
                return Severity::Warning;
            }
        }
    }
    if vuln.id.starts_with("GHSA-") || vuln.id.starts_with("CVE-") {
        Severity::Warning
    } else {
        Severity::Info
    }
}

fn get_fixed_version(vuln: &OsvVulnerability, dep: &Dependency) -> Option<String> {
    for a in &vuln.affected {
        if let Some(p) = &a.package {
            if p.name != dep.name {
                continue;
            }
        }
        for r in &a.ranges {
            for e in &r.events {
                if let Some(f) = &e.fixed {
                    return Some(f.clone());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_cargo_lock() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("Cargo.lock"),
            "[[package]]\nname = \"serde\"\nversion = \"1.0.130\"",
        )
        .unwrap();
        let scanner = Scanner::new(tmp.path().to_path_buf());
        let deps = parse_cargo_lock(&scanner).unwrap();
        assert_eq!(deps.len(), 1);
        assert!(deps
            .iter()
            .any(|d| d.name == "serde" && d.version == "1.0.130"));
    }

    #[test]
    fn test_ecosystem_as_str() {
        assert_eq!(Ecosystem::Cargo.as_str(), "crates.io");
        assert_eq!(Ecosystem::Npm.as_str(), "npm");
    }

    #[test]
    fn test_determine_severity() {
        let vuln = OsvVulnerability {
            id: "GHSA-test".to_string(),
            summary: None,
            details: None,
            aliases: vec![],
            severity: vec![OsvSeverity {
                severity_type: "CVSS_V3".to_string(),
                score: "9.8".to_string(),
            }],
            affected: vec![],
            references: vec![],
        };
        assert_eq!(determine_severity(&vuln), Severity::Critical);
    }

    #[tokio::test]
    async fn test_check_vulnerabilities_no_lock_files() {
        let tmp = TempDir::new().unwrap();
        let scanner = Scanner::new(tmp.path().to_path_buf());
        let config = Config::default();
        let findings = check_vulnerabilities(&scanner, &config).await.unwrap();
        assert!(findings.is_empty());
    }

    #[tokio::test]
    async fn test_dependency_rules_category_name() {
        let rules = DependencyRules;
        assert_eq!(rules.name(), "dependencies");
    }
}
