//! RPA Server configuration.

use rpa_core::context::{BackoffStrategy, RetryConfig};
use rpa_engine::{DebugConfig, VmConfig};
use rpa_engine::vm::FindStrategy;
use serde::Deserialize;
use std::path::Path;

/// Server configuration loaded from config.toml.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub engine: EngineConfig,
    pub js: JsConfig,
    pub plugins: PluginsConfig,
    pub debug: DebugConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub http_addr: String,
    pub grpc_addr: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub default_timeout_ms: u64,
    pub find_strategy: String,
    pub retry_max: u32,
    pub retry_delay_ms: u64,
}

impl From<&EngineConfig> for VmConfig {
    fn from(cfg: &EngineConfig) -> Self {
        let find_strategy = match cfg.find_strategy.as_str() {
            "sequential" => FindStrategy::Sequential,
            s if s.starts_with('[') => {
                // Try to parse as custom strategy list
                FindStrategy::Sequential
            }
            _ => FindStrategy::Sequential,
        };

        let retry = RetryConfig {
            max_retries: cfg.retry_max,
            delay_ms: cfg.retry_delay_ms,
            backoff: BackoffStrategy::Fixed,
        };

        VmConfig {
            retry,
            default_timeout_ms: cfg.default_timeout_ms,
            find_strategy,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct JsConfig {
    pub max_memory_mb: u64,
    pub max_execution_time_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct PluginsConfig {
    pub load_paths: Vec<String>,
    pub auto_load: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                http_addr: "0.0.0.0:8080".to_string(),
                grpc_addr: "0.0.0.0:50051".to_string(),
            },
            engine: EngineConfig {
                default_timeout_ms: 30_000,
                find_strategy: "sequential".to_string(),
                retry_max: 3,
                retry_delay_ms: 1000,
            },
            js: JsConfig {
                max_memory_mb: 64,
                max_execution_time_ms: 60_000,
            },
            plugins: PluginsConfig {
                load_paths: vec!["./plugins".to_string()],
                auto_load: true,
            },
            debug: DebugConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn from_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
