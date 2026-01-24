//! Plan command - Analyze repository and show planned actions

use anyhow::{Context, Result};
use std::path::PathBuf;

use super::{OutputFormat, PlanArgs};
use crate::cli::output::{TerminalOutput, JsonOutput, SarifOutput, OutputRenderer};
use crate::config::Config;
use crate::rules::engine::RulesEngine;
use crate::scanner::Scanner;
use crate::actions::planner::ActionPlanner;
use crate::exit_codes;

pub async fn execute(args: PlanArgs) -> Result<i32> {
    // Load configuration
    let config = Config::load_or_default()?;

    // Initialize scanner
    let scanner = Scanner::new(PathBuf::from("."));

    // Run the rules engine
    let mut engine = RulesEngine::new(config.clone());

    // Apply filters if specified
    if let Some(only) = &args.only {
        engine.set_only_categories(only.clone());
    }
    if let Some(skip) = &args.skip {
        engine.set_skip_categories(skip.clone());
    }

    // Execute audit
    let audit_results = engine.run(&scanner).await
        .context("Failed to run audit")?;

    // Generate action plan
    let planner = ActionPlanner::new(config);
    let action_plan = planner.create_plan(&audit_results);

    // Render output
    let output: Box<dyn OutputRenderer> = match args.format {
        OutputFormat::Terminal => Box::new(TerminalOutput::new()),
        OutputFormat::Json => Box::new(JsonOutput::new()),
        OutputFormat::Sarif => Box::new(SarifOutput::new()),
    };

    let rendered = output.render_plan(&audit_results, &action_plan)?;

    // Write output
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &rendered)
            .context("Failed to write output file")?;
        eprintln!("Plan written to: {}", output_path.display());
    } else {
        println!("{rendered}");
    }

    // Determine exit code based on findings
    let exit_code = if audit_results.has_critical() {
        exit_codes::CRITICAL_ISSUES
    } else if audit_results.has_warnings() {
        exit_codes::WARNINGS
    } else {
        exit_codes::SUCCESS
    };

    Ok(exit_code)
}
