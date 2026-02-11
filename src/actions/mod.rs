//! # Actions Module
//!
//! This module handles the planning and execution of remediation actions
//! identified during an audit.
//!
//! ## Overview
//!
//! The actions system provides:
//!
//! - **Planning** - Analyze audit findings and determine required actions
//! - **Execution** - Apply fixes to the repository (files, GitHub settings)
//! - **Templates** - Generate missing files from templates
//!
//! ## Submodules
//!
//! - [`planner`] - Action plan generation from audit results
//! - [`executor`] - Action execution engine
//! - [`plan`] - Action plan data structures
//! - [`git`] - Git-related actions
//!
//! ## Supported Actions
//!
//! | Action | Description |
//! |--------|-------------|
//! | Create LICENSE | Generate LICENSE file from template |
//! | Create README | Generate README.md skeleton |
//! | Create CONTRIBUTING | Generate CONTRIBUTING.md |
//! | Create CODE_OF_CONDUCT | Generate CODE_OF_CONDUCT.md |
//! | Create SECURITY | Generate SECURITY.md policy |
//! | Update .gitignore | Add recommended entries |
//! | Branch Protection | Configure GitHub branch rules |
//! | Repository Settings | Configure GitHub repo settings |
//!
//! ## Examples
//!
//! ### Creating an Action Plan
//!
//! ```rust,no_run
//! use repolens::{
//!     actions::planner::ActionPlanner,
//!     config::Config,
//!     scanner::Scanner,
//!     rules::engine::RulesEngine,
//! };
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), repolens::RepoLensError> {
//! let config = Config::default();
//! let scanner = Scanner::new(PathBuf::from("."));
//!
//! // First run the audit to get results
//! let engine = RulesEngine::new(config.clone());
//! let results = engine.run(&scanner).await?;
//!
//! // Then create an action plan from the results
//! let planner = ActionPlanner::new(config);
//! let plan = planner.create_plan(&results).await?;
//! println!("Planned {} actions", plan.actions().len());
//! # Ok(())
//! # }
//! ```

mod branch_protection;
pub mod executor;
pub mod git;
mod github_settings;
mod gitignore;
pub mod plan;
pub mod planner;
mod templates;
