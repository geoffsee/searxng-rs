//! Search orchestration module
//!
//! Coordinates search execution across multiple engines,
//! aggregates results, and handles timing.

mod executor;
mod models;

pub use executor::Search;
pub use models::*;
