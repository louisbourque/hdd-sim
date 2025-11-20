use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::config::{Config, DriveConfig};

const POLL_INTERVAL_MS: u64 = 50;

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub drives: HashMap<String, DriveConfig>,
}

impl From<&Config> for MonitorConfig {
    fn from(config: &Config) -> Self {
        MonitorConfig {
            drives: config.drives.clone(),
        }
    }
}

pub struct Monitor {
    config_sender: mpsc::Sender<MonitorConfig>,
}

#[derive(Debug)]
struct DriveStats {
    read_ios: u64,
    write_ios: u64,
    read_sectors: u64,
    write_sectors: u64,
}

fn parse_drive_stat(device_name: &str) -> Option<DriveStats> {
    let stat_path = Path::new("/sys/block").join(device_name).join("stat");
    let content = fs::read_to_string(&stat_path).ok()?;

    let parts: Vec<&str> = content.split_whitespace().collect();
    if parts.len() < 8 {
        return None;
    }

    Some(DriveStats {
        read_ios: parts[0].parse().ok()?,
        read_sectors: parts[2].parse().ok()?,
        write_ios: parts[4].parse().ok()?,
        write_sectors: parts[6].parse().ok()?,
    })
}

impl Monitor {
    pub fn new() -> Self {
        let (config_sender, config_receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut current_config = MonitorConfig {
                drives: HashMap::new(),
            };
            let mut previous_stats: HashMap<String, DriveStats> = HashMap::new();

            loop {
                // Check for config updates (non-blocking)
                while let Ok(new_config) = config_receiver.try_recv() {
                    current_config = new_config;
                    // Reset stats when config changes to avoid false positives
                    previous_stats.clear();
                }

                // Monitor enabled drives
                for (device_name, drive_config) in &current_config.drives {
                    if !drive_config.enabled {
                        continue;
                    }

                    let Some(current_stats) = parse_drive_stat(device_name) else {
                        continue;
                    };

                    if let Some(prev_stats) = previous_stats.get(device_name) {
                        // Check for read activity
                        if drive_config.read && current_stats.read_ios > prev_stats.read_ios {
                            let sectors = current_stats.read_sectors - prev_stats.read_sectors;
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            eprintln!(
                                "[{}] {}: READ activity detected (sectors: {})",
                                timestamp, device_name, sectors
                            );
                        }

                        // Check for write activity
                        if drive_config.write && current_stats.write_ios > prev_stats.write_ios {
                            let sectors = current_stats.write_sectors - prev_stats.write_sectors;
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            eprintln!(
                                "[{}] {}: WRITE activity detected (sectors: {})",
                                timestamp, device_name, sectors
                            );
                        }
                    }

                    previous_stats.insert(device_name.clone(), current_stats);
                }

                thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
            }
        });

        Monitor { config_sender }
    }

    pub fn update_config(&self, config: &Config) {
        let monitor_config = MonitorConfig::from(config);
        if let Err(e) = self.config_sender.send(monitor_config) {
            eprintln!("Failed to send config update to monitor: {}", e);
        }
    }
}
