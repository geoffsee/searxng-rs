//! Google search engine implementation

use super::traits::*;
use crate::results::Result;
use anyhow::Result as AnyhowResult;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// Google web search engine
pub struct Google {
    base_url: String,
}

impl Google {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.google.com/search".to_string(),
        }
    }

    fn parse_results(&self, html: &str, engine_name: &str) -> Vec<Result> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Main result selector
        let result_selector = Selector::parse("div.g").unwrap();
        let title_selector = Selector::parse("h3").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let snippet_selector = Selector::parse("div.VwiC3b, span.aCOpRe").unwrap();

        let mut position = 1u32;

        for element in document.select(&result_selector) {
            // Get title
            let title = element
                .select(&title_selector)
                .next()
                .map(|t| t.text().collect::<String>())
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            // Get URL
            let url = element
                .select(&link_selector)
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(|h| h.to_string())
                .unwrap_or_default();

            if url.is_empty() || url.starts_with('/') || url.starts_with('#') {
                continue;
            }

            // Get snippet
            let snippet = element
                .select(&snippet_selector)
                .next()
                .map(|s| s.text().collect::<String>());

            let mut result = Result::new(url, title, engine_name.to_string());
            if let Some(content) = snippet {
                result = result.with_content(content);
            }
            result = result.with_position(position);
            position += 1;

            results.push(result);
        }

        results
    }
}

impl Default for Google {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for Google {
    fn name(&self) -> &str {
        "google"
    }

    fn about(&self) -> EngineAbout {
        EngineAbout::new()
            .website("https://www.google.com")
            .official_api(false)
            .results_format("HTML")
    }

    fn categories(&self) -> Vec<&str> {
        vec!["general", "web"]
    }

    fn supports_paging(&self) -> bool {
        true
    }

    fn supports_time_range(&self) -> bool {
        true
    }

    fn supports_safesearch(&self) -> bool {
        true
    }

    fn request(&self, params: &RequestParams) -> AnyhowResult<EngineRequest> {
        let mut query_params = HashMap::new();
        query_params.insert("q".to_string(), params.query.clone());
        query_params.insert("hl".to_string(), params.lang.clone());
        query_params.insert("num".to_string(), "10".to_string());

        // Pagination
        if params.pageno > 1 {
            let start = (params.pageno - 1) * 10;
            query_params.insert("start".to_string(), start.to_string());
        }

        // Safe search
        match params.safesearch {
            2 => {
                query_params.insert("safe".to_string(), "active".to_string());
            }
            1 => {
                query_params.insert("safe".to_string(), "medium".to_string());
            }
            _ => {}
        }

        // Time range
        if let Some(ref time_range) = params.time_range {
            let tbs = match time_range {
                crate::query::TimeRange::Day => "qdr:d",
                crate::query::TimeRange::Week => "qdr:w",
                crate::query::TimeRange::Month => "qdr:m",
                crate::query::TimeRange::Year => "qdr:y",
            };
            query_params.insert("tbs".to_string(), tbs.to_string());
        }

        let mut request = EngineRequest::get(&self.base_url);
        request.params = query_params;

        Ok(request)
    }

    fn response(&self, response: EngineResponse) -> AnyhowResult<EngineResults> {
        if !response.is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status));
        }

        if response.is_captcha() {
            return Err(anyhow::anyhow!("CAPTCHA detected"));
        }

        let results = self.parse_results(&response.text, self.name());
        Ok(EngineResults::with_results(results))
    }
}

/// Google Images search engine
pub struct GoogleImages {
    base_url: String,
}

impl GoogleImages {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.google.com/search".to_string(),
        }
    }
}

impl Default for GoogleImages {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for GoogleImages {
    fn name(&self) -> &str {
        "google_images"
    }

    fn about(&self) -> EngineAbout {
        EngineAbout::new()
            .website("https://images.google.com")
            .official_api(false)
            .results_format("HTML")
    }

