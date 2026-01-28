//! Audit results caching module
//!
//! This module provides a caching system for audit results to avoid
//! re-auditing files that haven't changed since the last audit.
//!
//! The cache uses SHA256 content hashing to detect file changes.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::RepoLensError;
use crate::rules::results::Finding;

/// Default cache directory name within the project
const DEFAULT_CACHE_DIR: &str = ".repolens/cache";

/// Default maximum age for cache entries in hours
const DEFAULT_MAX_AGE_HOURS: u64 = 24;

/// Cache file name
const CACHE_FILE_NAME: &str = "audit_cache.json";

/// A single cache entry for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Relative path to the file from repository root
    pub file_path: String,

    /// SHA256 hash of the file content
    pub content_hash: String,

    /// Findings for this file
    pub findings: Vec<Finding>,

    /// Timestamp when the entry was created (seconds since UNIX epoch)
    pub timestamp: u64,
}

impl CacheEntry {
    /// Create a new cache entry
    #[allow(dead_code)]
    pub fn new(file_path: String, content_hash: String, findings: Vec<Finding>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        Self {
            file_path,
            content_hash,
            findings,
            timestamp,
        }
    }

    /// Check if the entry is expired based on max age
    pub fn is_expired(&self, max_age_hours: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        let max_age_secs = max_age_hours * 3600;
        now.saturating_sub(self.timestamp) > max_age_secs
    }

    /// Check if the entry matches the current file hash
    #[allow(dead_code)]
    pub fn matches_hash(&self, current_hash: &str) -> bool {
        self.content_hash == current_hash
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether caching is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Maximum age for cache entries in hours
    #[serde(default = "default_max_age_hours")]
    pub max_age_hours: u64,

    /// Cache directory path (relative to project root or absolute)
    #[serde(default = "default_directory")]
    pub directory: String,
}

fn default_enabled() -> bool {
    true
}

fn default_max_age_hours() -> u64 {
    DEFAULT_MAX_AGE_HOURS
}

fn default_directory() -> String {
    DEFAULT_CACHE_DIR.to_string()
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_age_hours: DEFAULT_MAX_AGE_HOURS,
            directory: DEFAULT_CACHE_DIR.to_string(),
        }
    }
}

/// Main audit cache structure
#[derive(Debug)]
pub struct AuditCache {
    /// Cache entries indexed by file path
    entries: HashMap<PathBuf, CacheEntry>,

    /// Path to the cache directory
    cache_dir: PathBuf,

    /// Cache configuration
    #[allow(dead_code)]
    config: CacheConfig,

    /// Whether the cache has been modified since loading
    dirty: bool,
}

impl AuditCache {
    /// Create a new audit cache with the given configuration
    ///
    /// # Arguments
    ///
    /// * `project_root` - The root directory of the project
    /// * `config` - Cache configuration
    ///
    /// # Returns
    ///
    /// A new `AuditCache` instance
    #[allow(dead_code)]
    pub fn new(project_root: &Path, config: CacheConfig) -> Self {
        let cache_dir = Self::resolve_cache_dir(project_root, &config.directory);

        Self {
            entries: HashMap::new(),
            cache_dir,
            config,
            dirty: false,
        }
    }

    /// Resolve the cache directory path
    fn resolve_cache_dir(project_root: &Path, directory: &str) -> PathBuf {
        let path = Path::new(directory);

        if path.is_absolute() {
            path.to_path_buf()
        } else if directory.starts_with("~") {
            // Handle home directory expansion
            if let Some(home) = dirs::home_dir() {
                home.join(directory.trim_start_matches("~/"))
            } else {
                project_root.join(directory)
            }
        } else {
            project_root.join(directory)
        }
    }

    /// Load cache from disk
    ///
    /// # Arguments
    ///
    /// * `project_root` - The root directory of the project
    /// * `config` - Cache configuration
    ///
    /// # Returns
    ///
    /// A loaded `AuditCache` instance, or a new empty cache if loading fails
    pub fn load(project_root: &Path, config: CacheConfig) -> Self {
        let cache_dir = Self::resolve_cache_dir(project_root, &config.directory);
        let cache_file = cache_dir.join(CACHE_FILE_NAME);

        let entries = if cache_file.exists() {
            match fs::read_to_string(&cache_file) {
                Ok(content) => match serde_json::from_str::<Vec<CacheEntry>>(&content) {
                    Ok(entries) => {
                        let mut map = HashMap::new();
                        for entry in entries {
                            // Skip expired entries during load
                            if !entry.is_expired(config.max_age_hours) {
                                map.insert(PathBuf::from(&entry.file_path), entry);
                            }
                        }
                        tracing::debug!(
                            "Loaded {} cache entries from {}",
                            map.len(),
                            cache_file.display()
                        );
                        map
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse cache file: {}", e);
                        HashMap::new()
                    }
                },
                Err(e) => {
                    tracing::debug!("Failed to read cache file: {}", e);
                    HashMap::new()
                }
            }
        } else {
            tracing::debug!("No cache file found at {}", cache_file.display());
            HashMap::new()
        };

        Self {
            entries,
            cache_dir,
            config,
            dirty: false,
        }
    }

