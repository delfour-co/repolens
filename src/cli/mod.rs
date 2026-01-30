//! CLI module - Command line interface definition and handlers

pub mod commands;
pub mod output;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use commands::{ApplyArgs, InitArgs, InstallHooksArgs, PlanArgs, ReportArgs};

/// RepoLens - Audit and prepare repositories for open source or enterprise standards
#[derive(Parser, Debug)]
#[command(name = "repolens")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Increase verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Path to configuration file
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Working directory (defaults to current directory)
    #[arg(short = 'C', long, global = true, value_name = "DIR")]
    pub directory: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new configuration file
    Init(InitArgs),

    /// Analyze repository and show planned actions
    Plan(PlanArgs),

    /// Apply planned changes to the repository
    Apply(ApplyArgs),

    /// Generate an audit report
    Report(ReportArgs),

    /// Install or remove Git hooks (pre-commit, pre-push)
    InstallHooks(InstallHooksArgs),
}
