//! YouTube search engine implementation (no API key required)

use super::traits::*;
use crate::results::{Result, ResultType};
use anyhow::Result as AnyhowResult;
use std::collections::HashMap;

/// YouTube video search engine
pub struct YouTube {
    base_url: String,
}

impl YouTube {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.youtube.com/results".to_string(),
        }
    }

    /// Extract text from YouTube's JSON structure
    fn get_text_from_json(element: &serde_json::Value) -> String {
        // Try "runs" format first
        if let Some(runs) = element.get("runs").and_then(|r| r.as_array()) {
            return runs
                .iter()
                .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("");
        }
        // Try "simpleText" format
        element
            .get("simpleText")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Extract the ytInitialData JSON from YouTube's HTML
    fn extract_initial_data(html: &str) -> Option<serde_json::Value> {
        // Find the ytInitialData JSON
        let start_marker = "ytInitialData = ";
        let start = html.find(start_marker)?;
        let json_start = start + start_marker.len();

        // Find the end of the JSON (ends with ";</script>")
        let end_marker = ";</script>";
        let end = html[json_start..].find(end_marker)?;

        let json_str = &html[json_start..json_start + end];
        serde_json::from_str(json_str).ok()
    }

    /// Parse video results from the initial data
    fn parse_video_results(&self, data: &serde_json::Value) -> Vec<Result> {
        let mut results = Vec::new();

        // Navigate to the contents array
        let sections = data
            .get("contents")
            .and_then(|c| c.get("twoColumnSearchResultsRenderer"))
            .and_then(|r| r.get("primaryContents"))
            .and_then(|p| p.get("sectionListRenderer"))
            .and_then(|s| s.get("contents"))
            .and_then(|c| c.as_array());

        let sections = match sections {
            Some(s) => s,
            None => return results,
        };

        let mut position = 1u32;

        for section in sections {
            // Get video containers from itemSectionRenderer
            let contents = section
                .get("itemSectionRenderer")
                .and_then(|r| r.get("contents"))
                .and_then(|c| c.as_array());

            let contents = match contents {
                Some(c) => c,
                None => continue,
            };

            for container in contents {
                // Get the video renderer
                let video = match container.get("videoRenderer") {
                    Some(v) => v,
                    None => continue,
                };

                // Get video ID
                let video_id = match video.get("videoId").and_then(|v| v.as_str()) {
                    Some(id) => id,
                    None => continue,
                };

                // Build the URL
                let url = format!("https://www.youtube.com/watch?v={}", video_id);

                // Get title
                let title = Self::get_text_from_json(
                    video.get("title").unwrap_or(&serde_json::Value::Null),
                );

                if title.is_empty() {
                    continue;
                }

                // Get description/content
                let content = Self::get_text_from_json(
                    video
                        .get("descriptionSnippet")
                        .unwrap_or(&serde_json::Value::Null),
                );

                // Get author/channel
                let author = Self::get_text_from_json(
                    video.get("ownerText").unwrap_or(&serde_json::Value::Null),
                );

                // Get duration
                let duration = Self::get_text_from_json(
                    video.get("lengthText").unwrap_or(&serde_json::Value::Null),
                );

                // Get view count
                let views = Self::get_text_from_json(
                    video
                        .get("viewCountText")
                        .unwrap_or(&serde_json::Value::Null),
                );

                // Build thumbnail URL
                let thumbnail = format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", video_id);

                // Build iframe URL for embedding
                let iframe_src = format!("https://www.youtube-nocookie.com/embed/{}", video_id);

                // Create result
                let mut result = Result::new(url, title, self.name().to_string());
                result.result_type = ResultType::Video;
                result = result.with_position(position);

                if !content.is_empty() {
                    result = result.with_content(content);
                }

                result.metadata.thumbnail = Some(thumbnail);
                result.metadata.author = if author.is_empty() {
                    None
                } else {
                    Some(author)
                };
                result.metadata.duration = if duration.is_empty() {
                    None
                } else {
                    Some(duration)
                };
                result.metadata.iframe_src = Some(iframe_src);
                result.metadata.template = Some("videos.html".to_string());

                // Parse view count
                if !views.is_empty() {
                    // Extract number from view count text
                    let views_num: String = views.chars().filter(|c| c.is_ascii_digit()).collect();
                    if let Ok(v) = views_num.parse::<u64>() {
                        result.metadata.views = Some(v);
                    }
                }

                results.push(result);
                position += 1;
            }
        }

        results
    }
}

impl Default for YouTube {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine for YouTube {
    fn name(&self) -> &str {
        "youtube"
    }

    fn about(&self) -> EngineAbout {
        EngineAbout::new()
            .website("https://www.youtube.com")
            .official_api(false)
            .results_format("HTML")
    }

    fn categories(&self) -> Vec<&str> {
        vec!["videos", "music"]
    }

    fn supports_paging(&self) -> bool {
        true
    }

    fn supports_time_range(&self) -> bool {
        true
    }

    fn request(&self, params: &RequestParams) -> AnyhowResult<EngineRequest> {
        let mut query_params = HashMap::new();
        query_params.insert("search_query".to_string(), params.query.clone());

        // Time range filter using YouTube's sp parameter
        if let Some(ref time_range) = params.time_range {
            let sp = match time_range {
                crate::query::TimeRange::Day => "EgIIAg%3D%3D", // Last hour
                crate::query::TimeRange::Week => "EgIIAw%3D%3D", // This week
                crate::query::TimeRange::Month => "EgIIBA%3D%3D", // This month
                crate::query::TimeRange::Year => "EgIIBQ%3D%3D", // This year
            };
            query_params.insert("sp".to_string(), sp.to_string());
        }

        let mut request = EngineRequest::get(&self.base_url);
        request.params = query_params;

        // Set CONSENT cookie to bypass consent page
        request
            .cookies
            .insert("CONSENT".to_string(), "YES+".to_string());

        Ok(request)
    }

    fn response(&self, response: EngineResponse) -> AnyhowResult<EngineResults> {
        if !response.is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status));
        }

        // Extract ytInitialData from the page
        let data = Self::extract_initial_data(&response.text);

        let results = match data {
            Some(d) => self.parse_video_results(&d),
            None => {
                tracing::warn!("Could not extract ytInitialData from YouTube response");
                vec![]
            }
        };

        Ok(EngineResults::with_results(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_request() {
        let youtube = YouTube::new();
        let params = RequestParams::new("rust programming");
        let request = youtube.request(&params).unwrap();

        assert!(request.url.contains("youtube.com"));
        assert!(request.params.contains_key("search_query"));
        assert!(request.cookies.contains_key("CONSENT"));
    }

    #[test]
    fn test_get_text_from_json() {
        // Test simpleText format
        let simple = serde_json::json!({
            "simpleText": "Hello World"
        });
        assert_eq!(YouTube::get_text_from_json(&simple), "Hello World");

        // Test runs format
        let runs = serde_json::json!({
            "runs": [
                {"text": "Hello "},
                {"text": "World"}
            ]
        });
        assert_eq!(YouTube::get_text_from_json(&runs), "Hello World");
    }
}
