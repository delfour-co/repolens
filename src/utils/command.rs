//! Command execution utilities
//!
//! Provides helper functions for executing external commands with consistent
//! error handling and output capture.
//!
//! Note: These utilities are part of the public API for future refactoring
//! of command execution throughout the codebase.

use std::path::Path;
use std::process::Command;

/// Result of a command execution
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Exit code of the command
    pub exit_code: i32,
    /// Standard output (stdout)
    pub stdout: String,
    /// Standard error (stderr)
    pub stderr: String,
}

impl CommandResult {
    /// Check if the command succeeded (exit code 0)
    #[allow(dead_code)]
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Execute a command and capture its output
///
/// # Arguments
/// * `program` - The program to execute
/// * `args` - Arguments to pass to the program
/// * `working_dir` - Optional working directory
///
/// # Returns
/// * `Ok(CommandResult)` - The command result with exit code, stdout, and stderr
/// * `Err(std::io::Error)` - If the command failed to start
#[allow(dead_code)]
pub fn execute_command(
    program: &str,
    args: &[&str],
    working_dir: Option<&Path>,
) -> std::io::Result<CommandResult> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output()?;

    Ok(CommandResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

/// Execute a command and return stdout if successful, error otherwise
///
/// # Arguments
/// * `program` - The program to execute
/// * `args` - Arguments to pass to the program
/// * `working_dir` - Optional working directory
///
/// # Returns
/// * `Ok(String)` - stdout if exit code is 0
/// * `Err(String)` - Error message with stderr if command failed
#[allow(dead_code)]
pub fn execute_command_checked(
    program: &str,
    args: &[&str],
    working_dir: Option<&Path>,
) -> Result<String, String> {
    let result = execute_command(program, args, working_dir)
        .map_err(|e| format!("Failed to execute '{}': {}", program, e))?;

    if result.success() {
        Ok(result.stdout)
    } else {
        Err(format!(
            "'{}' failed with exit code {}: {}",
            program, result.exit_code, result.stderr
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_success() {
        let result = execute_command("echo", &["hello"], None).unwrap();
        assert!(result.success());
        assert_eq!(result.stdout, "hello");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_execute_command_failure() {
        let result = execute_command("false", &[], None).unwrap();
        assert!(!result.success());
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn test_execute_command_checked_success() {
        let stdout = execute_command_checked("echo", &["test"], None).unwrap();
        assert_eq!(stdout, "test");
    }

    #[test]
    fn test_execute_command_checked_failure() {
        let result = execute_command_checked("false", &[], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_command_not_found() {
        let result = execute_command("nonexistent_command_xyz", &[], None);
        assert!(result.is_err());
    }
}
