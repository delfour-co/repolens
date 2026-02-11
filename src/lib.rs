//! # RepoLens Library
//!
//! RepoLens is a comprehensive CLI tool for auditing GitHub repositories against
//! best practices, security standards, and compliance requirements.
//!
//! This crate provides the core functionality for:
//! - Scanning repositories for secrets, security issues, and configuration problems
//! - Checking compliance with open-source or enterprise standards
//! - Generating detailed audit reports in multiple formats
//! - Planning and applying remediation actions
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use repolens::{config, scanner::Scanner, rules::engine::RulesEngine};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), repolens::RepoLensError> {
//! // Load configuration
//! let config = config::Config::load_or_default()?;
//!
//! // Create a scanner for the repository
//! let scanner = Scanner::new(PathBuf::from("."));
//!
//! // Create and run the rules engine
//! let engine = RulesEngine::new(config);
//! let results = engine.run(&scanner).await?;
//!
//! // Check results
//! println!("Found {} issues", results.findings().len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The library is organized into the following modules:
//!
//! - [`config`] - Configuration loading, presets, and rule settings
//! - [`rules`] - Audit rules engine and finding categories
//! - [`scanner`] - File system and git repository scanning
//! - [`actions`] - Action planning and execution for remediation
//! - [`providers`] - External service integrations (GitHub API)
//! - [`cache`] - Audit results caching for performance
//! - [`compare`] - Report comparison and diff generation
//! - [`hooks`] - Git hooks management
//! - [`error`] - Error types and handling
//!
//! ## Presets
//!
//! RepoLens supports three built-in presets:
//!
//! - **opensource** - Standard open-source project requirements
//! - **enterprise** - Enterprise-grade security and compliance
//! - **strict** - Maximum security and documentation requirements
//!
//! ## Rule Categories
//!
//! The audit engine checks the following categories:
//!
//! | Category | Description |
//! |----------|-------------|
//! | `secrets` | Detect exposed secrets and credentials |
//! | `files` | Check for required files (README, LICENSE, etc.) |
//! | `docs` | Documentation quality checks |
//! | `security` | Security best practices |
//! | `workflows` | CI/CD and GitHub Actions checks |
//! | `quality` | Code quality standards |
//! | `dependencies` | Dependency security and licensing |
//! | `docker` | Docker configuration checks |
//! | `git` | Git configuration and history checks |

pub mod actions;
pub mod cache;
pub mod cli;
pub mod compare;
pub mod config;
pub mod error;
pub mod hooks;
pub mod providers;
pub mod rules;
pub mod scanner;
pub mod utils;

pub use error::RepoLensError;

// Re-export exit_codes from cli module for public API
pub use cli::exit_codes;
