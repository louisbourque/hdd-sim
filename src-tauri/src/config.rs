use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_drive_config_default() {
        let config = DriveConfig::default();
        assert!(!config.enabled);
        assert!(!config.read);
        assert!(!config.write);
        assert_eq!(config.volume, 35);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.drives.len(), 0);
        assert!(config.active);
        assert_eq!(config.poll_interval_ms, 100);
    }

    #[test]
    fn test_load_config_from_path_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        let config = load_config_from_path(&config_path);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_load_config_from_path_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let toml_content = r#"
active = false
poll_interval_ms = 200
[drives.sda]
enabled = true
read = true
write = false
volume = 50
"#;
        fs::write(&config_path, toml_content).unwrap();

        let config = load_config_from_path(&config_path);
        assert!(!config.active);
        assert_eq!(config.poll_interval_ms, 200);
        assert_eq!(config.drives.len(), 1);
        let drive_config = config.drives.get("sda").unwrap();
        assert!(drive_config.enabled);
        assert!(drive_config.read);
        assert!(!drive_config.write);
        assert_eq!(drive_config.volume, 50);
    }

    #[test]
    fn test_load_config_from_path_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "invalid toml content {").unwrap();

        let config = load_config_from_path(&config_path);
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_save_config_to_path() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            active: false,
            poll_interval_ms: 250,
            theme: "system".to_string(),
            drives: {
                let mut drives = HashMap::new();
                drives.insert(
                    "sdb".to_string(),
                    DriveConfig {
                        enabled: true,
                        volume: 75,
                        ..DriveConfig::default()
                    },
                );
                drives
            },
        };

        save_config_to_path(&config, &config_path).unwrap();
        assert!(config_path.exists());

        let loaded = load_config_from_path(&config_path);
        assert!(!loaded.active);
        assert_eq!(loaded.poll_interval_ms, 250);
        assert_eq!(loaded.drives.len(), 1);
        let loaded_drive = loaded.drives.get("sdb").unwrap();
        assert!(loaded_drive.enabled);
        assert_eq!(loaded_drive.volume, 75);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            active: false,
            poll_interval_ms: 500,
            theme: "system".to_string(),
            drives: {
                let mut drives = HashMap::new();
                drives.insert(
                    "sda".to_string(),
                    DriveConfig {
                        enabled: true,
                        read: true,
                        write: true,
                        volume: 80,
                    },
                );
                drives.insert(
                    "sdb".to_string(),
                    DriveConfig {
                        enabled: false,
                        read: true,
                        write: false,
                        volume: 20,
                    },
                );
                drives
            },
        };

        save_config_to_path(&original, &config_path).unwrap();
        let loaded = load_config_from_path(&config_path);

        assert_eq!(loaded.active, original.active);
        assert_eq!(loaded.poll_interval_ms, original.poll_interval_ms);
        assert_eq!(loaded.drives.len(), original.drives.len());
        for (key, value) in &original.drives {
            let loaded_value = loaded.drives.get(key).unwrap();
            assert_eq!(loaded_value.enabled, value.enabled);
            assert_eq!(loaded_value.read, value.read);
            assert_eq!(loaded_value.write, value.write);
            assert_eq!(loaded_value.volume, value.volume);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct DriveConfig {
    pub enabled: bool,
    pub read: bool,
    pub write: bool,
    pub volume: u8,
}

impl Default for DriveConfig {
    fn default() -> Self {
        DriveConfig {
            enabled: false,
            read: false,
            write: false,
            volume: 35,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    pub drives: HashMap<String, DriveConfig>,
    pub active: bool,
    pub poll_interval_ms: u64,
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            drives: HashMap::new(),
            active: true,
            poll_interval_ms: 100,
            theme: "system".to_string(),
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

fn load_config_from_path(config_path: &PathBuf) -> Config {
    if !config_path.exists() {
        return Config::default();
    }

    match fs::read_to_string(config_path) {
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

fn save_config_to_path(config: &Config, config_path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(config_path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

pub fn load_config() -> Config {
    let config_path = match get_config_path() {
        Some(path) => path,
        None => {
            return Config::default();
        }
    };
    load_config_from_path(&config_path)
}

pub fn save_config(config: &Config) {
    let config_path = match get_config_path() {
        Some(path) => path,
        None => {
            eprintln!("Failed to get config directory");
            return;
        }
    };
    if let Err(e) = save_config_to_path(config, &config_path) {
        eprintln!("{}", e);
    }
}
