//! SearXNG-RS: A privacy-respecting metasearch engine written in Rust
//!
//! This is the main entry point for the application.

use anyhow::Result;
use searxng_rs::{
    config::{self, Settings},
    engines::EngineLoader,
    network::HttpClient,
    web::{create_router, AppState},
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting SearXNG-RS v{}", searxng_rs::VERSION);

    // Load configuration
    let settings = load_settings()?;
    info!("Loaded configuration for instance: {}", settings.general.instance_name);

    // Initialize HTTP client
    let client = HttpClient::with_settings(&settings.outgoing)?;
    info!("HTTP client initialized");

    // Load engines
    let registry = EngineLoader::load(&settings)?;
    info!("Loaded {} search engines", registry.len());

    // Create application state
    let state = AppState::new(settings.clone(), registry, client)?;
    info!("Application state initialized");

    // Create router
    let app = create_router(state);

    // Bind address
    let addr = SocketAddr::new(
        settings.server.bind_address.parse()?,
        settings.server.port,
    );

    info!("Starting server on http://{}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Load settings from file or use defaults
fn load_settings() -> Result<Settings> {
    // Check for settings file in various locations
    let paths = [
        PathBuf::from("settings.yml"),
        PathBuf::from("config/settings.yml"),
        PathBuf::from("/etc/searxng/settings.yml"),
        dirs::config_dir()
            .map(|p| p.join("searxng-rs/settings.yml"))
            .unwrap_or_default(),
    ];

    // Check environment variable first
    if let Ok(path) = std::env::var("SEARXNG_SETTINGS_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            info!("Loading settings from: {}", path.display());
            let mut settings = Settings::from_file(&path)?;
            settings.merge_env();
            return Ok(settings);
        }
    }

    // Try each default path
    for path in paths.iter() {
        if path.exists() {
            info!("Loading settings from: {}", path.display());
            let mut settings = Settings::from_file(path)?;
            settings.merge_env();
            return Ok(settings);
        }
    }

    // Use defaults
    info!("No settings file found, using defaults");
    let mut settings = Settings::default();
    settings.merge_env();
    Ok(settings)
}

/// Print usage information
fn print_usage() {
    println!(
        r#"
SearXNG-RS v{}
A privacy-respecting metasearch engine written in Rust

USAGE:
    searxng-rs [OPTIONS]

OPTIONS:
    -c, --config <FILE>    Path to configuration file
    -h, --help             Print help information
    -V, --version          Print version information

ENVIRONMENT VARIABLES:
    SEARXNG_SETTINGS_PATH  Path to settings.yml
    SEARXNG_DEBUG          Enable debug mode (true/false)
    SEARXNG_PORT           Server port
    SEARXNG_BIND_ADDRESS   Bind address
    SEARXNG_SECRET_KEY     Secret key for sessions

For more information, visit: https://github.com/searxng/searxng-rs
"#,
        searxng_rs::VERSION
    );
}
