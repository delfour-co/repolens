use criterion::{black_box, criterion_group, criterion_main, Criterion};
use repolens::rules::categories::dependencies::{
    parse_cargo_lock, parse_package_lock, parse_requirements_txt,
};
use repolens::scanner::Scanner;
use std::fs;
use tempfile::TempDir;

fn generate_large_cargo_lock() -> String {
    let mut content = String::from("[[package]]\nname = \"repolens\"\nversion = \"0.1.0\"\n");
    for i in 0..100 {
        content.push_str(&format!(
            "[[package]]\nname = \"dep{}\"\nversion = \"1.0.{}\"\n",
            i, i
        ));
    }
    content
}

fn benchmark_parse_cargo_lock(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let lock_content = generate_large_cargo_lock();
    fs::write(temp_dir.path().join("Cargo.lock"), lock_content).unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());

    c.bench_function("parse_cargo_lock", |b| {
        b.iter(|| parse_cargo_lock(black_box(&scanner)))
    });
}

fn benchmark_parse_package_lock(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let lock_content = r#"{"packages": {"": {"name": "test", "version": "1.0.0"}}}"#;
    fs::write(temp_dir.path().join("package-lock.json"), lock_content).unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());

    c.bench_function("parse_package_lock", |b| {
        b.iter(|| parse_package_lock(black_box(&scanner)))
    });
}

fn benchmark_parse_requirements_txt(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let req_content = "requests==2.28.0\nflask>=2.0.0\npytest~=7.0.0\n";
    fs::write(temp_dir.path().join("requirements.txt"), req_content).unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());

    c.bench_function("parse_requirements_txt", |b| {
        b.iter(|| parse_requirements_txt(black_box(&scanner)))
    });
}

criterion_group!(
    benches,
    benchmark_parse_cargo_lock,
    benchmark_parse_package_lock,
    benchmark_parse_requirements_txt
);
criterion_main!(benches);
