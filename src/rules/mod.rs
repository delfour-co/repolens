//! # Rules Module
//!
//! This module provides the audit rules engine and finding management for RepoLens.
//!
//! ## Architecture
//!
//! The rules system is organized as follows:
//!
//! - [`engine`] - The main rules evaluation engine that orchestrates rule execution
//! - [`categories`] - Individual rule category implementations (secrets, files, docs, etc.)
//! - [`results`] - Finding and severity types for audit results
//! - [`constants`] - Rule category constants and filtering
//!
//! ## Rule Categories
//!
//! Each category implements the [`engine::RuleCategory`] trait:
//!
//! | Category | Description |
//! |----------|-------------|
//! | `files` | `.gitignore` presence and recommended entries |
//! | `docs` | Required docs (README, LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CHANGELOG) |
//! | `security` | Repo security settings: branch protection, vulnerability alerts, secret scanning, Actions permissions |
//! | `git` | `.gitattributes` presence, sensitive files that should be gitignored |
//! | `codeowners` | CODEOWNERS file presence and syntax |
//! | `metadata` | Repository description, topics/tags, homepage |
//!
//! ## Examples
//!
//! ### Running the Rules Engine
//!
//! ```rust,no_run
//! use repolens::{config::Config, rules::engine::RulesEngine, scanner::Scanner};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), repolens::RepoLensError> {
//! let config = Config::default();
//! let scanner = Scanner::new(PathBuf::from("."));
//! let engine = RulesEngine::new(config);
//!
//! let results = engine.run(&scanner).await?;
//!
//! for finding in results.findings() {
//!     println!("[{:?}] {}: {}", finding.severity, finding.rule_id, finding.message);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Filtering Categories
//!
//! ```rust,no_run
//! use repolens::{config::Config, rules::engine::RulesEngine, scanner::Scanner};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), repolens::RepoLensError> {
//! let config = Config::default();
//! let scanner = Scanner::new(PathBuf::from("."));
//! let mut engine = RulesEngine::new(config);
//!
//! // Only run secrets detection
//! engine.set_only_categories(vec!["secrets".to_string()]);
//!
//! let results = engine.run(&scanner).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Working with Findings
//!
//! ```rust
//! use repolens::rules::{Finding, Severity};
//!
//! let finding = Finding::new("SEC001", "secrets", Severity::Critical, "API key detected")
//!     .with_location("src/config.rs:42")
//!     .with_description("A hardcoded API key was found in the source code")
//!     .with_remediation("Move the API key to environment variables");
//!
//! assert_eq!(finding.severity, Severity::Critical);
//! ```

pub mod categories;
pub mod constants;
pub mod engine;
pub mod results;

pub use constants::filter_valid_categories;
pub use results::{Finding, Severity};
