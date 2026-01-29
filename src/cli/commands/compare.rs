//! Compare command - Compare two audit report JSON files

use colored::Colorize;
use std::path::PathBuf;

use super::CompareArgs;
use crate::compare::{compare_results, format_json, format_markdown, format_terminal};
use crate::error::RepoLensError;
use crate::exit_codes;
use crate::rules::results::AuditResults;

/// Load an AuditResults from a JSON file
fn load_report(path: &PathBuf) -> Result<AuditResults, RepoLensError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        RepoLensError::Action(crate::error::ActionError::ExecutionFailed {
            message: format!("Failed to read report file '{}': {}", path.display(), e),
        })
    })?;
    let results: AuditResults = serde_json::from_str(&content)?;
    Ok(results)
}

pub async fn execute(args: CompareArgs) -> Result<i32, RepoLensError> {
    // Load base and head reports
    let base_results = load_report(&args.base_file)?;
    let head_results = load_report(&args.head_file)?;

    let base_label = args
        .base_file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "base".to_string());
    let head_label = args
        .head_file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "head".to_string());

    let report = compare_results(&base_results, &head_results, &base_label, &head_label);

    // Format output
    let output_str = match args.format {
        super::CompareFormat::Terminal => format_terminal(&report),
        super::CompareFormat::Json => format_json(&report).map_err(|e| {
            RepoLensError::Action(crate::error::ActionError::ExecutionFailed {
                message: format!("Failed to serialize compare report: {}", e),
            })
        })?,
        super::CompareFormat::Markdown => format_markdown(&report),
    };

    // Write output
    if let Some(output_path) = &args.output {
        std::fs::write(output_path, &output_str).map_err(|e| {
            RepoLensError::Action(crate::error::ActionError::FileWrite {
                path: output_path.display().to_string(),
                source: e,
            })
        })?;
        println!(
            "{} Comparison report written to: {}",
            "Success:".green().bold(),
            output_path.display().to_string().cyan()
        );
    } else {
        print!("{}", output_str);
    }

    // Determine exit code
    if args.fail_on_regression && report.has_regressions() {
        eprintln!(
            "{} {} new issue(s) detected (regression).",
            "Error:".red().bold(),
            report.added_findings.len()
        );
        Ok(exit_codes::CRITICAL_ISSUES)
    } else {
        Ok(exit_codes::SUCCESS)
    }
}