    fn categories(&self) -> Vec<&str> {
        vec!["images"]
    }

    fn supports_safesearch(&self) -> bool {
        true
    }

    fn request(&self, params: &RequestParams) -> AnyhowResult<EngineRequest> {
        let mut query_params = HashMap::new();
        query_params.insert("q".to_string(), params.query.clone());
        query_params.insert("tbm".to_string(), "isch".to_string());
        query_params.insert("hl".to_string(), params.lang.clone());

        if params.safesearch >= 2 {
            query_params.insert("safe".to_string(), "active".to_string());
        }

        let mut request = EngineRequest::get(&self.base_url);
        request.params = query_params;

        Ok(request)
    }

    fn response(&self, response: EngineResponse) -> AnyhowResult<EngineResults> {
        if !response.is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status));
        }

        // Parse image results from the HTML
        let document = Html::parse_document(&response.text);
        let mut results = Vec::new();

        // Google images uses complex JSON embedded in the page
        // For now, return empty - would need more complex parsing
        // TODO: Implement proper Google Images parsing

        Ok(EngineResults::with_results(results))
    }
}

/// Google News search engine
pub struct GoogleNews {
    base_url: String,
}

impl GoogleNews {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.google.com/search".to_string(),
        }
    }
}

impl Default for GoogleNews {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for GoogleNews {
    fn name(&self) -> &str {
        "google_news"
    }

    fn about(&self) -> EngineAbout {
        EngineAbout::new()
            .website("https://news.google.com")
            .official_api(false)
            .results_format("HTML")
    }

    fn categories(&self) -> Vec<&str> {
        vec!["news"]
    }

    fn supports_time_range(&self) -> bool {
        true
    }

    fn request(&self, params: &RequestParams) -> AnyhowResult<EngineRequest> {
        let mut query_params = HashMap::new();
        query_params.insert("q".to_string(), params.query.clone());
        query_params.insert("tbm".to_string(), "nws".to_string());
        query_params.insert("hl".to_string(), params.lang.clone());

        if let Some(ref time_range) = params.time_range {
            let tbs = match time_range {
                crate::query::TimeRange::Day => "qdr:d",
                crate::query::TimeRange::Week => "qdr:w",
                crate::query::TimeRange::Month => "qdr:m",
                crate::query::TimeRange::Year => "qdr:y",
            };
            query_params.insert("tbs".to_string(), tbs.to_string());
        }

        let mut request = EngineRequest::get(&self.base_url);
        request.params = query_params;

        Ok(request)
    }

    fn response(&self, response: EngineResponse) -> AnyhowResult<EngineResults> {
        if !response.is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status));
        }

        let document = Html::parse_document(&response.text);
        let mut results = Vec::new();

        // News result parsing
        let result_selector = Selector::parse("div.SoaBEf, div.xuvV6b").unwrap();
        let title_selector = Selector::parse("div.mCBkyc, div.n0jPhd").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let snippet_selector = Selector::parse("div.GI74Re, div.Y3v8qd").unwrap();

        let mut position = 1u32;

        for element in document.select(&result_selector) {
            let title = element
                .select(&title_selector)
                .next()
                .map(|t| t.text().collect::<String>())
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            let url = element
                .select(&link_selector)
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(|h| h.to_string())
                .unwrap_or_default();

            if url.is_empty() {
                continue;
            }

            let snippet = element
                .select(&snippet_selector)
                .next()
                .map(|s| s.text().collect::<String>());

            let mut result = Result::new(url, title, self.name().to_string());
            if let Some(content) = snippet {
                result = result.with_content(content);
            }
            result = result.with_position(position);
            result.result_type = crate::results::ResultType::News;
            position += 1;

            results.push(result);
        }

        Ok(EngineResults::with_results(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_request() {
        let google = Google::new();
        let params = RequestParams::new("rust programming");
        let request = google.request(&params).unwrap();

        assert!(request.url.contains("google.com"));
        assert!(request.params.contains_key("q"));
    }
}
