mod config;
mod monitor;
mod ui;

use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::{
    App, AppContext, Bounds, QuitMode, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};

use config::load_config;
use monitor::Monitor;

#[derive(Debug, Clone)]
pub struct Drive {
    pub name: String,
    pub model: String,
    pub size_display: String,
}

fn get_drive_model(device_path: &Path) -> Option<String> {
    // Try /sys/block/{device}/device/model first (most common for SCSI/SATA)
    let model_path = device_path.join("device/model");
    if model_path.exists()
        && let Ok(model) = fs::read_to_string(&model_path)
    {
        let trimmed = model.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    // Fallback: try /sys/block/{device}/../model (for some device types)
    if let Some(parent) = device_path.parent() {
        let alt_model_path = parent.join("model");
        if alt_model_path.exists()
            && let Ok(model) = fs::read_to_string(&alt_model_path)
        {
            let trimmed = model.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    None
}

fn format_drive_size(sectors: u64) -> String {
    // Convert sectors (512 bytes) to bytes, then to TB
    let bytes = sectors * 512;
    let tb = bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0);
    if tb >= 1.0 {
        return format!("{:.1} TB", tb);
    }
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    format!("{:.1} GB", gb)
}

fn get_drive_size_display(device_name: &str) -> String {
    let size_path = Path::new("/sys/block").join(device_name).join("size");
    if let Ok(size_content) = fs::read_to_string(&size_path)
        && let Ok(sectors) = size_content.trim().parse::<u64>()
    {
        return format_drive_size(sectors);
    }
    "Unknown".to_string()
}

fn is_virtual_device(device_name: &str) -> bool {
    device_name.starts_with("loop")
        || device_name.starts_with("ram")
        || device_name.starts_with("zram")
        || device_name.starts_with("dm-")
        || device_name.starts_with("sr")
        || device_name.starts_with("fd")
}

pub fn scan_hard_drives() -> Vec<Drive> {
    let sys_block = Path::new("/sys/block");
    let mut drives = Vec::new();

    if let Ok(entries) = fs::read_dir(sys_block) {
        for entry in entries.flatten() {
            let device_name = entry.file_name();
            let device_str = device_name.to_string_lossy().to_string();

            // Filter out virtual devices (loop, ram, zram, etc.)
            if is_virtual_device(&device_str) {
                continue;
            }

            // Check if it's a real block device by verifying it has a size
            let size_path = entry.path().join("size");
            if size_path.exists()
                && let Ok(size_content) = fs::read_to_string(&size_path)
                && let Ok(size) = size_content.trim().parse::<u64>()
                && size > 0
            {
                // Try to get the model name, fall back to device name if not available
                let display_name =
                    get_drive_model(&entry.path()).unwrap_or_else(|| device_str.clone());
                let size_display = get_drive_size_display(&device_str);
                drives.push(Drive {
                    name: device_str,
                    model: display_name,
                    size_display,
                });
            }
        }
    }

    drives.sort_by(|a, b| a.model.cmp(&b.model));
    drives
}

enum TrayCmd {
    Restore,
    Quit,
}

struct Tray {
    tx: smol::channel::Sender<TrayCmd>,
    icon: ksni::Icon,
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }
    fn title(&self) -> String {
        "HDD Simulator".into()
    }
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![self.icon.clone()]
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send_blocking(TrayCmd::Restore);
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::StandardItem;
        vec![
            StandardItem {
                label: "Restore".into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.tx.send_blocking(TrayCmd::Restore);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|t: &mut Self| {
                    let _ = t.tx.send_blocking(TrayCmd::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn tray_icon() -> ksni::Icon {
    let img = image::load_from_memory(include_bytes!("../assets/app-icon.png"))
        .expect("bundled icon is a valid PNG")
        .into_rgba8();
    let (width, height) = img.dimensions();
    let mut data = img.into_vec();
    // RGBA -> ARGB
    for pixel in data.as_chunks_mut::<4>().0 {
        pixel.rotate_right(1);
    }
    ksni::Icon {
        width: width as i32,
        height: height as i32,
        data,
    }
}

fn open_main_window(monitor: Rc<Monitor>, cx: &mut App) {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(900.), px(600.)),
            cx,
        ))),
        window_min_size: Some(size(px(640.), px(480.))),
        titlebar: Some(TitlebarOptions {
            title: Some("HDD Simulator".into()),
            ..Default::default()
        }),
        // Wayland picks the icon from the matching .desktop file (see README), X11 uses `icon`
        app_id: Some("ca.rusticotter.hdd-simulator".into()),
        icon: Some(Arc::new(
            image::load_from_memory(include_bytes!("../assets/app-icon.png"))
                .expect("bundled icon is a valid PNG")
                .into_rgba8(),
        )),
        ..Default::default()
    };
    gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| ui::HddApp::new(monitor, window, cx))
    })
    .expect("failed to open window");
}

