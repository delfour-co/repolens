//! Issues and Pull Requests hygiene rules
//!
//! This module provides rules for checking issue and PR hygiene:
//! - Stale issues (ISSUE001)
//! - Stale PRs (ISSUE002)
//! - Issues without labels (ISSUE003)
//! - PRs without reviewers (PR001)
//! - Abandoned draft PRs (PR002)

use crate::config::Config;
use crate::error::RepoLensError;
use crate::providers::github::GitHubProvider;
use crate::rules::engine::RuleCategory;
use crate::rules::results::{Finding, Severity};
use crate::scanner::Scanner;

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Threshold for stale issues (90 days)
const STALE_ISSUE_DAYS: i64 = 90;
/// Threshold for stale PRs (30 days)
const STALE_PR_DAYS: i64 = 30;
/// Threshold for abandoned draft PRs (14 days)
const ABANDONED_DRAFT_DAYS: i64 = 14;

#[derive(Debug, Deserialize)]
struct Issue {
    #[allow(dead_code)]
    number: u64,
    #[allow(dead_code)]
    title: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    labels: Vec<Label>,
}

#[derive(Debug, Deserialize)]
struct Label {
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    #[allow(dead_code)]
    number: u64,
    #[allow(dead_code)]
    title: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    #[serde(rename = "reviewRequests")]
    review_requests: ReviewRequests,
    reviews: Reviews,
}

#[derive(Debug, Deserialize)]
struct ReviewRequests {
    #[serde(rename = "totalCount")]
    total_count: u64,
}

#[derive(Debug, Deserialize)]
struct Reviews {
    #[serde(rename = "totalCount")]
    total_count: u64,
}

/// Rules for checking issue and PR hygiene
pub struct IssuesRules;

#[async_trait::async_trait]
impl RuleCategory for IssuesRules {
    fn name(&self) -> &'static str {
        "issues"
    }

    async fn run(
        &self,
        _scanner: &Scanner,
        config: &Config,
    ) -> Result<Vec<Finding>, RepoLensError> {
        let mut findings = Vec::new();

        if !GitHubProvider::is_available() {
            return Ok(findings);
        }

        let provider = match GitHubProvider::new() {
            Ok(p) => p,
            Err(_) => return Ok(findings),
        };

        if config.is_rule_enabled("issues/stale-issues") {
            findings.extend(check_stale_issues(&provider));
        }

        if config.is_rule_enabled("issues/stale-prs") {
            findings.extend(check_stale_prs(&provider));
        }

        if config.is_rule_enabled("issues/unlabeled") {
            findings.extend(check_unlabeled_issues(&provider));
        }

        if config.is_rule_enabled("issues/pr-reviewers") {
            findings.extend(check_pr_reviewers(&provider));
        }

        if config.is_rule_enabled("issues/abandoned-drafts") {
            findings.extend(check_abandoned_drafts(&provider));
        }

        Ok(findings)
    }
}

/// ISSUE001: Check for stale issues (> 90 days without activity)
fn check_stale_issues(provider: &GitHubProvider) -> Vec<Finding> {
    let mut findings = Vec::new();

    let output = std::process::Command::new("gh")
        .args([
            "issue",
            "list",
            "--repo",
            &format!("{}/{}", provider.owner(), provider.name()),
            "--state",
            "open",
            "--json",
            "number,title,updatedAt,labels",
            "--limit",
            "100",
        ])
        .output();

    let issues: Vec<Issue> = match output {
        Ok(out) if out.status.success() => serde_json::from_slice(&out.stdout).unwrap_or_default(),
        _ => return findings,
    };

    let now = Utc::now();
    let mut stale_count = 0;

    for issue in &issues {
        if let Ok(updated) = issue.updated_at.parse::<DateTime<Utc>>() {
            let days_since = (now - updated).num_days();
            if days_since > STALE_ISSUE_DAYS {
                stale_count += 1;
            }
        }
    }

    if stale_count > 0 {
        findings.push(
            Finding::new(
                "ISSUE001",
                "issues",
                Severity::Info,
                format!(
                    "{} stale issue(s) with no activity for over {} days",
                    stale_count, STALE_ISSUE_DAYS
                ),
            )
            .with_description(
                "Stale issues may indicate abandoned features, forgotten bugs, or \
                 lack of triage. Regular cleanup improves project visibility and contributor experience.",
            )
            .with_remediation(
                "Review stale issues and close those that are no longer relevant. \
                 Consider using a stale bot (e.g., actions/stale) to automate cleanup.",
            ),
        );
    }

    findings
}

