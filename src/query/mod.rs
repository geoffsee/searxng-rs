//! Query parsing module
//!
//! Handles parsing of user queries including special syntax like:
//! - Language specifiers: `:en`, `:de`
//! - Category/engine bangs: `!images`, `!google`
//! - External bangs: `!g`, `!yt`
//! - Timeout specifiers: `<3`
//! - Safe search toggle: `!safesearch`
//! - Time range: `!day`, `!week`, `!month`, `!year`

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Parsed search query with extracted special syntax
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedQuery {
    /// The cleaned search query (without special syntax)
    pub query: String,
    /// Original raw query
    pub raw_query: String,
    /// Detected language codes
    pub languages: Vec<String>,
    /// Requested categories
    pub categories: Vec<String>,
    /// Specific engines requested
    pub engines: Vec<String>,
    /// External bang (e.g., !g for Google redirect)
    pub external_bang: Option<String>,
    /// Custom timeout in seconds
    pub timeout: Option<f64>,
    /// Safe search override (None = use default)
    pub safesearch: Option<u8>,
    /// Time range filter
    pub time_range: Option<TimeRange>,
    /// Page number
    pub pageno: u32,
    /// Redirect to first result
    pub redirect_to_first: bool,
}

impl ParsedQuery {
    /// Parse a raw query string
    pub fn parse(raw: &str) -> Self {
        let mut query = raw.to_string();
        let mut languages = Vec::new();
        let mut categories = Vec::new();
        let mut engines = Vec::new();
        let mut external_bang = None;
        let mut timeout = None;
        let mut safesearch = None;
        let mut time_range = None;
        let mut redirect_to_first = false;

        // Parse language specifiers :xx or :xx-XX
        let lang_re = Regex::new(r":([a-z]{2}(?:-[A-Z]{2})?)(?:\s|$)").unwrap();
        for cap in lang_re.captures_iter(&query) {
            languages.push(cap[1].to_string());
        }
        query = lang_re.replace_all(&query, " ").to_string();

        // Parse timeout <N or <Nms
        let timeout_re = Regex::new(r"<(\d+(?:\.\d+)?)(ms)?(?:\s|$)").unwrap();
        if let Some(cap) = timeout_re.captures(&query) {
            let value: f64 = cap[1].parse().unwrap_or(5.0);
            timeout = Some(if cap.get(2).is_some() {
                value / 1000.0
            } else {
                value
            });
        }
        query = timeout_re.replace_all(&query, " ").to_string();

        // Parse safesearch toggle
        if query.contains("!safesearch") {
            safesearch = Some(2); // Strict
            query = query.replace("!safesearch", " ");
        }
        if query.contains("!nosafesearch") {
            safesearch = Some(0); // Off
            query = query.replace("!nosafesearch", " ");
        }

        // Parse time range
        let time_ranges = [
            ("!day", TimeRange::Day),
            ("!week", TimeRange::Week),
            ("!month", TimeRange::Month),
            ("!year", TimeRange::Year),
        ];
        for (pattern, range) in time_ranges {
            if query.contains(pattern) {
                time_range = Some(range);
                query = query.replace(pattern, " ");
                break;
            }
        }

        // Parse redirect to first result
        if query.starts_with('!') && query.chars().nth(1).map(|c| c == ' ').unwrap_or(true) {
            redirect_to_first = true;
            query = query.trim_start_matches('!').to_string();
        }
        if query.starts_with("!!") {
            redirect_to_first = true;
            query = query.trim_start_matches("!!").to_string();
        }

        // Parse category bangs (!images, !videos, etc.)
        let category_bangs = [
            ("!images", "images"),
            ("!videos", "videos"),
            ("!news", "news"),
            ("!music", "music"),
            ("!files", "files"),
            ("!it", "it"),
            ("!science", "science"),
            ("!social", "social"),
            ("!maps", "maps"),
        ];
        for (bang, category) in category_bangs {
            if query.contains(bang) {
                categories.push(category.to_string());
                query = query.replace(bang, " ");
            }
        }

        // Parse engine bangs (!google, !ddg, etc.)
        let engine_re = Regex::new(r"!(\w+)(?:\s|$)").unwrap();
        let engine_bangs = Self::get_engine_bangs();
        let mut remaining_bangs = Vec::new();

        for cap in engine_re.captures_iter(&query) {
            let bang = cap[1].to_lowercase();
            if let Some(engine) = engine_bangs.get(bang.as_str()) {
                engines.push(engine.to_string());
            } else if Self::is_external_bang(&bang) {
                external_bang = Some(bang);
            } else {
                remaining_bangs.push(format!("!{}", bang));
            }
        }

        // Remove processed bangs
        query = engine_re.replace_all(&query, " ").to_string();

        // Add back unrecognized bangs
        for bang in remaining_bangs {
            query = format!("{} {}", bang, query);
        }

        // Clean up whitespace
        query = query.split_whitespace().collect::<Vec<_>>().join(" ");

        Self {
            query,
            raw_query: raw.to_string(),
            languages,
            categories,
            engines,
            external_bang,
            timeout,
            safesearch,
            time_range,
            pageno: 1,
            redirect_to_first,
        }
    }

