# Zephyr

<div align="center">

![Zephyr](./icon.svg)

**ASUS ROG Laptop Control Center for Linux**

_Communicates with the [asusd](https://github.com/asus-linux/asusd) daemon via D-Bus to provide hardware control capabilities including fan curves, Aura lighting, power profiles, GPU switching, and more._

[![Tauri](https://img.shields.io/badge/Tauri-2.x-FFC107?style=flat-square&logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-61DAFB?style=flat-square&logo=react)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/license-GPL--3.0-blue?style=flat-square)](LICENSE)

</div>

---

## Features

### 🎛️ Hardware Control
- **Fan Curves** — Independent CPU / GPU / MID fan control with multi-point temperature-PWM curve editing
- **Power Profiles** — One-click switching between Balanced / Performance / Quiet / LowPower / Custom
- **GPU Mode** — Switch between Dedicated / Hybrid / Integrated graphics (hardware dependent)
- **Battery Management** — Charge limit settings and Battery One-Shot mode

### 💡 Lighting Control
- **Aura RGB** — Motherboard and peripheral lighting with multiple color modes
- **Anime LED** — Notebook A-cover LED panel effects
- **Slash Lighting** — Slash light strip control
- **Keyboard Backlight** — Backlight brightness and mode adjustment

### 🖥️ System Monitoring
- **Real-time Dashboard** — CPU / GPU temperature, frequency, power usage, memory usage
- **Fan Speed** — Current RPM and curve status for all fans
- **GPU Status** — NVIDIA GPU info (frequency, VRAM, temperature, power)

### ⚙️ System Integration
- **System Tray** — Minimize to tray, background operation, quick power profile switching
- **Auto-start** — Optional startup with system boot
- **Silent Launch** — Optional silent start with window hidden
- **Cross-distro** — Communicates via D-Bus, no direct dependency on specific distributions

---

## Prerequisites

| Dependency | Description | Required |
|------------|-------------|----------|
| **asusd** | ASUS laptop D-Bus daemon, provides fan/lighting/power control interfaces | Yes |
| **power-profiles-daemon** | GNOME power profiles daemon (`ppd`) | Yes |
| **nvml** | NVIDIA Management Library (`libnvidia-ml.so`) | NVIDIA GPU users |
| **D-Bus session bus** | D-Bus session bus | Yes |

### Install asusd

> ⚠️ asusd is actively developed — some features may require the latest version.

```bash
# Arch Linux (AUR)
yay -S asusd

# Build from source
git clone https://github.com/asus-linux/asusd
cd asusd
cargo build --release
sudo cp target/release/asusd /usr/local/bin/
sudo cp asusd.service /etc/systemd/system/
sudo systemctl enable --now asusd
```

### Verify asusd is running

```bash
busctl --user list | grep asusd
# Should see org.asusd service

busctl --user tree org.asusd
# List all available interfaces
```

---

## Installation

### Build from source

#### Dependencies

- **Rust** ≥ 1.75
- **Node.js** ≥ 18
- **pnpm** ≥ 8 (recommended) or npm
- **Tauri CLI** — `cargo install tauri-cli`
- **System development packages**

```bash
# Debian/Ubuntu
sudo apt install libdbus-1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev

# Arch Linux
sudo pacman -S dbus openssl gtk3 libappindicator-gtk3

# Fedora
sudo dnf install dbus-devel openssl-devel gtk3-devel libappindicator-gtk3-devel
```

#### Build

```bash
# Clone the project
git clone https://github.com/MareDevi/Zephyr.git
cd Zephyr

# Install frontend dependencies
pnpm install

# Build Tauri app
cargo tauri build

# Executable location
ls src-tauri/target/release/zephyr
```

### Development mode

```bash
# Start frontend dev server + Tauri hot reload
cargo tauri dev
```

---

## Project Structure

```
Zephyr
├── src/                          # React frontend (TypeScript)
│   ├── routes/                    # Page routes
│   │   ├── OverviewPage.tsx      # System overview / Dashboard
│   │   ├── FanCurvesPage.tsx     # Fan curve editor
│   │   ├── LightingPage.tsx      # Lighting controls
│   │   ├── ControlsPage.tsx      # Hardware controls (GPU/battery)
│   │   ├── ProfilesPage.tsx      # Power profile management
│   │   └── SettingsPage.tsx      # Application settings
│   ├── features/                 # Feature modules
│   ├── layout/                   # Layout components (AppShell)
│   ├── themes/                   # Themes (Catppuccin)
│   └── main.tsx                  # Frontend entry point
│
├── src-tauri/                     # Rust backend (Tauri)
│   └── src/
│       ├── lib.rs               # Tauri app main logic
│       ├── ipc/                 # IPC commands (frontend invoke API)
│       ├── dbus/                # D-Bus communication layer
│       │   ├── asusd.rs         # asusd daemon interface
│       │   ├── ppd.rs           # power-profiles-daemon interface
│       │   ├── fan_curves.rs    # Fan curves D-Bus operations
│       │   ├── aura.rs          # Aura lighting D-Bus operations
│       │   └── ...
│       ├── services/            # Business logic services
│       │   ├── dashboard.rs     # Dashboard data collection
│       │   ├── fan_curves.rs    # Fan curve management
│       │   ├── gpu.rs           # GPU status monitoring
│       │   └── ...
│       ├── settings.rs         # Settings persistence
│       ├── tray.rs              # System tray
│       └── logging.rs           # Logging configuration
│
├── index.html
├── vite.config.ts
├── package.json
└── tauri.conf.json
```

---

## Tech Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Desktop Framework | **Tauri 2** | Rust + WebView, lighter and more secure than Electron |
| Frontend | **React 18** + **TypeScript** | Type-safe UI development |
| Styling | **Catppuccin** | Soft, pleasant dark theme |
| State Management | React Context + Tauri IPC | Frontend state and Rust backend bridge |
| IPC Binding | **Specta** | Auto-generate TypeScript types from Rust |
| Hardware Communication | **zbus** (D-Bus) | Communicate with asusd / ppd daemons |
| GPU Monitoring | **nvml-wrapper** | NVIDIA GPU metrics |
| Tray / Notifications | **tauri-plugin-*** | System tray, notifications, auto-start |

---

## FAQ

### Q: Fan control doesn't work?

Check if the asusd service is running and if the D-Bus interface exposes fan control:

```bash
busctl --user introspect org.asusd /org/asusd/FanCurves
```

### Q: Aura lighting options not available?

Some ASUS laptops may not expose Aura hardware interfaces. Verify your device model supports Aura RGB.

### Q: NVIDIA GPU info not showing?

Make sure NVIDIA drivers and `libnvidia-ml.so` are installed (usually bundled with `nvidia-utils`).

### Q: No window on startup?

"Silent launch" is enabled in settings. Right-click the system tray icon and select "Show Window".

---

## Contributing

Issues and Pull Requests are welcome!

## License

[GPL-3.0](./LICENSE)
