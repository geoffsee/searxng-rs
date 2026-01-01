//! Plugin traits and types

use crate::results::{Answer, Result};
use crate::search::SearchQuery;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Plugin information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Default enabled state
    pub default_on: bool,
}

/// Main plugin trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin info
    fn info(&self) -> PluginInfo;

    /// Keywords that trigger this plugin
    fn keywords(&self) -> Vec<&str> {
        vec![]
    }

    /// Called before search execution
    /// Return false to skip the search
    fn pre_search(&self, _query: &mut SearchQuery) -> PreSearchResult {
        PreSearchResult::Continue
    }

    /// Called for each result before aggregation
    /// Return false to filter out the result
    fn on_result(&self, _query: &SearchQuery, _result: &mut Result) -> bool {
        true
    }

    /// Called after search completion
    fn post_search(&self, _query: &SearchQuery, _results: &mut Vec<Result>) {}

    /// Check if query matches this plugin's keywords
    fn matches_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.keywords()
            .iter()
            .any(|k| query_lower.starts_with(k))
    }

    /// Process query and return an answer if applicable
    fn process(&self, _query: &str) -> Option<Answer> {
        None
    }
}

/// Result of pre_search hook
#[derive(Debug, Clone)]
pub enum PreSearchResult {
    /// Continue with normal search
    Continue,
    /// Skip search and return provided answer
    Answer(Answer),
    /// Skip search entirely
    Skip,
    /// Modify query and continue
    ModifyQuery(String),
}

/// Plugin that provides instant answers
pub trait AnswerPlugin: Plugin {
    /// Check if this plugin can answer the query
    fn can_answer(&self, query: &str) -> bool;

    /// Generate an answer for the query
    fn answer(&self, query: &str) -> Option<Answer>;
}
