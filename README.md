# HDD Simulator

Makes your drives click like an old spinning hard disk. Pick a drive, turn on read and/or write, and every burst of disk activity plays a seek click. Built in Rust with [GPUI](https://github.com/longbridge/gpui-kit).

Linux only: drive activity comes from `/sys/block`.

## Requirements

Install Rust with [rustup](https://rustup.rs/), then the system libraries:

```bash
# Fedora
sudo dnf install libxkbcommon-devel libxkbcommon-x11-devel wayland-devel vulkan-loader-devel \
    fontconfig-devel freetype-devel libxcb-devel alsa-lib-devel dbus-devel

# Debian/Ubuntu
sudo apt install libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libvulkan-dev \
    libfontconfig-dev libfreetype-dev libxcb1-dev libasound2-dev libdbus-1-dev
```

## Run

```bash
cargo run            # development
cargo build --release  # binary at target/release/hdd-simulator
cargo test
```

### Desktop entry (window and taskbar icon)

On Wayland the compositor looks up the icon from a `.desktop` file matching the app ID, so install it once:

```bash
install -Dm644 assets/ca.rusticotter.hdd-simulator.desktop -t ~/.local/share/applications/
install -Dm644 assets/app-icon.png ~/.local/share/icons/hicolor/256x256/apps/ca.rusticotter.hdd-simulator.png
```

Closing the window keeps the simulator running in the system tray (needs a StatusNotifierItem host, e.g. KDE, or GNOME with the AppIndicator extension). Use the tray menu to restore the window or quit. Without a tray host, closing the window quits.

Settings live in `~/.config/hdd-simulator/config.toml`.

## Project structure

- `src/main.rs` – drive discovery, tray, app startup
- `src/ui.rs` – the window (RusticOtter brand colours and fonts)
- `src/monitor.rs` – polls `/sys/block/*/stat` and synthesizes the click sound
- `src/config.rs` – TOML config
- `assets/` – icon, mascot, bundled Inter and Zilla Slab fonts (SIL OFL)
