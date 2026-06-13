# DDC Brightness

Lightweight Linux desktop application for controlling external monitor brightness via **DDC/CI** using `ddcutil`.

Built with **Rust**, **GTK4**, and **libadwaita**. Minimal, native GNOME UI — no dependencies beyond `ddcutil` and standard GTK4/libadwaita libraries.

## Features

- 🔍 **Monitor detection** — discovers all DDC/CI-compatible monitors on startup
- 🎚️ **Brightness slider** — smooth `GtkScale` with live percentage display
- ⏱️ **Debounced writes** — 200ms debounce before calling `ddcutil setvcp`, avoids hammering the I2C bus
- ⏳ **Loading states** — spinner while querying monitors and reading brightness
- 🚫 **Error handling** — toast notifications for missing `ddcutil`, unsupported monitors, and command failures
- 🌗 **Native look** — libadwaita styling, light/dark theme support, matches GNOME Settings

## Requirements

- Linux with a DDC/CI-capable monitor
- `ddcutil` (≥ 2.0) — `pacman -S ddcutil` / `apt install ddcutil`
- GTK4 and libadwaita runtime libraries

## Install

### From source

```bash
git clone https://github.com/yourusername/ddc_brightness
cd ddc_brightness
make && sudo make install PREFIX=/usr
```

Or manually:

```bash
cargo build --release
sudo install -Dm755 target/release/ddc_brightness /usr/bin/ddc_brightness
sudo install -Dm644 ddc_brightness.desktop /usr/share/applications/ddc_brightness.desktop
```

### Arch Linux (AUR)

```
yay -S ddc_brightness
```

## Usage

Launch from the application menu or run:

```bash
ddc_brightness
```

1. Select your monitor from the dropdown.
2. Drag the brightness slider.

That's it.

## Project structure

```
src/
├── main.rs              # Application entry point
├── models/
│   └── monitor.rs       # Monitor data struct
├── ddc/
│   ├── detect.rs        # Monitor discovery via ddcutil detect
│   ├── brightness.rs    # Read/set brightness via ddcutil getvcp/setvcp
│   └── parser.rs        # Parse ddcutil output
└── ui/
    └── window.rs        # Main window, widgets, threading, debounce
```

## Build

```bash
cargo build --release
```

## License

MIT
