use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::types::NetworkId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub validator: ValidatorConfig,
    pub api: ApiConfig,
    pub metrics: MetricsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network_id: NetworkId,
    pub listen_address: String,
    pub port: u16,
    pub max_peers: usize,
    pub bootstrap_nodes: Vec<String>,
    pub enable_mdns: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub db_url: Option<String>,
    pub cache_size: usize,
    pub sync_mode: SyncMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    pub enabled: bool,
    pub keystore_path: Option<PathBuf>,
    pub keystore_password: Option<String>,
    pub graffiti: Option<String>,
    pub fee_recipient: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enabled: bool,
    pub listen_address: String,
    pub cors_origins: Vec<String>,
    pub max_request_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub listen_address: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMode {
    Full,
    Fast,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            validator: ValidatorConfig::default(),
            api: ApiConfig::default(),
            metrics: MetricsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            network_id: NetworkId::Devnet,
            listen_address: "0.0.0.0".to_string(),
            port: 9000,
            max_peers: 50,
            bootstrap_nodes: Vec::new(),
            enable_mdns: true,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            data_dir: PathBuf::from("./data"),
            db_url: None,
            cache_size: 1024 * 1024 * 100, // 100MB
            sync_mode: SyncMode::Full,
        }
    }
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        ValidatorConfig {
            enabled: false,
            keystore_path: None,
            keystore_password: None,
            graffiti: None,
            fee_recipient: None,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            enabled: true,
            listen_address: "127.0.0.1:8080".to_string(),
            cors_origins: vec!["*".to_string()],
            max_request_size: 1024 * 1024, // 1MB
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        MetricsConfig {
            enabled: false,
            listen_address: "127.0.0.1:9090".to_string(),
            namespace: "production_pos".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            file: None,
        }
    }
}

impl NodeConfig {
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: NodeConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}