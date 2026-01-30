//! Facet Server - Remote execution server for Facet desktop application
//!
//! This binary starts the Warp web server with configured routes and middleware.

use facet_server::{server, Config};
use std::env;
use std::path::PathBuf;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut dev_mode = false;
    let mut mock_mode = false;
    let mut config_path = None;

    for i in 0..args.len() {
        match args[i].as_str() {
            "--dev" => dev_mode = true,
            "--mock" => mock_mode = true,
            "--config" => {
                if i + 1 < args.len() {
                    config_path = Some(PathBuf::from(&args[i + 1]));
                }
            }
            _ => {}
        }
    }

    // Load configuration
    let mut config = if let Some(path) = config_path {
        Config::from_file(path)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
    } else if dev_mode {
        Config::dev_default()
    } else {
        // Default to looking for config.toml in current directory
        match Config::from_file("config.toml") {
            Ok(c) => c,
            Err(_) => {
                // specific fallback for dev convenience if config missing
                Config::dev_default()
            }
        }
    };

    // Override mock mode if specified on command line
    if mock_mode {
        config.claude.mock_mode = true;
    }

    // Initialize logging
    init_logging(&config);

    info!("Starting Facet Server v{}", env!("CARGO_PKG_VERSION"));

    // Run server
    server::run(config).await
}

/// Initializes logging based on configuration
fn init_logging(config: &Config) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.logging.level));

    if config.logging.pretty_print {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .pretty()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    }
}
