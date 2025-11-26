mod config;
mod monitor;

use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tauri::WebviewUrl;
use tauri::{
    Manager, State,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

use config::{Config, DriveConfig, load_config, save_config};
use monitor::Monitor;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Drive {
    name: String,
    model: String,
    size_display: String,
    config: DriveConfig,
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

fn get_drive_size_display(device_name: &str) -> String {
    let size_path = Path::new("/sys/block").join(device_name).join("size");
    if let Ok(size_content) = fs::read_to_string(&size_path)
        && let Ok(sectors) = size_content.trim().parse::<u64>()
    {
        // Convert sectors (512 bytes) to bytes, then to TB
        let bytes = sectors * 512;
        let tb = bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0);
        if tb >= 1.0 {
            return format!("{:.1} TB", tb);
        }
        let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        return format!("{:.1} GB", gb);
    }
    "Unknown".to_string()
}

fn scan_hard_drives(config: &Config) -> Vec<Drive> {
    let sys_block = Path::new("/sys/block");
    let mut drives = Vec::new();

    if let Ok(entries) = fs::read_dir(sys_block) {
        for entry in entries.flatten() {
            let device_name = entry.file_name();
            let device_str = device_name.to_string_lossy().to_string();

            // Filter out virtual devices (loop, ram, zram, etc.)
            if device_str.starts_with("loop")
                || device_str.starts_with("ram")
                || device_str.starts_with("zram")
                || device_str.starts_with("dm-")
                || device_str.starts_with("sr")
                || device_str.starts_with("fd")
            {
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
                let default_config = DriveConfig::default();
                let drive_config = config.drives.get(&device_str).unwrap_or(&default_config);
                let size_display = get_drive_size_display(&device_str);
                drives.push(Drive {
                    name: device_str,
                    model: display_name,
                    size_display,
                    config: drive_config.to_owned(),
                });
            }
        }
    }

    drives.sort_by(|a, b| a.model.cmp(&b.model));
    drives
}

#[tauri::command]
fn get_drives(monitor_state: State<'_, Arc<Mutex<Monitor>>>) -> Result<Vec<Drive>, String> {
    let config = load_config();
    Ok(scan_hard_drives(&config))
}

#[tauri::command]
fn load_config_command() -> Result<Config, String> {
    Ok(load_config())
}

#[tauri::command]
fn save_config_command(
    config: Config,
    monitor_state: State<'_, Arc<Mutex<Monitor>>>,
) -> Result<(), String> {
    save_config(&config);
    let monitor = monitor_state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    monitor.update_config(&config);
    Ok(())
}

#[tauri::command]
fn update_drive_config(
    device_name: String,
    drive_config: DriveConfig,
    monitor_state: State<'_, Arc<Mutex<Monitor>>>,
) -> Result<(), String> {
    let mut config = load_config();
    config
        .drives
        .insert(device_name.clone(), drive_config.clone());
    save_config(&config);
    let monitor = monitor_state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    monitor.update_config(&config);
    Ok(())
}

#[tauri::command]
fn update_global_config(
    active: Option<bool>,
    poll_interval_ms: Option<u64>,
    monitor_state: State<'_, Arc<Mutex<Monitor>>>,
) -> Result<(), String> {
    let mut config = load_config();
    if let Some(a) = active {
        config.active = a;
    }
    if let Some(interval) = poll_interval_ms {
        config.poll_interval_ms = interval.clamp(50, 1000);
    }
    save_config(&config);
    let monitor = monitor_state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    monitor.update_config(&config);
    Ok(())
}

fn main() {
    let config = load_config();
    let monitor = Arc::new(Mutex::new(Monitor::new()));
    monitor.lock().unwrap().update_config(&config);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(monitor)
        .invoke_handler(tauri::generate_handler![
            get_drives,
            load_config_command,
            save_config_command,
            update_drive_config,
            update_global_config
        ])
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let restore_i = MenuItem::with_id(app, "restore", "Restore", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&restore_i, &quit_i])?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app_handle, event| match event.id.as_ref() {
                    "quit" => {
                        println!("quit menu item was clicked");
                        app_handle.exit(99);
                    }
                    "restore" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        } else {
                            tauri::WebviewWindowBuilder::new(
                                app_handle,
                                "main",
                                WebviewUrl::App("index.html".into()),
                            )
                            .title("HDD Simulator")
                            .build()
                            .unwrap();
                        }
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() || code.unwrap() != 99 {
                    api.prevent_exit();
                }
                for (_label, window) in app.webview_windows() {
                    window.close().unwrap();
                }
            }
        });
}
