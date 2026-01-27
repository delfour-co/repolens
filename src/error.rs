//! Error types for RepoLens
//!
//! This module defines custom error types using `thiserror` for better error handling
//! and more descriptive error messages throughout the application.

use thiserror::Error;

/// Main error type for RepoLens
#[derive(Error, Debug)]
pub enum RepoLensError {
    /// Scan-related errors
    #[error("Scan error: {0}")]
    Scan(#[from] ScanError),
}

/// Errors that occur during repository scanning
#[derive(Error, Debug)]
pub enum ScanError {
    /// Failed to read a file
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        /// Path to the file that failed to read
        path: String,
        /// The underlying I/O error
        source: std::io::Error,
    },
}

// Allow conversion from std::io::Error for convenience
impl From<std::io::Error> for RepoLensError {
    fn from(err: std::io::Error) -> Self {
        RepoLensError::Scan(ScanError::FileRead {
            path: "unknown".to_string(),
            source: err,
        })
    }
}
