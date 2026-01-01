//! Tracker URL remover plugin

use super::traits::{Plugin, PluginInfo};
use crate::results::Result;
use crate::search::SearchQuery;
use regex::Regex;
use url::Url;

/// Plugin that removes tracking parameters from URLs
pub struct TrackerRemoverPlugin {
    tracking_params: Vec<&'static str>,
    tracking_patterns: Vec<Regex>,
}

impl TrackerRemoverPlugin {
    pub fn new() -> Self {
        Self {
            tracking_params: vec![
                // Google
                "utm_source",
                "utm_medium",
                "utm_campaign",
                "utm_term",
                "utm_content",
                "gclid",
                "gclsrc",
                // Facebook
                "fbclid",
                "fb_action_ids",
                "fb_action_types",
                "fb_source",
                "fb_ref",
                // Microsoft
                "msclkid",
                // Twitter
                "twclid",
                // Mailchimp
                "mc_eid",
                "mc_cid",
                // HubSpot
                "_hsenc",
                "_hsmi",
                "__hstc",
                "__hsfp",
                "hsCtaTracking",
                // Adobe
                "s_kwcid",
                // General
                "ref",
                "ref_",
                "source",
                "click_id",
                "campaign_id",
                "ad_id",
            ],
            tracking_patterns: vec![
                Regex::new(r"^utm_.*$").unwrap(),
                Regex::new(r"^_ga.*$").unwrap(),
            ],
        }
    }

    fn clean_url(&self, url: &str) -> String {
        match Url::parse(url) {
            Ok(mut parsed) => {
                // Collect query pairs, filtering out tracking params
                let cleaned_pairs: Vec<(String, String)> = parsed
                    .query_pairs()
                    .filter(|(key, _)| !self.is_tracking_param(key))
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect();

                // Clear and rebuild query string
                parsed.set_query(None);
                if !cleaned_pairs.is_empty() {
                    let query_string = cleaned_pairs
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&");
                    parsed.set_query(Some(&query_string));
                }

                parsed.to_string()
            }
            Err(_) => url.to_string(),
        }
    }

    fn is_tracking_param(&self, param: &str) -> bool {
        // Check exact matches
        if self.tracking_params.contains(&param) {
            return true;
        }

        // Check patterns
        for pattern in &self.tracking_patterns {
            if pattern.is_match(param) {
                return true;
            }
        }

        false
    }
}

impl Default for TrackerRemoverPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TrackerRemoverPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "tracker_url_remover".to_string(),
            name: "Tracker URL Remover".to_string(),
            description: "Remove tracking parameters from result URLs".to_string(),
            default_on: true,
        }
    }

    fn on_result(&self, _query: &SearchQuery, result: &mut Result) -> bool {
        // Clean the main URL
        result.url = self.clean_url(&result.url);

        // Update parsed URL
        result.parsed_url = Url::parse(&result.url).ok();

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_utm_params() {
        let plugin = TrackerRemoverPlugin::new();
        let url = "https://example.com/page?foo=bar&utm_source=google&utm_medium=cpc";
        let cleaned = plugin.clean_url(url);
        assert_eq!(cleaned, "https://example.com/page?foo=bar");
    }

    #[test]
    fn test_remove_fbclid() {
        let plugin = TrackerRemoverPlugin::new();
        let url = "https://example.com/?fbclid=IwAR123456";
        let cleaned = plugin.clean_url(url);
        assert_eq!(cleaned, "https://example.com/");
    }

    #[test]
    fn test_keep_non_tracking_params() {
        let plugin = TrackerRemoverPlugin::new();
        let url = "https://example.com/search?q=test&page=2";
        let cleaned = plugin.clean_url(url);
        assert!(cleaned.contains("q=test"));
        assert!(cleaned.contains("page=2"));
    }
}
