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

    /// Configuration-related errors
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    /// Provider-related errors (GitHub API, etc.)
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    /// Action execution errors
    #[error("Action error: {0}")]
    Action(#[from] ActionError),

    /// Rule execution errors
    #[error("Rule error: {0}")]
    Rule(#[from] RuleError),
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

/// Errors that occur during configuration loading and parsing
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to read configuration file
    #[error("Failed to read configuration file '{path}': {source}")]
    FileRead {
        /// Path to the configuration file
        path: String,
        /// The underlying I/O error
        source: std::io::Error,
    },

    /// Failed to parse configuration file
    #[error("Failed to parse configuration file: {message}")]
    Parse {
        /// Error message describing the parse failure
        message: String,
    },

    /// Failed to serialize configuration
    #[error("Failed to serialize configuration: {message}")]
    Serialize {
        /// Error message describing the serialization failure
        message: String,
    },

    /// Invalid preset name
    #[error("Invalid preset name: {name}")]
    InvalidPreset {
        /// The invalid preset name
        name: String,
    },
}

/// Errors that occur when interacting with external providers (GitHub API, etc.)
#[derive(Error, Debug)]
pub enum ProviderError {
    /// Command execution failed
    #[error("Command execution failed: {command}")]
    CommandFailed {
        /// The command that failed
        command: String,
    },

    /// Failed to parse JSON response
    #[error("Failed to parse JSON response: {message}")]
    JsonParse {
        /// Error message describing the parse failure
        message: String,
    },

    /// Not in a GitHub repository or not authenticated
    #[error("Not in a GitHub repository or not authenticated")]
    NotAuthenticated,

    /// Invalid repository name format
    #[error("Invalid repository name format: {name}")]
    InvalidRepoName {
        /// The invalid repository name
        name: String,
    },

    /// GitHub CLI not available
    #[error("GitHub CLI (gh) is not available or not authenticated")]
    GitHubCliNotAvailable,
}

/// Errors that occur during action execution
#[derive(Error, Debug)]
pub enum ActionError {
    /// Failed to create file
    #[error("Failed to create file '{path}': {source}")]
    #[allow(dead_code)]
    FileCreate {
        /// Path to the file that failed to create
        path: String,
        /// The underlying I/O error
        source: std::io::Error,
    },

    /// Failed to write file
    #[error("Failed to write file '{path}': {source}")]
    FileWrite {
        /// Path to the file that failed to write
        path: String,
        /// The underlying I/O error
        source: std::io::Error,
    },

    /// Failed to create directory
    #[error("Failed to create directory '{path}': {source}")]
    DirectoryCreate {
        /// Path to the directory that failed to create
        path: String,
        /// The underlying I/O error
        source: std::io::Error,
    },

    /// Unknown template
    #[error("Unknown template: {name}")]
    UnknownTemplate {
        /// The unknown template name
        name: String,
    },

    /// Action execution failed
    #[error("Action execution failed: {message}")]
    ExecutionFailed {
        /// Error message describing the failure
        message: String,
    },
}

/// Errors that occur during rule execution
#[derive(Error, Debug)]
pub enum RuleError {
    /// Rule execution failed
    #[error("Rule execution failed: {message}")]
    ExecutionFailed {
        /// Error message describing the failure
        message: String,
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

// Conversion from toml::de::Error
impl From<toml::de::Error> for RepoLensError {
    fn from(err: toml::de::Error) -> Self {
        RepoLensError::Config(ConfigError::Parse {
            message: err.to_string(),
        })
    }
}

// Conversion from toml::ser::Error
impl From<toml::ser::Error> for RepoLensError {
    fn from(err: toml::ser::Error) -> Self {
        RepoLensError::Config(ConfigError::Serialize {
            message: err.to_string(),
        })
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for RepoLensError {
    fn from(err: serde_json::Error) -> Self {
        RepoLensError::Provider(ProviderError::JsonParse {
            message: err.to_string(),
        })
    }
}
