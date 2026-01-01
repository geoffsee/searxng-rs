//! HTTP networking module
//!
//! Provides HTTP client functionality for making requests to search engines.

mod client;
mod user_agent;

pub use client::HttpClient;
pub use user_agent::generate_user_agent;
