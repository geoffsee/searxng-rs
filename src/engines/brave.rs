//! Brave search engine implementation

use super::traits::*;
use crate::results::Result;
use anyhow::Result as AnyhowResult;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// Brave web search engine
pub struct Brave {
    base_url: String,
}

impl Brave {
    pub fn new() -> Self {
        Self {
            base_url: "https://search.brave.com/search".to_string(),
        }
    }

    fn parse_results(&self, html: &str) -> Vec<Result> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Brave result selectors
        let result_selector = Selector::parse("div.snippet").unwrap();
        let title_selector = Selector::parse("a.result-header").unwrap();
        let snippet_selector = Selector::parse("p.snippet-description").unwrap();

        let mut position = 1u32;

        for element in document.select(&result_selector) {
            // Get title and URL
            let title_elem = match element.select(&title_selector).next() {
                Some(t) => t,
                None => continue,
            };

            let title = title_elem.text().collect::<String>().trim().to_string();
            if title.is_empty() {
                continue;
            }

            let url = title_elem
                .value()
                .attr("href")
                .map(|h| h.to_string())
                .unwrap_or_default();

            if url.is_empty() || url.starts_with('/') {
                continue;
            }

            // Get snippet
            let snippet = element
                .select(&snippet_selector)
                .next()
                .map(|s| s.text().collect::<String>().trim().to_string());

            let mut result = Result::new(url, title, self.name().to_string());
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

impl Default for Brave {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for Brave {
    fn name(&self) -> &str {
        "brave"
    }

    fn about(&self) -> EngineAbout {
        EngineAbout::new()
            .website("https://search.brave.com")
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
        query_params.insert("source".to_string(), "web".to_string());

        // Pagination
        if params.pageno > 1 {
            let offset = (params.pageno - 1) * 20;
            query_params.insert("offset".to_string(), offset.to_string());
        }

        // Time range
        if let Some(ref time_range) = params.time_range {
            let tf = match time_range {
                crate::query::TimeRange::Day => "pd",
                crate::query::TimeRange::Week => "pw",
                crate::query::TimeRange::Month => "pm",
                crate::query::TimeRange::Year => "py",
            };
            query_params.insert("tf".to_string(), tf.to_string());
        }

        // Safe search
        let safesearch = match params.safesearch {
            2 => "strict",
            1 => "moderate",
            _ => "off",
        };
        query_params.insert("safesearch".to_string(), safesearch.to_string());

        let mut request = EngineRequest::get(&self.base_url);
        request.params = query_params;

        Ok(request)
    }

    fn response(&self, response: EngineResponse) -> AnyhowResult<EngineResults> {
        if !response.is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status));
        }

        let results = self.parse_results(&response.text);
        Ok(EngineResults::with_results(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brave_request() {
        let brave = Brave::new();
        let params = RequestParams::new("rust programming");
        let request = brave.request(&params).unwrap();

        assert!(request.url.contains("brave.com"));
        assert!(request.params.contains_key("q"));
    }
}
