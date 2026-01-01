//! Search engine module
//!
//! Defines the Engine trait and provides a registry for all search engines.

mod loader;
mod registry;
mod traits;

// Engine implementations
pub mod bing;
pub mod brave;
pub mod duckduckgo;
pub mod google;
pub mod wikipedia;

pub use loader::EngineLoader;
pub use registry::EngineRegistry;
pub use traits::*;
