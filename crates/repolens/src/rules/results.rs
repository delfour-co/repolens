//! # Audit Results Structures
//!
//! This module defines the data structures for representing audit findings
//! and results.
//!
//! ## Overview
//!
//! - [`Severity`] - Finding severity levels (Critical, Warning, Info)
//! - [`Finding`] - Individual audit finding with location and remediation
//! - [`AuditResults`] - Collection of findings from an audit run
//!
//! ## Examples
//!
//! ### Creating Findings
//!
//! ```rust
//! use repolens::rules::{Finding, Severity};
//!
//! let finding = Finding::new("SEC001", "secrets", Severity::Critical, "API key detected")
//!     .with_location("src/config.rs:42")
//!     .with_description("A hardcoded API key was found")
//!     .with_remediation("Use environment variables instead");
//! ```
//!
//! ### Working with Audit Results
//!
//! ```rust
//! use repolens::rules::results::{AuditResults, Finding, Severity};
//!
//! let mut results = AuditResults::new("my-repo", "opensource");
//!
//! results.add_finding(Finding::new("FILE001", "files", Severity::Warning, "README missing"));
//!
//! // Query results
//! println!("Has critical issues: {}", results.has_critical());
//! println!("Warning count: {}", results.count_by_severity(Severity::Warning));
//! ```

use serde::{Deserialize, Serialize};

/// Severity levels for audit findings.
///
/// Severity determines the urgency and importance of a finding:
///
/// - **Critical** - Must be resolved before release (e.g., exposed secrets)
/// - **Warning** - Should be addressed (e.g., missing documentation)
/// - **Info** - Suggestions for improvement (e.g., best practices)
///
/// # Examples
///
/// ```rust
/// use repolens::rules::Severity;
///
/// let severity = Severity::Critical;
///
/// // Parse from string
/// let parsed = Severity::from_string("warning");
/// assert_eq!(parsed, Some(Severity::Warning));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Critical issues that must be resolved immediately.
    /// Examples: exposed secrets, security vulnerabilities.
    Critical,
    /// Warnings that should be addressed before release.
    /// Examples: missing README, incomplete documentation.
    Warning,
    /// Informational suggestions for improvement.
    /// Examples: best practice recommendations, style suggestions.
    Info,
}

impl Severity {
    #[allow(dead_code)]
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "critical" | "error" => Some(Self::Critical),
            "warning" | "warn" => Some(Self::Warning),
            "info" | "information" | "note" => Some(Self::Info),
            _ => None,
        }
    }
}

/// A single audit finding representing an issue detected in the repository.
///
/// Findings are created by rule categories during the audit process and
/// contain all information needed to understand and resolve the issue.
///
/// # Examples
///
/// ```rust
/// use repolens::rules::{Finding, Severity};
///
/// // Create a basic finding
/// let finding = Finding::new(
///     "SEC001",
///     "secrets",
///     Severity::Critical,
///     "Exposed API key detected"
/// );
///
/// // Create a finding with full details
/// let detailed = Finding::new("DOC001", "docs", Severity::Warning, "README is missing")
///     .with_location(".")
///     .with_description("A README.md file helps users understand your project")
///     .with_remediation("Create a README.md with project description and usage");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique rule identifier (e.g., "SEC001", "DOC001").
    /// Used to reference specific rules in configuration.
    pub rule_id: String,

    /// Category of the rule (e.g., "secrets", "docs", "security").
    /// Corresponds to the rule category that generated this finding.
    pub category: String,

    /// Severity of the finding.
    pub severity: Severity,

    /// Short message describing the finding.
    /// Should be concise but informative.
    pub message: String,

    /// Optional file location where the issue was found.
    /// Format: "path/to/file:line" or just "path/to/file".
    pub location: Option<String>,

    /// Detailed description of the issue.
    /// Provides context about why this is a problem.
    pub description: Option<String>,

    /// Suggested remediation steps.
    /// Provides actionable guidance for fixing the issue.
    pub remediation: Option<String>,
}

