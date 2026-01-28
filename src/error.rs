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

    /// Cache-related errors
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
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

/// Errors that occur during cache operations
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum CacheError {
    /// Failed to read cache file
    #[error("Failed to read cache file '{path}': {message}")]
    FileRead {
        /// Path to the cache file
        path: String,
        /// Error message
        message: String,
    },

    /// Failed to write cache file
    #[error("Failed to write cache file '{path}': {message}")]
    FileWrite {
        /// Path to the cache file
        path: String,
        /// Error message
        message: String,
    },

    /// Failed to parse cache file
    #[error("Failed to parse cache file: {message}")]
    Parse {
        /// Error message describing the parse failure
        message: String,
    },

    /// Failed to delete cache
    #[error("Failed to delete cache: {message}")]
    Delete {
        /// Error message
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_error_display() {
        let err = ScanError::FileRead {
            path: "test.txt".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("test.txt"));
        assert!(msg.contains("Failed to read file"));
    }

    #[test]
    fn test_config_error_file_read_display() {
        let err = ConfigError::FileRead {
            path: "config.toml".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("config.toml"));
    }

    #[test]
    fn test_config_error_parse_display() {
        let err = ConfigError::Parse {
            message: "invalid syntax".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("invalid syntax"));
    }

    #[test]
    fn test_config_error_serialize_display() {
        let err = ConfigError::Serialize {
            message: "serialization failed".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("serialization failed"));
    }

    #[test]
    fn test_config_error_invalid_preset_display() {
        let err = ConfigError::InvalidPreset {
            name: "unknown".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("unknown"));
    }

    #[test]
    fn test_provider_error_command_failed_display() {
        let err = ProviderError::CommandFailed {
            command: "gh pr list".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("gh pr list"));
    }

    #[test]
    fn test_provider_error_json_parse_display() {
        let err = ProviderError::JsonParse {
            message: "unexpected token".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("unexpected token"));
    }

    #[test]
    fn test_provider_error_not_authenticated_display() {
        let err = ProviderError::NotAuthenticated;
        let msg = format!("{}", err);
        assert!(msg.contains("not authenticated") || msg.contains("Not in a GitHub repository"));
    }

    #[test]
    fn test_provider_error_invalid_repo_name_display() {
        let err = ProviderError::InvalidRepoName {
            name: "invalid-name".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("invalid-name"));
    }

    #[test]
    fn test_provider_error_github_cli_not_available_display() {
        let err = ProviderError::GitHubCliNotAvailable;
        let msg = format!("{}", err);
        assert!(msg.contains("GitHub CLI") || msg.contains("gh"));
    }

    #[test]
    fn test_action_error_file_write_display() {
        let err = ActionError::FileWrite {
            path: "output.txt".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("output.txt"));
    }

    #[test]
    fn test_action_error_directory_create_display() {
        let err = ActionError::DirectoryCreate {
            path: "/some/dir".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("/some/dir"));
    }

    #[test]
    fn test_action_error_unknown_template_display() {
        let err = ActionError::UnknownTemplate {
            name: "my-template".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("my-template"));
    }

    #[test]
    fn test_action_error_execution_failed_display() {
        let err = ActionError::ExecutionFailed {
            message: "action failed".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("action failed"));
    }

    #[test]
    fn test_rule_error_execution_failed_display() {
        let err = RuleError::ExecutionFailed {
            message: "rule failed".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("rule failed"));
    }

    #[test]
    fn test_cache_error_file_read_display() {
        let err = CacheError::FileRead {
            path: "cache.json".to_string(),
            message: "read error".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("cache.json"));
    }

    #[test]
    fn test_cache_error_file_write_display() {
        let err = CacheError::FileWrite {
            path: "cache.json".to_string(),
            message: "write error".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("cache.json"));
    }

    #[test]
    fn test_cache_error_parse_display() {
        let err = CacheError::Parse {
            message: "parse error".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("parse error"));
    }

    #[test]
    fn test_cache_error_delete_display() {
        let err = CacheError::Delete {
            message: "delete error".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("delete error"));
    }

    #[test]
    fn test_repolens_error_from_scan_error() {
        let scan_err = ScanError::FileRead {
            path: "test.txt".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        };
        let err: RepoLensError = scan_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Scan error"));
    }

    #[test]
    fn test_repolens_error_from_config_error() {
        let config_err = ConfigError::Parse {
            message: "parse error".to_string(),
        };
        let err: RepoLensError = config_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Config error"));
    }

    #[test]
    fn test_repolens_error_from_provider_error() {
        let provider_err = ProviderError::NotAuthenticated;
        let err: RepoLensError = provider_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Provider error"));
    }

    #[test]
    fn test_repolens_error_from_action_error() {
        let action_err = ActionError::ExecutionFailed {
            message: "failed".to_string(),
        };
        let err: RepoLensError = action_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Action error"));
    }

    #[test]
    fn test_repolens_error_from_rule_error() {
        let rule_err = RuleError::ExecutionFailed {
            message: "failed".to_string(),
        };
        let err: RepoLensError = rule_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Rule error"));
    }

    #[test]
    fn test_repolens_error_from_cache_error() {
        let cache_err = CacheError::Parse {
            message: "failed".to_string(),
        };
        let err: RepoLensError = cache_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Cache error"));
    }

    #[test]
    fn test_repolens_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err: RepoLensError = io_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Scan error"));
    }
}
