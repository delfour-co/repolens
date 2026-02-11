//! # Providers Module
//!
//! This module handles integrations with external services, primarily GitHub.
//!
//! ## GitHub Integration
//!
//! The [`github`] module provides functionality for:
//!
//! - Repository information retrieval
//! - Branch protection rule management
//! - Repository settings configuration
//! - Authentication via `gh` CLI
//!
//! ## Prerequisites
//!
//! GitHub operations require:
//!
//! 1. GitHub CLI (`gh`) installed and in PATH
//! 2. Authentication via `gh auth login`
//! 3. Appropriate repository permissions
//!
//! ## Examples
//!
//! ### Checking GitHub Authentication
//!
//! ```rust,no_run
//! use repolens::providers::github::GitHubProvider;
//!
//! // Check if gh CLI is available
//! if GitHubProvider::is_available() {
//!     println!("GitHub CLI is available");
//! } else {
//!     eprintln!("GitHub CLI not available");
//! }
//! ```

pub mod github;
