use std::fs;
use std::path::Path;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Label, ListBox, ScrolledWindow, glib};

const APP_ID: &str = "ca.rusticotter.hdd-simulator";

#[derive(Debug, Clone)]
struct Drive {
    name: String,
    model: String,
}

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
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

fn scan_hard_drives() -> Vec<Drive> {
    let sys_block = Path::new("/sys/block");
    let mut drives = Vec::new();

    if let Ok(entries) = fs::read_dir(sys_block) {
        for entry in entries.flatten() {
            dbg!(&entry);
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
                drives.push(Drive {
                    name: device_str,
                    model: display_name,
                });
            }
        }
    }

    drives.sort_by(|a, b| a.model.cmp(&b.model));
    drives
}

fn build_ui(application: &Application) {
    let drives = scan_hard_drives();

    let list_box = ListBox::builder()
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    if drives.is_empty() {
        let label = Label::builder()
            .label("No hard drives found")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        list_box.append(&label);
    } else {
        for drive in drives {
            let label = Label::builder()
                .label(&drive.model)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(12)
                .margin_end(12)
                .halign(gtk::Align::Start)
                .build();
            list_box.append(&label);
        }
    }

    let scrolled = ScrolledWindow::builder().child(&list_box).build();

    let window = ApplicationWindow::builder()
        .application(application)
        .title("HDD Simulator")
        .default_width(400)
        .default_height(600)
        .child(&scrolled)
        .build();

    window.present();
}
