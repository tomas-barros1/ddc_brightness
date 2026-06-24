# DDC Brightness

[![Crates.io](https://img.shields.io/crates/v/ddc_brightness?style=flat-square)](https://crates.io/crates/ddc_brightness)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![GTK4](https://img.shields.io/badge/GTK-4-blue?style=flat-square&logo=gtk)](https://gtk.org)
[![Libadwaita](https://img.shields.io/badge/libadwaita-1.6-blue?style=flat-square)](https://gnome.pages.gitlab.gnome.org/libadwaita/)

**Lightweight Linux desktop application for controlling external monitor brightness via DDC/CI.** A minimal, native GTK4/Libadwaita GUI frontend for `ddcutil` — discovers DDC/CI-compatible monitors and lets you adjust brightness with a slider or by typing a value.

Built with **Rust**, **GTK4**, and **libadwaita**. No web technologies, no configuration files, no background services — just a clean GNOME-native brightness control panel.

> Compatible with any monitor that supports DDC/CI over I2C. Works on GNOME, KDE, Sway, Hyprland, and any Linux desktop with GTK4/libadwaita.

## Features

- 🔍 **Automatic monitor detection** — discovers all DDC/CI-compatible monitors at startup via `ddcutil detect`
- 🎚️ **Brightness slider + numeric input** — drag the `GtkScale` or type a value directly in the `SpinButton`
- ⌨️ **Keyboard shortcuts** — arrow keys (5% steps), Enter for immediate apply
- 🖱️ **Mouse wheel support** — scroll on the slider to adjust brightness
- ⏱️ **Debounced writes** — 200ms debounce before calling `ddcutil setvcp` to avoid saturating the I2C bus
- 🔒 **Concurrent access safety** — serializes DDC/CI commands to prevent I2C bus race conditions
- ⏳ **Loading states** — spinner while querying monitors and reading brightness values
- 🚫 **Error handling** — toast notifications for missing `ddcutil`, unsupported monitors, and command failures
- 🌗 **Native look & feel** — libadwaita styling, automatic light/dark theme support, matches GNOME Settings

## Requirements

- **Linux** with a **DDC/CI-capable monitor** connected via DisplayPort, HDMI, or USB-C
- **`ddcutil`** (≥ 2.0) — the underlying DDC/CI communication tool
  - Arch: `sudo pacman -S ddcutil`
  - Debian/Ubuntu: `sudo apt install ddcutil`
  - Fedora: `sudo dnf install ddcutil`
- **GTK4** and **libadwaita** runtime libraries (included in most modern Linux desktops)

## Install

### AUR (Arch Linux)

```bash
yay -S ddc_brightness
```

### From source

```bash
git clone https://github.com/tomas-barros1/ddc_brightness.git
cd ddc_brightness
make && sudo make install PREFIX=/usr
```

Or manually:

```bash
cargo build --release
sudo install -Dm755 target/release/ddc_brightness /usr/bin/ddc_brightness
sudo install -Dm644 ddc_brightness.desktop /usr/share/applications/ddc_brightness.desktop
```

### Cargo

```bash
cargo install ddc_brightness
```

## Usage

Launch from the application menu (`DDC Brightness`) or run:

```bash
ddc_brightness
```

1. Select your monitor from the dropdown.
2. Adjust brightness by dragging the slider, scrolling, pressing arrow keys (↑/↓), or typing a value.
3. Press Enter in the numeric field to apply immediately.

That's it.

## Build

```bash
# Debug
cargo build

# Release (with LTO, symbol stripping, codegen-units=1)
cargo build --release
```

### Optimized release profile

```toml
[profile.release]
lto = true
codegen-units = 1
strip = "symbols"
opt-level = 3
```

### Project structure

```
src/
├── main.rs              # Application entry point (adw::Application)
├── models/
│   └── monitor.rs       # Monitor data struct
├── ddc/
│   ├── detect.rs        # Monitor discovery via ddcutil detect
│   ├── brightness.rs    # Read/set brightness via ddcutil getvcp/setvcp
│   └── parser.rs        # Parse ddcutil CLI output
└── ui/
    └── window.rs        # Main window, widgets, threading, debounce, shortcuts
```

## Keywords / Related

- **DDC/CI** — Display Data Channel / Command Interface
- **Monitor brightness control** — external display brightness on Linux
- **I2C** — hardware communication bus used by DDC/CI
- **GNOME brightness app** — alternative to `gnome-control-center` display settings
- **Rust GTK4** — Rust bindings for the GTK4 toolkit
- **Libadwaita** — GNOME's adaptive widget library

## License

MIT — see [LICENSE](LICENSE) for details.
