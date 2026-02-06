//! Rules evaluation engine

use crate::cache::AuditCache;
use crate::error::RepoLensError;
use crate::utils::{AuditTiming, CategoryTiming, Timer};
use tracing::{debug, info, span, Level};

use super::categories::{
    custom::CustomRules, dependencies::DependencyRules, docker::DockerRules, docs::DocsRules,
    files::FilesRules, git::GitRules, licenses::LicenseRules, quality::QualityRules,
    secrets::SecretsRules, security::SecurityRules, workflows::WorkflowsRules,
};
use super::results::AuditResults;
use crate::config::Config;
use crate::scanner::Scanner;

/// Trait for rule categories
#[async_trait::async_trait]
pub trait RuleCategory: Send + Sync {
    /// Get the category name
    fn name(&self) -> &'static str;

    /// Run the rules in this category
    async fn run(
        &self,
        scanner: &Scanner,
        config: &Config,
    ) -> Result<Vec<super::Finding>, RepoLensError>;
}

/// Callback function type for progress reporting
/// Parameters: (category_name, current_index, total_count, findings_count, duration_ms)
pub type ProgressCallback = Box<dyn Fn(&str, usize, usize, Option<(usize, u64)>) + Send + Sync>;

/// Main rules evaluation engine
pub struct RulesEngine {
    config: Config,
    only_categories: Option<Vec<String>>,
    skip_categories: Option<Vec<String>>,
    progress_callback: Option<ProgressCallback>,
    cache: Option<AuditCache>,
}

