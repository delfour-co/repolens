//! Utility modules for RepoLens

pub mod language_detection;
pub mod prerequisites;

pub use language_detection::{detect_languages, get_gitignore_entries_with_descriptions};
