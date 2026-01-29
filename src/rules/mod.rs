//! Rules module - Audit rules and evaluation engine

pub mod categories;
pub mod engine;
pub mod patterns;
pub mod results;

pub use results::{Finding, Severity};
