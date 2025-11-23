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
    // Echo/reverb delay buffer (stores samples for delayed mixing)
    delay_buffer: Vec<f32>,
    delay_buffer_pos: usize,
    // Low-pass filter state for muffling effect
    lowpass_state: f32,
    // Volume multiplier (0.0 to 1.0)
    volume: f32,
}

impl ClickSound {
    fn new(sample_rate: u32, duration_ms: u32, volume: u8) -> Self {
        let duration_samples = (sample_rate as u64 * duration_ms as u64 / 1000) as u32;
        // Create delay buffer for echo (approximately 5-10ms delays)
        let delay_samples = (sample_rate as f32 * 0.008) as usize; // 8ms delay
        let delay_buffer = vec![0.0; delay_samples];
        // Convert volume from 0-100 to 0.0-1.0 multiplier
        let volume_multiplier = volume as f32 / 100.0;
        ClickSound {
            sample_rate,
            duration_samples,
            current_sample: 0,
            delay_buffer,
            delay_buffer_pos: 0,
            lowpass_state: 0.0,
            volume: volume_multiplier,
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

        // Generate a heavy, clunky mechanical sound (muffled as if inside a drive enclosure)
        let base_frequency = 40.0; // Lower frequency for clunk sound
        let phase = 2.0 * std::f32::consts::PI * base_frequency * t;

        // Strong fundamental for heavy bass
        let fundamental = phase.sin() * 2.0;

        // Strong subharmonic for deep, heavy thunk
        let subharmonic = (phase * 0.5).sin() * 0.6;

        // Reduced harmonics for muffled effect (high frequencies are damped by enclosure)
        let second_harmonic = (2.0 * phase).sin() * 0.12;
        let third_harmonic = (3.0 * phase).sin() * 0.06;

        // Add low-frequency noise for mechanical texture
        let noise = (t * 100.0).sin() * 0.08;

        // Combine all components (no high-frequency transients - muffled by enclosure)
        let raw_signal = fundamental + subharmonic + second_harmonic + third_harmonic + noise;

        // Apply envelope
        let attack_time = 0.0001; // 0.1ms - sharp attack
        let envelope = if t < attack_time {
            // Exponential attack for sharper onset
            (t / attack_time * 3.0).exp() / (3.0_f32.exp())
        } else {
            // Slower decay for heavier, more sustained clunk
            (-(t - attack_time) * 18.0).exp()
        };

        let dry_signal = envelope * raw_signal;

        // Apply low-pass filter for muffling effect (simulates sound damped by drive enclosure)
        // Simple first-order low-pass filter
        let cutoff = 800.0; // Low cutoff frequency for muffled sound
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let dt = 1.0 / self.sample_rate as f32;
        let alpha = dt / (rc + dt);
        self.lowpass_state = alpha * dry_signal + (1.0 - alpha) * self.lowpass_state;
        let muffled_signal = self.lowpass_state;

        // Add echo/reverb effect (sound bouncing inside the drive enclosure)
        let echo_delay = self.delay_buffer[self.delay_buffer_pos];
        let echo_amount = 0.35; // Amount of echo
        let echo_signal = muffled_signal + echo_delay * echo_amount;

        // Store current sample in delay buffer for echo
        self.delay_buffer[self.delay_buffer_pos] = muffled_signal * 0.6; // Slightly reduced for natural decay
        self.delay_buffer_pos = (self.delay_buffer_pos + 1) % self.delay_buffer.len();

        // Additional subtle echo (longer delay)
        let longer_delay_pos =
            (self.delay_buffer_pos + self.delay_buffer.len() / 2) % self.delay_buffer.len();
        let longer_echo = self.delay_buffer[longer_delay_pos] * 0.15;
        let final_signal = echo_signal + longer_echo;

        // Apply volume multiplier to the final signal
        Some(0.5 * final_signal * self.volume)
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
fn play_click(stream_handle: &OutputStreamHandle, sectors: u64, volume: u8) {
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
        let volume_clone = volume;

        thread::spawn(move || {
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }

            match Sink::try_new(&handle) {
                Ok(sink) => {
                    let click = ClickSound::new(44100, 25, volume_clone);
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
                                        play_click(handle, sectors, drive_config.volume);
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
                                        play_click(handle, sectors, drive_config.volume);
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
