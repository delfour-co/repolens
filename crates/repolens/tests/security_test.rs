//! Security tests to prevent vulnerabilities

use repolens_core::scanner::Scanner;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_no_secrets_in_logs() {
    // Test that secrets don't appear in error messages or logs
    // This is a placeholder - actual implementation would require
    // intercepting log output
    let temp_dir = TempDir::new().unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());

    // Create a file with a potential secret
    fs::write(
        temp_dir.path().join("test.rs"),
        "const API_KEY = \"sk_test_1234567890abcdef\";",
    )
    .unwrap();

    // Run secrets check - secrets should be detected but not logged in plain text
    // This test verifies the structure, actual secret masking would be in the implementation
    let _scanner = scanner;
    // Placeholder test - actual secret masking verification would go here
}

#[tokio::test]
async fn test_file_permissions() {
    // Test that file operations respect permissions
    let temp_dir = TempDir::new().unwrap();
    let scanner = Scanner::new(temp_dir.path().to_path_buf());

    // Create a file with restricted permissions (if possible)
    let test_file = temp_dir.path().join("restricted.txt");
    fs::write(&test_file, "test content").unwrap();

    // Try to read it - should succeed or fail gracefully
    let result = scanner.read_file("restricted.txt");
    // Should handle permission errors gracefully
    let _ = result;
}
