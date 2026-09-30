use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::config::{Config, DriveConfig};
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use tokio::time;

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

fn parse_drive_stat_content(content: &str) -> Option<DriveStats> {
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

fn parse_drive_stat(device_name: &str) -> Option<DriveStats> {
    let stat_path = Path::new("/sys/block").join(device_name).join("stat");
    let content = fs::read_to_string(&stat_path).ok()?;
    parse_drive_stat_content(&content)
}

// Tiny xorshift PRNG so clicks and their spacing vary without pulling in `rand`
fn xorshift(state: &mut u32) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 17;
    *state ^= *state << 5;
    *state as f32 / u32::MAX as f32
}

fn random_seed() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos ^ COUNTER.fetch_add(0x9E37_79B9, Ordering::Relaxed)) | 1
}

// Programmatically generated HDD seek click: a sharp noise tick from the actuator,
// a short metallic ring from arm/chassis resonances, a quieter settle tick, a low
// thump from the arm's mass, and a bassy echo from the case. All muffled by the enclosure.
struct ClickSound {
    sample_rate: u32,
    duration_samples: u32,
    current_sample: u32,
    rng: u32,
    // Per-click jittered resonance frequencies (Hz)
    resonances: [f32; 3],
    // When the head settle tick lands (seconds)
    settle_at: f32,
    // High-pass filter state
    hp_prev_in: f32,
    hp_prev_out: f32,
    // Two cascaded low-pass stages: enclosure + case muffling
    lp: [f32; 2],
    // Case echo: delay line with a low-passed feedback loop, so each bounce gets bassier
    echo_buf: Vec<f32>,
    echo_pos: usize,
    echo_lp: f32,
    // Volume multiplier (0.0 to 1.0)
    volume: f32,
}

impl ClickSound {
    // Tuning knobs, tweak by ear
    const RESONANCES_HZ: [f32; 3] = [1440.0, 2100.0, 2880.0];
    const RESONANCE_GAIN: [f32; 3] = [0.35, 0.25, 0.15];
    const RING_DECAY_S: [f32; 3] = [0.0012, 0.0009, 0.0007];
    const RESONANCE_JITTER: f32 = 0.08;
    const TICK_GAIN: f32 = 1.0;
    const SETTLE_GAIN: f32 = 0.25;
    const SETTLE_MIN_S: f32 = 0.0015;
    const SETTLE_MAX_S: f32 = 0.003;
    const TICK_DECAY_S: f32 = 0.0007;
    const HIGHPASS_HZ: f32 = 330.0;
    const MUFFLE_HZ: f32 = 2000.0;
    const ECHO_DELAY_S: f32 = 0.004;
    const ECHO_FEEDBACK: f32 = 0.21;
    const ECHO_LOWPASS_HZ: f32 = 50.0;
    const ECHO_GAIN: f32 = 0.8;
    const THUMP_HZ: f32 = 150.0;
    const THUMP_GAIN: f32 = 0.25;
    const THUMP_DECAY_S: f32 = 0.0025;
    const OUTPUT_GAIN: f32 = 0.9;

    fn new(sample_rate: u32, duration_ms: u32, volume: u8) -> Self {
        let duration_samples = (sample_rate as u64 * duration_ms as u64 / 1000) as u32;
        let mut rng = random_seed();
        let resonances = Self::RESONANCES_HZ
            .map(|f| f * (1.0 + Self::RESONANCE_JITTER * (2.0 * xorshift(&mut rng) - 1.0)));
        let settle_at =
            Self::SETTLE_MIN_S + (Self::SETTLE_MAX_S - Self::SETTLE_MIN_S) * xorshift(&mut rng);
        // Convert volume from 0-100 to 0.0-1.0 multiplier
        let volume_multiplier = volume as f32 / 100.0;
        ClickSound {
            sample_rate,
            duration_samples,
            current_sample: 0,
            rng,
            resonances,
            settle_at,
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            lp: [0.0; 2],
            echo_buf: vec![0.0; ((sample_rate as f32 * Self::ECHO_DELAY_S) as usize).max(1)],
            echo_pos: 0,
            echo_lp: 0.0,
            volume: volume_multiplier,
        }
    }

