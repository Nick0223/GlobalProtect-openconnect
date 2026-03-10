use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use gpapi::portal::PortalConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub auto_connect: bool,
    pub portals: Vec<PortalConfig>,
    pub last_connected_portal: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_connect: false,
            portals: Vec::new(),
            last_connected_portal: None,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }
    
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
    
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut config_dir = dirs::config_dir().ok_or("Could not determine config directory")?;
        config_dir.push("globalprotect-gui");
        config_dir.push("config.json");
        Ok(config_dir)
    }
}