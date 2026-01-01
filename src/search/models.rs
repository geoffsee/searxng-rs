//! Search query and related data models

use crate::query::{ParsedQuery, TimeRange};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reference to an engine with its category context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EngineRef {
    /// Engine name
    pub name: String,
    /// Category context for this engine
    pub category: String,
}

impl EngineRef {
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: category.into(),
        }
    }
}

/// Complete search query with all parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// The search query string
    pub query: String,
    /// Engines to search
    pub engine_refs: Vec<EngineRef>,
    /// Language code
    pub lang: String,
    /// Safe search level (0, 1, 2)
    pub safesearch: u8,
    /// Page number (1-indexed)
    pub pageno: u32,
    /// Time range filter
    pub time_range: Option<TimeRange>,
    /// Custom timeout in seconds
    pub timeout_limit: Option<f64>,
    /// External bang (redirect to external search)
    pub external_bang: Option<String>,
    /// Redirect to first result
    pub redirect_to_first: bool,
    /// Per-engine state data
    #[serde(default)]
    pub engine_data: HashMap<String, serde_json::Value>,
}

impl SearchQuery {
    /// Create a new search query from parsed query
    pub fn from_parsed(parsed: ParsedQuery, default_engines: Vec<EngineRef>) -> Self {
        let engine_refs = if !parsed.engines.is_empty() {
            // Use explicitly requested engines
            parsed
                .engines
                .iter()
                .map(|e| EngineRef::new(e, "general"))
                .collect()
        } else {
            // Use default engines for requested categories (or default categories)
            default_engines
        };

        Self {
            query: parsed.query,
            engine_refs,
            lang: parsed.languages.first().cloned().unwrap_or_else(|| "all".to_string()),
            safesearch: parsed.safesearch.unwrap_or(0),
            pageno: parsed.pageno,
            time_range: parsed.time_range,
            timeout_limit: parsed.timeout,
            external_bang: parsed.external_bang,
            redirect_to_first: parsed.redirect_to_first,
            engine_data: HashMap::new(),
        }
    }

    /// Create a simple query for a single string
    pub fn simple(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            engine_refs: vec![],
            lang: "all".to_string(),
            safesearch: 0,
            pageno: 1,
            time_range: None,
            timeout_limit: None,
            external_bang: None,
            redirect_to_first: false,
            engine_data: HashMap::new(),
        }
    }

    /// Add an engine to the query
    pub fn add_engine(&mut self, name: impl Into<String>, category: impl Into<String>) {
        self.engine_refs.push(EngineRef::new(name, category));
    }

    /// Set language
    pub fn with_lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// Set safe search
    pub fn with_safesearch(mut self, level: u8) -> Self {
        self.safesearch = level.min(2);
        self
    }

    /// Set page number
    pub fn with_page(mut self, page: u32) -> Self {
        self.pageno = page.max(1);
        self
    }

    /// Set time range
    pub fn with_time_range(mut self, range: TimeRange) -> Self {
        self.time_range = Some(range);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, seconds: f64) -> Self {
        self.timeout_limit = Some(seconds);
        self
    }

    /// Get effective timeout
    pub fn effective_timeout(&self, default: f64, max: f64) -> f64 {
        self.timeout_limit
            .map(|t| t.min(max))
            .unwrap_or(default)
    }

    /// Check if query is empty
    pub fn is_empty(&self) -> bool {
        self.query.trim().is_empty()
    }

    /// Get categories from engine refs
    pub fn categories(&self) -> Vec<String> {
        self.engine_refs
            .iter()
            .map(|e| e.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Store engine-specific data
    pub fn set_engine_data(&mut self, engine: &str, key: &str, value: serde_json::Value) {
        let data = self
            .engine_data
            .entry(format!("{}:{}", engine, key))
            .or_insert(serde_json::Value::Null);
        *data = value;
    }

    /// Get engine-specific data
    pub fn get_engine_data(&self, engine: &str, key: &str) -> Option<&serde_json::Value> {
        self.engine_data.get(&format!("{}:{}", engine, key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let query = SearchQuery::simple("hello world");
        assert_eq!(query.query, "hello world");
        assert_eq!(query.pageno, 1);
        assert_eq!(query.safesearch, 0);
    }

    #[test]
    fn test_query_builder() {
        let query = SearchQuery::simple("test")
            .with_lang("en")
            .with_safesearch(2)
            .with_page(3);

        assert_eq!(query.lang, "en");
        assert_eq!(query.safesearch, 2);
        assert_eq!(query.pageno, 3);
    }

    #[test]
    fn test_engine_refs() {
        let mut query = SearchQuery::simple("test");
        query.add_engine("google", "general");
        query.add_engine("google_images", "images");

        assert_eq!(query.engine_refs.len(), 2);
        let cats = query.categories();
        assert!(cats.contains(&"general".to_string()));
        assert!(cats.contains(&"images".to_string()));
    }
}
