//! Exit codes for the CLI
//!
//! Standard exit codes used by RepoLens CLI for CI/CD integration.
//!
//! # Exit Code Reference
//!
//! | Code | Constant | Meaning | Example |
//! |------|----------|---------|---------|
//! | 0 | `SUCCESS` | Success | Audit completed, no critical issues |
//! | 1 | `CRITICAL_ISSUES` | Critical issues | Secrets exposed, critical vulnerabilities |
//! | 2 | `WARNINGS` | Warnings | Missing files, non-critical findings |
//! | 3 | `ERROR` | Runtime error | File not found, network error |
//! | 4 | `INVALID_ARGS` | Invalid arguments | Unknown category, invalid preset |
//!
//! # Usage
//!
//! ```rust,ignore
//! use repolens::cli::exit_codes;
//!
//! // Return success
//! std::process::exit(exit_codes::SUCCESS);
//!
//! // Return critical issues
//! std::process::exit(exit_codes::CRITICAL_ISSUES);
//! ```

/// Success - no issues found or operation completed successfully.
///
/// Used when:
/// - Audit completed with no findings
/// - All actions applied successfully
/// - Command completed normally
pub const SUCCESS: i32 = 0;

/// Critical issues detected (secrets exposed, critical vulnerabilities).
///
/// Used when:
/// - Secrets are detected in the repository
/// - Critical security vulnerabilities are found
/// - All applied actions failed
/// - Regressions detected (compare command with --fail-on-regression)
pub const CRITICAL_ISSUES: i32 = 1;

/// Warning issues detected (missing files, non-critical findings).
///
/// Used when:
/// - Missing recommended files (README, LICENSE, etc.)
/// - Non-critical security findings
/// - Some actions failed (partial success)
pub const WARNINGS: i32 = 2;

/// Runtime error (file not found, network error, etc.).
///
/// Used when:
/// - Configuration file not found or invalid
/// - File system errors
/// - Network errors
/// - Prerequisites check failed
/// - All actions failed during apply
pub const ERROR: i32 = 3;

/// Invalid arguments (unknown category, invalid preset, etc.).
///
/// Used when:
/// - Unknown preset name provided
/// - Invalid category specified
/// - Invalid command-line arguments
pub const INVALID_ARGS: i32 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes_are_distinct() {
        let codes = [SUCCESS, CRITICAL_ISSUES, WARNINGS, ERROR, INVALID_ARGS];
        for i in 0..codes.len() {
            for j in (i + 1)..codes.len() {
                assert_ne!(
                    codes[i], codes[j],
                    "Exit codes should be unique: {} and {} are both {}",
                    i, j, codes[i]
                );
            }
        }
    }

    #[test]
    fn test_exit_codes_values() {
        assert_eq!(SUCCESS, 0);
        assert_eq!(CRITICAL_ISSUES, 1);
        assert_eq!(WARNINGS, 2);
        assert_eq!(ERROR, 3);
        assert_eq!(INVALID_ARGS, 4);
    }
}