    // One impulse hitting the mechanism, `t` seconds after it lands
    fn excitation(&self, t: f32, noise: f32) -> f32 {
        let tick = noise * Self::TICK_GAIN * (-t / Self::TICK_DECAY_S).exp();
        let ring: f32 = (0..3)
            .map(|i| {
                (2.0 * std::f32::consts::PI * self.resonances[i] * t).sin()
                    * Self::RESONANCE_GAIN[i]
                    * (-t / Self::RING_DECAY_S[i]).exp()
            })
            .sum();
        tick + ring
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

        let noise = 2.0 * xorshift(&mut self.rng) - 1.0;
        let mut signal = self.excitation(t, noise);
        if t >= self.settle_at {
            signal += Self::SETTLE_GAIN * self.excitation(t - self.settle_at, noise);
        }

        // First-order high-pass to strip rumble; a seek click has almost no bass
        let rc = 1.0 / (2.0 * std::f32::consts::PI * Self::HIGHPASS_HZ);
        let dt = 1.0 / self.sample_rate as f32;
        let alpha = rc / (rc + dt);
        self.hp_prev_out = alpha * (self.hp_prev_out + signal - self.hp_prev_in);
        self.hp_prev_in = signal;

        // Enclosure and case soak up the highs
        let lp_rc = 1.0 / (2.0 * std::f32::consts::PI * Self::MUFFLE_HZ);
        let lp_alpha = dt / (lp_rc + dt);
        self.lp[0] += lp_alpha * (self.hp_prev_out - self.lp[0]);
        self.lp[1] += lp_alpha * (self.lp[0] - self.lp[1]);

        // Low thump bouncing around the case
        let echo_rc = 1.0 / (2.0 * std::f32::consts::PI * Self::ECHO_LOWPASS_HZ);
        let echo_alpha = dt / (echo_rc + dt);
        let delayed = self.echo_buf[self.echo_pos];
        self.echo_lp += echo_alpha * (signal + Self::ECHO_FEEDBACK * delayed - self.echo_lp);
        self.echo_buf[self.echo_pos] = self.echo_lp;
        self.echo_pos = (self.echo_pos + 1) % self.echo_buf.len();

        // Low body thump from the arm's mass jolting the drive, skips the bass cut
        let thump = (2.0 * std::f32::consts::PI * Self::THUMP_HZ * t).sin()
            * Self::THUMP_GAIN
            * (-t / Self::THUMP_DECAY_S).exp();

        // Short fade-out so long tails don't get chopped into a pop at the end
        let fade_samples = self.sample_rate as f32 * 0.003;
        let fade = ((self.duration_samples - self.current_sample) as f32 / fade_samples).min(1.0);

        let out = (self.lp[1] + Self::ECHO_GAIN * delayed + thump) * fade;
        Some((Self::OUTPUT_GAIN * out * self.volume).clamp(-1.0, 1.0))
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
// Capped at 20 clicks max to avoid spawning excessive tasks
fn play_click(
    runtime_handle: &tokio::runtime::Handle,
    stream_handle: &OutputStreamHandle,
    sectors: u64,
    volume: u8,
) {
    let click_count = if sectors > 10000 {
        (sectors / 10000).min(20)
    } else if sectors > 5000 {
        2
    } else if sectors > 1000 {
        1
    } else {
        return;
    };

    // Irregular 8-35ms gaps between clicks, like real seek patterns
    let mut rng = random_seed();
    let mut delay_ms = 0;
    for i in 0..click_count {
        if i > 0 {
            delay_ms += 8 + (27.0 * xorshift(&mut rng)) as u64;
        }
        let handle = stream_handle.clone();
        let volume_clone = volume;

        runtime_handle.spawn(async move {
            if delay_ms > 0 {
                time::sleep(Duration::from_millis(delay_ms)).await;
            }

            match Sink::try_new(&handle) {
                Ok(sink) => {
                    let click = ClickSound::new(44100, 30, volume_clone);
                    sink.append(click);
                    tokio::task::spawn_blocking(move || {
                        sink.sleep_until_end();
                    })
                    .await
                    .ok();
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
            // Create tokio runtime for async tasks
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            let handle = rt.handle().clone();

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
                                    Some(ref audio_handle) => {
                                        play_click(
                                            &handle,
                                            audio_handle,
                                            sectors,
                                            drive_config.volume,
                                        );
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
                                    Some(ref audio_handle) => {
                                        play_click(
                                            &handle,
                                            audio_handle,
                                            sectors,
                                            drive_config.volume,
                                        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_monitor_config_from() {
        let config = Config {
            active: false,
            poll_interval_ms: 250,
            close_to_tray: true,
            drives: {
                let mut drives = HashMap::new();
                drives.insert(
                    "sda".to_string(),
                    DriveConfig {
                        enabled: true,
                        volume: 60,
                        ..DriveConfig::default()
                    },
                );
                drives
            },
        };

        let monitor_config = MonitorConfig::from(&config);
        assert!(!monitor_config.active);
        assert_eq!(monitor_config.poll_interval_ms, 250);
        assert_eq!(monitor_config.drives.len(), 1);
        let drive = monitor_config.drives.get("sda").unwrap();
        assert!(drive.enabled);
        assert_eq!(drive.volume, 60);
    }

    #[test]
    fn test_parse_drive_stat_content_valid() {
        // Format: read_ios read_merges read_sectors read_ticks write_ios write_merges write_sectors write_ticks
        let content = "12345 0 67890 0 11111 0 22222 0 0 0 0 0 0 0";
        let stats = parse_drive_stat_content(content).unwrap();
        assert_eq!(stats.read_ios, 12345);
        assert_eq!(stats.read_sectors, 67890);
        assert_eq!(stats.write_ios, 11111);
        assert_eq!(stats.write_sectors, 22222);
    }

    #[test]
    fn test_parse_drive_stat_content_minimal() {
        let content = "100 0 200 0 300 0 400 0";
        let stats = parse_drive_stat_content(content).unwrap();
        assert_eq!(stats.read_ios, 100);
        assert_eq!(stats.read_sectors, 200);
        assert_eq!(stats.write_ios, 300);
        assert_eq!(stats.write_sectors, 400);
    }

    #[test]
    fn test_parse_drive_stat_content_insufficient_fields() {
        let content = "100 0 200";
        assert!(parse_drive_stat_content(content).is_none());
    }

    #[test]
    fn test_parse_drive_stat_content_invalid_numbers() {
        let content = "abc 0 200 0 300 0 400 0";
        assert!(parse_drive_stat_content(content).is_none());
    }

    #[test]
    fn test_parse_drive_stat_content_empty() {
        assert!(parse_drive_stat_content("").is_none());
    }

    #[test]
    fn test_parse_drive_stat_content_whitespace() {
        let content = "   100   0   200   0   300   0   400   0   ";
        let stats = parse_drive_stat_content(content).unwrap();
        assert_eq!(stats.read_ios, 100);
        assert_eq!(stats.read_sectors, 200);
        assert_eq!(stats.write_ios, 300);
        assert_eq!(stats.write_sectors, 400);
    }

    #[test]
    fn test_click_sound_creation() {
        let click = ClickSound::new(44100, 25, 50);
        assert_eq!(click.sample_rate, 44100);
        assert_eq!(click.channels(), 1);
        assert_eq!(click.sample_rate(), 44100);
        assert!(click.volume > 0.0 && click.volume <= 1.0);
    }

    #[test]
    fn test_click_sound_volume_scaling() {
        let click_0 = ClickSound::new(44100, 25, 0);
        assert_eq!(click_0.volume, 0.0);

        let click_50 = ClickSound::new(44100, 25, 50);
        assert_eq!(click_50.volume, 0.5);

        let click_100 = ClickSound::new(44100, 25, 100);
        assert_eq!(click_100.volume, 1.0);
    }

    #[test]
    fn test_click_sound_generates_samples() {
        let mut click = ClickSound::new(44100, 25, 50);
        let mut sample_count = 0;
        let mut max_amplitude = 0.0_f32;
        let mut min_amplitude = 0.0_f32;

        // Generate a few samples to verify it works
        for _ in 0..100 {
            if let Some(sample) = click.next() {
                sample_count += 1;
                max_amplitude = max_amplitude.max(sample.abs());
                min_amplitude = min_amplitude.min(sample.abs());
            } else {
                break;
            }
        }

        assert!(sample_count > 0, "Should generate at least some samples");
        assert!(max_amplitude > 0.0, "Should have non-zero amplitude");
        // Samples should be in valid audio range (typically -1.0 to 1.0)
        assert!(max_amplitude <= 1.0, "Amplitude should not exceed 1.0");
    }

    #[test]
    fn test_click_sound_duration() {
        let click = ClickSound::new(44100, 25, 50);
        let duration = click.total_duration().unwrap();
        // 25ms should be approximately 25ms (allow small rounding)
        assert!(duration.as_millis() >= 24 && duration.as_millis() <= 26);
    }

    #[test]
    fn test_click_sound_is_sharp_transient() {
        let samples: Vec<f32> = ClickSound::new(44100, 30, 100).collect();
        assert!(samples.iter().all(|s| s.abs() <= 1.0));
        let peak_at = samples
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .unwrap()
            .0;
        // Loudest point should land in the first ~1ms, not a slow thump
        assert!(peak_at < 44, "peak at sample {}", peak_at);
        // Tail should have died down by the end
        let tail = samples[samples.len() - 20..]
            .iter()
            .fold(0.0_f32, |m, s| m.max(s.abs()));
        assert!(tail < 0.05, "tail amplitude {}", tail);
    }

    #[test]
    fn test_click_sound_finishes() {
        let mut click = ClickSound::new(1000, 10, 50); // 10ms at 1kHz = ~10 samples
        let mut count = 0;
        while click.next().is_some() {
            count += 1;
            if count > 1000 {
                panic!("Click sound should finish, but generated too many samples");
            }
        }
        assert!(count > 0, "Should generate some samples before finishing");
    }
}
