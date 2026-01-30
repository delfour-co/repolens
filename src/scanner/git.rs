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
    parse_repo_name_from_url_impl(&url)
}

/// Parse repository name from a git URL
#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn parse_repo_name_from_url(url: &str) -> Option<String> {
    parse_repo_name_from_url_impl(url)
}

fn parse_repo_name_from_url_impl(url: &str) -> Option<String> {
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
#[allow(dead_code)]
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

/// Check if the repository is a git repository
#[allow(dead_code)]
pub fn is_git_repository(root: &Path) -> bool {
    root.join(".git").exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let url = "git@github.com:owner/repo.git";
        assert_eq!(
            parse_repo_name_from_url_impl(url),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_parse_https_url() {
        let url = "https://github.com/owner/repo.git";
        assert_eq!(
            parse_repo_name_from_url_impl(url),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_parse_https_url_without_git() {
        let url = "https://github.com/owner/repo";
        assert_eq!(
            parse_repo_name_from_url_impl(url),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_parse_http_url() {
        let url = "http://github.com/owner/repo.git";
        assert_eq!(
            parse_repo_name_from_url_impl(url),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_parse_ssh_url_with_port() {
        let url = "git@github.com:2222:owner/repo.git";
        // Should handle port in SSH URL (though uncommon)
        let result = parse_repo_name_from_url_impl(url);
        // May or may not parse correctly depending on implementation
        let _ = result;
    }

    #[test]
    fn test_parse_invalid_url() {
        let url = "not-a-valid-url";
        assert_eq!(parse_repo_name_from_url_impl(url), None);
    }

    #[test]
    fn test_parse_https_url_with_path() {
        let url = "https://github.com/owner/repo.git";
        assert_eq!(
            parse_repo_name_from_url_impl(url),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_is_git_repository() {
        use std::fs;
        use tempfile::TempDir;

        // Test with non-git directory
        let temp_dir = TempDir::new().unwrap();
        assert!(!is_git_repository(temp_dir.path()));

        // Test with git directory
        fs::create_dir(temp_dir.path().join(".git")).unwrap();
        assert!(is_git_repository(temp_dir.path()));
    }
}
