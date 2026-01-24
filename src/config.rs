//! Configuration module for Portable Superset Launcher

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

const CONFIG_FILE: &str = "config.json";

/// Launcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Port for Superset server
    pub port: u16,
    /// Whether to open browser on start
    pub open_browser: bool,
    /// Host to bind to
    pub host: String,
    /// Path to Python executable (relative to root)
    pub python_path: String,
    /// Superset home directory (relative to root)
    pub superset_home: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8088,
            open_browser: true,
            host: "127.0.0.1".to_string(),
            python_path: "python/python.exe".to_string(),
            superset_home: "superset_home".to_string(),
        }
    }
}

impl Config {
    /// Load config from file or create default
    pub fn load_or_create(root: &Path) -> Result<Self> {
        let config_path = root.join(CONFIG_FILE);
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save(root)?;
            Ok(config)
        }
    }
    
    /// Save config to file
    pub fn save(&self, root: &Path) -> Result<()> {
        let config_path = root.join(CONFIG_FILE);
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
