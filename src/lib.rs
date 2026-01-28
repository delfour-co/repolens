//! RepoLens Library
//!
//! This crate provides the core functionality for auditing repositories
//! and preparing them for open source or enterprise standards.

pub mod actions;
pub mod cache;
pub mod cli;
pub mod config;
pub mod error;
pub mod providers;
pub mod rules;
pub mod scanner;
pub mod utils;

pub use error::RepoLensError;

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
