//! Output formatting module for CLI

mod terminal;
mod json;
mod sarif;
mod markdown;
mod html;

pub use terminal::TerminalOutput;
pub use json::JsonOutput;
pub use sarif::SarifOutput;
pub use markdown::MarkdownReport;
pub use html::HtmlReport;

use anyhow::Result;
use crate::rules::results::AuditResults;
use crate::actions::plan::ActionPlan;

/// Trait for rendering plan output
pub trait OutputRenderer {
    fn render_plan(&self, results: &AuditResults, plan: &ActionPlan) -> Result<String>;
}

/// Trait for rendering report output
pub trait ReportRenderer {
    fn render_report(&self, results: &AuditResults) -> Result<String>;
}
