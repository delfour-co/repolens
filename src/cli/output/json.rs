//! JSON output formatting

use crate::error::RepoLensError;
use serde::Serialize;

use super::{OutputRenderer, ReportRenderer};
use crate::actions::plan::ActionPlan;
use crate::rules::results::AuditResults;

pub struct JsonOutput;

impl JsonOutput {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize)]
struct PlanOutput<'a> {
    version: &'static str,
    repository: &'a str,
    preset: &'a str,
    audit: AuditSummary<'a>,
    actions: Vec<ActionSummary<'a>>,
}

#[derive(Serialize)]
struct AuditSummary<'a> {
    critical_count: usize,
    warning_count: usize,
    info_count: usize,
    findings: &'a [crate::rules::results::Finding],
}

#[derive(Serialize)]
struct ActionSummary<'a> {
    category: &'a str,
    description: &'a str,
    details: &'a [String],
}

impl OutputRenderer for JsonOutput {
    fn render_plan(
        &self,
        results: &AuditResults,
        plan: &ActionPlan,
    ) -> Result<String, RepoLensError> {
        let output = PlanOutput {
            version: env!("CARGO_PKG_VERSION"),
            repository: &results.repository_name,
            preset: &results.preset,
            audit: AuditSummary {
                critical_count: results
                    .count_by_severity(crate::rules::results::Severity::Critical),
                warning_count: results.count_by_severity(crate::rules::results::Severity::Warning),
                info_count: results.count_by_severity(crate::rules::results::Severity::Info),
                findings: results.findings(),
            },
            actions: plan
                .actions()
                .iter()
                .map(|a| ActionSummary {
                    category: a.category(),
                    description: a.description(),
                    details: a.details(),
                })
                .collect(),
        };

        Ok(serde_json::to_string_pretty(&output)?)
    }
}

impl ReportRenderer for JsonOutput {
    fn render_report(&self, results: &AuditResults) -> Result<String, RepoLensError> {
        Ok(serde_json::to_string_pretty(results)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::results::{Finding, Severity};

    fn create_test_results() -> AuditResults {
        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "SEC001",
            "secrets",
            Severity::Critical,
            "Secret exposed",
        ));
        results.add_finding(Finding::new(
            "DOC001",
            "docs",
            Severity::Warning,
            "README missing",
        ));
        results
    }

    #[test]
    fn test_json_output_new() {
        let _output = JsonOutput::new();
        // JsonOutput is a unit struct
    }

    #[test]
    fn test_json_output_default() {
        let _output = JsonOutput;
        // JsonOutput is a unit struct
    }

    #[test]
    fn test_render_plan() {
        use crate::actions::plan::{Action, ActionOperation};

        let output = JsonOutput::new();
        let results = create_test_results();
        let mut plan = ActionPlan::new();
        plan.add(
            Action::new(
                "action1",
                "files",
                "Update gitignore",
                ActionOperation::UpdateGitignore {
                    entries: vec!["*.log".to_string()],
                },
            )
            .with_detail("Adding *.log"),
        );

        let rendered = output.render_plan(&results, &plan).unwrap();
        let json: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(json["repository"], "test-repo");
        assert_eq!(json["preset"], "opensource");
        assert_eq!(json["audit"]["critical_count"], 1);
        assert_eq!(json["audit"]["warning_count"], 1);
        assert!(!json["actions"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_render_plan_empty() {
        let output = JsonOutput::new();
        let results = AuditResults::new("empty-repo", "strict");
        let plan = ActionPlan::new();

        let rendered = output.render_plan(&results, &plan).unwrap();
        let json: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(json["repository"], "empty-repo");
        assert_eq!(json["preset"], "strict");
        assert_eq!(json["audit"]["critical_count"], 0);
        assert!(json["actions"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_render_report() {
        let output = JsonOutput::new();
        let results = create_test_results();

        let rendered = output.render_report(&results).unwrap();
        let json: serde_json::Value = serde_json::from_str(&rendered).unwrap();

        assert_eq!(json["repository_name"], "test-repo");
        assert_eq!(json["preset"], "opensource");
    }
}