impl Finding {
    /// Create a new finding
    pub fn new(
        rule_id: impl Into<String>,
        category: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            category: category.into(),
            severity,
            message: message.into(),
            location: None,
            description: None,
            remediation: None,
        }
    }

    /// Set the location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the remediation
    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }
}

/// Collection of audit results from a complete audit run.
///
/// `AuditResults` aggregates all findings from all rule categories
/// and provides methods for querying and filtering results.
///
/// # Examples
///
/// ```rust
/// use repolens::rules::results::{AuditResults, Finding, Severity};
///
/// // Create new results
/// let mut results = AuditResults::new("my-repo", "opensource");
///
/// // Add findings
/// results.add_finding(Finding::new("SEC001", "secrets", Severity::Critical, "Secret found"));
/// results.add_findings(vec![
///     Finding::new("DOC001", "docs", Severity::Warning, "README missing"),
///     Finding::new("DOC002", "docs", Severity::Info, "Consider adding examples"),
/// ]);
///
/// // Query results
/// assert_eq!(results.count_by_severity(Severity::Critical), 1);
/// assert!(results.has_critical());
///
/// // Filter by category
/// let doc_findings: Vec<_> = results.findings_by_category("docs").collect();
/// assert_eq!(doc_findings.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResults {
    /// Name of the repository that was audited.
    pub repository_name: String,

    /// Preset used for the audit (opensource, enterprise, strict).
    pub preset: String,

    /// List of all findings from the audit.
    findings: Vec<Finding>,
}

impl AuditResults {
    /// Create new audit results
    pub fn new(repository_name: impl Into<String>, preset: impl Into<String>) -> Self {
        Self {
            repository_name: repository_name.into(),
            preset: preset.into(),
            findings: Vec::new(),
        }
    }

    /// Add a finding
    #[allow(dead_code)]
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// Add multiple findings
    pub fn add_findings(&mut self, findings: impl IntoIterator<Item = Finding>) {
        self.findings.extend(findings);
    }

