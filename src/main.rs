mod config;
mod monitor;

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use gdk4::Key;
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Button, EventControllerKey, Label, ListBox, ListBoxRow,
    Paned, Scale, ScrolledWindow, Switch, glib,
};

use config::{Config, DriveConfig, load_config, save_config};
use monitor::Monitor;

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
                let default_config = DriveConfig::default();
                let config = config.drives.get(&device_str).unwrap_or(&default_config);
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

    // Create and start the monitor
    let monitor = Monitor::new();
    monitor.update_config(&config);

    // Create the left sidebar with drive list
    let list_box = ListBox::builder().css_classes(vec!["sidebar"]).build();

    // Store drives in a shared container for access in callbacks
    let drives_rc = Rc::new(RefCell::new(drives));
    let monitor_rc = Rc::new(monitor);

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

    // Create main container with paned and global settings
    let main_container = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    paned.set_vexpand(true);
    main_container.append(&paned);

    // Create global settings section
    let config_rc = Rc::new(RefCell::new(config));
    let monitor_clone_for_global = monitor_rc.clone();
    let drives_clone_for_global = drives_rc.clone();
    let global_settings = create_global_settings_view(
        config_rc.clone(),
        monitor_clone_for_global,
        drives_clone_for_global,
    );
    main_container.append(&global_settings);

    // Handle drive selection
    let empty_state_clone = empty_state.clone();
    let settings_container_clone = settings_container.clone();
    let drives_clone = drives_rc.clone();
    let monitor_clone_for_selection = monitor_rc.clone();
    let config_clone_for_selection = config_rc.clone();
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
                let settings_view = create_settings_view(
                    drive,
                    drive_index,
                    drives_clone.clone(),
                    monitor_clone_for_selection.clone(),
                    config_clone_for_selection.clone(),
                );
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
        .child(&main_container)
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

fn save_full_config(
    drives: &Rc<RefCell<Vec<Drive>>>,
    global_config: &Rc<RefCell<Config>>,
    monitor: &Monitor,
) {
    let config_borrow = global_config.borrow();
    let mut config = Config {
        drives: std::collections::HashMap::new(),
        active: config_borrow.active,
        poll_interval_ms: config_borrow.poll_interval_ms,
    };
    for drive in drives.borrow().iter() {
        config
            .drives
            .insert(drive.name.clone(), drive.config.to_owned());
    }
    save_config(&config);
    monitor.update_config(&config);
}

fn create_settings_view(
    drive: &Drive,
    drive_index: usize,
    drives: Rc<RefCell<Vec<Drive>>>,
    monitor: Rc<Monitor>,
    global_config: Rc<RefCell<Config>>,
) -> Box {
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
        .spacing(20)
        .build();

    let section_title = Label::builder()
        .label(format!("{} Settings", drive.model))
        .css_classes(vec!["title-2"])
        .halign(gtk::Align::Start)
        .build();

    // Enabled toggle (separate section)
    let enabled_row = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();

    let enabled_label = Label::builder()
        .label("Enabled")
        .halign(gtk::Align::Start)
        .width_chars(8)
        .build();

    let enabled_switch = Switch::builder()
        .active(drive.config.enabled)
        .halign(gtk::Align::End)
        .build();

    enabled_row.append(&enabled_label);
    enabled_row.append(&enabled_switch);

    // Group Read and Write together for better visual alignment
    let io_settings_group = Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();

    // Read toggle
    let read_row = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();

    let read_label = Label::builder()
        .label("Read")
        .halign(gtk::Align::Start)
        .width_chars(8)
        .build();

    let read_switch = Switch::builder()
        .active(drive.config.read)
        .halign(gtk::Align::End)
        .sensitive(drive.config.enabled)
        .build();

    let drives_clone = drives.clone();
    let monitor_clone = monitor.clone();
    let global_config_clone = global_config.clone();
    read_switch.connect_state_set(move |_switch, new_state| {
        drives_clone.borrow_mut()[drive_index].config.read = new_state;
        save_full_config(&drives_clone, &global_config_clone, &monitor_clone);
        glib::Propagation::Proceed
    });

    read_row.append(&read_label);
    read_row.append(&read_switch);

    // Write toggle
    let write_row = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();

    let write_label = Label::builder()
        .label("Write")
        .halign(gtk::Align::Start)
        .width_chars(8)
        .build();

    let write_switch = Switch::builder()
        .active(drive.config.write)
        .halign(gtk::Align::End)
        .sensitive(drive.config.enabled)
        .build();

    let drives_clone = drives.clone();
    let monitor_clone = monitor.clone();
    let global_config_clone = global_config.clone();
    write_switch.connect_state_set(move |_switch, new_state| {
        drives_clone.borrow_mut()[drive_index].config.write = new_state;
        save_full_config(&drives_clone, &global_config_clone, &monitor_clone);
        glib::Propagation::Proceed
    });

    write_row.append(&write_label);
    write_row.append(&write_switch);

    io_settings_group.append(&read_row);
    io_settings_group.append(&write_row);

    // Volume slider (0-100)
    let volume_row = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();

    let volume_label = Label::builder()
        .label("Volume")
        .halign(gtk::Align::Start)
        .width_chars(8)
        .build();

    let volume_value_label = Label::builder()
        .label(drive.config.volume.to_string())
        .halign(gtk::Align::End)
        .width_chars(3)
        .build();

    let adjustment = gtk::Adjustment::new(drive.config.volume as f64, 0.0, 100.0, 1.0, 5.0, 0.0);

    let volume_scale = Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .adjustment(&adjustment)
        .draw_value(false)
        .hexpand(true)
        .width_request(100)
        .sensitive(drive.config.enabled)
        .build();

    let drives_clone = drives.clone();
    let monitor_clone = monitor.clone();
    let global_config_clone = global_config.clone();
    let volume_value_label_clone = volume_value_label.clone();
    volume_scale.connect_value_changed(move |scale| {
        let value = scale.value() as u8;
        volume_value_label_clone.set_label(&format!("{}", value));
        drives_clone.borrow_mut()[drive_index].config.volume = value;
        save_full_config(&drives_clone, &global_config_clone, &monitor_clone);
    });

    volume_row.append(&volume_label);
    volume_row.append(&volume_scale);
    volume_row.append(&volume_value_label);

    // Connect enabled switch to update sensitivity of all other settings
    let read_switch_clone = read_switch.clone();
    let write_switch_clone = write_switch.clone();
    let volume_scale_clone = volume_scale.clone();
    let drives_clone = drives.clone();
    let monitor_clone = monitor.clone();
    let global_config_clone = global_config.clone();
    enabled_switch.connect_state_set(move |_switch, new_state| {
        drives_clone.borrow_mut()[drive_index].config.enabled = new_state;
        save_full_config(&drives_clone, &global_config_clone, &monitor_clone);

        // Update sensitivity of all settings based on enabled state
        read_switch_clone.set_sensitive(new_state);
        write_switch_clone.set_sensitive(new_state);
        volume_scale_clone.set_sensitive(new_state);

        glib::Propagation::Proceed
    });

    settings_section.append(&section_title);
    settings_section.append(&enabled_row);
    settings_section.append(&io_settings_group);
    settings_section.append(&volume_row);

    container.append(&settings_section);

    container
}

