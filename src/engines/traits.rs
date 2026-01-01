//! Engine traits and types

use crate::config::EngineConfig;
use crate::network::HttpClient;
use crate::query::TimeRange;
use crate::results::{Answer, InfoBox, Result, Suggestion};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of an engine search
#[derive(Debug, Clone, Default)]
pub struct EngineResults {
    /// Search results
    pub results: Vec<Result>,
    /// Direct answers
    pub answers: Vec<Answer>,
    /// Search suggestions
    pub suggestions: Vec<Suggestion>,
    /// Information boxes
    pub infoboxes: Vec<InfoBox>,
    /// Number of total results (if known)
    pub number_of_results: Option<u64>,
}

impl EngineResults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_results(results: Vec<Result>) -> Self {
        Self {
            results,
            ..Default::default()
        }
    }

    pub fn add_result(&mut self, result: Result) {
        self.results.push(result);
    }

    pub fn add_answer(&mut self, answer: Answer) {
        self.answers.push(answer);
    }

    pub fn add_suggestion(&mut self, suggestion: Suggestion) {
        self.suggestions.push(suggestion);
    }

    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
            && self.answers.is_empty()
            && self.suggestions.is_empty()
            && self.infoboxes.is_empty()
    }
}

/// Parameters for building a search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestParams {
    /// Search query string
    pub query: String,
    /// Page number (1-indexed)
    pub pageno: u32,
    /// Language code
    pub lang: String,
    /// Safe search level
    pub safesearch: u8,
    /// Time range filter
    pub time_range: Option<TimeRange>,
    /// Category context
    pub category: String,
    /// Engine-specific data
    #[serde(default)]
    pub engine_data: HashMap<String, serde_json::Value>,
}

impl RequestParams {
    /// Create new request parameters
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            pageno: 1,
            lang: "en".to_string(),
            safesearch: 0,
            time_range: None,
            category: "general".to_string(),
            engine_data: HashMap::new(),
        }
    }
}

/// HTTP request to be made by the engine
#[derive(Debug, Clone)]
pub struct EngineRequest {
    /// URL to request
    pub url: String,
    /// HTTP method
    pub method: HttpMethod,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Query parameters
    pub params: HashMap<String, String>,
    /// POST body data
    pub data: Option<RequestBody>,
    /// Cookies to send
    pub cookies: HashMap<String, String>,
}

impl EngineRequest {
    /// Create a GET request
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            params: HashMap::new(),
            data: None,
            cookies: HashMap::new(),
        }
    }

    /// Create a POST request
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            params: HashMap::new(),
            data: None,
            cookies: HashMap::new(),
        }
    }

    /// Add a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Add a query parameter
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Add form data (sets content-type to form-urlencoded)
    pub fn form(mut self, data: HashMap<String, String>) -> Self {
        self.data = Some(RequestBody::Form(data));
        self
    }

    /// Add JSON body
    pub fn json(mut self, data: serde_json::Value) -> Self {
        self.data = Some(RequestBody::Json(data));
        self
    }

    /// Add a cookie
    pub fn cookie(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.cookies.insert(key.into(), value.into());
        self
    }
}

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

/// Request body types
#[derive(Debug, Clone)]
pub enum RequestBody {
    Form(HashMap<String, String>),
    Json(serde_json::Value),
    Raw(Vec<u8>),
}

/// HTTP response from engine request
#[derive(Debug)]
pub struct EngineResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body as text
    pub text: String,
    /// Response URL (after redirects)
    pub url: String,
}

impl EngineResponse {
    /// Parse response as JSON
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<T> {
        Ok(serde_json::from_str(&self.text)?)
    }

    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if response indicates rate limiting
    pub fn is_rate_limited(&self) -> bool {
        self.status == 429
    }

    /// Check if response indicates CAPTCHA
    pub fn is_captcha(&self) -> bool {
        // Common CAPTCHA indicators
        self.text.contains("captcha")
            || self.text.contains("CAPTCHA")
            || self.text.contains("unusual traffic")
            || self.text.contains("automated requests")
    }
}

/// Main engine trait that all search engines must implement
#[async_trait]
pub trait Engine: Send + Sync {
    /// Engine name
    fn name(&self) -> &str;

    /// Short description of the engine
    fn about(&self) -> EngineAbout {
        EngineAbout::default()
    }

    /// Supported categories (e.g., "general", "images")
    fn categories(&self) -> Vec<&str> {
        vec!["general"]
    }

    /// Whether this engine supports pagination
    fn supports_paging(&self) -> bool {
        true
    }

    /// Whether this engine supports time range filtering
    fn supports_time_range(&self) -> bool {
        false
    }

    /// Whether this engine supports safe search
    fn supports_safesearch(&self) -> bool {
        false
    }

    /// Default weight for result scoring
    fn weight(&self) -> f64 {
        1.0
    }

    /// Default timeout in seconds
    fn timeout(&self) -> f64 {
        5.0
    }

    /// Number of results per page
    fn results_per_page(&self) -> u32 {
        10
    }

    /// Build the HTTP request for a search
    fn request(&self, params: &RequestParams) -> anyhow::Result<EngineRequest>;

    /// Parse the HTTP response into results
    fn response(&self, response: EngineResponse) -> anyhow::Result<EngineResults>;

    /// Optional initialization (called once on startup)
    fn init(&mut self, _config: &EngineConfig) -> anyhow::Result<()> {
        Ok(())
    }

    /// Optional validation of configuration
    fn validate(&self, _config: &EngineConfig) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Engine metadata
#[derive(Debug, Clone, Default)]
pub struct EngineAbout {
    /// Website URL
    pub website: Option<String>,
    /// Wiki/documentation URL
    pub wikidata_id: Option<String>,
    /// Whether it uses the official API
    pub use_official_api: bool,
    /// Whether an API key is required
    pub require_api_key: bool,
    /// Result format (HTML, JSON, XML)
    pub results: String,
}

impl EngineAbout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn website(mut self, url: impl Into<String>) -> Self {
        self.website = Some(url.into());
        self
    }

    pub fn official_api(mut self, uses: bool) -> Self {
        self.use_official_api = uses;
        self
    }

    pub fn api_key_required(mut self, required: bool) -> Self {
        self.require_api_key = required;
        self
    }

    pub fn results_format(mut self, format: impl Into<String>) -> Self {
        self.results = format.into();
        self
    }
}

/// Trait for engines that need async initialization
#[async_trait]
pub trait AsyncEngine: Engine {
    /// Async initialization
    async fn async_init(&mut self, client: &HttpClient) -> anyhow::Result<()>;
}