    /// Get all findings
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// Get findings by severity
    pub fn findings_by_severity(&self, severity: Severity) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(move |f| f.severity == severity)
    }

    /// Get findings by category
    pub fn findings_by_category<'a>(
        &'a self,
        category: &'a str,
    ) -> impl Iterator<Item = &'a Finding> {
        self.findings.iter().filter(move |f| f.category == category)
    }

    /// Count findings by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .count()
    }

    /// Check if there are any critical findings
    pub fn has_critical(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == Severity::Critical)
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == Severity::Warning)
    }

    /// Get total number of findings
    #[allow(dead_code)]
    pub fn total_count(&self) -> usize {
        self.findings.len()
    }

    /// Check if there are no findings
    #[allow(dead_code)]
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_builder() {
        let finding = Finding::new("SEC001", "secrets", Severity::Critical, "Secret detected")
            .with_location("src/config.ts:42")
            .with_description("A hardcoded secret was found")
            .with_remediation("Move the secret to environment variables");

        assert_eq!(finding.rule_id, "SEC001");
        assert_eq!(finding.location, Some("src/config.ts:42".to_string()));
    }

    #[test]
    fn test_audit_results() {
        let mut results = AuditResults::new("test-repo", "opensource");

        results.add_finding(Finding::new(
            "SEC001",
            "secrets",
            Severity::Critical,
            "Secret found",
        ));
        results.add_finding(Finding::new(
            "DOC001",
            "docs",
            Severity::Warning,
            "README missing",
        ));

        assert_eq!(results.total_count(), 2);
        assert!(results.has_critical());
        assert!(results.has_warnings());
        assert_eq!(results.count_by_severity(Severity::Critical), 1);
    }

    #[test]
    fn test_severity_from_string() {
        assert_eq!(Severity::from_string("critical"), Some(Severity::Critical));
        assert_eq!(Severity::from_string("error"), Some(Severity::Critical));
        assert_eq!(Severity::from_string("CRITICAL"), Some(Severity::Critical));

        assert_eq!(Severity::from_string("warning"), Some(Severity::Warning));
        assert_eq!(Severity::from_string("warn"), Some(Severity::Warning));
        assert_eq!(Severity::from_string("WARNING"), Some(Severity::Warning));

        assert_eq!(Severity::from_string("info"), Some(Severity::Info));
        assert_eq!(Severity::from_string("information"), Some(Severity::Info));
        assert_eq!(Severity::from_string("note"), Some(Severity::Info));
        assert_eq!(Severity::from_string("INFO"), Some(Severity::Info));

        assert_eq!(Severity::from_string("unknown"), None);
        assert_eq!(Severity::from_string(""), None);
    }

    #[test]
    fn test_finding_new() {
        let finding = Finding::new("TEST001", "test", Severity::Info, "Test message");

        assert_eq!(finding.rule_id, "TEST001");
        assert_eq!(finding.category, "test");
        assert_eq!(finding.severity, Severity::Info);
        assert_eq!(finding.message, "Test message");
        assert!(finding.location.is_none());
        assert!(finding.description.is_none());
        assert!(finding.remediation.is_none());
    }

    #[test]
    fn test_finding_with_all_fields() {
        let finding = Finding::new("TEST001", "test", Severity::Warning, "Test")
            .with_location("file.rs:10")
            .with_description("Test description")
            .with_remediation("Test remediation");

        assert_eq!(finding.location, Some("file.rs:10".to_string()));
        assert_eq!(finding.description, Some("Test description".to_string()));
        assert_eq!(finding.remediation, Some("Test remediation".to_string()));
    }

    #[test]
    fn test_audit_results_add_findings() {
        let mut results = AuditResults::new("test-repo", "opensource");

        let findings = vec![
            Finding::new("TEST001", "test", Severity::Info, "Info 1"),
            Finding::new("TEST002", "test", Severity::Warning, "Warning 1"),
        ];

        results.add_findings(findings);
        assert_eq!(results.total_count(), 2);
    }

    #[test]
    fn test_audit_results_findings_by_category() {
        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "SEC001",
            "secrets",
            Severity::Critical,
            "Secret",
        ));
        results.add_finding(Finding::new("DOC001", "docs", Severity::Warning, "Doc"));
        results.add_finding(Finding::new(
            "SEC002",
            "secrets",
            Severity::Warning,
            "Another secret",
        ));

        let secrets: Vec<_> = results.findings_by_category("secrets").collect();
        assert_eq!(secrets.len(), 2);

        let docs: Vec<_> = results.findings_by_category("docs").collect();
        assert_eq!(docs.len(), 1);

        let other: Vec<_> = results.findings_by_category("other").collect();
        assert_eq!(other.len(), 0);
    }

    #[test]
    fn test_audit_results_is_clean() {
        let results = AuditResults::new("test-repo", "opensource");
        assert!(results.is_clean());

        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new("TEST", "test", Severity::Info, "Test"));
        assert!(!results.is_clean());
    }

    #[test]
    fn test_audit_results_no_critical_or_warnings() {
        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new("INFO", "info", Severity::Info, "Info only"));

        assert!(!results.has_critical());
        assert!(!results.has_warnings());
    }

    #[test]
    fn test_audit_results_count_by_severity() {
        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new("C1", "test", Severity::Critical, "C1"));
        results.add_finding(Finding::new("C2", "test", Severity::Critical, "C2"));
        results.add_finding(Finding::new("W1", "test", Severity::Warning, "W1"));
        results.add_finding(Finding::new("I1", "test", Severity::Info, "I1"));
        results.add_finding(Finding::new("I2", "test", Severity::Info, "I2"));
        results.add_finding(Finding::new("I3", "test", Severity::Info, "I3"));

        assert_eq!(results.count_by_severity(Severity::Critical), 2);
        assert_eq!(results.count_by_severity(Severity::Warning), 1);
        assert_eq!(results.count_by_severity(Severity::Info), 3);
    }
}