    /// Get map of engine shortcuts to engine names
    fn get_engine_bangs() -> std::collections::HashMap<&'static str, &'static str> {
        let mut map = std::collections::HashMap::new();
        map.insert("g", "google");
        map.insert("google", "google");
        map.insert("ddg", "duckduckgo");
        map.insert("duckduckgo", "duckduckgo");
        map.insert("bi", "bing");
        map.insert("bing", "bing");
        map.insert("br", "brave");
        map.insert("brave", "brave");
        map.insert("wp", "wikipedia");
        map.insert("wikipedia", "wikipedia");
        map.insert("yt", "youtube");
        map.insert("youtube", "youtube");
        map.insert("gh", "github");
        map.insert("github", "github");
        map.insert("so", "stackoverflow");
        map.insert("stackoverflow", "stackoverflow");
        map.insert("arx", "arxiv");
        map.insert("arxiv", "arxiv");
        map
    }

    /// Check if a bang should redirect to external site
    fn is_external_bang(bang: &str) -> bool {
        // External bangs (redirect to external search)
        let external = ["g", "yt", "w", "wa", "amazon", "imdb"];
        external.contains(&bang)
    }

    /// Check if query is empty after parsing
    pub fn is_empty(&self) -> bool {
        self.query.trim().is_empty()
    }

    /// Get the effective categories (requested or default)
    pub fn effective_categories(&self, default: &[String]) -> Vec<String> {
        if self.categories.is_empty() {
            default.to_vec()
        } else {
            self.categories.clone()
        }
    }
}

/// Time range filter for search results
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TimeRange {
    Day,
    Week,
    Month,
    Year,
}

impl TimeRange {
    /// Get the string representation for API calls
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        }
    }
}

impl std::fmt::Display for TimeRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query() {
        let parsed = ParsedQuery::parse("hello world");
        assert_eq!(parsed.query, "hello world");
        assert!(parsed.languages.is_empty());
        assert!(parsed.categories.is_empty());
    }

    #[test]
    fn test_language_parsing() {
        let parsed = ParsedQuery::parse("hello :en world");
        assert_eq!(parsed.query, "hello world");
        assert_eq!(parsed.languages, vec!["en"]);
    }

    #[test]
    fn test_timeout_parsing() {
        let parsed = ParsedQuery::parse("hello <3 world");
        assert_eq!(parsed.query, "hello world");
        assert_eq!(parsed.timeout, Some(3.0));
    }

    #[test]
    fn test_category_bang() {
        let parsed = ParsedQuery::parse("rust tutorial !images");
        assert_eq!(parsed.query, "rust tutorial");
        assert_eq!(parsed.categories, vec!["images"]);
    }

    #[test]
    fn test_engine_bang() {
        let parsed = ParsedQuery::parse("rust !google");
        assert_eq!(parsed.query, "rust");
        assert_eq!(parsed.engines, vec!["google"]);
    }

    #[test]
    fn test_time_range() {
        let parsed = ParsedQuery::parse("news !week");
        assert_eq!(parsed.query, "news");
        assert_eq!(parsed.time_range, Some(TimeRange::Week));
    }

    #[test]
    fn test_safesearch() {
        let parsed = ParsedQuery::parse("query !safesearch");
        assert_eq!(parsed.safesearch, Some(2));
    }
}
