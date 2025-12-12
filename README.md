# HDD Simulator

A desktop application for simulating hard disk drives, built with Tauri and Vue.js.

## Requirements

### Rust

Install Rust using [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, ensure Rust is in your PATH and verify:

```bash
rustc --version
cargo --version
```

### Node.js and npm

Install Node.js (version 18 or higher recommended) from [nodejs.org](https://nodejs.org/).

Verify installation:

```bash
node --version
npm --version
```

### System Dependencies

#### Linux

Install system dependencies:

```bash
# Debian/Ubuntu
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel.x86_64 \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator \
    librsvg2-devel

# Arch Linux
sudo pacman -S webkit2gtk \
    base-devel \
    curl \
    wget \
    openssl \
    appmenu-gtk-module \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    libvips
```

#### macOS

Install Xcode Command Line Tools:

```bash
xcode-select --install
```

#### Windows

Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) or Visual Studio with C++ support.

## Installation

1. Clone the repository:

```bash
git clone <repository-url>
cd hdd-sim
```

2. Install npm dependencies (this includes Tauri CLI as a dev dependency):

```bash
npm ci
```

## Development

Run the application in development mode:

```bash
npm run tauri:dev
```

This will:
- Start the Vite dev server for the frontend
- Build and run the Tauri application
- Enable hot-reload for both frontend and backend changes

## Building

Build the application for production:

```bash
npm run tauri:build
```

The built application will be located in `src-tauri/target/release/`:
- **Linux**: `bundle/appimage/` or `bundle/deb/`
- **macOS**: `bundle/macos/`
- **Windows**: `bundle/msi/` or `bundle/nsis/`

## Project Structure

- `src/` - Vue.js frontend source code
- `src-tauri/` - Rust backend source code
- `src-tauri/src/` - Rust application code
- `dist/` - Frontend build output (generated)

## Scripts

- `npm run dev` - Start Vite dev server only
- `npm run build` - Build frontend only
- `npm run tauri:dev` - Run Tauri in development mode
- `npm run tauri:build` - Build Tauri application for production
- `npm run tauri` - Run Tauri CLI commands directly

## Troubleshooting

### Tauri CLI not found

If you encounter issues with Tauri CLI, ensure npm dependencies are installed:

```bash
npm ci
```

### Build errors on Linux

Ensure all system dependencies are installed (see System Dependencies section above).

### Port 5173 already in use

The dev server uses port 5173. If it's occupied, you can modify `vite.config.js` to use a different port.
