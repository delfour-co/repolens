//! Pattern definitions for secret detection

mod secrets;

// Retained as a shared pattern library. Currently unused internally after the
// secrets rule category was removed; kept available for future rule categories.
#[allow(unused_imports)]
pub use secrets::SECRET_PATTERNS;
