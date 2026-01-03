//! Search engine module
//!
//! Defines the Engine trait and provides a registry for all search engines.

mod loader;
mod registry;
mod traits;

// Engine implementations
pub mod arxiv;
pub mod bing;
pub mod brave;
pub mod duckduckgo;
pub mod github;
pub mod google;
pub mod stackoverflow;
pub mod wikipedia;
pub mod youtube;

pub use loader::EngineLoader;
pub use registry::EngineRegistry;
pub use traits::*;
