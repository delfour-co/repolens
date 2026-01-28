//! Apply command - Apply planned changes to the repository

use colored::Colorize;
use dialoguer::Confirm;
use std::path::{Path, PathBuf};

use super::ApplyArgs;
use crate::actions::executor::ActionExecutor;
use crate::actions::git;
use crate::actions::planner::ActionPlanner;
use crate::config::Config;
use crate::error::RepoLensError;
use crate::exit_codes;
use crate::providers::github::GitHubProvider;
use crate::rules::engine::RulesEngine;
use crate::scanner::Scanner;

pub async fn execute(args: ApplyArgs) -> Result<i32, RepoLensError> {
    // Load configuration
    let config = Config::load_or_default()?;

    // Initialize scanner
    let scanner = Scanner::new(PathBuf::from("."));

    // Run the rules engine to get current state
    let engine = RulesEngine::new(config.clone());
    let audit_results = engine.run(&scanner).await?;

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
            .interact()
            .map_err(|e| {
                RepoLensError::Action(crate::error::ActionError::ExecutionFailed {
                    message: format!("Failed to get user input: {}", e),
                })
            })?;

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
            println!(
                "  {} {} - {}",
                "✗".red(),
                result.action_name,
                result.error.as_deref().unwrap_or("Unknown error")
            );
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

    // Handle git operations and PR creation if there were successful file changes
    if success_count > 0 {
        let repo_root = PathBuf::from(".");
        let should_create_pr = if args.no_pr {
            false
        } else {
            args.create_pr
                .unwrap_or_else(|| git::is_git_repository(&repo_root))
        };

        if should_create_pr && git::is_git_repository(&repo_root) {
            if let Err(e) = handle_git_operations(&repo_root, &action_plan, &results).await {
                eprintln!(
                    "{} {}",
                    "⚠".yellow(),
                    format!("Failed to create PR: {}", e).yellow()
                );
                // Don't fail the whole command if PR creation fails
            }
        }
    }

    if error_count > 0 {
        Ok(exit_codes::ERROR)
    } else {
        Ok(exit_codes::SUCCESS)
    }
}

/// Handle git operations: create branch, commit, push, and create PR
async fn handle_git_operations(
    repo_root: &Path,
    action_plan: &crate::actions::plan::ActionPlan,
    results: &[crate::actions::executor::ActionResult],
) -> Result<(), RepoLensError> {
    use crate::actions::plan::ActionOperation;

    // Check if there are any file-related changes by checking the action plan
    let has_file_changes = action_plan.actions().iter().any(|action| {
        matches!(
            action.operation(),
            ActionOperation::CreateFile { .. } | ActionOperation::UpdateGitignore { .. }
        )
    });

    // Also check if any file-related actions succeeded
    let has_successful_file_changes = results.iter().any(|r| {
        if !r.success {
            return false;
        }
        // Match successful file operations by checking action names
        r.action_name.contains("file")
            || r.action_name.contains("gitignore")
            || r.action_name.contains("Create")
            || r.action_name.contains("Update")
    });

    if !has_file_changes && !has_successful_file_changes {
        // Only file changes trigger PR creation
        return Ok(());
    }

    // Check if there are actual changes to commit
    if !git::has_changes(repo_root)? {
        println!(
            "{}",
            "No file changes detected, skipping PR creation.".dimmed()
        );
        return Ok(());
    }

    println!();
    println!(
        "{}",
        "Création de la branche et préparation de la PR...".dimmed()
    );

    // Create new branch
    let branch_name = git::create_branch(repo_root)?;
    println!("  {} Branche créée: {}", "✓".green(), branch_name.cyan());

    // Stage all changes
    git::stage_all_changes(repo_root)?;
    println!("  {} Changements stagés", "✓".green());

    // Create commit message
    let commit_message = format!(
        "chore: apply RepoLens fixes\n\n{}\n\nActions appliquées:\n{}",
        "Ce commit contient les corrections automatiques appliquées par RepoLens.",
        action_plan
            .actions()
            .iter()
            .map(|a| format!("- {}", a.description()))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Create commit
    git::create_commit(repo_root, &commit_message)?;
    println!("  {} Commit créé", "✓".green());

    // Push branch
    git::push_branch(repo_root, &branch_name)?;
    println!("  {} Branche poussée vers origin", "✓".green());

    // Create PR - check if GitHub CLI is available first
    if !GitHubProvider::is_available() {
        println!(
            "{} {}",
            "⚠".yellow(),
            "GitHub CLI non disponible, PR non créée. Les changements sont dans la branche locale."
                .yellow()
        );
        return Ok(());
    }

    let github_provider = match GitHubProvider::new() {
        Ok(provider) => provider,
        Err(e) => {
            println!(
                "{} {}",
                "⚠".yellow(),
                format!(
                    "Impossible de créer la PR: {}. Les changements sont dans la branche locale.",
                    e
                )
                .yellow()
            );
            return Ok(());
        }
    };

    let default_branch = git::get_default_branch(repo_root).unwrap_or_else(|| "main".to_string());

    let pr_title = format!("RepoLens: Corrections automatiques ({})", {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M");
        timestamp
    });

    let pr_body = format!(
        "# Corrections automatiques RepoLens\n\n\
        Cette PR contient les corrections automatiques appliquées par RepoLens.\n\n\
        ## Actions appliquées\n\n\
        {}\n\n\
        ## Détails\n\n\
        {}",
        action_plan
            .actions()
            .iter()
            .map(|a| format!("- **{}**: {}", a.category(), a.description()))
            .collect::<Vec<_>>()
            .join("\n"),
        results
            .iter()
            .filter(|r| r.success)
            .map(|r| format!("- ✓ {}", r.action_name))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let pr_url = github_provider.create_pull_request(
        &pr_title,
        &pr_body,
        &branch_name,
        Some(&default_branch),
    )?;

    println!();
    println!(
        "{} {}",
        "✓".green(),
        format!("Pull Request créée: {}", pr_url.cyan()).green()
    );

    Ok(())
}
