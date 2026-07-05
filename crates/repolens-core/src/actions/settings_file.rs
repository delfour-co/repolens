//! Merge missing branch-protection sections into an EXISTING
//! `.github/settings.yml` without clobbering the user's existing content.
//!
//! This is the review-bug-#12 remediation: SEC008 (`branches:` section
//! entirely absent), SEC009 (`required_pull_request_reviews` absent) and
//! SEC010 (`required_status_checks` absent) previously had no genuine
//! auto-fix for a settings file that already exists (only the file-creation
//! path for a *missing* file — SEC007 — was wired). This module parses the
//! existing YAML, adds only the missing keys, and writes it back.

use crate::error::{ActionError, RepoLensError};
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::Path;

/// Which sections of `.github/settings.yml` must be present after the merge.
#[derive(Debug, Clone)]
pub struct SettingsFileUpdate {
    /// Branch the protection block should target.
    pub branch: String,
    /// Required approving review count when adding
    /// `required_pull_request_reviews`.
    pub required_approvals: u32,
    /// Whether the `branches:` key itself is missing and must be added.
    pub ensure_branches_block: bool,
    /// Whether `required_pull_request_reviews` must be added.
    pub ensure_pr_reviews: bool,
    /// Whether `required_status_checks` must be added.
    pub ensure_status_checks: bool,
}

/// Merge the missing sections into `<root>/<path>`.
///
/// Reads the existing file (or starts from an empty mapping if it does not
/// exist), parses it as YAML, adds only the keys named by `update` that are
/// currently absent, and writes the result back. Existing keys — including
/// ones this function doesn't know about — are left untouched.
pub fn update_settings_file_at(
    root: &Path,
    path: &str,
    update: &SettingsFileUpdate,
) -> Result<(), RepoLensError> {
    let settings_path = root.join(path);

    let content = if settings_path.exists() {
        fs::read_to_string(&settings_path).map_err(|e| {
            RepoLensError::Scan(crate::error::ScanError::FileRead {
                path: settings_path.display().to_string(),
                source: e,
            })
        })?
    } else {
        String::new()
    };

    let mut doc: Value = if content.trim().is_empty() {
        Value::Mapping(Mapping::new())
    } else {
        serde_yaml::from_str(&content).map_err(|e| {
            RepoLensError::Action(ActionError::ExecutionFailed {
                message: format!("Failed to parse {path}: {e}"),
            })
        })?
    };

    let root_map = doc.as_mapping_mut().ok_or_else(|| {
        RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("{path} does not contain a YAML mapping at the top level"),
        })
    })?;

    merge_branch_protection(root_map, update);

    let serialized = serde_yaml::to_string(&doc).map_err(|e| {
        RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to serialize {path}: {e}"),
        })
    })?;

    if let Some(parent) = settings_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                RepoLensError::Action(ActionError::DirectoryCreate {
                    path: parent.display().to_string(),
                    source: e,
                })
            })?;
        }
    }

    fs::write(&settings_path, serialized).map_err(|e| {
        RepoLensError::Action(ActionError::FileWrite {
            path: settings_path.display().to_string(),
            source: e,
        })
    })?;

    Ok(())
}

