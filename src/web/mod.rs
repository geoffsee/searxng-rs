//! Web server module
//!
//! Provides the HTTP API and web interface for SearXNG-RS.

mod handlers;
mod routes;
mod state;
mod templates;

pub use routes::create_router;
pub use state::AppState;
pub use templates::Templates;
