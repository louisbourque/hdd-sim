use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DriveConfig {
    pub enabled: bool,
    pub read: bool,
    pub write: bool,
    pub tone: u8,
    pub volume: u8,
}

impl Default for DriveConfig {
    fn default() -> Self {
        DriveConfig {
            enabled: false,
            read: false,
            write: false,
            tone: 1,
            volume: 50,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub drives: HashMap<String, DriveConfig>,
    pub active: bool,
    pub poll_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            drives: HashMap::new(),
            active: true,
            poll_interval_ms: 100,
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("hdd-simulator");
        path.push("config.toml");
        path
    })
}

pub fn load_config() -> Config {
    let config_path = match get_config_path() {
        Some(path) => path,
        None => {
            return Config::default();
        }
    };

    if !config_path.exists() {
        return Config::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to parse config file: {}", e);
                Config::default()
            }
        },
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            Config::default()
        }
    }
}

pub fn save_config(config: &Config) {
    let config_path = match get_config_path() {
        Some(path) => path,
        None => {
            eprintln!("Failed to get config directory");
            return;
        }
    };

    if let Some(parent) = config_path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        eprintln!("Failed to create config directory: {}", e);
        return;
    }

    match toml::to_string_pretty(config) {
        Ok(content) => {
            if let Err(e) = fs::write(&config_path, content) {
                eprintln!("Failed to write config file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize config: {}", e);
        }
    }
}
