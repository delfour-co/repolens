//! Git repository utilities

use std::path::Path;
use std::process::Command;

/// Get the repository name from git remote
pub fn get_repository_name(root: &Path) -> Option<String> {
    // Try to get the remote URL
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    parse_repo_name_from_url(&url)
}

/// Parse repository name from a git URL
fn parse_repo_name_from_url(url: &str) -> Option<String> {
    // Handle SSH URLs: git@github.com:owner/repo.git
    if url.starts_with("git@") {
        let parts: Vec<&str> = url.split(':').collect();
        if parts.len() == 2 {
            let repo_path = parts[1].trim_end_matches(".git");
            return Some(repo_path.to_string());
        }
    }

    // Handle HTTPS URLs: https://github.com/owner/repo.git
    if url.starts_with("https://") || url.starts_with("http://") {
        let url = url.trim_end_matches(".git");
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2];
            let repo = parts[parts.len() - 1];
            return Some(format!("{}/{}", owner, repo));
        }
    }

    None
}

/// Get the default branch name
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
            .args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch)])
            .current_dir(root)
            .output()
            .ok()?;

        if output.status.success() {
            return Some(branch.to_string());
        }
    }

    None
}

/// Check if the repository is a git repository
pub fn is_git_repository(root: &Path) -> bool {
    root.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let url = "git@github.com:owner/repo.git";
        assert_eq!(parse_repo_name_from_url(url), Some("owner/repo".to_string()));
    }

    #[test]
    fn test_parse_https_url() {
        let url = "https://github.com/owner/repo.git";
        assert_eq!(parse_repo_name_from_url(url), Some("owner/repo".to_string()));
    }

    #[test]
    fn test_parse_https_url_without_git() {
        let url = "https://github.com/owner/repo";
        assert_eq!(parse_repo_name_from_url(url), Some("owner/repo".to_string()));
    }
}
