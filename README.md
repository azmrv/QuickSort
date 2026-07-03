# QuickSort

**QuickSort** – a Windows 10/11 utility that adds a cascading "QuickSort" item to the Explorer context menu with your favorite folders. One click – and files are moved instantly.  
No more dragging, extra windows, or hunting for the right directory.

Tauri + React + TypeScript

---

## Table of Contents
- [Features](#-features)
- [Technologies](#-technologies)
- [Dependencies](#-dependencies)
- [Installation and Build](#-installation-and-build)
- [Usage](#-usage)
- [Acknowledgments](#-acknowledgments)
- [License](#-license)

---

## ✨ Features

- **Cascading context menu** – favorite folders appear directly in the Explorer menu (no UAC or extra windows).
- **Instant moving** – selecting a folder immediately moves the files (using atomic `rename`).
- **Folder picker** – access all folders from the configuration (via the “📂 Other folders...” button).
- **Folder editor** – a convenient Tauri + React GUI to manage the list: add, rename, mark as favorite.
- **Event log** – a local history of all operations (moves, config changes).
- **Dark theme** – built‑in support for light and dark color schemes.
- **System tray** – the app can run in the background, keeping the taskbar clean.

---

## 🛠️ Technologies

- **Rust** – core of the application, COM server, CLI mode, registry handling.
- **Tauri 2** – lightweight GUI framework (instead of Electron).
- **React + TypeScript** – editor frontend.
- **Ant Design** – UI library (components, icons, theming).
- **Windows COM** – Shell Extension (DLL) loaded by Explorer.
- **win-ctx** – high‑level registry wrapper for context menus.
- **windows-rs** – official Microsoft WinAPI bindings for Rust.

---

## 📖 Usage

1. Launch `QuickSort.exe`. An icon appears in the system tray.
2. In the editor window, add folders (click “Add Folder”), mark favorites with a star, and click “Apply”.
3. Now, right‑click any file or folder in Explorer and you’ll see the **QuickSort** menu with your favorite folders.  
   Choose a folder – the file moves instantly.
4. To access all folders (not just favorites), use “📂 Other folders...” – a picker window opens.
5. Every action is recorded in the “Log” tab. When you close the window, the app keeps running in the tray.

---

## 🙏 Acknowledgments

This project is inspired by the work of many talented Rust developers:

- **[PaulDance](https://gist.github.com/PaulDance)** – for the excellent [Gist with a Shell Extension example](https://gist.github.com/PaulDance), which became the foundation of our COM server.
- **[ahaoboy](https://github.com/ahaoboy)** – for [rcm-com](https://github.com/ahaoboy/rcm-com) and [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b), which helped understand the architecture.
- **[ppound](https://github.com/ppound)** – for [xmp-reader](https://github.com/ppound/xmp-reader), another excellent Shell Extension example in Rust.
- **[acdvs](https://github.com/acdvs)** – for the [winctx-rs](https://github.com/acdvs/winctx-rs) library, which simplified context menu handling.
- **[Microsoft](https://github.com/microsoft)** – for [windows-rs](https://github.com/microsoft/windows-rs), opening WinAPI access from Rust.

Special thanks to all maintainers of the crates used in this project.

---

## 📄 License

This project is distributed under the **MIT** license.  
It is an educational project, but everyone is free to use, modify, and distribute it for any purpose.

Full license text: [LICENSE](LICENSE)

---

**QuickSort** – tidy up your files as fast as you name a folder!  
If you have ideas or suggestions, open an issue or pull request. Let’s make Explorer more convenient together.

## 📦 Dependencies

### Rust (core / Tauri)
```toml
tauri = "2"
tauri-plugin-dialog = "2"
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
winreg = "0.56"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
directories = "6"
uuid = { version = "1", features = ["v4", "serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
parking_lot = "0.12"
winctx = "1.4"
```

### Frontend (React)
```json
"react": "^18",
"antd": "^5",
"@ant-design/icons": "^5",
"@tauri-apps/api": "^2",
"@tauri-apps/plugin-dialog": "^2",
"vite": "^7"
```

### COM server (context-menu-dll)
```toml
windows = "0.61"
parking_lot = "0.12"
log = "0.4"
simplelog = "0.12"
```

See the full list of dependencies in `src-tauri/Cargo.toml`, `context-menu-dll/Cargo.toml`, and `package.json`.

---

## 🔧 Installation and Build

### Prerequisites
- Windows 10/11 (64‑bit)
- [Rust](https://rustup.rs) (stable, `x86_64-pc-windows-msvc`)
- [Node.js](https://nodejs.org) (LTS)
- [Tauri CLI](https://tauri.app) (`cargo install tauri-cli`)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (component “Desktop development with C++” with Windows SDK)

### Build
```bash
git clone https://github.com/yourname/quicksort.git
cd quicksort
npm install
cargo build --all --release
```

### COM server registration (for dynamic menu)
1. Build the DLL: `cd context-menu-dll && cargo build --release`.
2. Run `register.reg` from the project root with administrator privileges.
3. Restart Explorer (`taskkill /f /im explorer.exe && start explorer.exe`).

### Launch the editor
```bash
npm run tauri dev   # development mode
# or
cargo run           # backend only (GUI without hot‑reload)
```

For production build:
```bash
npm run tauri build
```

The installer will be in `src-tauri/target/release/bundle`.

---
