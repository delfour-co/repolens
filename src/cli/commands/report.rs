//! Report command - Generate an audit report

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;

use super::{ReportArgs, ReportFormat};
use crate::cli::output::{HtmlReport, MarkdownReport, JsonOutput, ReportRenderer};
use crate::config::Config;
use crate::rules::engine::RulesEngine;
use crate::scanner::Scanner;
use crate::exit_codes;

pub async fn execute(args: ReportArgs) -> Result<i32> {
    // Load configuration
    let config = Config::load_or_default()?;

    // Initialize scanner
    let scanner = Scanner::new(PathBuf::from("."));

    // Run the rules engine
    let engine = RulesEngine::new(config);
    let audit_results = engine.run(&scanner).await
        .context("Failed to run audit")?;

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

    std::fs::write(&output_path, &report)
        .context("Failed to write report file")?;

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
