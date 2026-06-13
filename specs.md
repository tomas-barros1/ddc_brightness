# DDC Brightness

## Overview

Build a lightweight Linux desktop application for controlling the brightness of external monitors through DDC/CI using `ddcutil`.

The application should focus on simplicity and provide only two core features:

1. Select a monitor.
2. Adjust its brightness using a slider.

The UI should feel native, modern, and minimal, similar to GNOME/Libadwaita applications.

---

# Technology Stack

## Backend

- Rust
- `ddcutil` for monitor communication
- System command execution through Rust

## Frontend

Preferred:

- GTK4
- Libadwaita

Alternative:

- GTK4 only

The application should follow modern GNOME Human Interface Guidelines where practical.

---

# Features

## Monitor Detection

On startup, discover all available DDC/CI-compatible monitors.

Run:

```bash
ddcutil detect --brief
```

or

```bash
ddcutil detect
```

Extract:

- Monitor name
- Display number
- I2C bus (if available)

Example output:

```text
Display 1
   I2C bus: /dev/i2c-7
   Monitor: LG UltraGear

Display 2
   I2C bus: /dev/i2c-9
   Monitor: Dell U2720Q
```

Populate a monitor picker with user-friendly names:

```text
LG UltraGear
Dell U2720Q
```

---

## Read Current Brightness

When a monitor is selected:

Run:

```bash
ddcutil getvcp 10 --display N
```

Example:

```text
VCP code 0x10 (Brightness): current value = 70, max value = 100
```

Extract:

- Current brightness
- Maximum brightness

Update the slider accordingly.

---

## Set Brightness

When the user moves the slider:

Run:

```bash
ddcutil setvcp 10 VALUE --display N
```

Example:

```bash
ddcutil setvcp 10 80 --display 1
```

---

# User Interface

## Window Layout

The application should be intentionally small and minimal.

Example:

```text
+----------------------------------+
|        DDC Brightness            |
|                                  |
| Monitor                          |
| [ LG UltraGear             ▼ ]   |
|                                  |
| Brightness                       |
|                                  |
| -----------●------------------   |
|                 70%              |
|                                  |
+----------------------------------+
```

---

## Components

### Header

Application title:

```text
DDC Brightness
```

No toolbar buttons are necessary.

---

### Monitor Picker

Use:

- ComboBox
- DropDown

Example:

```text
[ LG UltraGear ▼ ]
```

When the selected monitor changes:

1. Read the monitor's current brightness.
2. Update the slider position.

---

### Brightness Slider

Range:

```text
0 → 100
```

Display the current brightness percentage beneath the slider.

Example:

```text
73%
```

---

# User Experience

## Debouncing

Do not execute:

```bash
ddcutil setvcp
```

for every slider movement event.

Implement a debounce of approximately:

```text
150–250 ms
```

Only send the brightness command after the user stops dragging.

---

## Loading State

While querying monitors or brightness values:

- Disable controls
- Show a loading spinner

---

## Error Handling

### DDC/CI Unsupported

Display:

```text
This monitor does not support DDC/CI.
```

### ddcutil Missing

Display:

```text
ddcutil is not installed or could not be found.
```

### Command Failure

Display a non-intrusive error banner:

```text
Failed to communicate with the monitor.
```

---

# Design Requirements

## Visual Style

The application should:

- Feel native on GNOME
- Use Libadwaita styling
- Support light and dark themes automatically
- Avoid custom theming
- Avoid unnecessary controls

Design inspiration:

- GNOME Settings
- GNOME Extensions Manager
- Loupe
- GNOME Calculator

---

# Performance Requirements

- Fast startup
- Low memory usage
- No background services
- No telemetry
- No network access

---

# Architecture

Suggested structure:

```text
src/
├── main.rs
├── ui/
│   ├── window.rs
│   └── widgets.rs
├── ddc/
│   ├── detect.rs
│   ├── brightness.rs
│   └── parser.rs
└── models/
    └── monitor.rs
```

---

# Future Enhancements (Not Part of MVP)

Do not implement the following in the initial version:

- System tray support
- Global brightness shortcuts
- Brightness profiles
- Multi-monitor control view
- Contrast controls
- RGB controls
- Auto-refresh
- Hyprland integration
- Keyboard shortcuts
- Configuration file

---

# MVP Scope

The first version should only include:

- Monitor discovery via `ddcutil`
- Monitor selection
- Reading current brightness
- Brightness adjustment through a slider
- Proper error handling
- Modern GTK4/Libadwaita interface

Keep the application focused, polished, and extremely simple.
