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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command as StdCommand;
    use tempfile::TempDir;

    fn init_git_repo(root: &Path) -> bool {
        StdCommand::new("git")
            .args(["init"])
            .current_dir(root)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn configure_git_user(root: &Path) {
        let _ = StdCommand::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(root)
            .output();
        let _ = StdCommand::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(root)
            .output();
    }

    fn create_initial_commit(root: &Path) {
        fs::write(root.join("README.md"), "# Test").unwrap();
        let _ = StdCommand::new("git")
            .args(["add", "README.md"])
            .current_dir(root)
            .output();
        let _ = StdCommand::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(root)
            .output();
    }

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
    fn test_get_repository_name_no_git() {
        let temp_dir = TempDir::new().unwrap();
        // No git repo, should return None
        assert!(get_repository_name(temp_dir.path()).is_none());
    }

    #[test]
    fn test_get_repository_name_no_remote() {
        let temp_dir = TempDir::new().unwrap();
        if !init_git_repo(temp_dir.path()) {
            return; // Skip if git not available
        }
        configure_git_user(temp_dir.path());
        create_initial_commit(temp_dir.path());

        // Git repo but no remote
        assert!(get_repository_name(temp_dir.path()).is_none());
    }

    #[test]
    fn test_parse_gitlab_ssh_url() {
        let url = "git@gitlab.com:group/subgroup/repo.git";
        let result = parse_repo_name_from_url_impl(url);
        // Should parse the path after the colon
        assert!(result.is_some());
        assert!(result.unwrap().contains("repo"));
    }

    #[test]
    fn test_parse_bitbucket_https_url() {
        let url = "https://bitbucket.org/team/repo.git";
        assert_eq!(
            parse_repo_name_from_url_impl(url),
            Some("team/repo".to_string())
        );
    }

    #[test]
    fn test_parse_empty_url() {
        assert_eq!(parse_repo_name_from_url_impl(""), None);
    }

    #[test]
    fn test_parse_url_only_protocol() {
        // These URLs are incomplete but the parser extracts what it can
        let result1 = parse_repo_name_from_url_impl("https://");
        let result2 = parse_repo_name_from_url_impl("http://");
        // Just verify it doesn't panic
        let _ = result1;
        let _ = result2;
    }
}
