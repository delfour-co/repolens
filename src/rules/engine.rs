//! Rules evaluation engine

use anyhow::Result;
use tracing::{debug, info};

use super::categories::{
    secrets::SecretsRules,
    files::FilesRules,
    docs::DocsRules,
    security::SecurityRules,
    workflows::WorkflowsRules,
    quality::QualityRules,
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
    async fn run(&self, scanner: &Scanner, config: &Config) -> Result<Vec<super::Finding>>;
}

/// Main rules evaluation engine
pub struct RulesEngine {
    config: Config,
    only_categories: Option<Vec<String>>,
    skip_categories: Option<Vec<String>>,
}

impl RulesEngine {
    /// Create a new rules engine with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            only_categories: None,
            skip_categories: None,
        }
    }

    /// Set categories to exclusively run
    pub fn set_only_categories(&mut self, categories: Vec<String>) {
        self.only_categories = Some(categories);
    }

    /// Set categories to skip
    pub fn set_skip_categories(&mut self, categories: Vec<String>) {
        self.skip_categories = Some(categories);
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
    pub async fn run(&self, scanner: &Scanner) -> Result<AuditResults> {
        info!("Starting audit with preset: {}", self.config.preset);

        let repo_name = scanner.repository_name();
        let mut results = AuditResults::new(repo_name, &self.config.preset);

        // Get all rule categories
        let categories: Vec<Box<dyn RuleCategory>> = vec![
            Box::new(SecretsRules),
            Box::new(FilesRules),
            Box::new(DocsRules),
            Box::new(SecurityRules),
            Box::new(WorkflowsRules),
            Box::new(QualityRules),
        ];

        // Run each category
        for category in categories {
            let category_name = category.name();

            if !self.should_run_category(category_name) {
                debug!("Skipping category: {}", category_name);
                continue;
            }

            debug!("Running category: {}", category_name);

            match category.run(scanner, &self.config).await {
                Ok(findings) => {
                    debug!("Category {} found {} issues", category_name, findings.len());
                    results.add_findings(findings);
                }
                Err(e) => {
                    tracing::warn!("Error running category {}: {}", category_name, e);
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
