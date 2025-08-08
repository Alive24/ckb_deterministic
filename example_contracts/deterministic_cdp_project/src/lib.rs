// Library interface for deterministic_cdp_project
// This allows tests to run without compiling the full binary

pub mod recipes;

// Re-export necessary items for tests
pub use deterministic_cdp_shared::*;