/// Add the missing `branches` / `protection` / `required_pull_request_reviews`
/// / `required_status_checks` keys to `root_map`, without touching anything
/// already present.
fn merge_branch_protection(root_map: &mut Mapping, update: &SettingsFileUpdate) {
    if !update.ensure_branches_block && !update.ensure_pr_reviews && !update.ensure_status_checks {
        return;
    }

    if !root_map.contains_key("branches") {
        root_map.insert(Value::from("branches"), Value::Sequence(Vec::new()));
    }

    let Some(branches_seq) = root_map
        .get_mut("branches")
        .and_then(Value::as_sequence_mut)
    else {
        // `branches:` exists but isn't a sequence -- an unexpected shape.
        // Leave the user's content alone rather than clobbering it.
        return;
    };

    // Target the entry for the configured branch if one exists; otherwise
    // fall back to the first entry. `check_branch_protection` merges
    // `has_pr_reviews`/`has_status_checks` across ALL entries, so any single
    // entry gaining the missing keys clears the finding.
    let target_index = branches_seq
        .iter()
        .position(|entry| {
            entry
                .as_mapping()
                .and_then(|m| m.get("name"))
                .and_then(Value::as_str)
                == Some(update.branch.as_str())
        })
        .unwrap_or_else(|| {
            let mut entry = Mapping::new();
            entry.insert(Value::from("name"), Value::from(update.branch.clone()));
            entry.insert(Value::from("protection"), Value::Mapping(Mapping::new()));
            branches_seq.push(Value::Mapping(entry));
            branches_seq.len() - 1
        });

    let Some(entry_map) = branches_seq[target_index].as_mapping_mut() else {
        return;
    };

    if !entry_map.contains_key("protection") {
        entry_map.insert(Value::from("protection"), Value::Mapping(Mapping::new()));
    }
    let Some(protection_map) = entry_map
        .get_mut("protection")
        .and_then(Value::as_mapping_mut)
    else {
        return;
    };

    // SEC008 (branches block missing entirely) implies both sub-sections
    // must be added too, so the freshly-created entry doesn't immediately
    // re-trigger SEC009/SEC010.
    if (update.ensure_pr_reviews || update.ensure_branches_block)
        && !protection_map.contains_key("required_pull_request_reviews")
    {
        let mut reviews = Mapping::new();
        reviews.insert(
            Value::from("required_approving_review_count"),
            Value::from(update.required_approvals),
        );
        protection_map.insert(
            Value::from("required_pull_request_reviews"),
            Value::Mapping(reviews),
        );
    }

    if (update.ensure_status_checks || update.ensure_branches_block)
        && !protection_map.contains_key("required_status_checks")
    {
        let mut checks = Mapping::new();
        checks.insert(Value::from("strict"), Value::Bool(true));
        checks.insert(Value::from("contexts"), Value::Sequence(Vec::new()));
        protection_map.insert(
            Value::from("required_status_checks"),
            Value::Mapping(checks),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn update_all(branch: &str) -> SettingsFileUpdate {
        SettingsFileUpdate {
            branch: branch.to_string(),
            required_approvals: 1,
            ensure_branches_block: true,
            ensure_pr_reviews: true,
            ensure_status_checks: true,
        }
    }

    #[test]
    fn test_update_settings_file_creates_branches_block_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir_all(root.join(".github")).unwrap();
        fs::write(
            root.join(".github/settings.yml"),
            "repository:\n  name: my-repo\n  description: A test repo\n",
        )
        .unwrap();

        update_settings_file_at(root, ".github/settings.yml", &update_all("main")).unwrap();

        let content = fs::read_to_string(root.join(".github/settings.yml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&content).unwrap();

        // Original content preserved.
        assert_eq!(
            parsed
                .get("repository")
                .and_then(|r| r.get("name"))
                .and_then(Value::as_str),
            Some("my-repo")
        );

        let branches = parsed.get("branches").and_then(Value::as_sequence).unwrap();
        assert_eq!(branches.len(), 1);
        let protection = branches[0].get("protection").unwrap();
        assert!(protection.get("required_pull_request_reviews").is_some());
        assert!(protection.get("required_status_checks").is_some());
    }

    #[test]
    fn test_update_settings_file_adds_only_missing_pr_reviews() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir_all(root.join(".github")).unwrap();
        fs::write(
            root.join(".github/settings.yml"),
            "branches:\n  - name: main\n    protection:\n      required_status_checks:\n        strict: true\n        contexts:\n          - ci\n",
        )
        .unwrap();

        let update = SettingsFileUpdate {
            branch: "main".to_string(),
            required_approvals: 2,
            ensure_branches_block: false,
            ensure_pr_reviews: true,
            ensure_status_checks: false,
        };
        update_settings_file_at(root, ".github/settings.yml", &update).unwrap();

        let content = fs::read_to_string(root.join(".github/settings.yml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&content).unwrap();
        let branches = parsed.get("branches").and_then(Value::as_sequence).unwrap();
        assert_eq!(branches.len(), 1);
        let protection = branches[0].get("protection").unwrap();

        let reviews = protection.get("required_pull_request_reviews").unwrap();
        assert_eq!(
            reviews
                .get("required_approving_review_count")
                .and_then(Value::as_u64),
            Some(2)
        );

        // Pre-existing required_status_checks (with a custom context) must
        // survive untouched.
        let checks = protection.get("required_status_checks").unwrap();
        assert_eq!(
            checks
                .get("contexts")
                .and_then(Value::as_sequence)
                .map(|s| s.len()),
            Some(1)
        );
    }

    #[test]
    fn test_update_settings_file_creates_file_when_absent() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        update_settings_file_at(root, ".github/settings.yml", &update_all("main")).unwrap();

        assert!(root.join(".github/settings.yml").exists());
    }

    #[test]
    fn test_update_settings_file_targets_matching_branch_name() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::create_dir_all(root.join(".github")).unwrap();
        fs::write(
            root.join(".github/settings.yml"),
            "branches:\n  - name: develop\n    protection: {}\n",
        )
        .unwrap();

        // Requesting protection for "main" (not "develop") should append a
        // new entry rather than mutating the unrelated "develop" entry.
        update_settings_file_at(root, ".github/settings.yml", &update_all("main")).unwrap();

        let content = fs::read_to_string(root.join(".github/settings.yml")).unwrap();
        let parsed: Value = serde_yaml::from_str(&content).unwrap();
        let branches = parsed.get("branches").and_then(Value::as_sequence).unwrap();
        assert_eq!(branches.len(), 2);
        assert!(
            branches[1]
                .get("protection")
                .and_then(|p| p.get("required_pull_request_reviews"))
                .is_some()
        );
    }
}
