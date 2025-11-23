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

        // Generate a heavy, clunky mechanical sound with sharp attack
        let base_frequency = 40.0; // Lower frequency for clunk sound
        let phase = 2.0 * std::f32::consts::PI * base_frequency * t;

        // Strong fundamental for heavy bass
        let fundamental = phase.sin() * 2.0;

        // Strong subharmonic for deep, heavy thunk
        let subharmonic = (phase * 0.5).sin() * 0.6;

        // Add harmonics for mechanical character with stronger high frequencies for sharpness
        let second_harmonic = (2.0 * phase).sin() * 0.25;
        let third_harmonic = (3.0 * phase).sin() * 0.18;
        let fourth_harmonic = (4.0 * phase).sin() * 0.12;
        let fifth_harmonic = (5.0 * phase).sin() * 0.08;
        let sixth_harmonic = (6.0 * phase).sin() * 0.05;

        // Add low-frequency noise for mechanical texture
        let noise = (t * 100.0).sin() * 0.1;

        // High-frequency transient for sharp attack (only at the very beginning)
        let high_freq_transient = if t < 0.001 {
            (2.0 * std::f32::consts::PI * 2000.0 * t).sin() * 0.3 * (1.0 - t * 1000.0)
        } else {
            0.0
        };

        // Combine all components
        let clunk_signal = fundamental
            + subharmonic
            + second_harmonic
            + third_harmonic
            + fourth_harmonic
            + fifth_harmonic
            + sixth_harmonic
            + noise
            + high_freq_transient;

        // Ultra-sharp attack (almost instant impact), then slower decay for heavy feel
        let attack_time = 0.0001; // 0.1ms - ultra-sharp attack
        let envelope = if t < attack_time {
            // Exponential attack for sharper onset
            (t / attack_time * 3.0).exp() / (3.0_f32.exp())
        } else {
            // Slower decay for heavier, more sustained clunk
            (-(t - attack_time) * 18.0).exp()
        };

        Some(0.5 * envelope * clunk_signal)
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
// Plays multiple times based on sector count: >5000 = 3 times, >1000 = 2 times, otherwise = 1 time
fn play_click(stream_handle: &OutputStreamHandle, sectors: u64) {
    let click_count = if sectors > 5000 {
        3
    } else if sectors > 1000 {
        2
    } else {
        1
    };

    for i in 0..click_count {
        let handle = stream_handle.clone();
        let delay_ms = i * 30; // 30ms delay between clicks

        thread::spawn(move || {
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }

            match Sink::try_new(&handle) {
                Ok(sink) => {
                    let click = ClickSound::new(44100, 25);
                    sink.append(click);
                    sink.sleep_until_end();
                }
                Err(e) => {
                    eprintln!("Error creating sink for click sound: {:?}", e);
                }
            }
        });
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
                                        play_click(handle, sectors);
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
                                        play_click(handle, sectors);
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
