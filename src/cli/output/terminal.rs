//! Terminal output formatting with colors

use anyhow::Result;
use colored::Colorize;

use super::{OutputRenderer, ReportRenderer};
use crate::rules::results::{AuditResults, Finding, Severity};
use crate::actions::plan::ActionPlan;

pub struct TerminalOutput;

impl TerminalOutput {
    pub fn new() -> Self {
        Self
    }

    fn format_header(&self, repo_name: &str, preset: &str) -> String {
        format!(
            r#"
{} v{}

{} {}
{} {}
"#,
            "repolens".cyan().bold(),
            env!("CARGO_PKG_VERSION"),
            "Repository:".dimmed(),
            repo_name.white().bold(),
            "Preset:".dimmed(),
            preset.yellow()
        )
    }

    fn format_findings(&self, results: &AuditResults) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\n{}\n{}\n\n",
            "━".repeat(50).dimmed(),
            "  AUDIT RESULTS".bold()
        ));

        // Critical findings
        let critical: Vec<_> = results.findings_by_severity(Severity::Critical).collect();
        if !critical.is_empty() {
            output.push_str(&format!("{} ({})\\n", "❌ CRITICAL".red().bold(), critical.len()));
            for finding in critical {
                output.push_str(&self.format_finding(finding));
            }
            output.push('\n');
        }

        // Warning findings
        let warnings: Vec<_> = results.findings_by_severity(Severity::Warning).collect();
        if !warnings.is_empty() {
            output.push_str(&format!("{} ({})\\n", "⚠️  WARNING".yellow().bold(), warnings.len()));
            for finding in warnings {
                output.push_str(&self.format_finding(finding));
            }
            output.push('\n');
        }

        // Info findings
        let info: Vec<_> = results.findings_by_severity(Severity::Info).collect();
        if !info.is_empty() {
            output.push_str(&format!("{} ({})\\n", "ℹ️  INFO".blue().bold(), info.len()));
            for finding in info {
                output.push_str(&self.format_finding(finding));
            }
            output.push('\n');
        }

        output
    }

    fn format_finding(&self, finding: &Finding) -> String {
        let mut output = format!(
            "  {} [{}] {}\n",
            "•".dimmed(),
            finding.rule_id.cyan(),
            finding.message
        );

        if let Some(location) = &finding.location {
            output.push_str(&format!("    {} {}\n", "└─".dimmed(), location.dimmed()));
        }

        output
    }

    fn format_actions(&self, plan: &ActionPlan) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\n{}\n{}\n\n",
            "━".repeat(50).dimmed(),
            "  PLANNED ACTIONS".bold()
        ));

        if plan.is_empty() {
            output.push_str(&format!("  {}\n", "No actions required.".green()));
            return output;
        }

        output.push_str("The following changes will be applied:\n\n");

        for action in plan.actions() {
            output.push_str(&format!(
                "  {} [{}] {}\n",
                "+".green(),
                action.category().cyan(),
                action.description()
            ));

            for detail in action.details() {
                output.push_str(&format!("      {} {}\n", "└─".dimmed(), detail.dimmed()));
            }
        }

        output
    }

    fn format_summary(&self, results: &AuditResults) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "\n{}\n{}\n\n",
            "━".repeat(50).dimmed(),
            "  SUMMARY".bold()
        ));

        let critical_count = results.count_by_severity(Severity::Critical);
        let warning_count = results.count_by_severity(Severity::Warning);
        let info_count = results.count_by_severity(Severity::Info);

        output.push_str(&format!(
            "Critical: {} │ Warnings: {} │ Info: {}\n",
            critical_count.to_string().red().bold(),
            warning_count.to_string().yellow().bold(),
            info_count.to_string().blue().bold()
        ));

        if critical_count > 0 {
            output.push_str(&format!(
                "\n{} {} critical issue(s) must be fixed manually.\n",
                "⚠️ ".yellow(),
                critical_count
            ));
        }

        output.push_str(&format!(
            "\nRun '{}' to execute planned actions.\n",
            "repolens apply".cyan()
        ));

        output
    }
}

impl Default for TerminalOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputRenderer for TerminalOutput {
    fn render_plan(&self, results: &AuditResults, plan: &ActionPlan) -> Result<String> {
        let mut output = String::new();

        output.push_str(&self.format_header(&results.repository_name, &results.preset));
        output.push_str(&self.format_findings(results));
        output.push_str(&self.format_actions(plan));
        output.push_str(&self.format_summary(results));

        Ok(output)
    }
}

impl ReportRenderer for TerminalOutput {
    fn render_report(&self, results: &AuditResults) -> Result<String> {
        let mut output = String::new();

        output.push_str(&self.format_header(&results.repository_name, &results.preset));
        output.push_str(&self.format_findings(results));
        output.push_str(&self.format_summary(results));

        Ok(output)
    }
}
