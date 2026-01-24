//! Actions module - Planned changes and execution

pub mod plan;
pub mod planner;
pub mod executor;
mod gitignore;
mod templates;
mod branch_protection;
mod github_settings;

pub use plan::{Action, ActionPlan};
pub use planner::ActionPlanner;
pub use executor::{ActionExecutor, ActionResult};
