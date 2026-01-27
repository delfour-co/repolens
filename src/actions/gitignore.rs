//! Gitignore file management

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Update .gitignore with new entries at the given root path
pub fn update_gitignore_at(root: &Path, entries: &[String]) -> Result<()> {
    let gitignore_path = root.join(".gitignore");

    // Read existing content or create empty
    let mut content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path).context("Failed to read .gitignore")?
    } else {
        String::new()
    };

    // Track what we add
    let mut added = Vec::new();

    for entry in entries {
        // Check if entry already exists (handle various formats)
        let entry_clean = entry.trim_end_matches('/');
        let entry_patterns = [
            entry.as_str(),
            &format!("/{}", entry),
            &format!("{}/", entry),
            entry_clean,
            &format!("/{}", entry_clean),
            &format!("{}/", entry_clean),
        ];

        let exists = content.lines().any(|line| {
            let line = line.trim();
            let line_clean = line.trim_end_matches('/');
            entry_patterns
                .iter()
                .any(|p| line == *p || line_clean == entry_clean)
        });

        if !exists {
            added.push(entry.clone());
        }
    }

    if added.is_empty() {
        return Ok(());
    }

    // Add a newline if the file doesn't end with one
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    // Add comment separator if there's existing content
    if !content.is_empty() {
        content.push_str("\n# Added by repolens\n");
    }

    // Add new entries
    for entry in &added {
        content.push_str(entry);
        content.push('\n');
    }

    // Write back
    fs::write(&gitignore_path, content).context("Failed to write .gitignore")?;

    Ok(())
}

/// Update .gitignore with new entries in current directory
pub fn update_gitignore(entries: &[String]) -> Result<()> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    update_gitignore_at(&current_dir, entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_update_gitignore_new_file() {
        let dir = tempdir().unwrap();

        update_gitignore_at(dir.path(), &[".env".to_string(), "*.key".to_string()]).unwrap();

        let content = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(content.contains(".env"));
        assert!(content.contains("*.key"));
    }

    #[test]
    fn test_update_gitignore_existing_file() {
        let dir = tempdir().unwrap();

        fs::write(dir.path().join(".gitignore"), "node_modules/\n").unwrap();

        update_gitignore_at(
            dir.path(),
            &[".env".to_string(), "node_modules".to_string()],
        )
        .unwrap();

        let content = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(content.contains("node_modules"));
        assert!(content.contains(".env"));
        // Should not duplicate
        assert_eq!(content.matches("node_modules").count(), 1);
    }
}