/// ISSUE002: Check for stale PRs (> 30 days without activity)
fn check_stale_prs(provider: &GitHubProvider) -> Vec<Finding> {
    let mut findings = Vec::new();

    let prs = match list_open_prs(provider) {
        Some(prs) => prs,
        None => return findings,
    };

    let now = Utc::now();
    let mut stale_count = 0;

    for pr in &prs {
        if let Ok(updated) = pr.updated_at.parse::<DateTime<Utc>>() {
            let days_since = (now - updated).num_days();
            if days_since > STALE_PR_DAYS {
                stale_count += 1;
            }
        }
    }

    if stale_count > 0 {
        findings.push(
            Finding::new(
                "ISSUE002",
                "issues",
                Severity::Info,
                format!(
                    "{} stale pull request(s) with no activity for over {} days",
                    stale_count, STALE_PR_DAYS
                ),
            )
            .with_description(
                "Stale pull requests can indicate blocked work, insufficient review bandwidth, \
                 or abandoned contributions. They accumulate merge conflicts over time.",
            )
            .with_remediation(
                "Review stale PRs: merge, close, or request updates from authors. \
                 Consider configuring branch protection rules with required reviews.",
            ),
        );
    }

    findings
}

/// ISSUE003: Check for issues without labels
fn check_unlabeled_issues(provider: &GitHubProvider) -> Vec<Finding> {
    let mut findings = Vec::new();

    let output = std::process::Command::new("gh")
        .args([
            "issue",
            "list",
            "--repo",
            &format!("{}/{}", provider.owner(), provider.name()),
            "--state",
            "open",
            "--json",
            "number,title,updatedAt,labels",
            "--limit",
            "100",
        ])
        .output();

    let issues: Vec<Issue> = match output {
        Ok(out) if out.status.success() => serde_json::from_slice(&out.stdout).unwrap_or_default(),
        _ => return findings,
    };

    let unlabeled_count = issues.iter().filter(|i| i.labels.is_empty()).count();

    if unlabeled_count > 0 {
        findings.push(
            Finding::new(
                "ISSUE003",
                "issues",
                Severity::Info,
                format!("{} issue(s) without labels", unlabeled_count),
            )
            .with_description(
                "Issues without labels are harder to triage, filter, and prioritize. \
                 Labels help organize work and provide visibility into issue categories.",
            )
            .with_remediation(
                "Add relevant labels to open issues (e.g., bug, enhancement, documentation). \
                 Consider using GitHub's default label set or creating a custom labeling scheme.",
            ),
        );
    }

    findings
}

/// PR001: Check for PRs without reviewers assigned
fn check_pr_reviewers(provider: &GitHubProvider) -> Vec<Finding> {
    let mut findings = Vec::new();

    let prs = match list_open_prs(provider) {
        Some(prs) => prs,
        None => return findings,
    };

    let no_reviewer_count = prs
        .iter()
        .filter(|pr| {
            !pr.is_draft && pr.review_requests.total_count == 0 && pr.reviews.total_count == 0
        })
        .count();

    if no_reviewer_count > 0 {
        findings.push(
            Finding::new(
                "PR001",
                "issues",
                Severity::Warning,
                format!(
                    "{} pull request(s) without reviewers assigned",
                    no_reviewer_count
                ),
            )
            .with_description(
                "Pull requests without reviewers may bypass code review processes. \
                 Code review is essential for maintaining code quality and knowledge sharing.",
            )
            .with_remediation(
                "Assign reviewers to all open pull requests. Consider configuring CODEOWNERS \
                 to automatically request reviews from the appropriate teams.",
            ),
        );
    }

    findings
}

/// PR002: Check for abandoned draft PRs (> 14 days)
fn check_abandoned_drafts(provider: &GitHubProvider) -> Vec<Finding> {
    let mut findings = Vec::new();

    let prs = match list_open_prs(provider) {
        Some(prs) => prs,
        None => return findings,
    };

    let now = Utc::now();
    let mut abandoned_count = 0;

    for pr in &prs {
        if pr.is_draft {
            if let Ok(updated) = pr.updated_at.parse::<DateTime<Utc>>() {
                let days_since = (now - updated).num_days();
                if days_since > ABANDONED_DRAFT_DAYS {
                    abandoned_count += 1;
                }
            }
        }
    }

    if abandoned_count > 0 {
        findings.push(
            Finding::new(
                "PR002",
                "issues",
                Severity::Info,
                format!(
                    "{} abandoned draft pull request(s) (no activity for over {} days)",
                    abandoned_count, ABANDONED_DRAFT_DAYS
                ),
            )
            .with_description(
                "Draft pull requests with no recent activity may represent abandoned work. \
                 They clutter the PR list and can accumulate merge conflicts.",
            )
            .with_remediation(
                "Review abandoned drafts: continue work, close, or convert to issues for tracking.",
            ),
        );
    }

    findings
}

/// Helper: list open pull requests
fn list_open_prs(provider: &GitHubProvider) -> Option<Vec<PullRequest>> {
    let output = std::process::Command::new("gh")
        .args([
            "pr",
            "list",
            "--repo",
            &format!("{}/{}", provider.owner(), provider.name()),
            "--state",
            "open",
            "--json",
            "number,title,updatedAt,isDraft,reviewRequests,reviews",
            "--limit",
            "100",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stale_issue_threshold() {
        assert_eq!(STALE_ISSUE_DAYS, 90);
    }

    #[test]
    fn test_stale_pr_threshold() {
        assert_eq!(STALE_PR_DAYS, 30);
    }

    #[test]
    fn test_abandoned_draft_threshold() {
        assert_eq!(ABANDONED_DRAFT_DAYS, 14);
    }
}
