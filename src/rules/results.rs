//! Audit results structures

use serde::{Deserialize, Serialize};

/// Severity levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Critical issues that must be resolved
    Critical,
    /// Warnings that should be addressed
    Warning,
    /// Informational suggestions
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

/// A single audit finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique rule identifier (e.g., "SEC001")
    pub rule_id: String,

    /// Category of the rule (e.g., "secrets", "docs")
    pub category: String,

    /// Severity of the finding
    pub severity: Severity,

    /// Short message describing the finding
    pub message: String,

    /// Optional file location (e.g., "src/config.ts:42")
    pub location: Option<String>,

    /// Detailed description of the issue
    pub description: Option<String>,

    /// Suggested remediation steps
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

/// Collection of audit results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResults {
    /// Repository name
    pub repository_name: String,

    /// Preset used for the audit
    pub preset: String,

    /// List of findings
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
}
