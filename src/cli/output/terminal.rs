//! Terminal output formatting with colors

use crate::error::RepoLensError;
use colored::Colorize;

use super::{OutputRenderer, ReportRenderer};
use crate::actions::plan::ActionPlan;
use crate::rules::results::{AuditResults, Finding, Severity};

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
            output.push_str(&format!(
                "{} ({})\\n",
                "❌ CRITICAL".red().bold(),
                critical.len()
            ));
            for finding in critical {
                output.push_str(&self.format_finding(finding));
            }
            output.push('\n');
        }

        // Warning findings
        let warnings: Vec<_> = results.findings_by_severity(Severity::Warning).collect();
        if !warnings.is_empty() {
            output.push_str(&format!(
                "{} ({})\\n",
                "⚠️  WARNING".yellow().bold(),
                warnings.len()
            ));
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
    fn render_plan(
        &self,
        results: &AuditResults,
        plan: &ActionPlan,
    ) -> Result<String, RepoLensError> {
        let mut output = String::new();

        output.push_str(&self.format_header(&results.repository_name, &results.preset));
        output.push_str(&self.format_findings(results));
        output.push_str(&self.format_actions(plan));
        output.push_str(&self.format_summary(results));

        Ok(output)
    }
}

impl ReportRenderer for TerminalOutput {
    fn render_report(&self, results: &AuditResults) -> Result<String, RepoLensError> {
        let mut output = String::new();

        output.push_str(&self.format_header(&results.repository_name, &results.preset));
        output.push_str(&self.format_findings(results));
        output.push_str(&self.format_summary(results));

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::results::Finding;

    fn create_test_results() -> AuditResults {
        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(
            Finding::new("SEC001", "secrets", Severity::Critical, "Secret exposed")
                .with_location("src/config.rs:42"),
        );
        results.add_finding(Finding::new(
            "DOC001",
            "docs",
            Severity::Warning,
            "README missing",
        ));
        results.add_finding(Finding::new(
            "INFO001",
            "info",
            Severity::Info,
            "Consider adding tests",
        ));
        results
    }

    fn create_empty_results() -> AuditResults {
        AuditResults::new("clean-repo", "opensource")
    }

    #[test]
    fn test_terminal_output_new() {
        let _output = TerminalOutput::new();
        // TerminalOutput is a unit struct, testing construction
    }

    #[test]
    fn test_terminal_output_default() {
        let _output: TerminalOutput = Default::default();
        // Verify Default trait impl works
    }

    #[test]
    fn test_format_header() {
        let output = TerminalOutput::new();
        let header = output.format_header("my-repo", "enterprise");
        assert!(header.contains("my-repo"));
        assert!(header.contains("enterprise"));
    }

    #[test]
    fn test_format_findings_with_all_severities() {
        let output = TerminalOutput::new();
        let results = create_test_results();
        let formatted = output.format_findings(&results);
        assert!(formatted.contains("SEC001"));
        assert!(formatted.contains("DOC001"));
        assert!(formatted.contains("INFO001"));
    }

    #[test]
    fn test_format_findings_empty() {
        let output = TerminalOutput::new();
        let results = create_empty_results();
        let formatted = output.format_findings(&results);
        assert!(formatted.contains("AUDIT RESULTS"));
    }

    #[test]
    fn test_format_finding_with_location() {
        let output = TerminalOutput::new();
        let finding = Finding::new("TEST001", "test", Severity::Warning, "Test message")
            .with_location("src/test.rs:10");
        let formatted = output.format_finding(&finding);
        assert!(formatted.contains("TEST001"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("src/test.rs:10"));
    }

    #[test]
    fn test_format_finding_without_location() {
        let output = TerminalOutput::new();
        let finding = Finding::new("TEST001", "test", Severity::Warning, "Test message");
        let formatted = output.format_finding(&finding);
        assert!(formatted.contains("TEST001"));
        assert!(formatted.contains("Test message"));
    }

    #[test]
    fn test_format_actions_empty() {
        let output = TerminalOutput::new();
        let plan = ActionPlan::new();
        let formatted = output.format_actions(&plan);
        assert!(formatted.contains("No actions required"));
    }

    #[test]
    fn test_format_actions_with_actions() {
        use crate::actions::plan::{Action, ActionOperation};

        let output = TerminalOutput::new();
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
        let formatted = output.format_actions(&plan);
        assert!(formatted.contains("PLANNED ACTIONS"));
        assert!(formatted.contains("Update gitignore"));
        assert!(formatted.contains("Adding *.log"));
    }

    #[test]
    fn test_format_summary() {
        let output = TerminalOutput::new();
        let results = create_test_results();
        let formatted = output.format_summary(&results);
        assert!(formatted.contains("SUMMARY"));
        assert!(formatted.contains("Critical:"));
        assert!(formatted.contains("Warnings:"));
        assert!(formatted.contains("Info:"));
    }

    #[test]
    fn test_format_summary_with_critical() {
        let output = TerminalOutput::new();
        let mut results = AuditResults::new("test-repo", "opensource");
        results.add_finding(Finding::new(
            "SEC001",
            "secrets",
            Severity::Critical,
            "Critical issue",
        ));
        let formatted = output.format_summary(&results);
        assert!(formatted.contains("critical issue(s) must be fixed manually"));
    }

    #[test]
    fn test_render_plan() {
        let output = TerminalOutput::new();
        let results = create_test_results();
        let plan = ActionPlan::new();
        let rendered = output.render_plan(&results, &plan).unwrap();
        assert!(rendered.contains("test-repo"));
        assert!(rendered.contains("SEC001"));
    }

    #[test]
    fn test_render_report() {
        let output = TerminalOutput::new();
        let results = create_test_results();
        let rendered = output.render_report(&results).unwrap();
        assert!(rendered.contains("test-repo"));
        assert!(rendered.contains("SEC001"));
    }
}
