//! StackOverflow search engine implementation
//!
//! Uses the StackExchange API to search for questions on StackOverflow.

use super::traits::*;
use crate::results::Result;
use anyhow::Result as AnyhowResult;
use std::collections::HashMap;

/// StackOverflow search engine
pub struct StackOverflow {
    api_url: String,
    site: String,
}

impl StackOverflow {
    pub fn new() -> Self {
        Self {
            api_url: "https://api.stackexchange.com/2.3/search/advanced".to_string(),
            site: "stackoverflow".to_string(),
        }
    }

    /// Create a StackExchange engine for a different site
    pub fn with_site(site: impl Into<String>) -> Self {
        Self {
            api_url: "https://api.stackexchange.com/2.3/search/advanced".to_string(),
            site: site.into(),
        }
    }

    /// Unescape HTML entities
    fn unescape_html(s: &str) -> String {
        s.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'")
    }
}

impl Default for StackOverflow {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for StackOverflow {
    fn name(&self) -> &str {
        "stackoverflow"
    }

    fn about(&self) -> EngineAbout {
        EngineAbout::new()
            .website("https://stackoverflow.com")
            .official_api(true)
            .results_format("JSON")
    }

    fn categories(&self) -> Vec<&str> {
        vec!["it", "q&a"]
    }

    fn supports_paging(&self) -> bool {
        true
    }

    fn request(&self, params: &RequestParams) -> AnyhowResult<EngineRequest> {
        let mut query_params = HashMap::new();
        query_params.insert("q".to_string(), params.query.clone());
        query_params.insert("site".to_string(), self.site.clone());
        query_params.insert("sort".to_string(), "relevance".to_string());
        query_params.insert("order".to_string(), "desc".to_string());
        query_params.insert("pagesize".to_string(), "10".to_string());
        query_params.insert("page".to_string(), params.pageno.to_string());

        // Include tags, owner, and answer info in the response
        query_params.insert(
            "filter".to_string(),
            "!-*jbN-o9Aeie".to_string(), // Filter for basic question info
        );

        let mut request = EngineRequest::get(&self.api_url);
        request.params = query_params;

        Ok(request)
    }

    fn response(&self, response: EngineResponse) -> AnyhowResult<EngineResults> {
        if !response.is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status));
        }

        let json: serde_json::Value = serde_json::from_str(&response.text)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;

        // Check for API errors
        if let Some(error_id) = json.get("error_id") {
            let error_msg = json
                .get("error_message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(anyhow::anyhow!(
                "StackExchange API error {}: {}",
                error_id,
                error_msg
            ));
        }

        let items = json
            .get("items")
            .and_then(|i| i.as_array())
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        let mut position = 1u32;

        for item in items {
            // Get question ID
            let question_id = match item.get("question_id").and_then(|id| id.as_u64()) {
                Some(id) => id,
                None => continue,
            };

            // Build URL
            let url = format!("https://{}.com/q/{}", self.site, question_id);

            // Get title
            let title = item
                .get("title")
                .and_then(|t| t.as_str())
                .map(Self::unescape_html)
                .unwrap_or_default();

            if title.is_empty() {
                continue;
            }

            // Build content from tags and metadata
            let mut content_parts = Vec::new();

            // Add tags
            if let Some(tags) = item.get("tags").and_then(|t| t.as_array()) {
                let tag_str: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
                if !tag_str.is_empty() {
                    content_parts.push(format!("[{}]", tag_str.join(", ")));
                }
            }

            // Add author
            if let Some(owner) = item.get("owner") {
                if let Some(name) = owner.get("display_name").and_then(|n| n.as_str()) {
                    content_parts.push(Self::unescape_html(name));
                }
            }

            // Add answered status
            let is_answered = item
                .get("is_answered")
                .and_then(|a| a.as_bool())
                .unwrap_or(false);

            if is_answered {
                content_parts.push("✓ answered".to_string());
            }

            // Add score
            if let Some(score) = item.get("score").and_then(|s| s.as_i64()) {
                content_parts.push(format!("score: {}", score));
            }

            let content = if content_parts.is_empty() {
                None
            } else {
                Some(content_parts.join(" // "))
            };

            // Get view count
            let views = item.get("view_count").and_then(|v| v.as_u64());

            // Get author avatar
            let thumbnail = item
                .get("owner")
                .and_then(|o| o.get("profile_image"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());

            // Build result
            let mut result = Result::new(url, title, self.name().to_string());
            result = result.with_position(position);

            if let Some(c) = content {
                result = result.with_content(c);
            }

            result.metadata.thumbnail = thumbnail;
            result.metadata.views = views;

            results.push(result);
            position += 1;
        }

        Ok(EngineResults::with_results(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stackoverflow_request() {
        let so = StackOverflow::new();
        let params = RequestParams::new("rust async");
        let request = so.request(&params).unwrap();

        assert!(request.url.contains("api.stackexchange.com"));
        assert!(request.params.contains_key("q"));
        assert_eq!(
            request.params.get("site"),
            Some(&"stackoverflow".to_string())
        );
    }

    #[test]
    fn test_unescape_html() {
        assert_eq!(
            StackOverflow::unescape_html("&amp;&lt;&gt;&quot;&#39;"),
            "&<>\"'"
        );
    }
}
