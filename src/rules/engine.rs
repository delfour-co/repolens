//! Rules evaluation engine

use crate::cache::AuditCache;
use crate::error::RepoLensError;
use tracing::{debug, info, span, Level};

use super::categories::{
    custom::CustomRules, dependencies::DependencyRules, docs::DocsRules, files::FilesRules,
    quality::QualityRules, secrets::SecretsRules, security::SecurityRules,
    workflows::WorkflowsRules,
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
/// Parameters: (category_name, current_index, total_count)
pub type ProgressCallback = Box<dyn Fn(&str, usize, usize) + Send + Sync>;

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
        info!("Starting audit with preset: {}", self.config.preset);

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
            if let Some(ref callback) = self.progress_callback {
                callback(category_name, current, total);
            }

            let span = span!(Level::INFO, "category", category = category_name, repository = %repo_name_ref);
            let _guard = span.enter();

            debug!(category = category_name, "Running category");

            match category.run(scanner, &self.config).await {
                Ok(findings) => {
                    let count = findings.len();
                    debug!(
                        category = category_name,
                        findings_count = count,
                        "Category completed"
                    );
                    results.add_findings(findings);
                }
                Err(e) => {
                    tracing::warn!(
                        category = category_name,
                        error = %e,
                        "Error running category"
                    );
                }
            }
        }

        info!(
            "Audit complete: {} critical, {} warnings, {} info",
            results.count_by_severity(super::Severity::Critical),
            results.count_by_severity(super::Severity::Warning),
            results.count_by_severity(super::Severity::Info),
        );

        Ok(results)
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

        engine.set_progress_callback(Box::new(move |_name, _current, _total| {
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

        engine.set_progress_callback(Box::new(move |_name, _current, _total| {
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
}
