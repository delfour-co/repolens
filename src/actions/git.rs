//! Git operations for creating branches, commits, and pushing changes

use crate::error::{ActionError, RepoLensError};
use chrono::Local;
use std::path::Path;
use std::process::Command;

/// Create a new git branch with a timestamp-based name
///
/// # Arguments
///
/// * `root` - The root directory of the git repository
///
/// # Returns
///
/// The name of the created branch
///
/// # Errors
///
/// Returns an error if git operations fail
pub fn create_branch(root: &Path) -> Result<String, RepoLensError> {
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let branch_name = format!("repolens/apply-{}", timestamp);

    // Create and checkout the new branch
    let output = Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .current_dir(root)
        .output()
        .map_err(|e| {
            RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to create branch: {}", e),
            })
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to create branch '{}': {}", branch_name, stderr),
        }));
    }

    Ok(branch_name)
}

/// Check if there are any uncommitted changes in the repository
///
/// # Arguments
///
/// * `root` - The root directory of the git repository
///
/// # Returns
///
/// `true` if there are uncommitted changes, `false` otherwise
pub fn has_changes(root: &Path) -> Result<bool, RepoLensError> {
    // Check if there are any changes (staged or unstaged)
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .map_err(|e| {
            RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to check git status: {}", e),
            })
        })?;

    if !status_output.status.success() {
        return Err(RepoLensError::Action(ActionError::ExecutionFailed {
            message: "Failed to check git status".to_string(),
        }));
    }

    let status = String::from_utf8_lossy(&status_output.stdout);
    Ok(!status.trim().is_empty())
}

/// Stage all changes in the repository
///
/// # Arguments
///
/// * `root` - The root directory of the git repository
///
/// # Errors
///
/// Returns an error if git add fails
#[allow(dead_code)]
pub fn stage_all_changes(root: &Path) -> Result<(), RepoLensError> {
    let output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(root)
        .output()
        .map_err(|e| {
            RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to stage changes: {}", e),
            })
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to stage changes: {}", stderr),
        }));
    }

    Ok(())
}

/// Stage specific files in the repository
///
/// # Arguments
///
/// * `root` - The root directory of the git repository
/// * `files` - The list of file paths to stage (relative to the repository root)
///
/// # Errors
///
/// Returns an error if git add fails
pub fn stage_files(root: &Path, files: &[String]) -> Result<(), RepoLensError> {
    if files.is_empty() {
        return Ok(());
    }

    let mut args = vec!["add", "--"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);

    let output = Command::new("git")
        .args(&args)
        .current_dir(root)
        .output()
        .map_err(|e| {
            RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to stage files: {}", e),
            })
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to stage files: {}", stderr),
        }));
    }

    Ok(())
}

/// Create a commit with the given message
///
/// # Arguments
///
/// * `root` - The root directory of the git repository
/// * `message` - The commit message
///
/// # Errors
///
/// Returns an error if git commit fails
pub fn create_commit(root: &Path, message: &str) -> Result<(), RepoLensError> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(root)
        .output()
        .map_err(|e| {
            RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to create commit: {}", e),
            })
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to create commit: {}", stderr),
        }));
    }

    Ok(())
}

/// Push the current branch to the remote repository
///
/// # Arguments
///
/// * `root` - The root directory of the git repository
/// * `branch_name` - The name of the branch to push
///
/// # Errors
///
/// Returns an error if git push fails
pub fn push_branch(root: &Path, branch_name: &str) -> Result<(), RepoLensError> {
    let output = Command::new("git")
        .args(["push", "-u", "origin", branch_name])
        .current_dir(root)
        .output()
        .map_err(|e| {
            RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to push branch: {}", e),
            })
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to push branch '{}': {}", branch_name, stderr),
        }));
    }

    Ok(())
}

/// Get the current branch name
///
/// # Arguments
///
/// * `root` - The root directory of the git repository
///
/// # Returns
///
/// The current branch name, or `None` if not in a git repository
#[allow(dead_code)]
pub fn get_current_branch(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        None
    } else {
        Some(branch)
    }
}

/// Get the default branch (main or master)
///
/// # Arguments
///
/// * `root` - The root directory of the git repository
///
/// # Returns
///
/// The default branch name, or `None` if not found
pub fn get_default_branch(root: &Path) -> Option<String> {
    // Try to get from git symbolic-ref
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD", "--short"])
        .current_dir(root)
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_start_matches("origin/")
            .to_string();
        if !branch.is_empty() {
            return Some(branch);
        }
    }

    // Fall back to checking for main or master
    let branches = ["main", "master"];
    for branch in branches {
        let output = Command::new("git")
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{}", branch),
            ])
            .current_dir(root)
            .output()
            .ok()?;

        if output.status.success() {
            return Some(branch.to_string());
        }
    }

    None
}

