//! Apply command - Apply planned changes to the repository

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use std::path::PathBuf;

use super::ApplyArgs;
use crate::config::Config;
use crate::rules::engine::RulesEngine;
use crate::scanner::Scanner;
use crate::actions::planner::ActionPlanner;
use crate::actions::executor::ActionExecutor;
use crate::exit_codes;

pub async fn execute(args: ApplyArgs) -> Result<i32> {
    // Load configuration
    let config = Config::load_or_default()?;

    // Initialize scanner
    let scanner = Scanner::new(PathBuf::from("."));

    // Run the rules engine to get current state
    let engine = RulesEngine::new(config.clone());
    let audit_results = engine.run(&scanner).await
        .context("Failed to run audit")?;

    // Generate action plan
    let planner = ActionPlanner::new(config.clone());
    let mut action_plan = planner.create_plan(&audit_results);

    // Apply filters if specified
    if let Some(only) = &args.only {
        action_plan.filter_only(only);
    }
    if let Some(skip) = &args.skip {
        action_plan.filter_skip(skip);
    }

    // Check if there are any actions to perform
    if action_plan.is_empty() {
        println!("{}", "No actions to perform.".green());
        return Ok(exit_codes::SUCCESS);
    }

    // Display plan summary
    println!("{}", "Planned actions:".bold());
    println!();
    for action in action_plan.actions() {
        println!("  {} {}", "+".green(), action.description());
    }
    println!();

    // Dry run mode
    if args.dry_run {
        println!("{}", "Dry run mode - no changes made.".yellow());
        return Ok(exit_codes::SUCCESS);
    }

    // Confirm execution
    if !args.yes {
        let confirm = Confirm::new()
            .with_prompt("Apply these changes?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("{}", "Aborted.".yellow());
            return Ok(exit_codes::SUCCESS);
        }
    }

    // Execute actions
    let executor = ActionExecutor::new(config);
    let results = executor.execute(&action_plan).await?;

    // Display results
    println!();
    let mut success_count = 0;
    let mut error_count = 0;

    for result in &results {
        if result.success {
            println!("  {} {}", "✓".green(), result.action_name);
            success_count += 1;
        } else {
            println!("  {} {} - {}", "✗".red(), result.action_name, result.error.as_deref().unwrap_or("Unknown error"));
            error_count += 1;
        }
    }

    println!();
    println!(
        "{}: {} succeeded, {} failed",
        "Summary".bold(),
        success_count.to_string().green(),
        error_count.to_string().red()
    );

    if error_count > 0 {
        Ok(exit_codes::ERROR)
    } else {
        Ok(exit_codes::SUCCESS)
    }
}
