//! Utility modules for RepoLens

pub mod command;
pub mod language_detection;
pub mod permissions;
pub mod prerequisites;
pub mod timing;

pub use language_detection::{detect_languages, get_gitignore_entries_with_descriptions};
pub use timing::{format_duration, AuditTiming, CategoryTiming, Timer};