    /// Save cache to disk
    ///
    /// # Returns
    ///
    /// `Ok(())` if the cache was saved successfully, or an error if it failed
    pub fn save(&self) -> Result<(), RepoLensError> {
        if !self.dirty {
            tracing::debug!("Cache not modified, skipping save");
            return Ok(());
        }

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&self.cache_dir).map_err(|e| {
            RepoLensError::Action(crate::error::ActionError::DirectoryCreate {
                path: self.cache_dir.display().to_string(),
                source: e,
            })
        })?;

        let cache_file = self.cache_dir.join(CACHE_FILE_NAME);
        let entries: Vec<&CacheEntry> = self.entries.values().collect();
        let content = serde_json::to_string_pretty(&entries)?;

        fs::write(&cache_file, content).map_err(|e| {
            RepoLensError::Action(crate::error::ActionError::FileWrite {
                path: cache_file.display().to_string(),
                source: e,
            })
        })?;

        tracing::debug!(
            "Saved {} cache entries to {}",
            entries.len(),
            cache_file.display()
        );

        Ok(())
    }

    /// Get cached findings for a file if the cache entry is valid
    ///
    /// # Arguments
    ///
    /// * `file_path` - Relative path to the file
    /// * `current_hash` - Current SHA256 hash of the file content
    ///
    /// # Returns
    ///
    /// `Some(&Vec<Finding>)` if a valid cache entry exists, `None` otherwise
    #[allow(dead_code)]
    pub fn get(&self, file_path: &Path, current_hash: &str) -> Option<&Vec<Finding>> {
        self.entries.get(file_path).and_then(|entry| {
            if entry.matches_hash(current_hash) && !entry.is_expired(self.config.max_age_hours) {
                tracing::trace!("Cache hit for {}", file_path.display());
                Some(&entry.findings)
            } else {
                tracing::trace!("Cache miss for {} (hash or expiry)", file_path.display());
                None
            }
        })
    }

    /// Insert or update a cache entry
    ///
    /// # Arguments
    ///
    /// * `file_path` - Relative path to the file
    /// * `hash` - SHA256 hash of the file content
    /// * `findings` - Findings for this file
    #[allow(dead_code)]
    pub fn insert(&mut self, file_path: PathBuf, hash: String, findings: Vec<Finding>) {
        let entry = CacheEntry::new(file_path.to_string_lossy().to_string(), hash, findings);
        self.entries.insert(file_path, entry);
        self.dirty = true;
    }

    /// Invalidate cache entry for a specific file
    ///
    /// # Arguments
    ///
    /// * `file_path` - Relative path to the file
    #[allow(dead_code)]
    pub fn invalidate(&mut self, file_path: &Path) {
        if self.entries.remove(file_path).is_some() {
            self.dirty = true;
            tracing::debug!("Invalidated cache for {}", file_path.display());
        }
    }

    /// Clear all cache entries
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        if !self.entries.is_empty() {
            self.entries.clear();
            self.dirty = true;
            tracing::info!("Cleared all cache entries");
        }
    }

    /// Get the number of cached entries
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.entries.len(),
            cache_dir: self.cache_dir.clone(),
        }
    }

    /// Check if caching is enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Cache directory path
    #[allow(dead_code)]
    pub cache_dir: PathBuf,
}

