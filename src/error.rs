//! # Error Types for RepoLens
//!
//! This module defines custom error types using `thiserror` for better error handling
//! and more descriptive error messages throughout the application.
//!
//! ## Error Hierarchy
//!
//! The main error type is [`RepoLensError`], which wraps more specific error types:
//!
//! - [`ScanError`] - Errors during repository scanning
//! - [`ConfigError`] - Configuration loading and parsing errors
//! - [`ProviderError`] - External service errors (GitHub API)
//! - [`ActionError`] - Remediation action execution errors
//! - [`RuleError`] - Rule evaluation errors
//! - [`CacheError`] - Caching operation errors
//!
//! ## Examples
//!
//! ### Handling Errors
//!
//! ```rust,no_run
//! use repolens::{config::Config, RepoLensError};
//!
//! fn load_config() -> Result<Config, RepoLensError> {
//!     Config::load_or_default()
//! }
//!
//! match load_config() {
//!     Ok(config) => println!("Loaded config with preset: {}", config.preset),
//!     Err(e) => {
//!         eprintln!("{}", e.display_formatted());
//!         if let Some(suggestion) = e.suggestion() {
//!             eprintln!("Hint: {}", suggestion);
//!         }
//!     }
//! }
//! ```
//!
//! ### Error Suggestions
//!
//! Many errors include helpful suggestions for resolution:
//!
//! ```rust
//! use repolens::error::{RepoLensError, ConfigError};
//!
//! let err = RepoLensError::Config(ConfigError::ConfigNotFound {
//!     path: ".repolens.toml".to_string(),
//! });
//!
//! // Get a suggestion for how to fix the error
//! if let Some(suggestion) = err.suggestion() {
//!     assert!(suggestion.contains("repolens init"));
//! }
//! ```

use colored::Colorize;
use thiserror::Error;

/// Valid preset names for error messages
pub const VALID_PRESETS: &[&str] = &["opensource", "enterprise", "strict"];

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

impl RepoLensError {
    /// Get a user-friendly suggestion for how to fix this error
    pub fn suggestion(&self) -> Option<String> {
        match self {
            RepoLensError::Config(ConfigError::ConfigNotFound { .. }) => {
                Some("Run 'repolens init' to create a configuration file.".to_string())
            }
            RepoLensError::Config(ConfigError::InvalidPreset { .. }) => {
                Some(format!("Valid presets are: {}", VALID_PRESETS.join(", ")))
            }
            RepoLensError::Config(ConfigError::Parse { .. }) => {
                Some("Check your .repolens.toml file for syntax errors.".to_string())
            }
            RepoLensError::Provider(ProviderError::GitNotRepository { .. }) => {
                Some("Run 'git init' to initialize a git repository.".to_string())
            }
            RepoLensError::Provider(ProviderError::NotAuthenticated) => {
                Some("Run 'gh auth login' to authenticate with GitHub.".to_string())
            }
            RepoLensError::Provider(ProviderError::GitHubCliNotAvailable) => {
                Some("Install GitHub CLI from https://cli.github.com/".to_string())
            }
            RepoLensError::Action(ActionError::FileWrite { path, .. }) => Some(format!(
                "Check that you have write permissions for '{}'.",
                path
            )),
            RepoLensError::Action(ActionError::DirectoryCreate { path, .. }) => Some(format!(
                "Check that you have permissions to create directories in '{}'.",
                path
            )),
            _ => None,
        }
    }

    /// Format the error for display with colors and suggestions
    pub fn display_formatted(&self) -> String {
        let mut output = String::new();

        // Main error message in red
        output.push_str(&format!("{} {}\n", "Error:".red().bold(), self));

        // Add suggestion if available
        if let Some(suggestion) = self.suggestion() {
            output.push_str(&format!("\n  {} {}\n", "Hint:".cyan().bold(), suggestion));
        }

        output
    }
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
    /// Configuration file not found
    #[allow(dead_code)]
    #[error("Configuration file not found")]
    ConfigNotFound {
        /// Path where configuration was expected
        path: String,
    },

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
    #[allow(dead_code)]
    #[error("Invalid preset '{name}'")]
    InvalidPreset {
        /// The invalid preset name
        name: String,
    },
}

impl ConfigError {
    /// Get a user-friendly description of the error
    #[allow(dead_code)]
    pub fn description(&self) -> String {
        match self {
            ConfigError::ConfigNotFound { path } => {
                format!("No .repolens.toml found at '{}'.", path)
            }
            ConfigError::InvalidPreset { name } => {
                format!(
                    "The preset '{}' is not valid. Valid presets are: {}",
                    name,
                    VALID_PRESETS.join(", ")
                )
            }
            _ => self.to_string(),
        }
    }
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

    /// Not in a git repository
    #[allow(dead_code)]
    #[error("Not a git repository")]
    GitNotRepository {
        /// The path that was checked
        path: String,
    },

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

impl ProviderError {
    /// Get a user-friendly description of the error
    #[allow(dead_code)]
    pub fn description(&self) -> String {
        match self {
            ProviderError::GitNotRepository { path } => {
                format!("The directory '{}' is not a git repository.", path)
            }
            ProviderError::GitHubCliNotAvailable => {
                "The GitHub CLI (gh) is not installed or not in PATH.".to_string()
            }
            ProviderError::NotAuthenticated => "You are not authenticated with GitHub.".to_string(),
            ProviderError::CommandFailed { command, .. } => {
                format!("The command '{}' failed to execute.", command)
            }
            _ => self.to_string(),
        }
    }
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

