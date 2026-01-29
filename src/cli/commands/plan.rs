//! Plan command - Analyze repository and show planned actions
//!
//! This module implements the `plan` command which analyzes a repository
//! and generates an action plan to fix detected issues.

use std::path::PathBuf;

use super::{OutputFormat, PlanArgs};
use crate::actions::planner::ActionPlanner;
use crate::cache::{delete_cache_directory, AuditCache};
use crate::cli::output::{JsonOutput, OutputRenderer, SarifOutput, TerminalOutput};
use crate::config::Config;
use crate::error::RepoLensError;
use crate::exit_codes;
use crate::rules::engine::RulesEngine;
use crate::scanner::Scanner;
use colored::Colorize;

/// Execute the plan command
///
/// Analyzes the repository, runs audit rules, generates an action plan,
/// and outputs the results in the requested format.
///
/// # Arguments
///
/// * `args` - Command line arguments for the plan command
///
/// # Returns
///
/// An exit code: 0 for success, 1 for critical issues, 2 for warnings
///
/// # Errors
///
/// Returns an error if the audit or plan generation fails
pub async fn execute(args: PlanArgs) -> Result<i32, RepoLensError> {
    eprintln!("{}", "Chargement de la configuration...".dimmed());
    // Load configuration
    let mut config = Config::load_or_default()?;

    // Handle cache directory override from CLI
    if let Some(ref cache_dir) = args.cache_dir {
        config.cache.directory = cache_dir.display().to_string();
    }

    // Disable cache if --no-cache is specified
    if args.no_cache {
        config.cache.enabled = false;
        eprintln!("{}", "Cache disabled.".dimmed());
    }

    // Clear cache if --clear-cache is specified
    let project_root = PathBuf::from(".");
    if args.clear_cache {
        eprintln!("{}", "Clearing cache...".dimmed());
        if let Err(e) = delete_cache_directory(&project_root, &config.cache) {
            eprintln!("{} Failed to clear cache: {}", "Warning:".yellow(), e);
        } else {
            eprintln!("{} {}", "✓".green(), "Cache cleared.".green());
        }
    }

    // Load or create cache
    let cache = if config.cache.enabled {
        let cache = AuditCache::load(&project_root, config.cache.clone());
        let stats = cache.stats();
        if stats.total_entries > 0 {
            eprintln!(
                "{} {} {}",
                "Cache:".dimmed(),
                stats.total_entries.to_string().cyan(),
                "entries loaded.".dimmed()
            );
        }
        Some(cache)
    } else {
        None
    };

    eprintln!("{}", "Analyse du dépôt...".dimmed());
    // Initialize scanner
    let scanner = Scanner::new(PathBuf::from("."));

    // Run the rules engine
    let mut engine = RulesEngine::new(config.clone());

    // Set cache in the engine
    if let Some(c) = cache {
        engine.set_cache(c);
    }

    // Apply filters if specified
    if let Some(only) = &args.only {
        engine.set_only_categories(only.clone());
    }
    if let Some(skip) = &args.skip {
        engine.set_skip_categories(skip.clone());
    }

    // Set up progress callback
    engine.set_progress_callback(Box::new(|category_name, current, total| {
        eprintln!(
            "  {} {} ({}/{})...",
            "→".dimmed(),
            category_name.cyan(),
            current,
            total
        );
    }));

    // Execute audit
    eprintln!("{}", "Exécution de l'audit...".dimmed());
    let audit_results = engine.run(&scanner).await?;

    // Save cache if enabled
    if let Some(cache) = engine.take_cache() {
        if let Err(e) = cache.save() {
            eprintln!("{} Failed to save cache: {}", "Warning:".yellow(), e);
        }
    }

    eprintln!("{} {}", "✓".green(), "Audit terminé.".green());

    eprintln!("{}", "Génération du plan d'action...".dimmed());
    // Generate action plan
    let planner = ActionPlanner::new(config);
    let action_plan = planner.create_plan(&audit_results).await?;

    eprintln!("{}", "Génération du rapport...".dimmed());
    // Render output
    let output: Box<dyn OutputRenderer> = match args.format {
        OutputFormat::Terminal => Box::new(TerminalOutput::new()),
        OutputFormat::Json => Box::new(JsonOutput::new()),
        OutputFormat::Sarif => Box::new(SarifOutput::new()),
    };

    let rendered = output.render_plan(&audit_results, &action_plan)?;

    // Write output
    if let Some(output_path) = args.output {
        std::fs::write(&output_path, &rendered).map_err(|e| {
            RepoLensError::Action(crate::error::ActionError::FileWrite {
                path: output_path.display().to_string(),
                source: e,
            })
        })?;
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
