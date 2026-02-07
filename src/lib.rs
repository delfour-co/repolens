//! RepoLens Library
//!
//! This crate provides the core functionality for auditing repositories
//! and preparing them for open source or enterprise standards.

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
