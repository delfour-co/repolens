//! RepoLens - A CLI tool to audit and prepare repositories for open source or enterprise standards
//!
//! This is the main entry point for the CLI application.

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod actions;
mod cache;
mod cli;
mod config;
mod error;
mod providers;
mod rules;
mod scanner;
mod utils;

use error::RepoLensError;

/// Exit codes for the CLI
pub mod exit_codes {
    /// Success - no issues found
    pub const SUCCESS: i32 = 0;
    /// Critical issues found that block release
    pub const CRITICAL_ISSUES: i32 = 1;
    /// Warnings found but not blocking
    pub const WARNINGS: i32 = 2;
    /// Configuration or runtime error
    pub const ERROR: i32 = 3;
}

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), RepoLensError> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Setup logging based on verbosity
    setup_logging(cli.verbose);

    // Execute the appropriate command
    let result = match cli.command {
        Commands::Init(args) => cli::commands::init::execute(args).await,
        Commands::Plan(args) => cli::commands::plan::execute(args).await,
        Commands::Apply(args) => cli::commands::apply::execute(args).await,
        Commands::Report(args) => cli::commands::report::execute(args).await,
        Commands::Schema(args) => cli::commands::schema::execute(args).await,
    };

    // Handle exit codes for CI integration
    match result {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn setup_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)))
        .init();
}
