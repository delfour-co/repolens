//! Tests for provider modules

use repolens::providers::github::{BranchProtection, GitHubProvider};

#[test]
fn test_github_provider_is_available() {
    // Test that the function doesn't panic
    // Result depends on environment
    let _ = GitHubProvider::is_available();
}

#[test]
fn test_branch_protection_deserialize() {
    let json = r#"{
        "required_status_checks": {
            "strict": true,
            "contexts": ["test"]
        },
        "enforce_admins": {
            "enabled": true
        },
        "required_pull_request_reviews": {
            "required_approving_review_count": 2
        },
        "required_linear_history": {
            "enabled": false
        },
        "allow_force_pushes": {
            "enabled": false
        },
        "allow_deletions": {
            "enabled": false
        }
    }"#;

    let protection: BranchProtection = serde_json::from_str(json).unwrap();
    assert!(protection.required_status_checks.is_some());
    assert!(protection.enforce_admins.is_some());
    assert!(protection.required_pull_request_reviews.is_some());
    if let Some(status_checks) = &protection.required_status_checks {
        assert!(status_checks.strict);
        assert_eq!(status_checks.contexts.len(), 1);
    }
    if let Some(reviews) = &protection.required_pull_request_reviews {
        assert_eq!(reviews.required_approving_review_count, 2);
    }
}

#[test]
fn test_branch_protection_deserialize_minimal() {
    // Test with minimal JSON (all fields optional)
    let json = r#"{}"#;
    let protection: BranchProtection = serde_json::from_str(json).unwrap();
    assert!(protection.required_status_checks.is_none());
    assert!(protection.enforce_admins.is_none());
}

#[test]
fn test_branch_protection_deserialize_partial() {
    let json = r#"{
        "required_status_checks": {
            "strict": false,
            "contexts": []
        }
    }"#;
    let protection: BranchProtection = serde_json::from_str(json).unwrap();
    assert!(protection.required_status_checks.is_some());
    assert!(protection.enforce_admins.is_none());
    if let Some(status_checks) = &protection.required_status_checks {
        assert!(!status_checks.strict);
        assert!(status_checks.contexts.is_empty());
    }
}
