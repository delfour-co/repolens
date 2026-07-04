//! # RepoLens core
//!
//! Pure auditing logic: scanning, rules, action planning, providers, caching,
//! comparison. No CLI, no `clap`. The `repolens` binary crate builds the CLI on
//! top of this library.

pub mod actions;
pub mod cache;
pub mod compare;
pub mod config;
pub mod error;
pub mod providers;
pub mod rules;
pub mod scanner;
pub mod utils;

pub use error::RepoLensError;
