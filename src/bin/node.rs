use clap::{Arg, Command};
use production_pos::{config::NodeConfig, Node};
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("production-pos")
        .version("0.1.0")
        .about("Production-grade Proof of Stake blockchain node")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
                .default_value("config.toml"),
        )
        .arg(
            Arg::new("data-dir")
                .short('d')
                .long("data-dir")
                .value_name("DIR")
                .help("Data directory path")
                .default_value("./data"),
        )
        .arg(
            Arg::new("network")
                .short('n')
                .long("network")
                .value_name("NETWORK")
                .help("Network to connect to")
                .value_parser(["mainnet", "testnet", "devnet"])
                .default_value("devnet"),
        )
        .arg(
            Arg::new("validator")
                .long("validator")
                .help("Enable validator mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("log-level")
                .short('v')
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level")
                .value_parser(["trace", "debug", "info", "warn", "error"])
                .default_value("info"),
        )
        .get_matches();

    // Initialize logging
    let log_level = matches.get_one::<String>("log-level").unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(format!("production_pos={}", log_level))
        .init();

    info!("Starting Production PoS Node v0.1.0");

    // Load configuration
    let config_path = PathBuf::from(matches.get_one::<String>("config").unwrap());
    let mut config = if config_path.exists() {
        info!("Loading configuration from: {:?}", config_path);
        NodeConfig::load_from_file(&config_path)?
    } else {
        info!("Using default configuration");
        NodeConfig::default()
    };

    // Override config with CLI arguments
    if let Some(data_dir) = matches.get_one::<String>("data-dir") {
        config.storage.data_dir = PathBuf::from(data_dir);
    }

    if let Some(network) = matches.get_one::<String>("network") {
        config.network.network_id = match network.as_str() {
            "mainnet" => production_pos::NetworkId::Mainnet,
            "testnet" => production_pos::NetworkId::Testnet,
            "devnet" => production_pos::NetworkId::Devnet,
            _ => production_pos::NetworkId::Devnet,
        };
    }

    if matches.get_flag("validator") {
        config.validator.enabled = true;
    }

    info!("Configuration: {:?}", config);

    // Create and start the node
    match Node::new(config).await {
        Ok(mut node) => {
            info!("Node initialized successfully");

            // Handle shutdown gracefully
            let shutdown_signal = async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for ctrl-c");
                info!("Received shutdown signal");
            };

            tokio::select! {
                result = node.start() => {
                    if let Err(e) = result {
                        error!("Node error: {}", e);
                        return Err(e.into());
                    }
                }
                _ = shutdown_signal => {
                    info!("Shutting down gracefully...");
                }
            }
        }
        Err(e) => {
            error!("Failed to initialize node: {}", e);
            return Err(e.into());
        }
    }

    info!("Node stopped");
    Ok(())
}