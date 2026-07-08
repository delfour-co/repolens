//! # RepoLens core
//!
//! Pure auditing logic: scanning, rules, action planning, providers, caching,
//! comparison. No CLI, no `clap`. The `repolens` binary crate builds the CLI on
//! top of this library.
//!
//! ## Rule Categories
//!
//! The audit engine checks the following categories:
//!
//! | Category | Description |
//! |----------|-------------|
//! | `files` | `.gitignore` presence and recommended entries |
//! | `docs` | Required docs (README, LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CHANGELOG) |
//! | `security` | Repo security settings: branch protection, vulnerability alerts, secret scanning, Actions permissions |
//! | `git` | `.gitattributes` presence, sensitive files that should be gitignored |
//! | `codeowners` | CODEOWNERS file presence and syntax |
//! | `metadata` | Repository description, topics/tags, homepage |

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
