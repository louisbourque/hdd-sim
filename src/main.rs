mod config;

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use gdk4::Key;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, EventControllerKey, Label, ListBox, ListBoxRow, Paned,
    ScrolledWindow, Switch, glib,
};

use config::{Config, DriveConfig, load_config, save_config};

const APP_ID: &str = "ca.rusticotter.hdd-simulator";

#[derive(Debug, Clone)]
struct Drive {
    name: String,
    model: String,
    config: DriveConfig,
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
                let config = config
                    .drives
                    .get(&device_str)
                    .unwrap_or(&DriveConfig { enabled: false });
                drives.push(Drive {
                    name: device_str,
                    model: display_name,
                    config: config.to_owned(),
                });
            }
        }
    }

    drives.sort_by(|a, b| a.model.cmp(&b.model));
    drives
}

fn build_ui(application: &Application) {
    let config = load_config();
    let drives = scan_hard_drives(&config);

    // Create the left sidebar with drive list
    let list_box = ListBox::builder().css_classes(vec!["sidebar"]).build();

    // Store drives in a shared container for access in callbacks
    let drives_rc = Rc::new(RefCell::new(drives));

    // Create drive list items
    if drives_rc.borrow().is_empty() {
        let label = Label::builder()
            .label("No hard drives found")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        list_box.append(&label);
    } else {
        for drive in drives_rc.borrow().iter() {
            let row = ListBoxRow::builder().build();

            // Create a box to hold the drive info
            let row_box = Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .margin_start(12)
                .margin_end(12)
                .margin_top(12)
                .margin_bottom(12)
                .spacing(4)
                .build();

            // Drive name label
            let name_label = Label::builder()
                .label(format!("{} Disk", get_drive_size_display(&drive.name)))
                .halign(gtk::Align::Start)
                .css_classes(vec!["title-4"])
                .build();

            // Model label
            let model_label = Label::builder()
                .label(&drive.model)
                .halign(gtk::Align::Start)
                .css_classes(vec!["dim-label"])
                .build();

            row_box.append(&name_label);
            row_box.append(&model_label);
            row_box.set_css_classes(&["drive-row"]);

            row.set_child(Some(&row_box));
            row.set_selectable(true);

            list_box.append(&row);
        }
    }

    let left_scrolled = ScrolledWindow::builder()
        .child(&list_box)
        .width_request(250)
        .build();

    // Create the right pane for settings
    let settings_container = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .margin_start(24)
        .margin_end(24)
        .margin_top(24)
        .margin_bottom(24)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .vexpand(true)
        .css_classes(vec!["settings-container"])
        .build();

    // Create empty state view
    let empty_state = create_empty_state_view();
    settings_container.append(&empty_state);

    // Ensure settings_container expands to fill available space
    settings_container.set_halign(gtk::Align::Fill);
    settings_container.set_valign(gtk::Align::Fill);

    let right_scrolled = ScrolledWindow::builder()
        .child(&settings_container)
        .min_content_width(1)
        .min_content_height(1)
        .build();

    // Create paned widget to split left and right
    let paned = Paned::builder()
        .orientation(gtk::Orientation::Horizontal)
        .shrink_start_child(false)
        .shrink_end_child(false)
        .resize_start_child(true)
        .resize_end_child(true)
        .position(250)
        .build();
    paned.set_start_child(Some(&left_scrolled));
    paned.set_end_child(Some(&right_scrolled));

    // Handle drive selection
    let empty_state_clone = empty_state.clone();
    let settings_container_clone = settings_container.clone();
    let drives_clone = drives_rc.clone();
    list_box.connect_row_selected(move |_, row| {
        // Remove existing settings view if present
        // empty_state is always the first child, so if last_child != first_child, remove it
        if let (Some(first_child), Some(last_child)) = (
            settings_container_clone.first_child(),
            settings_container_clone.last_child(),
        ) && first_child != last_child
        {
            settings_container_clone.remove(&last_child);
        }

        if let Some(row) = row {
            let drive_index = row.index() as usize;
            if let Some(drive) = drives_clone.borrow().get(drive_index) {
                // Create and add new settings view with drive state
                let settings_view = create_settings_view(drive, drive_index, drives_clone.clone());
                settings_container_clone.append(&settings_view);
                empty_state_clone.set_visible(false);
            } else {
                // No drive found - show empty state
                empty_state_clone.set_visible(true);
            }
        } else {
            // No selection - show empty state
            empty_state_clone.set_visible(true);
        }
    });

    let window = ApplicationWindow::builder()
        .application(application)
        .title("HDD Simulator")
        .default_width(800)
        .default_height(600)
        .child(&paned)
        .build();

    // Handle ESC key to unselect all rows
    let list_box_clone = list_box.clone();
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == Key::Escape {
            list_box_clone.unselect_all();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    window.present();
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

fn create_empty_state_view() -> Box {
    // Outer container that expands to fill available space
    let outer_container = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(vec!["debug-outer"])
        .build();
    outer_container.set_hexpand(true);
    outer_container.set_vexpand(true);

    // Inner container that holds the content and doesn't expand
    let inner_container = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .css_classes(vec!["debug-inner"])
        .build();

    // Icon placeholder (using a large label as icon)
    let icon_label = Label::builder()
        .label("💾")
        .css_classes(vec!["empty-state-icon"])
        .build();
    icon_label.set_markup("<span size='xx-large'>💾</span>");

    let title_label = Label::builder()
        .label("No Device Selected")
        .css_classes(vec!["title-1"])
        .halign(gtk::Align::Center)
        .build();

    let subtitle_label = Label::builder()
        .label("Select a device to manage.")
        .css_classes(vec!["dim-label"])
        .halign(gtk::Align::Center)
        .build();

    inner_container.append(&icon_label);
    inner_container.append(&title_label);
    inner_container.append(&subtitle_label);

    outer_container.append(&inner_container);

    outer_container
}

fn create_settings_view(drive: &Drive, drive_index: usize, drives: Rc<RefCell<Vec<Drive>>>) -> Box {
    let container = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(24)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Start)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Settings section
    let settings_section = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();

    let section_title = Label::builder()
        .label("Settings")
        .css_classes(vec!["title-2"])
        .halign(gtk::Align::Start)
        .build();

    // Enable/Disable toggle
    let toggle_row = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();

    let toggle_label = Label::builder()
        .label("Enabled")
        .halign(gtk::Align::Start)
        .build();

    let toggle_switch = Switch::builder()
        .active(drive.config.enabled)
        .halign(gtk::Align::End)
        .build();

    // Connect toggle switch to update drive.enabled when state changes
    let drives_clone = drives.clone();
    toggle_switch.connect_state_set(move |_switch, new_state| {
        // new_state is the state being set, use it directly
        drives_clone.borrow_mut()[drive_index].config.enabled = new_state;

        // Save config to persist the change
        let mut config = Config {
            drives: std::collections::HashMap::new(),
        };
        for drive in drives_clone.borrow().iter() {
            config
                .drives
                .insert(drive.name.clone(), drive.config.to_owned());
        }
        save_config(&config);

        glib::Propagation::Proceed
    });

    toggle_row.append(&toggle_label);
    toggle_row.append(&toggle_switch);

    settings_section.append(&section_title);
    settings_section.append(&toggle_row);

    container.append(&settings_section);

    container
}
