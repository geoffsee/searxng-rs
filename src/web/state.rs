//! Application state shared across handlers

use crate::config::Settings;
use crate::engines::EngineRegistry;
use crate::network::HttpClient;
use crate::search::Search;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Global settings
    pub settings: Arc<Settings>,
    /// Engine registry
    pub registry: Arc<EngineRegistry>,
    /// Search executor
    pub search: Arc<Search>,
    /// Template renderer
    pub templates: Arc<super::Templates>,
    /// HTTP client for autocomplete and other requests
    pub http_client: Arc<HttpClient>,
}

impl AppState {
    /// Create new application state
    pub fn new(
        settings: Settings,
        registry: EngineRegistry,
        client: HttpClient,
    ) -> anyhow::Result<Self> {
        let settings = Arc::new(settings);
        let registry = Arc::new(registry);
        let http_client = Arc::new(client.clone());
        let search = Arc::new(Search::new(client, registry.clone()));
        let templates = Arc::new(super::Templates::new()?);

        Ok(Self {
            settings,
            registry,
            search,
            templates,
            http_client,
        })
    }

    /// Get instance name
    pub fn instance_name(&self) -> &str {
        &self.settings.general.instance_name
    }

    /// Check if instance is public
    pub fn is_public(&self) -> bool {
        self.settings.server.public_instance
    }

    /// Get configured autocomplete backend name
    pub fn autocomplete_backend(&self) -> Option<&str> {
        self.settings.search.autocomplete.as_deref()
    }
}