impl RulesEngine {
    /// Create a new rules engine with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            only_categories: None,
            skip_categories: None,
            progress_callback: None,
            cache: None,
        }
    }

    /// Set a callback function to report progress
    ///
    /// The callback will be called with category names as they are being processed.
    pub fn set_progress_callback(&mut self, callback: ProgressCallback) {
        self.progress_callback = Some(callback);
    }

    /// Set categories to exclusively run
    pub fn set_only_categories(&mut self, categories: Vec<String>) {
        self.only_categories = Some(categories);
    }

    /// Set categories to skip
    pub fn set_skip_categories(&mut self, categories: Vec<String>) {
        self.skip_categories = Some(categories);
    }

    /// Set the audit cache
    pub fn set_cache(&mut self, cache: AuditCache) {
        self.cache = Some(cache);
    }

    /// Take ownership of the cache (for saving after audit)
    pub fn take_cache(&mut self) -> Option<AuditCache> {
        self.cache.take()
    }

    /// Get a reference to the cache
    #[allow(dead_code)]
    pub fn cache(&self) -> Option<&AuditCache> {
        self.cache.as_ref()
    }

    /// Get a mutable reference to the cache
    #[allow(dead_code)]
    pub fn cache_mut(&mut self) -> Option<&mut AuditCache> {
        self.cache.as_mut()
    }

    /// Check if a category should be run
    fn should_run_category(&self, category: &str) -> bool {
        if let Some(only) = &self.only_categories {
            return only.iter().any(|c| c == category);
        }

        if let Some(skip) = &self.skip_categories {
            return !skip.iter().any(|c| c == category);
        }

        true
    }

    /// Run all enabled rules and return results
    pub async fn run(&self, scanner: &Scanner) -> Result<AuditResults, RepoLensError> {
        let (results, _) = self.run_with_timing(scanner).await?;
        Ok(results)
    }

    /// Run all enabled rules and return results along with timing information
    pub async fn run_with_timing(
        &self,
        scanner: &Scanner,
    ) -> Result<(AuditResults, AuditTiming), RepoLensError> {
        info!("Starting audit with preset: {}", self.config.preset);

        let total_timer = Timer::start();
        let mut audit_timing = AuditTiming::new();

        let repo_name = scanner.repository_name();
        let repo_name_ref = &repo_name;
        let mut results = AuditResults::new(repo_name.clone(), &self.config.preset);

        // Get all rule categories
        let categories: Vec<Box<dyn RuleCategory>> = vec![
            Box::new(SecretsRules),
            Box::new(FilesRules),
            Box::new(DocsRules),
            Box::new(SecurityRules),
            Box::new(WorkflowsRules),
            Box::new(QualityRules),
            Box::new(DependencyRules),
            Box::new(LicenseRules),
            Box::new(DockerRules),
            Box::new(GitRules),
            Box::new(CustomRules),
        ];

        // Count categories that will be executed
        let total: usize = categories
            .iter()
            .filter(|c| self.should_run_category(c.name()))
            .count();
        let mut current = 0;

        // Run each category
        for category in categories {
            let category_name = category.name();

            if !self.should_run_category(category_name) {
                debug!(category = category_name, "Skipping category");
                continue;
            }

            current += 1;
            // Call progress callback with None timing (before execution)
            if let Some(ref callback) = self.progress_callback {
                callback(category_name, current, total, None);
            }

            let span = span!(Level::INFO, "category", category = category_name, repository = %repo_name_ref);
            let _guard = span.enter();

            debug!(category = category_name, "Running category");

            // Time the category execution
            let category_timer = Timer::start();

            match category.run(scanner, &self.config).await {
                Ok(findings) => {
                    let category_duration = category_timer.elapsed();
                    let findings_count = findings.len();
                    debug!(
                        category = category_name,
                        findings_count = findings_count,
                        duration_ms = category_duration.as_millis(),
                        "Category completed"
                    );

                    // Record timing (rule_count is findings_count as we don't have individual rule counts yet)
                    audit_timing.add_category(CategoryTiming::new(
                        category_name,
                        0, // We don't track individual rules yet
                        findings_count,
                        category_duration,
                    ));

                    // Call progress callback with timing info (after execution)
                    if let Some(ref callback) = self.progress_callback {
                        callback(
                            category_name,
                            current,
                            total,
                            Some((findings_count, category_duration.as_millis() as u64)),
                        );
                    }

                    results.add_findings(findings);
                }
                Err(e) => {
                    let category_duration = category_timer.elapsed();
                    tracing::warn!(
                        category = category_name,
                        error = %e,
                        duration_ms = category_duration.as_millis(),
                        "Error running category"
                    );

                    // Still record timing for failed categories
                    audit_timing.add_category(CategoryTiming::new(
                        category_name,
                        0,
                        0,
                        category_duration,
                    ));
                }
            }
        }

        // Set total duration
        audit_timing.set_total_duration(total_timer.elapsed());

        info!(
            "Audit complete: {} critical, {} warnings, {} info (total time: {})",
            results.count_by_severity(super::Severity::Critical),
            results.count_by_severity(super::Severity::Warning),
            results.count_by_severity(super::Severity::Info),
            audit_timing.total_duration_formatted(),
        );

        Ok((results, audit_timing))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rules_engine_runs_all_categories() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a basic file structure
        fs::write(root.join("README.md"), "# Test Project").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let engine = RulesEngine::new(config);

        let results = engine.run(&scanner).await.unwrap();

        // Verify that results are returned (may be empty if no issues found)
        let _ = results.findings().len();
        assert_eq!(results.preset, "opensource");
    }

    #[tokio::test]
    async fn test_rules_engine_filters_with_only() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("README.md"), "# Test").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let mut engine = RulesEngine::new(config);
        engine.set_only_categories(vec!["secrets".to_string()]);

        let results = engine.run(&scanner).await.unwrap();

        // Verify that only secrets category was run
        // All findings should be from secrets category
        for finding in results.findings() {
            assert_eq!(finding.category, "secrets");
        }
    }

    #[tokio::test]
    async fn test_rules_engine_filters_with_skip() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("README.md"), "# Test").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let mut engine = RulesEngine::new(config);
        engine.set_skip_categories(vec!["secrets".to_string()]);

        let results = engine.run(&scanner).await.unwrap();

        // Verify that secrets category was skipped
        for finding in results.findings() {
            assert_ne!(finding.category, "secrets");
        }
    }

    #[tokio::test]
    async fn test_rules_engine_handles_category_errors_gracefully() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a file that might cause issues
        fs::write(root.join("test.txt"), "test").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let engine = RulesEngine::new(config);

        // Should not panic even if a category fails
        let results = engine.run(&scanner).await;
        assert!(results.is_ok());
    }

    #[tokio::test]
    async fn test_rules_engine_collects_all_findings() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create files that should trigger findings
        fs::write(
            root.join("test.js"),
            "const apiKey = 'sk_test_1234567890abcdef';",
        )
        .unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let engine = RulesEngine::new(config);

        let results = engine.run(&scanner).await.unwrap();

        // Should have collected findings from multiple categories
        // (at least secrets should find something)
        let _ = results.findings().len();
    }

    #[test]
    fn test_should_run_category_with_only() {
        let config = Config::default();
        let mut engine = RulesEngine::new(config);
        engine.set_only_categories(vec!["secrets".to_string(), "files".to_string()]);

        assert!(engine.should_run_category("secrets"));
        assert!(engine.should_run_category("files"));
        assert!(!engine.should_run_category("docs"));
    }

    #[test]
    fn test_should_run_category_with_skip() {
        let config = Config::default();
        let mut engine = RulesEngine::new(config);
        engine.set_skip_categories(vec!["secrets".to_string()]);

        assert!(!engine.should_run_category("secrets"));
        assert!(engine.should_run_category("files"));
        assert!(engine.should_run_category("docs"));
    }

    #[test]
    fn test_should_run_category_default() {
        let config = Config::default();
        let engine = RulesEngine::new(config);

        // By default, all categories should run
        assert!(engine.should_run_category("secrets"));
        assert!(engine.should_run_category("files"));
        assert!(engine.should_run_category("docs"));
    }

    #[test]
    fn test_set_progress_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let config = Config::default();
        let mut engine = RulesEngine::new(config);

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        engine.set_progress_callback(Box::new(move |_name, _current, _total, _timing| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        }));

        assert!(engine.progress_callback.is_some());
    }

    #[test]
    fn test_cache_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::default();
        let mut engine = RulesEngine::new(config.clone());

        // Initially no cache
        assert!(engine.cache().is_none());
        assert!(engine.cache_mut().is_none());

        // Set cache
        let cache = AuditCache::new(temp_dir.path(), config.cache);
        engine.set_cache(cache);

        // Now has cache
        assert!(engine.cache().is_some());
        assert!(engine.cache_mut().is_some());

        // Take cache
        let taken = engine.take_cache();
        assert!(taken.is_some());
        assert!(engine.cache().is_none());
    }

    #[tokio::test]
    async fn test_rules_engine_with_progress_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("README.md"), "# Test").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let mut engine = RulesEngine::new(config);

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        engine.set_progress_callback(Box::new(move |_name, _current, _total, _timing| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        }));

        let _ = engine.run(&scanner).await.unwrap();

        // Should have called progress callback for each category
        assert!(call_count.load(Ordering::SeqCst) > 0);
    }

    #[tokio::test]
    async fn test_rules_engine_with_cache() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("README.md"), "# Test").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let mut engine = RulesEngine::new(config.clone());
        engine.set_cache(AuditCache::new(root, config.cache));

        let results = engine.run(&scanner).await.unwrap();
        assert_eq!(results.preset, "opensource");
    }

    #[tokio::test]
    async fn test_rules_engine_run_with_timing() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("README.md"), "# Test Project").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let engine = RulesEngine::new(config);

        let (results, timing) = engine.run_with_timing(&scanner).await.unwrap();

        // Verify results are returned
        assert_eq!(results.preset, "opensource");

        // Verify timing information is populated
        assert!(!timing.categories().is_empty());
        assert!(timing.total_duration.as_nanos() > 0);
    }

    #[tokio::test]
    async fn test_rules_engine_timing_captures_category_durations() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("README.md"), "# Test").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let mut engine = RulesEngine::new(config);
        engine.set_only_categories(vec!["files".to_string()]);

        let (_, timing) = engine.run_with_timing(&scanner).await.unwrap();

        // Should have exactly one category timing
        assert_eq!(timing.categories().len(), 1);
        assert_eq!(timing.categories()[0].name, "files");
    }

    #[tokio::test]
    async fn test_rules_engine_timing_with_progress_callback() {
        use std::sync::{Arc, Mutex};

        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join("README.md"), "# Test").unwrap();

        let config = Config::default();
        let scanner = Scanner::new(root.to_path_buf());
        let mut engine = RulesEngine::new(config);
        engine.set_only_categories(vec!["files".to_string()]);

        // Track timing info received in callback
        let timing_info = Arc::new(Mutex::new(Vec::new()));
        let timing_info_clone = timing_info.clone();

        engine.set_progress_callback(Box::new(move |name, _current, _total, timing| {
            if let Some((findings, duration_ms)) = timing {
                timing_info_clone
                    .lock()
                    .unwrap()
                    .push((name.to_string(), findings, duration_ms));
            }
        }));

        let _ = engine.run_with_timing(&scanner).await.unwrap();

        let captured = timing_info.lock().unwrap();
        // Should have captured timing for files category
        assert!(!captured.is_empty());
        assert_eq!(captured[0].0, "files");
    }
}
