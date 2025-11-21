use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::config::{Config, DriveConfig};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub drives: HashMap<String, DriveConfig>,
    pub active: bool,
    pub poll_interval_ms: u64,
}

impl From<&Config> for MonitorConfig {
    fn from(config: &Config) -> Self {
        MonitorConfig {
            drives: config.drives.clone(),
            active: config.active,
            poll_interval_ms: config.poll_interval_ms,
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

// Programmatically generated click sound
struct ClickSound {
    sample_rate: u32,
    duration_samples: u32,
    current_sample: u32,
}

impl ClickSound {
    fn new(sample_rate: u32, duration_ms: u32) -> Self {
        let duration_samples = (sample_rate as u64 * duration_ms as u64 / 1000) as u32;
        ClickSound {
            sample_rate,
            duration_samples,
            current_sample: 0,
        }
    }
}

impl Iterator for ClickSound {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.duration_samples {
            return None;
        }

        let t = self.current_sample as f32 / self.sample_rate as f32;
        self.current_sample += 1;

        // Generate a short click: brief sine wave burst with exponential decay
        // This creates a "tick" sound similar to a hard drive click
        let frequency = 2000.0; // 2kHz click
        let phase = 2.0 * std::f32::consts::PI * frequency * t;

        // Exponential decay envelope for natural sound
        let envelope = (-t * 30.0).exp(); // Fast decay

        // Short burst of sine wave
        Some(0.3 * envelope * phase.sin())
    }
}

impl Source for ClickSound {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1 // Mono
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(
            (self.duration_samples as u64 * 1000) / self.sample_rate as u64,
        ))
    }
}

// Helper to play click sound (non-blocking)
fn play_click(stream_handle: &OutputStreamHandle) {
    match Sink::try_new(stream_handle) {
        Ok(sink) => {
            let click = ClickSound::new(44100, 15); // 15ms click at 44.1kHz
            sink.append(click);
            // Spawn a thread to keep the sink alive until playback completes
            thread::spawn(move || {
                sink.sleep_until_end();
            });
        }
        Err(e) => {
            eprintln!("Error creating sink for click sound: {:?}", e);
        }
    }
}

impl Monitor {
    pub fn new() -> Self {
        let (config_sender, config_receiver) = mpsc::channel();

        thread::spawn(move || {
            // Initialize audio output once for the entire monitoring thread
            // We must keep the OutputStream alive for the handle to remain valid
            let (stream_opt, stream_handle_opt) = match OutputStream::try_default() {
                Ok((stream, handle)) => (Some(stream), Some(handle)),
                Err(e) => {
                    eprintln!(
                        "Warning: Could not initialize audio output: {:?}. Clicks will be silent.",
                        e
                    );
                    (None, None)
                }
            };

            // Keep the stream alive by moving it into a variable that lives for the thread lifetime
            let _audio_stream = stream_opt;

            let mut current_config = MonitorConfig {
                drives: HashMap::new(),
                active: true,
                poll_interval_ms: 100,
            };
            let mut previous_stats: HashMap<String, DriveStats> = HashMap::new();

            loop {
                // Check for config updates (non-blocking)
                while let Ok(new_config) = config_receiver.try_recv() {
                    current_config = new_config;
                    // Reset stats when config changes to avoid false positives
                    previous_stats.clear();
                }

                // Only monitor if active
                if current_config.active {
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
                                match stream_handle_opt {
                                    Some(ref handle) => {
                                        play_click(handle);
                                    }
                                    None => eprintln!(
                                        "Warning: Audio stream handle not available, cannot play click sound"
                                    ),
                                }
                            }

                            // Check for write activity
                            if drive_config.write && current_stats.write_ios > prev_stats.write_ios
                            {
                                let sectors =
                                    current_stats.write_sectors - prev_stats.write_sectors;
                                let timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                eprintln!(
                                    "[{}] {}: WRITE activity detected (sectors: {})",
                                    timestamp, device_name, sectors
                                );
                                match stream_handle_opt {
                                    Some(ref handle) => {
                                        play_click(handle);
                                    }
                                    None => eprintln!(
                                        "Warning: Audio stream handle not available, cannot play click sound"
                                    ),
                                }
                            }
                        }

                        previous_stats.insert(device_name.clone(), current_stats);
                    }
                }

                thread::sleep(Duration::from_millis(current_config.poll_interval_ms));
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
