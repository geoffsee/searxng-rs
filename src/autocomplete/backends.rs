//! Autocomplete backend implementations

use crate::network::HttpClient;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait for autocomplete backends
#[async_trait]
pub trait AutocompleteBackend: Send + Sync {
    /// Backend name
    fn name(&self) -> &str;

    /// Fetch suggestions for a query
    async fn suggest(&self, client: &HttpClient, query: &str, lang: &str) -> Result<Vec<String>>;
}

/// Get a backend by name
pub fn get_backend(name: &str) -> Option<Box<dyn AutocompleteBackend>> {
    match name.to_lowercase().as_str() {
        "duckduckgo" | "ddg" => Some(Box::new(DuckDuckGo)),
        "google" => Some(Box::new(Google)),
        "wikipedia" | "wiki" => Some(Box::new(Wikipedia)),
        "brave" => Some(Box::new(Brave)),
        "qwant" => Some(Box::new(Qwant)),
        _ => None,
    }
}

/// List available backends
pub fn list_backends() -> Vec<&'static str> {
    vec!["duckduckgo", "google", "wikipedia", "brave", "qwant"]
}

/// DuckDuckGo autocomplete backend
pub struct DuckDuckGo;

#[async_trait]
impl AutocompleteBackend for DuckDuckGo {
    fn name(&self) -> &str {
        "duckduckgo"
    }

    async fn suggest(&self, client: &HttpClient, query: &str, _lang: &str) -> Result<Vec<String>> {
        let url = "https://duckduckgo.com/ac/";
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        params.insert("type".to_string(), "list".to_string());

        let response = client.get_with_params(url, params).await?;

        if !response.is_success() {
            return Ok(vec![]);
        }

        // DuckDuckGo returns: [query, [suggestions...]]
        let json: serde_json::Value = serde_json::from_str(&response.text)?;

        let suggestions = json
            .as_array()
            .and_then(|arr| arr.get(1))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(suggestions)
    }
}

/// Google autocomplete backend
pub struct Google;

#[async_trait]
impl AutocompleteBackend for Google {
    fn name(&self) -> &str {
        "google"
    }

    async fn suggest(&self, client: &HttpClient, query: &str, lang: &str) -> Result<Vec<String>> {
        let url = "https://www.google.com/complete/search";
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        params.insert("client".to_string(), "firefox".to_string());
        params.insert("hl".to_string(), lang.to_string());

        let response = client.get_with_params(url, params).await?;

        if !response.is_success() {
            return Ok(vec![]);
        }

        // Google returns: [query, [suggestions...]]
        let json: serde_json::Value = serde_json::from_str(&response.text)?;

        let suggestions = json
            .as_array()
            .and_then(|arr| arr.get(1))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(suggestions)
    }
}

/// Wikipedia autocomplete backend
pub struct Wikipedia;

#[async_trait]
impl AutocompleteBackend for Wikipedia {
    fn name(&self) -> &str {
        "wikipedia"
    }

    async fn suggest(&self, client: &HttpClient, query: &str, lang: &str) -> Result<Vec<String>> {
        // Use language-specific Wikipedia
        let wiki_lang = if lang.len() >= 2 { &lang[..2] } else { "en" };
        let url = format!("https://{}.wikipedia.org/w/api.php", wiki_lang);

        let mut params = HashMap::new();
        params.insert("action".to_string(), "opensearch".to_string());
        params.insert("format".to_string(), "json".to_string());
        params.insert("formatversion".to_string(), "2".to_string());
        params.insert("search".to_string(), query.to_string());
        params.insert("namespace".to_string(), "0".to_string());
        params.insert("limit".to_string(), "10".to_string());

        let response = client.get_with_params(&url, params).await?;

        if !response.is_success() {
            return Ok(vec![]);
        }

        // Wikipedia returns: [query, [suggestions...], [...], [...]]
        let json: serde_json::Value = serde_json::from_str(&response.text)?;

        let suggestions = json
            .as_array()
            .and_then(|arr| arr.get(1))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(suggestions)
    }
}

/// Brave autocomplete backend
pub struct Brave;

#[async_trait]
impl AutocompleteBackend for Brave {
    fn name(&self) -> &str {
        "brave"
    }

    async fn suggest(&self, client: &HttpClient, query: &str, _lang: &str) -> Result<Vec<String>> {
        let url = "https://search.brave.com/api/suggest";
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        let response = client.get_with_params(url, params).await?;

        if !response.is_success() {
            return Ok(vec![]);
        }

        // Brave returns: [query, [suggestions...]]
        let json: serde_json::Value = serde_json::from_str(&response.text)?;

        let suggestions = json
            .as_array()
            .and_then(|arr| arr.get(1))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(suggestions)
    }
}

/// Qwant autocomplete backend
pub struct Qwant;

#[async_trait]
impl AutocompleteBackend for Qwant {
    fn name(&self) -> &str {
        "qwant"
    }

    async fn suggest(&self, client: &HttpClient, query: &str, lang: &str) -> Result<Vec<String>> {
        let url = "https://api.qwant.com/v3/suggest";

        // Map language to Qwant locale format (e.g., "en" -> "en_US")
        let locale = match lang.get(..2) {
            Some("de") => "de_DE",
            Some("fr") => "fr_FR",
            Some("es") => "es_ES",
            Some("it") => "it_IT",
            Some("nl") => "nl_NL",
            Some("pt") => "pt_PT",
            _ => "en_US",
        };

        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        params.insert("locale".to_string(), locale.to_string());
        params.insert("version".to_string(), "2".to_string());

        let response = client.get_with_params(url, params).await?;

        if !response.is_success() {
            return Ok(vec![]);
        }

        // Qwant returns: {"status": "success", "data": {"items": [{"value": "..."}]}}
        let json: serde_json::Value = serde_json::from_str(&response.text)?;

        let suggestions = json
            .get("data")
            .and_then(|d| d.get("items"))
            .and_then(|items| items.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("value").and_then(|v| v.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_backends() {
        let backends = list_backends();
        assert!(backends.contains(&"duckduckgo"));
        assert!(backends.contains(&"google"));
        assert!(backends.contains(&"wikipedia"));
    }

    #[test]
    fn test_get_backend() {
        assert!(get_backend("duckduckgo").is_some());
        assert!(get_backend("ddg").is_some());
        assert!(get_backend("google").is_some());
        assert!(get_backend("wikipedia").is_some());
        assert!(get_backend("wiki").is_some());
        assert!(get_backend("unknown").is_none());
    }
}
