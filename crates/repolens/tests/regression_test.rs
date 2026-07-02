//! Regression tests to prevent bugs from reoccurring

use repolens::rules::categories::dependencies::{
    parse_cargo_lock, parse_package_lock, parse_requirements_txt,
};
use repolens::scanner::Scanner;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cargo_lock_with_special_characters() {
    // Regression: Cargo.lock with special characters in package names
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("Cargo.lock"),
        r#"[[package]]
name = "test-package"
version = "1.0.0"

[[package]]
name = "test_package_2"
version = "2.0.0"
"#,
    )
    .unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());
    let deps = parse_cargo_lock(&scanner).unwrap();
    assert_eq!(deps.len(), 2);
}

#[test]
fn test_package_lock_empty_packages() {
    // Regression: package-lock.json with empty packages object
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("package-lock.json"),
        r#"{"packages": {}}"#,
    )
    .unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());
    let deps = parse_package_lock(&scanner).unwrap();
    assert!(deps.is_empty());
}

#[test]
fn test_requirements_txt_with_comments() {
    // Regression: requirements.txt with comments and empty lines
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("requirements.txt"),
        r#"# This is a comment
requests==2.28.0
# Another comment
flask>=2.0.0

pytest~=7.0.0
"#,
    )
    .unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());
    let deps = parse_requirements_txt(&scanner).unwrap();
    assert_eq!(deps.len(), 3);
}

#[test]
fn test_requirements_txt_with_extras() {
    // Regression: requirements.txt with extras markers [dev]
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("requirements.txt"),
        "requests[security]==2.28.0\npytest[testing]>=7.0.0\n",
    )
    .unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());
    let deps = parse_requirements_txt(&scanner).unwrap();
    // Should parse package name without extras
    assert!(deps.iter().any(|d| d.name == "requests"));
    assert!(deps.iter().any(|d| d.name == "pytest"));
}

#[test]
fn test_cargo_lock_missing_packages() {
    // Regression: Cargo.lock without [[package]] section
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("Cargo.lock"), "[metadata]\n").unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());
    let deps = parse_cargo_lock(&scanner).unwrap();
    assert!(deps.is_empty());
}

#[test]
fn test_package_lock_v1_format() {
    // Regression: package-lock.json v1 format (dependencies instead of packages)
    let temp_dir = TempDir::new().unwrap();
    fs::write(
        temp_dir.path().join("package-lock.json"),
        r#"{"dependencies": {"test-package": {"version": "1.0.0"}}}"#,
    )
    .unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());
    let deps = parse_package_lock(&scanner).unwrap();
    assert!(!deps.is_empty());
}