/// Calculate SHA256 hash of a file's content
///
/// # Arguments
///
/// * `path` - Path to the file
///
/// # Returns
///
/// The SHA256 hash as a hexadecimal string
#[allow(dead_code)]
pub fn calculate_file_hash(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculate SHA256 hash of content bytes
///
/// # Arguments
///
/// * `content` - The content to hash
///
/// # Returns
///
/// The SHA256 hash as a hexadecimal string
#[allow(dead_code)]
pub fn calculate_content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

/// Delete the cache directory and all its contents
///
/// # Arguments
///
/// * `project_root` - The root directory of the project
/// * `config` - Cache configuration
///
/// # Returns
///
/// `Ok(())` if the cache was cleared successfully
pub fn delete_cache_directory(project_root: &Path, config: &CacheConfig) -> io::Result<()> {
    let cache_dir = AuditCache::resolve_cache_dir(project_root, &config.directory);

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
        tracing::info!("Deleted cache directory: {}", cache_dir.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::results::Severity;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_finding(rule_id: &str, location: Option<&str>) -> Finding {
        let mut finding = Finding::new(rule_id, "test", Severity::Warning, "Test finding");
        if let Some(loc) = location {
            finding = finding.with_location(loc);
        }
        finding
    }

    #[test]
    fn test_cache_entry_creation() {
        let entry = CacheEntry::new(
            "test.rs".to_string(),
            "abc123".to_string(),
            vec![create_test_finding("TEST001", Some("test.rs:1"))],
        );

        assert_eq!(entry.file_path, "test.rs");
        assert_eq!(entry.content_hash, "abc123");
        assert_eq!(entry.findings.len(), 1);
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_cache_entry_expiry() {
        let mut entry = CacheEntry::new("test.rs".to_string(), "abc123".to_string(), vec![]);

        // Fresh entry should not be expired
        assert!(!entry.is_expired(24));

        // Set timestamp to 25 hours ago
        entry.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (25 * 3600);

        // Should now be expired with 24-hour max age
        assert!(entry.is_expired(24));

        // But not with 48-hour max age
        assert!(!entry.is_expired(48));
    }

    #[test]
    fn test_cache_entry_hash_matching() {
        let entry = CacheEntry::new("test.rs".to_string(), "abc123".to_string(), vec![]);

        assert!(entry.matches_hash("abc123"));
        assert!(!entry.matches_hash("def456"));
    }

    #[test]
    fn test_audit_cache_new() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::default();
        let cache = AuditCache::new(temp_dir.path(), config);

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert!(cache.is_enabled());
    }

    #[test]
    fn test_audit_cache_insert_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::default();
        let mut cache = AuditCache::new(temp_dir.path(), config);

        let findings = vec![create_test_finding("TEST001", Some("test.rs:1"))];
        cache.insert(
            PathBuf::from("test.rs"),
            "abc123".to_string(),
            findings.clone(),
        );

        // Cache hit with matching hash
        let result = cache.get(Path::new("test.rs"), "abc123");
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);

        // Cache miss with different hash
        let result = cache.get(Path::new("test.rs"), "def456");
        assert!(result.is_none());

        // Cache miss for non-existent file
        let result = cache.get(Path::new("other.rs"), "abc123");
        assert!(result.is_none());
    }

    #[test]
    fn test_audit_cache_invalidate() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::default();
        let mut cache = AuditCache::new(temp_dir.path(), config);

        cache.insert(PathBuf::from("test.rs"), "abc123".to_string(), vec![]);
        assert_eq!(cache.len(), 1);

        cache.invalidate(Path::new("test.rs"));
        assert!(cache.is_empty());
    }

    #[test]
    fn test_audit_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::default();
        let mut cache = AuditCache::new(temp_dir.path(), config);

        cache.insert(PathBuf::from("test1.rs"), "abc123".to_string(), vec![]);
        cache.insert(PathBuf::from("test2.rs"), "def456".to_string(), vec![]);
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_audit_cache_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::default();

        // Create and populate cache
        let mut cache = AuditCache::new(temp_dir.path(), config.clone());
        let findings = vec![create_test_finding("TEST001", Some("test.rs:1"))];
        cache.insert(
            PathBuf::from("test.rs"),
            "abc123".to_string(),
            findings.clone(),
        );

        // Save cache
        cache.save().unwrap();

        // Load cache in a new instance
        let loaded_cache = AuditCache::load(temp_dir.path(), config);
        assert_eq!(loaded_cache.len(), 1);

        let result = loaded_cache.get(Path::new("test.rs"), "abc123");
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_calculate_file_hash() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "Hello, World!").unwrap();

        let hash = calculate_file_hash(&file_path).unwrap();

        // Verify hash is a valid SHA256 hex string
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_calculate_content_hash() {
        let hash = calculate_content_hash(b"Hello, World!");

        // Should produce the same hash for the same content
        let hash2 = calculate_content_hash(b"Hello, World!");
        assert_eq!(hash, hash2);

        // Different content should produce different hash
        let hash3 = calculate_content_hash(b"Different content");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_delete_cache_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::default();

        // Create cache and save it
        let mut cache = AuditCache::new(temp_dir.path(), config.clone());
        cache.insert(PathBuf::from("test.rs"), "abc123".to_string(), vec![]);
        cache.save().unwrap();

        // Verify cache directory exists
        let cache_dir = temp_dir.path().join(".repolens/cache");
        assert!(cache_dir.exists());

        // Delete cache directory
        delete_cache_directory(temp_dir.path(), &config).unwrap();
        assert!(!cache_dir.exists());
    }

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_age_hours, 24);
        assert_eq!(config.directory, ".repolens/cache");
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig::default();
        let mut cache = AuditCache::new(temp_dir.path(), config);

        cache.insert(PathBuf::from("test1.rs"), "abc123".to_string(), vec![]);
        cache.insert(PathBuf::from("test2.rs"), "def456".to_string(), vec![]);

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
    }
}