fn create_global_settings_view(
    config: Rc<RefCell<Config>>,
    monitor: Rc<Monitor>,
    drives: Rc<RefCell<Vec<Drive>>>,
) -> Box {
    let container = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(16)
        .margin_start(24)
        .margin_end(24)
        .margin_top(16)
        .margin_bottom(16)
        .halign(gtk::Align::Fill)
        .css_classes(vec!["global-settings"])
        .build();

    // Play/Pause button for Active state
    let active_button = Button::builder()
        .icon_name(if config.borrow().active {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        })
        .build();

    let config_clone = config.clone();
    let monitor_clone = monitor.clone();
    let drives_clone = drives.clone();
    active_button.connect_clicked(move |button| {
        let mut config_borrow = config_clone.borrow_mut();
        config_borrow.active = !config_borrow.active;
        let is_active = config_borrow.active;
        button.set_icon_name(if is_active {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        });
        drop(config_borrow);
        save_full_config(&drives_clone, &config_clone, &monitor_clone);
    });

    // Interval section with better alignment
    let interval_row = Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .build();

    // Interval label
    let interval_label = Label::builder()
        .label("Interval")
        .halign(gtk::Align::Start)
        .width_chars(8)
        .build();

    // Interval value label
    let interval_value_label = Label::builder()
        .label(format!("{}ms", config.borrow().poll_interval_ms))
        .halign(gtk::Align::End)
        .width_chars(6)
        .build();

    // Interval slider (50ms - 1000ms)
    let current_value = config.borrow().poll_interval_ms as f64;
    let clamped_value = current_value.clamp(50.0, 1000.0);

    // Update config if value was clamped
    if (clamped_value as u64) != config.borrow().poll_interval_ms {
        config.borrow_mut().poll_interval_ms = clamped_value as u64;
        interval_value_label.set_label(&format!("{}ms", clamped_value as u64));
        save_full_config(&drives, &config, &monitor);
    }

    let adjustment = gtk::Adjustment::new(clamped_value, 50.0, 1000.0, 10.0, 100.0, 0.0);

    let interval_scale = Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .adjustment(&adjustment)
        .draw_value(false)
        .hexpand(true)
        .width_request(200)
        .build();

    let config_clone = config.clone();
    let monitor_clone = monitor.clone();
    let drives_clone = drives.clone();
    let interval_value_label_clone = interval_value_label.clone();
    interval_scale.connect_value_changed(move |scale| {
        let value = scale.value() as u64;
        interval_value_label_clone.set_label(&format!("{}ms", value));
        config_clone.borrow_mut().poll_interval_ms = value;
        save_full_config(&drives_clone, &config_clone, &monitor_clone);
    });

    interval_row.append(&interval_label);
    interval_row.append(&interval_scale);
    interval_row.append(&interval_value_label);

    container.append(&active_button);
    container.append(&interval_row);

    container
}
