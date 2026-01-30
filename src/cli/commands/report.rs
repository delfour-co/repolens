//! Report command - Generate an audit report

use colored::Colorize;
use std::path::PathBuf;

use super::{ReportArgs, ReportFormat};
use crate::cli::output::{HtmlReport, JsonOutput, MarkdownReport, ReportRenderer};
use crate::config::Config;
use crate::error::RepoLensError;
use crate::exit_codes;
use crate::rules::engine::RulesEngine;
use crate::scanner::Scanner;

pub async fn execute(args: ReportArgs) -> Result<i32, RepoLensError> {
    // Load configuration
    let config = Config::load_or_default()?;

    // Initialize scanner
    let scanner = Scanner::new(PathBuf::from("."));

    // Run the rules engine
    let engine = RulesEngine::new(config);
    let audit_results = engine.run(&scanner).await?;

    // Generate report
    let renderer: Box<dyn ReportRenderer> = match args.format {
        ReportFormat::Html => Box::new(HtmlReport::new(args.detailed)),
        ReportFormat::Markdown => Box::new(MarkdownReport::new(args.detailed)),
        ReportFormat::Json => Box::new(JsonOutput::new()),
    };

    let report = renderer.render_report(&audit_results)?;

    // Write output
    let output_path = args.output.unwrap_or_else(|| {
        let extension = match args.format {
            ReportFormat::Html => "html",
            ReportFormat::Markdown => "md",
            ReportFormat::Json => "json",
        };
        PathBuf::from(format!("repolens-report.{extension}"))
    });

    std::fs::write(&output_path, &report).map_err(|e| {
        RepoLensError::Action(crate::error::ActionError::FileWrite {
            path: output_path.display().to_string(),
            source: e,
        })
    })?;

    println!(
        "{} Report written to: {}",
        "Success:".green().bold(),
        output_path.display().to_string().cyan()
    );

    // Return exit code based on findings
    let exit_code = if audit_results.has_critical() {
        exit_codes::CRITICAL_ISSUES
    } else if audit_results.has_warnings() {
        exit_codes::WARNINGS
    } else {
        exit_codes::SUCCESS
    };

    Ok(exit_code)
}
