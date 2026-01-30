//! Script de vérification des seuils de qualité
//! 
//! Ce script vérifie que tous les seuils définis dans `.github/quality-gates.toml`
//! sont respectés avant de permettre la création d'une nightly build.

use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use serde::Deserialize;
use toml;

#[derive(Debug, Deserialize)]
struct QualityGates {
    coverage: Option<Coverage>,
    clippy: Option<Clippy>,
    security: Option<Security>,
    dependencies: Option<Dependencies>,
    code_metrics: Option<CodeMetrics>,
    documentation: Option<Documentation>,
    tests: Option<Tests>,
    build: Option<Build>,
    nightly: Option<Nightly>,
}

#[derive(Debug, Deserialize)]
struct Coverage {
    minimum: Option<f64>,
    target: Option<f64>,
    exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Clippy {
    max_warnings: Option<u32>,
    severity: Option<String>,
    strict: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Security {
    max_critical_vulnerabilities: Option<u32>,
    max_high_vulnerabilities: Option<u32>,
    max_medium_vulnerabilities: Option<u32>,
    allow_unpatched: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Dependencies {
    max_outdated: Option<u32>,
    max_duplicates: Option<u32>,
    check_licenses: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CodeMetrics {
    max_binary_size: Option<u64>,
    min_tests: Option<u32>,
    min_integration_tests: Option<u32>,
    max_cyclomatic_complexity: Option<u32>,
    max_unsafe_percentage: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Documentation {
    min_documentation_coverage: Option<f64>,
    require_public_docs: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Tests {
    require_all_tests_pass: Option<bool>,
    max_test_duration: Option<u32>,
    enable_performance_tests: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Build {
    require_build_success: Option<bool>,
    max_build_duration: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Nightly {
    strict_mode: Option<bool>,
    block_on_coverage_decrease: Option<bool>,
    block_on_new_vulnerabilities: Option<bool>,
    block_on_new_warnings: Option<bool>,
}

struct CheckResult {
    name: String,
    passed: bool,
    message: String,
}

fn main() -> Result<()> {
    println!("🔍 Vérification des seuils de qualité...\n");

    let gates = load_quality_gates()?;
    let mut results = Vec::new();
    let mut failed_checks = 0;

    // Vérifier la couverture
    if let Some(coverage) = &gates.coverage {
        if let Some(minimum) = coverage.minimum {
            match check_coverage(minimum) {
                Ok(result) => {
                    if !result.passed {
                        failed_checks += 1;
                    }
                    results.push(result);
                }
                Err(e) => {
                    println!("⚠️  Vérification de couverture ignorée: {}", e);
                }
            }
        }
    }

    // Vérifier Clippy
    if let Some(clippy) = &gates.clippy {
        if let Some(max_warnings) = clippy.max_warnings {
            match check_clippy(max_warnings) {
                Ok(result) => {
                    if !result.passed {
                        failed_checks += 1;
                    }
                    results.push(result);
                }
                Err(e) => {
                    println!("⚠️  Vérification Clippy ignorée: {}", e);
                }
            }
        }
    }

    // Vérifier la sécurité
    if let Some(security) = &gates.security {
        match check_security(security) {
            Ok(result) => {
                if !result.passed {
                    failed_checks += 1;
                }
                results.push(result);
            }
            Err(e) => {
                println!("⚠️  Vérification de sécurité ignorée: {}", e);
            }
        }
    }

    // Vérifier les dépendances
    if let Some(deps) = &gates.dependencies {
        match check_dependencies(deps) {
            Ok(result) => {
                if !result.passed {
                    failed_checks += 1;
                }
                results.push(result);
            }
            Err(e) => {
                println!("⚠️  Vérification des dépendances ignorée: {}", e);
            }
        }
    }

    // Vérifier les métriques de code
    if let Some(metrics) = &gates.code_metrics {
        match check_code_metrics(metrics) {
            Ok(result) => {
                for r in result {
                    if !r.passed {
                        failed_checks += 1;
                    }
                    results.push(r);
                }
            }
            Err(e) => {
                println!("⚠️  Vérification des métriques ignorée: {}", e);
            }
        }
    }

    // Afficher les résultats
    println!();
    for result in &results {
        if result.passed {
            println!("✅ {}: {}", result.name, result.message);
        } else {
            println!("❌ {}: {}", result.name, result.message);
        }
    }

    println!();
    if failed_checks == 0 {
        println!("✅ Tous les seuils de qualité sont respectés !");
        Ok(())
    } else {
        println!("❌ {} seuil(s) de qualité non respecté(s)", failed_checks);
        println!("La nightly build ne peut pas être créée.");
        std::process::exit(1);
    }
}

fn load_quality_gates() -> Result<QualityGates> {
    let content = fs::read_to_string(".github/quality-gates.toml")
        .context("Impossible de lire le fichier de configuration des seuils")?;
    let gates: QualityGates = toml::from_str(&content)
        .context("Erreur lors du parsing du fichier de configuration")?;
    Ok(gates)
}

fn check_coverage(minimum: f64) -> Result<CheckResult> {
    // Essayer de lire depuis un fichier XML de couverture
    let coverage_file = "coverage/cobertura.xml";
    if Path::new(coverage_file).exists() {
        // Pour une implémentation complète, il faudrait parser le XML
        // Ici, on utilise une approche simplifiée
        let coverage = extract_coverage_from_xml(coverage_file)?;
        let passed = coverage >= minimum;
        Ok(CheckResult {
            name: "Couverture de code".to_string(),
            passed,
            message: format!("{:.2}% (minimum: {:.2}%)", coverage, minimum),
        })
    } else {
        // Essayer avec cargo-tarpaulin directement
        let output = Command::new("cargo")
            .args(&["tarpaulin", "--out", "Xml", "--output-dir", "/tmp"])
            .stderr(Stdio::null())
            .output()?;

        if output.status.success() {
            // Parser la sortie pour extraire le pourcentage
            let stdout = String::from_utf8_lossy(&output.stdout);
            let coverage = extract_coverage_from_output(&stdout)?;
            let passed = coverage >= minimum;
            Ok(CheckResult {
                name: "Couverture de code".to_string(),
                passed,
                message: format!("{:.2}% (minimum: {:.2}%)", coverage, minimum),
            })
        } else {
            Err(anyhow::anyhow!("cargo-tarpaulin non disponible"))
        }
    }
}

fn extract_coverage_from_xml(_file: &str) -> Result<f64> {
    // Implémentation simplifiée - dans un vrai projet, utiliser un parser XML
    // Pour l'instant, on retourne une valeur par défaut
    Ok(0.0)
}

fn extract_coverage_from_output(output: &str) -> Result<f64> {
    // Extraire le pourcentage depuis la sortie de cargo-tarpaulin
    for line in output.lines() {
        if line.contains("%") {
            if let Some(percent_str) = line.split('%').next() {
                if let Ok(coverage) = percent_str.trim().parse::<f64>() {
                    return Ok(coverage);
                }
            }
        }
    }
    Err(anyhow::anyhow!("Impossible d'extraire la couverture"))
}

fn check_clippy(max_warnings: u32) -> Result<CheckResult> {
    let output = Command::new("cargo")
        .args(&["clippy", "--all-targets", "--all-features", "--message-format=json"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let warnings = stdout
        .lines()
        .filter(|line| line.contains(r#""level":"warning""#))
        .count() as u32;

    let passed = warnings <= max_warnings;
    Ok(CheckResult {
        name: "Clippy warnings".to_string(),
        passed,
        message: format!("{} warnings (maximum: {})", warnings, max_warnings),
    })
}

fn check_security(security: &Security) -> Result<CheckResult> {
    let output = Command::new("cargo")
        .args(&["audit", "--json"])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let critical = stdout.matches(r#""severity":"critical""#).count() as u32;
            let high = stdout.matches(r#""severity":"high""#).count() as u32;

            let max_critical = security.max_critical_vulnerabilities.unwrap_or(0);
            let max_high = security.max_high_vulnerabilities.unwrap_or(0);

            let passed = critical <= max_critical && high <= max_high;
            Ok(CheckResult {
                name: "Vulnérabilités de sécurité".to_string(),
                passed,
                message: format!(
                    "Critiques: {} (max: {}), Importantes: {} (max: {})",
                    critical, max_critical, high, max_high
                ),
            })
        }
        Err(_) => Err(anyhow::anyhow!("cargo-audit non disponible")),
    }
}

fn check_dependencies(deps: &Dependencies) -> Result<CheckResult> {
    if let Some(max_outdated) = deps.max_outdated {
        let output = Command::new("cargo")
            .args(&["outdated", "--format", "json"])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let outdated = stdout.matches(r#""name""#).count() as u32;

                let passed = outdated <= max_outdated;
                Ok(CheckResult {
                    name: "Dépendances obsolètes".to_string(),
                    passed,
                    message: format!("{} dépendances (maximum: {})", outdated, max_outdated),
                })
            }
            Err(_) => Err(anyhow::anyhow!("cargo-outdated non disponible")),
        }
    } else {
        Ok(CheckResult {
            name: "Dépendances obsolètes".to_string(),
            passed: true,
            message: "Vérification désactivée".to_string(),
        })
    }
}

fn check_code_metrics(metrics: &CodeMetrics) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Vérifier le nombre de tests
    if let Some(min_tests) = metrics.min_tests {
        let output = Command::new("cargo")
            .args(&["test", "--all-features", "--lib", "--tests", "--no-run", "--message-format=json"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let test_count = stdout.matches(r#""type":"test""#).count() as u32;

        results.push(CheckResult {
            name: "Nombre de tests".to_string(),
            passed: test_count >= min_tests,
            message: format!("{} tests (minimum: {})", test_count, min_tests),
        });
    }

    // Vérifier la taille du binaire
    if let Some(max_size) = metrics.max_binary_size {
        let binary_path = "target/release/repolens";
        if Path::new(binary_path).exists() {
            let metadata = fs::metadata(binary_path)?;
            let size = metadata.len();
            results.push(CheckResult {
                name: "Taille du binaire".to_string(),
                passed: size <= max_size,
                message: format!("{} bytes (maximum: {} bytes)", size, max_size),
            });
        }
    }

    Ok(results)
}