fn main() {
    let monitor = Rc::new(Monitor::new());
    monitor.update_config(&load_config());

    let (tx, rx) = smol::channel::unbounded();
    let tray = ksni::blocking::TrayMethods::spawn(Tray {
        tx,
        icon: tray_icon(),
    });
    // Without a tray there's no way back to a closed window, so closing it quits
    let quit_mode = match &tray {
        Ok(_) => QuitMode::Explicit,
        Err(e) => {
            eprintln!("Tray unavailable ({e}), closing the window will quit");
            QuitMode::LastWindowClosed
        }
    };

    gpui_kit::application()
        .with_quit_mode(quit_mode)
        .run(move |cx| {
            gpui_kit::init(cx);
            ui::init(cx);
            open_main_window(monitor.clone(), cx);

            // With QuitMode::Explicit, closing the window only quits if the user opted out of the tray
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() && !load_config().close_to_tray {
                    cx.quit();
                }
            })
            .detach();

            cx.spawn(async move |cx| {
                let _tray = tray;
                while let Ok(cmd) = rx.recv().await {
                    cx.update(|cx| match cmd {
                        TrayCmd::Quit => cx.quit(),
                        TrayCmd::Restore => match cx.windows().first() {
                            Some(window) => {
                                let _ = window.update(cx, |_, window, _| window.activate_window());
                            }
                            None => open_main_window(monitor.clone(), cx),
                        },
                    });
                }
            })
            .detach();
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_drive_size_tb() {
        // 1 TB = 1024^4 bytes = 1099511627776 bytes = 2147483648 sectors (512 bytes each)
        let sectors_1tb = 1_099_511_627_776 / 512;
        let result = format_drive_size(sectors_1tb);
        assert!(result.contains("TB"));
        // Result should be close to 1.0 TB (allow for formatting precision)
        let value: f64 = result.split_whitespace().next().unwrap().parse().unwrap();
        assert!(
            (0.9..=1.1).contains(&value),
            "Expected ~1.0 TB, got: {}",
            result
        );
    }

    #[test]
    fn test_format_drive_size_gb() {
        // 500 GB = 500 * 1024^3 bytes = 536870912000 bytes = 1048576000 sectors
        let sectors_500gb = 536_870_912_000 / 512;
        let result = format_drive_size(sectors_500gb);
        assert!(result.contains("GB"));
        assert!(result.starts_with("500.0") || result.starts_with("500.1")); // Allow small rounding
    }

    #[test]
    fn test_format_drive_size_small() {
        // 10 GB
        let sectors_10gb = 10 * 1024 * 1024 * 1024 / 512;
        let result = format_drive_size(sectors_10gb);
        assert!(result.contains("GB"));
        assert!(result.starts_with("10.0"));
    }

    #[test]
    fn test_format_drive_size_large() {
        // 2 TB
        let sectors_2tb = 2 * 1024 * 1024 * 1024 * 1024 / 512;
        let result = format_drive_size(sectors_2tb);
        assert!(result.contains("TB"));
        assert!(result.starts_with("2.0"));
    }

    #[test]
    fn test_format_drive_size_zero() {
        let result = format_drive_size(0);
        assert!(result.contains("GB")); // Should format as GB even if 0
    }

    #[test]
    fn test_is_virtual_device_loop() {
        assert!(is_virtual_device("loop0"));
        assert!(is_virtual_device("loop1"));
        assert!(!is_virtual_device("sda"));
    }

    #[test]
    fn test_is_virtual_device_ram() {
        assert!(is_virtual_device("ram0"));
        assert!(is_virtual_device("ram1"));
        assert!(!is_virtual_device("sda"));
    }

    #[test]
    fn test_is_virtual_device_zram() {
        assert!(is_virtual_device("zram0"));
        assert!(!is_virtual_device("sda"));
    }

    #[test]
    fn test_is_virtual_device_dm() {
        assert!(is_virtual_device("dm-0"));
        assert!(is_virtual_device("dm-1"));
        assert!(!is_virtual_device("sda"));
    }

    #[test]
    fn test_is_virtual_device_sr() {
        assert!(is_virtual_device("sr0"));
        assert!(is_virtual_device("sr1"));
        assert!(!is_virtual_device("sda"));
    }

    #[test]
    fn test_is_virtual_device_fd() {
        assert!(is_virtual_device("fd0"));
        assert!(!is_virtual_device("sda"));
    }

    #[test]
    fn test_is_virtual_device_real_drives() {
        assert!(!is_virtual_device("sda"));
        assert!(!is_virtual_device("sdb"));
        assert!(!is_virtual_device("nvme0n1"));
        assert!(!is_virtual_device("nvme1n1p1"));
        assert!(!is_virtual_device("hda"));
    }
}