    #[test]
    fn test_repolens_error_from_toml_de_error() {
        let toml_err = toml::from_str::<toml::Value>("invalid [[[toml").unwrap_err();
        let err: RepoLensError = toml_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Config error"));
    }

    #[test]
    fn test_repolens_error_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: RepoLensError = json_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Provider error"));
    }

    #[test]
    fn test_action_error_file_create_display() {
        let err = ActionError::FileCreate {
            path: "new_file.txt".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::AlreadyExists, "already exists"),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("new_file.txt"));
        assert!(msg.contains("Failed to create file"));
    }

    #[test]
    fn test_repolens_error_display_variants() {
        let scan_err = RepoLensError::Scan(ScanError::FileRead {
            path: "test.rs".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        });
        assert!(format!("{}", scan_err).contains("Scan error"));

        let rule_err = RepoLensError::Rule(RuleError::ExecutionFailed {
            message: "test failure".to_string(),
        });
        assert!(format!("{}", rule_err).contains("Rule error"));

        let provider_err = RepoLensError::Provider(ProviderError::NotAuthenticated);
        assert!(format!("{}", provider_err).contains("Provider error"));
    }

    #[test]
    fn test_repolens_error_from_toml_ser_error() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(vec![1, 2, 3], "value");
        let ser_err = toml::to_string(&map).unwrap_err();
        let err: RepoLensError = ser_err.into();
        let msg = format!("{}", err);
        assert!(msg.contains("Config error"));
    }

    // New tests for improved error handling

    #[test]
    fn test_config_not_found_error() {
        let err = ConfigError::ConfigNotFound {
            path: ".repolens.toml".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Configuration file not found"));
    }

    #[test]
    fn test_config_not_found_description() {
        let err = ConfigError::ConfigNotFound {
            path: ".repolens.toml".to_string(),
        };
        let desc = err.description();
        assert!(desc.contains(".repolens.toml"));
    }

    #[test]
    fn test_invalid_preset_description() {
        let err = ConfigError::InvalidPreset {
            name: "foo".to_string(),
        };
        let desc = err.description();
        assert!(desc.contains("foo"));
        assert!(desc.contains("opensource"));
        assert!(desc.contains("enterprise"));
        assert!(desc.contains("strict"));
    }

    #[test]
    fn test_git_not_repository_error() {
        let err = ProviderError::GitNotRepository {
            path: "/some/path".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Not a git repository"));
    }

    #[test]
    fn test_git_not_repository_description() {
        let err = ProviderError::GitNotRepository {
            path: "/some/path".to_string(),
        };
        let desc = err.description();
        assert!(desc.contains("/some/path"));
        assert!(desc.contains("not a git repository"));
    }

    #[test]
    fn test_repolens_error_suggestion_config_not_found() {
        let err = RepoLensError::Config(ConfigError::ConfigNotFound {
            path: ".repolens.toml".to_string(),
        });
        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("repolens init"));
    }

    #[test]
    fn test_repolens_error_suggestion_invalid_preset() {
        let err = RepoLensError::Config(ConfigError::InvalidPreset {
            name: "foo".to_string(),
        });
        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        let s = suggestion.unwrap();
        assert!(s.contains("opensource"));
        assert!(s.contains("enterprise"));
        assert!(s.contains("strict"));
    }

    #[test]
    fn test_repolens_error_suggestion_git_not_repo() {
        let err = RepoLensError::Provider(ProviderError::GitNotRepository {
            path: ".".to_string(),
        });
        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("git init"));
    }

    #[test]
    fn test_repolens_error_suggestion_not_authenticated() {
        let err = RepoLensError::Provider(ProviderError::NotAuthenticated);
        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("gh auth login"));
    }

    #[test]
    fn test_repolens_error_suggestion_github_cli_not_available() {
        let err = RepoLensError::Provider(ProviderError::GitHubCliNotAvailable);
        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("cli.github.com"));
    }

    #[test]
    fn test_repolens_error_suggestion_file_write() {
        let err = RepoLensError::Action(ActionError::FileWrite {
            path: "/some/file.txt".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied"),
        });
        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("/some/file.txt"));
    }

    #[test]
    fn test_repolens_error_no_suggestion() {
        let err = RepoLensError::Cache(CacheError::Parse {
            message: "some error".to_string(),
        });
        let suggestion = err.suggestion();
        assert!(suggestion.is_none());
    }

    #[test]
    fn test_display_formatted_with_suggestion() {
        let err = RepoLensError::Config(ConfigError::ConfigNotFound {
            path: ".repolens.toml".to_string(),
        });
        let formatted = err.display_formatted();
        // Should contain the error message
        assert!(formatted.contains("Configuration file not found"));
    }

    #[test]
    fn test_display_formatted_without_suggestion() {
        let err = RepoLensError::Cache(CacheError::Parse {
            message: "parse error".to_string(),
        });
        let formatted = err.display_formatted();
        // Should contain the error message
        assert!(formatted.contains("parse error"));
    }

    #[test]
    fn test_valid_presets_constant() {
        assert!(VALID_PRESETS.contains(&"opensource"));
        assert!(VALID_PRESETS.contains(&"enterprise"));
        assert!(VALID_PRESETS.contains(&"strict"));
        assert_eq!(VALID_PRESETS.len(), 3);
    }
}
