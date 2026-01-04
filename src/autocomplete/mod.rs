//! Autocomplete backends for search suggestions
//!
//! Provides autocomplete functionality from various search engines.

mod backends;

pub use backends::{get_backend, list_backends, AutocompleteBackend};

use crate::network::HttpClient;
use anyhow::Result;

/// Fetch autocomplete suggestions from a backend
pub async fn fetch_suggestions(
    client: &HttpClient,
    backend: &str,
    query: &str,
    lang: &str,
) -> Result<Vec<String>> {
    let backend = get_backend(backend)
        .ok_or_else(|| anyhow::anyhow!("Unknown autocomplete backend: {}", backend))?;

    backend.suggest(client, query, lang).await
}
