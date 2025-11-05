use clap::{Arg, Command};
use proof_of_stake::{config::NodeConfig, Node};
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
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Network port to listen on")
                .value_parser(clap::value_parser!(u16))
                .default_value("9000"),
        )
        .arg(
            Arg::new("node-id")
                .long("node-id")
                .value_name("ID")
                .help("Local node ID for testing (0-9)")
                .value_parser(clap::value_parser!(u8)),
        )
        .get_matches();

    // Initialize logging
    let log_level = matches.get_one::<String>("log-level").unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(format!("proof_of_stake={}", log_level))
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
            "mainnet" => proof_of_stake::NetworkId::Mainnet,
            "testnet" => proof_of_stake::NetworkId::Testnet,
            "devnet" => proof_of_stake::NetworkId::Devnet,
            _ => proof_of_stake::NetworkId::Devnet,
        };
    }

    if matches.get_flag("validator") {
        config.validator.enabled = true;
    }

    // Handle networking arguments
    if let Some(port) = matches.get_one::<u16>("port") {
        config.network.port = *port;
    }

    // If node-id is specified, configure for local testing
    if let Some(node_id) = matches.get_one::<u8>("node-id") {
        config.network.listen_address = "127.0.0.1".to_string();
        config.network.enable_mdns = true;

        // Add bootstrap nodes for a typical 3-node setup
        // Assume nodes are on consecutive ports starting from a base
        let node_port = config.network.port;
        let base_port = if node_port >= *node_id as u16 {
            node_port - (*node_id as u16)
        } else {
            9000 // fallback to default base
        };

        config.network.bootstrap_nodes.clear();
        for i in 0..3u8 {
            if i != *node_id {
                let bootstrap_port = base_port + (i as u16);
                config.network.bootstrap_nodes.push(
                    format!("/ip4/127.0.0.1/tcp/{}", bootstrap_port)
                );
            }
        }
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