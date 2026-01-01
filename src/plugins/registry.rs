//! Plugin registry for managing plugins

use super::traits::{Plugin, PreSearchResult};
use crate::results::{Answer, Result};
use crate::search::SearchQuery;
use std::sync::Arc;

/// Registry of all loaded plugins
pub struct PluginRegistry {
    plugins: Vec<Arc<dyn Plugin>>,
    enabled: Vec<String>,
}

impl PluginRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            enabled: Vec::new(),
        }
    }

    /// Create registry with default plugins
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register built-in plugins
        registry.register(Arc::new(super::calculator::CalculatorPlugin::new()));
        registry.register(Arc::new(super::hash_plugin::HashPlugin::new()));
        registry.register(Arc::new(super::tracker_remover::TrackerRemoverPlugin::new()));
        registry.register(Arc::new(super::unit_converter::UnitConverterPlugin::new()));

        registry
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        let info = plugin.info();
        if info.default_on {
            self.enabled.push(info.id.clone());
        }
        self.plugins.push(plugin);
    }

    /// Enable a plugin by ID
    pub fn enable(&mut self, id: &str) {
        if !self.enabled.contains(&id.to_string()) {
            self.enabled.push(id.to_string());
        }
    }

    /// Disable a plugin by ID
    pub fn disable(&mut self, id: &str) {
        self.enabled.retain(|e| e != id);
    }

    /// Check if a plugin is enabled
    pub fn is_enabled(&self, id: &str) -> bool {
        self.enabled.contains(&id.to_string())
    }

    /// Get all enabled plugins
    fn enabled_plugins(&self) -> Vec<&Arc<dyn Plugin>> {
        self.plugins
            .iter()
            .filter(|p| self.is_enabled(&p.info().id))
            .collect()
    }

    /// Run pre_search hooks on all enabled plugins
    pub fn pre_search(&self, query: &mut SearchQuery) -> Option<Answer> {
        for plugin in self.enabled_plugins() {
            match plugin.pre_search(query) {
                PreSearchResult::Continue => continue,
                PreSearchResult::Answer(answer) => return Some(answer),
                PreSearchResult::Skip => return None,
                PreSearchResult::ModifyQuery(new_query) => {
                    query.query = new_query;
                }
            }
        }
        None
    }

    /// Run on_result hooks on all enabled plugins
    pub fn on_result(&self, query: &SearchQuery, result: &mut Result) -> bool {
        for plugin in self.enabled_plugins() {
            if !plugin.on_result(query, result) {
                return false;
            }
        }
        true
    }

    /// Run post_search hooks on all enabled plugins
    pub fn post_search(&self, query: &SearchQuery, results: &mut Vec<Result>) {
        for plugin in self.enabled_plugins() {
            plugin.post_search(query, results);
        }
    }

    /// Try to get an instant answer from plugins
    pub fn try_answer(&self, query: &str) -> Option<Answer> {
        for plugin in self.enabled_plugins() {
            if plugin.matches_query(query) {
                if let Some(answer) = plugin.process(query) {
                    return Some(answer);
                }
            }
        }
        None
    }

    /// Get list of all plugins with their info
    pub fn list(&self) -> Vec<super::traits::PluginInfo> {
        self.plugins.iter().map(|p| p.info()).collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
