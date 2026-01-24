//! JSON output formatting

use anyhow::Result;
use serde::Serialize;

use super::{OutputRenderer, ReportRenderer};
use crate::rules::results::AuditResults;
use crate::actions::plan::ActionPlan;

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
    fn render_plan(&self, results: &AuditResults, plan: &ActionPlan) -> Result<String> {
        let output = PlanOutput {
            version: env!("CARGO_PKG_VERSION"),
            repository: &results.repository_name,
            preset: &results.preset,
            audit: AuditSummary {
                critical_count: results.count_by_severity(crate::rules::results::Severity::Critical),
                warning_count: results.count_by_severity(crate::rules::results::Severity::Warning),
                info_count: results.count_by_severity(crate::rules::results::Severity::Info),
                findings: results.findings(),
            },
            actions: plan.actions()
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
    fn render_report(&self, results: &AuditResults) -> Result<String> {
        Ok(serde_json::to_string_pretty(results)?)
    }
}