/// Check if the current directory is a git repository
///
/// # Arguments
///
/// * `root` - The directory to check
///
/// # Returns
///
/// `true` if it's a git repository, `false` otherwise
pub fn is_git_repository(root: &Path) -> bool {
    root.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn init_git_repo(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Command::new("git")
            .args(["init"])
            .current_dir(root)
            .output()?;

        // Configure git user (required for commits)
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(root)
            .output()?;

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(root)
            .output()?;

        // Create initial commit
        fs::write(root.join("README.md"), "# Test Repo")?;
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(root)
            .output()?;

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(root)
            .output()?;

        Ok(())
    }

    #[test]
    fn test_is_git_repository() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Test with non-git directory
        assert!(!is_git_repository(root));

        // Test with git directory
        fs::create_dir(root.join(".git")).unwrap();
        assert!(is_git_repository(root));
    }

    #[test]
    #[serial]
    fn test_create_branch() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

        // Initialize git repo
        init_git_repo(&root_abs).expect("Failed to init git repo");

        // Create branch
        let branch_name = create_branch(&root_abs).expect("Failed to create branch");

        // Verify branch name format
        assert!(branch_name.starts_with("repolens/apply-"));

        // Verify we're on the new branch
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&root_abs)
            .output()
            .unwrap();

        let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(current_branch, branch_name);

        let _ = std::env::set_current_dir(&original_dir);
    }

    #[test]
    #[serial]
    fn test_has_changes() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

        // Initialize git repo
        init_git_repo(&root_abs).expect("Failed to init git repo");

        // Initially no changes
        assert!(!has_changes(&root_abs).expect("Failed to check changes"));

        // Create a new file
        fs::write(root_abs.join("test.txt"), "test content").unwrap();

        // Now there should be changes
        assert!(has_changes(&root_abs).expect("Failed to check changes"));

        let _ = std::env::set_current_dir(&original_dir);
    }

    #[test]
    #[serial]
    fn test_stage_all_changes() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

        // Initialize git repo
        init_git_repo(&root_abs).expect("Failed to init git repo");

        // Create a new file
        fs::write(root_abs.join("test.txt"), "test content").unwrap();

        // Stage changes
        stage_all_changes(&root_abs).expect("Failed to stage changes");

        // Verify file is staged
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&root_abs)
            .output()
            .unwrap();

        let status = String::from_utf8_lossy(&output.stdout);
        assert!(status.contains("test.txt"));
        assert!(status.starts_with("A")); // A = Added

        let _ = std::env::set_current_dir(&original_dir);
    }

    #[test]
    #[serial]
    fn test_create_commit() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

        // Initialize git repo
        init_git_repo(&root_abs).expect("Failed to init git repo");

        // Create and stage a file
        fs::write(root_abs.join("test.txt"), "test content").unwrap();
        stage_all_changes(&root_abs).expect("Failed to stage changes");

        // Create commit
        let commit_message = "Test commit";
        create_commit(&root_abs, commit_message).expect("Failed to create commit");

        // Verify commit was created
        let output = Command::new("git")
            .args(["log", "--oneline", "-1"])
            .current_dir(&root_abs)
            .output()
            .unwrap();

        let log = String::from_utf8_lossy(&output.stdout);
        assert!(log.contains(commit_message));

        let _ = std::env::set_current_dir(&original_dir);
    }

    #[test]
    #[serial]
    fn test_stage_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

        // Initialize git repo
        init_git_repo(&root_abs).expect("Failed to init git repo");

        // Create multiple files
        fs::write(root_abs.join("file1.txt"), "content 1").unwrap();
        fs::write(root_abs.join("file2.txt"), "content 2").unwrap();
        fs::write(root_abs.join("file3.txt"), "content 3").unwrap();

        // Stage only specific files
        let files = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        stage_files(&root_abs, &files).expect("Failed to stage files");

        // Verify only file1.txt and file2.txt are staged, file3.txt is not
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&root_abs)
            .output()
            .unwrap();

        let status = String::from_utf8_lossy(&output.stdout);
        assert!(
            status.contains("A  file1.txt"),
            "file1.txt should be staged"
        );
        assert!(
            status.contains("A  file2.txt"),
            "file2.txt should be staged"
        );
        assert!(
            status.contains("?? file3.txt"),
            "file3.txt should be untracked, not staged"
        );

        let _ = std::env::set_current_dir(&original_dir);
    }

    #[test]
    #[serial]
    fn test_stage_files_empty() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

        // Initialize git repo
        init_git_repo(&root_abs).expect("Failed to init git repo");

        // Stage empty list should succeed without error
        let files: Vec<String> = vec![];
        stage_files(&root_abs, &files).expect("Staging empty file list should succeed");

        let _ = std::env::set_current_dir(&original_dir);
    }

    #[test]
    #[serial]
    fn test_get_default_branch() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

        // Initialize git repo
        init_git_repo(&root_abs).expect("Failed to init git repo");

        // Get default branch (should be main or master)
        let default_branch = get_default_branch(&root_abs);

        // Should find main or master
        assert!(default_branch.is_some());
        let branch = default_branch.unwrap();
        assert!(branch == "main" || branch == "master");

        let _ = std::env::set_current_dir(&original_dir);
    }

    #[test]
    #[serial]
    fn test_get_current_branch() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));

        if std::env::current_dir().is_err() {
            let _ = std::env::set_current_dir("/tmp");
        }

        std::env::set_current_dir(&root_abs).expect("Failed to change to temp directory");

        // Initialize git repo
        init_git_repo(&root_abs).expect("Failed to init git repo");

        // Get current branch
        let current_branch = get_current_branch(&root_abs);

        // Should find a branch (main or master)
        assert!(current_branch.is_some());
        let branch = current_branch.unwrap();
        assert!(branch == "main" || branch == "master");

        let _ = std::env::set_current_dir(&original_dir);
    }
}
