//! Bing search engine implementation

use super::traits::*;
use crate::results::Result;
use anyhow::Result as AnyhowResult;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// Bing web search engine
pub struct Bing {
    base_url: String,
}

impl Bing {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.bing.com/search".to_string(),
        }
    }

    /// Decode Bing's encoded URLs
    /// Bing often returns URLs like https://www.bing.com/ck/a?...&u=a1<base64>&...
    /// The real URL is base64 encoded in the 'u' parameter (after removing 'a1' prefix)
    fn decode_bing_url(&self, url: &str) -> String {
        if !url.starts_with("https://www.bing.com/ck/a?") {
            return url.to_string();
        }

        // Parse URL to get query params
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(u_param) = parsed.query_pairs().find(|(k, _)| k == "u") {
                let encoded = u_param.1.to_string();
                // Remove "a1" prefix
                if encoded.len() > 2 {
                    let encoded_url = &encoded[2..];
                    // Add padding if needed
                    let padding = (4 - encoded_url.len() % 4) % 4;
                    let padded = format!("{}{}", encoded_url, "=".repeat(padding));

                    // Decode base64
                    if let Ok(decoded_bytes) = base64::Engine::decode(
                        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                        padded.trim_end_matches('='),
                    ) {
                        if let Ok(decoded_url) = String::from_utf8(decoded_bytes) {
                            return decoded_url;
                        }
                    }
                    // Try standard base64 as fallback
                    if let Ok(decoded_bytes) = base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        &padded,
                    ) {
                        if let Ok(decoded_url) = String::from_utf8(decoded_bytes) {
                            return decoded_url;
                        }
                    }
                }
            }
        }

        url.to_string()
    }

    fn parse_results(&self, html: &str) -> Vec<Result> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Bing result selectors - matching reference SearXNG implementation
        // Reference uses: //ol[@id="b_results"]/li[contains(@class, "b_algo")]
        let results_container = Selector::parse("#b_results").unwrap();
        let result_selector = Selector::parse("li.b_algo").unwrap();
        let title_selector = Selector::parse("h2 a").unwrap();
        let snippet_selector = Selector::parse("p").unwrap();

        let mut position = 1u32;

        // Try to find results in #b_results container first
        let search_area = document
            .select(&results_container)
            .next()
            .map(|e| e.html())
            .unwrap_or_else(|| document.html());

        let search_doc = Html::parse_document(&search_area);

        for element in search_doc.select(&result_selector) {
            // Get title and URL
            let title_elem = match element.select(&title_selector).next() {
                Some(t) => t,
                None => continue,
            };

            let title = title_elem.text().collect::<String>().trim().to_string();
            if title.is_empty() {
                continue;
            }

            let raw_url = title_elem
                .value()
                .attr("href")
                .map(|h| h.to_string())
                .unwrap_or_default();

            if raw_url.is_empty() || raw_url.starts_with('/') {
                continue;
            }

            // Decode Bing's encoded URLs
            let url = self.decode_bing_url(&raw_url);

            // Get snippet - filter out algoSlug_icon elements by getting text carefully
            let snippet = element
                .select(&snippet_selector)
                .next()
                .map(|s| {
                    s.text()
                        .collect::<String>()
                        .trim()
                        .to_string()
                })
                .filter(|s| !s.is_empty() && !s.contains("Web"));

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

impl Default for Bing {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for Bing {
    fn name(&self) -> &str {
        "bing"
    }

    fn about(&self) -> EngineAbout {
        EngineAbout::new()
            .website("https://www.bing.com")
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
        query_params.insert("setlang".to_string(), params.lang.clone());

        // Pagination
        if params.pageno > 1 {
            let first = ((params.pageno - 1) * 10) + 1;
            query_params.insert("first".to_string(), first.to_string());
        }

        // Time range
        if let Some(ref time_range) = params.time_range {
            let filters = match time_range {
                crate::query::TimeRange::Day => "ex1:\"ez1\"",
                crate::query::TimeRange::Week => "ex1:\"ez2\"",
                crate::query::TimeRange::Month => "ex1:\"ez3\"",
                crate::query::TimeRange::Year => "ex1:\"ez5\"",
            };
            query_params.insert("filters".to_string(), filters.to_string());
        }

        let mut request = EngineRequest::get(&self.base_url);
        request.params = query_params;

        // Safe search via cookie
        let safe_cookie = match params.safesearch {
            2 => "STRICT",
            1 => "MODERATE",
            _ => "OFF",
        };
        request = request.cookie("SRCHHPGUSR", &format!("ADLT={}", safe_cookie));

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

/// Bing Images search engine
pub struct BingImages {
    base_url: String,
}

impl BingImages {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.bing.com/images/search".to_string(),
        }
    }
}

impl Default for BingImages {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for BingImages {
    fn name(&self) -> &str {
        "bing_images"
    }

    fn about(&self) -> EngineAbout {
        EngineAbout::new()
            .website("https://www.bing.com/images")
            .official_api(false)
            .results_format("HTML")
    }

    fn categories(&self) -> Vec<&str> {
        vec!["images"]
    }

    fn supports_paging(&self) -> bool {
        true
    }

    fn supports_safesearch(&self) -> bool {
        true
    }

    fn request(&self, params: &RequestParams) -> AnyhowResult<EngineRequest> {
        let mut query_params = HashMap::new();
        query_params.insert("q".to_string(), params.query.clone());
        query_params.insert("form".to_string(), "HDRSC2".to_string());

        if params.pageno > 1 {
            let first = ((params.pageno - 1) * 35) + 1;
            query_params.insert("first".to_string(), first.to_string());
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

        // Image result parsing
        let result_selector = Selector::parse("a.iusc").unwrap();

        let mut position = 1u32;

        for element in document.select(&result_selector) {
            // Bing stores image data in a JSON attribute
            if let Some(m_attr) = element.value().attr("m") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(m_attr) {
                    let url = json.get("purl").and_then(|v| v.as_str()).unwrap_or_default();
                    let img_src = json.get("murl").and_then(|v| v.as_str()).unwrap_or_default();
                    let title = json.get("t").and_then(|v| v.as_str()).unwrap_or("Image");

                    if !url.is_empty() {
                        let mut result = Result::new(
                            url.to_string(),
                            title.to_string(),
                            self.name().to_string(),
                        );
                        result.metadata.img_src = Some(img_src.to_string());
                        result.result_type = crate::results::ResultType::Image;
                        result = result.with_position(position);
                        position += 1;

                        results.push(result);
                    }
                }
            }
        }

        Ok(EngineResults::with_results(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bing_request() {
        let bing = Bing::new();
        let params = RequestParams::new("rust programming");
        let request = bing.request(&params).unwrap();

        assert!(request.url.contains("bing.com"));
        assert!(request.params.contains_key("q"));
    }
}
