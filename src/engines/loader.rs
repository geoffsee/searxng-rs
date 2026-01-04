//! Engine loader for initializing engines from configuration

use super::registry::EngineRegistry;
use super::traits::Engine;
use super::{arxiv, bing, brave, duckduckgo, github, google, stackoverflow, wikipedia, youtube};
use crate::config::{EngineConfig, Settings};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

/// Loader for initializing engines from configuration
pub struct EngineLoader;

impl EngineLoader {
    /// Load all engines from settings
    pub fn load(settings: &Settings) -> Result<EngineRegistry> {
        let mut registry = EngineRegistry::new();

        for config in &settings.engines {
            if config.disabled {
                info!("Skipping disabled engine: {}", config.name);
                continue;
            }

            match Self::create_engine(&config.engine, config) {
                Ok(engine) => {
                    info!("Loaded engine: {} ({})", config.name, config.engine);
                    registry.register(engine, config.clone());
                }
                Err(e) => {
                    warn!("Failed to load engine {}: {}", config.name, e);
                }
            }
        }

        info!("Loaded {} engines", registry.len());
        Ok(registry)
    }

    /// Create an engine instance by name
    fn create_engine(engine_type: &str, config: &EngineConfig) -> Result<Arc<dyn Engine>> {
        let mut engine: Box<dyn Engine> = match engine_type {
            "google" => Box::new(google::Google::new()),
            "google_images" => Box::new(google::GoogleImages::new()),
            "google_news" => Box::new(google::GoogleNews::new()),
            "duckduckgo" => Box::new(duckduckgo::DuckDuckGo::new()),
            "bing" => Box::new(bing::Bing::new()),
            "bing_images" => Box::new(bing::BingImages::new()),
            "brave" => Box::new(brave::Brave::new()),
            "wikipedia" => Box::new(wikipedia::Wikipedia::new()),
            "youtube" => Box::new(youtube::YouTube::new()),
            "github" => Box::new(github::GitHub::new()),
            "stackoverflow" => Box::new(stackoverflow::StackOverflow::new()),
            "arxiv" => Box::new(arxiv::ArXiv::new()),
            _ => {
                return Err(anyhow::anyhow!("Unknown engine type: {}", engine_type));
            }
        };

        // Initialize the engine
        engine.init(config)?;

        // Validate configuration
        engine.validate(config)?;

        Ok(Arc::from(engine))
    }

    /// Get list of available engine types
    pub fn available_engines() -> Vec<&'static str> {
        vec![
            "google",
            "google_images",
            "google_news",
            "duckduckgo",
            "bing",
            "bing_images",
            "brave",
            "wikipedia",
            "youtube",
            "github",
            "stackoverflow",
            "arxiv",
        ]
    }
}
