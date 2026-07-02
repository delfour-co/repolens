//! Install-hooks command - Install or remove Git hooks

use colored::Colorize;

use super::InstallHooksArgs;
use crate::config::Config;
use crate::error::{ActionError, RepoLensError};
use crate::exit_codes;
use crate::hooks::{HooksConfig, HooksManager};

pub async fn execute(args: InstallHooksArgs) -> Result<i32, RepoLensError> {
    let root = std::env::current_dir().map_err(|e| {
        RepoLensError::Action(ActionError::ExecutionFailed {
            message: format!("Failed to get current directory: {}", e),
        })
    })?;

    // Load configuration (uses hooks defaults if not specified in .repolens.toml)
    let config = Config::load_or_default()?;

    // Determine which hooks to install based on CLI flags
    let hooks_config = build_hooks_config(&args, &config.hooks);

    let manager = HooksManager::new(&root, hooks_config)?;

    if args.remove {
        println!("{}", "Removing Git hooks...".bold());
        let messages = manager.remove()?;
        for msg in &messages {
            println!("  {} {}", "->".green(), msg);
        }
        println!();
        println!("{}", "Done!".green().bold());
    } else {
        println!("{}", "Installing Git hooks...".bold());
        let messages = manager.install(args.force)?;
        if messages.is_empty() {
            println!(
                "  {} No hooks selected for installation. Use --pre-commit, --pre-push, or --all.",
                "!".yellow()
            );
        } else {
            for msg in &messages {
                println!("  {} {}", "->".green(), msg);
            }
        }
        println!();
        println!("{}", "Done!".green().bold());
    }

    Ok(exit_codes::SUCCESS)
}

/// Build the HooksConfig based on CLI arguments, falling back to config file values
fn build_hooks_config(args: &InstallHooksArgs, file_config: &HooksConfig) -> HooksConfig {
    // If --all is specified, or no specific hook flag is given, use config file defaults
    if args.all || (!args.pre_commit && !args.pre_push) {
        return file_config.clone();
    }

    // Otherwise, only install the specifically requested hooks
    HooksConfig {
        pre_commit: args.pre_commit,
        pre_push: args.pre_push,
        fail_on_warnings: file_config.fail_on_warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_hooks_config_defaults() {
        let args = InstallHooksArgs {
            pre_commit: false,
            pre_push: false,
            all: false,
            remove: false,
            force: false,
        };
        let file_config = HooksConfig::default();
        let result = build_hooks_config(&args, &file_config);
        assert!(result.pre_commit);
        assert!(result.pre_push);
        assert!(!result.fail_on_warnings);
    }

    #[test]
    fn test_build_hooks_config_all_flag() {
        let args = InstallHooksArgs {
            pre_commit: false,
            pre_push: false,
            all: true,
            remove: false,
            force: false,
        };
        let file_config = HooksConfig {
            pre_commit: true,
            pre_push: false,
            fail_on_warnings: true,
        };
        let result = build_hooks_config(&args, &file_config);
        assert!(result.pre_commit);
        assert!(!result.pre_push);
        assert!(result.fail_on_warnings);
    }

    #[test]
    fn test_build_hooks_config_pre_commit_only() {
        let args = InstallHooksArgs {
            pre_commit: true,
            pre_push: false,
            all: false,
            remove: false,
            force: false,
        };
        let file_config = HooksConfig::default();
        let result = build_hooks_config(&args, &file_config);
        assert!(result.pre_commit);
        assert!(!result.pre_push);
    }

    #[test]
    fn test_build_hooks_config_pre_push_only() {
        let args = InstallHooksArgs {
            pre_commit: false,
            pre_push: true,
            all: false,
            remove: false,
            force: false,
        };
        let file_config = HooksConfig::default();
        let result = build_hooks_config(&args, &file_config);
        assert!(!result.pre_commit);
        assert!(result.pre_push);
    }

    #[test]
    fn test_build_hooks_config_both_specific() {
        let args = InstallHooksArgs {
            pre_commit: true,
            pre_push: true,
            all: false,
            remove: false,
            force: false,
        };
        let file_config = HooksConfig::default();
        let result = build_hooks_config(&args, &file_config);
        assert!(result.pre_commit);
        assert!(result.pre_push);
    }

    #[test]
    fn test_build_hooks_config_preserves_fail_on_warnings() {
        let args = InstallHooksArgs {
            pre_commit: true,
            pre_push: false,
            all: false,
            remove: false,
            force: false,
        };
        let file_config = HooksConfig {
            pre_commit: true,
            pre_push: true,
            fail_on_warnings: true,
        };
        let result = build_hooks_config(&args, &file_config);
        assert!(result.fail_on_warnings);
    }
}
