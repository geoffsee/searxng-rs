//! Engine registry for managing available search engines

use super::traits::Engine;
use crate::config::EngineConfig;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of all available search engines
pub struct EngineRegistry {
    /// Engines by name
    engines: HashMap<String, Arc<dyn Engine>>,
    /// Engine shortcuts (e.g., "g" -> "google")
    shortcuts: HashMap<String, String>,
    /// Engines by category
    categories: HashMap<String, Vec<String>>,
    /// Engine configurations
    configs: HashMap<String, EngineConfig>,
}

impl EngineRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
            shortcuts: HashMap::new(),
            categories: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Register an engine
    pub fn register(
        &mut self,
        engine: Arc<dyn Engine>,
        config: EngineConfig,
    ) {
        let name = engine.name().to_string();

        // Register shortcut
        if !config.shortcut.is_empty() {
            self.shortcuts.insert(config.shortcut.clone(), name.clone());
        }

        // Register in categories
        for category in engine.categories() {
            self.categories
                .entry(category.to_string())
                .or_default()
                .push(name.clone());
        }

        // Store engine and config
        self.engines.insert(name.clone(), engine);
        self.configs.insert(name, config);
    }

    /// Get an engine by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Engine>> {
        self.engines.get(name)
    }

    /// Get an engine by shortcut
    pub fn get_by_shortcut(&self, shortcut: &str) -> Option<&Arc<dyn Engine>> {
        self.shortcuts
            .get(shortcut)
            .and_then(|name| self.engines.get(name))
    }

    /// Get engine config
    pub fn get_config(&self, name: &str) -> Option<&EngineConfig> {
        self.configs.get(name)
    }

    /// Get all engines in a category
    pub fn get_by_category(&self, category: &str) -> Vec<&Arc<dyn Engine>> {
        self.categories
            .get(category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.engines.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all enabled engines
    pub fn enabled(&self) -> Vec<&Arc<dyn Engine>> {
        self.configs
            .iter()
            .filter(|(_, config)| !config.disabled)
            .filter_map(|(name, _)| self.engines.get(name))
            .collect()
    }

    /// Get all engine names
    pub fn names(&self) -> Vec<&str> {
        self.engines.keys().map(|s| s.as_str()).collect()
    }

    /// Get all category names
    pub fn category_names(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }

    /// Check if an engine exists
    pub fn contains(&self, name: &str) -> bool {
        self.engines.contains_key(name)
    }

    /// Get number of registered engines
    pub fn len(&self) -> usize {
        self.engines.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.engines.is_empty()
    }

    /// Resolve a name or shortcut to an engine name
    pub fn resolve_name<'a>(&'a self, name_or_shortcut: &'a str) -> Option<&'a str> {
        if self.engines.contains_key(name_or_shortcut) {
            Some(name_or_shortcut)
        } else {
            self.shortcuts.get(name_or_shortcut).map(|s| s.as_str())
        }
    }

    /// Get effective timeout for an engine
    pub fn get_timeout(&self, name: &str, default: f64) -> f64 {
        self.configs
            .get(name)
            .and_then(|c| c.timeout)
            .or_else(|| self.engines.get(name).map(|e| e.timeout()))
            .unwrap_or(default)
    }

    /// Get effective weight for an engine
    pub fn get_weight(&self, name: &str) -> f64 {
        self.configs
            .get(name)
            .map(|c| c.weight)
            .unwrap_or_else(|| {
                self.engines.get(name).map(|e| e.weight()).unwrap_or(1.0)
            })
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::google::Google;

    #[test]
    fn test_registry() {
        let mut registry = EngineRegistry::new();
        let google = Arc::new(Google::new()) as Arc<dyn Engine>;
        let config = EngineConfig {
            name: "google".to_string(),
            engine: "google".to_string(),
            shortcut: "g".to_string(),
            ..Default::default()
        };

        registry.register(google, config);

        assert!(registry.contains("google"));
        assert!(registry.get_by_shortcut("g").is_some());
    }
}
