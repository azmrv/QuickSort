<div align="center">

# QuickSort

**A next-generation file manager for Windows.**

QuickSort combines the speed of a shell extension with the power of a modern file management system. Right-click any file in Explorer, pick a folder, and let QuickSort handle the rest — with duplicate detection, operation history, and undo support.

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?style=flat&logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?style=flat&logo=react)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

**English** | [Русский](docs/translation/ru/README.md) | [中文](docs/translation/cn/README.md) | [Deutsch](docs/translation/de/README.md) | [Español](docs/translation/es/README.md)

---

## Features

### Core

- **Cascading context menu** — favorite folders appear directly in Explorer (no UAC)
- **Instant move/copy** — atomic file operations with cross-drive support
- **Duplicate detection** — pre-operation checks by name, size, or SHA-256 content hash
- **Operation history** — full audit trail of every file operation with undo support
- **Configurable defaults** — default operation type, overwrite policy, and duplicate check mode

### Interface

- **Folder editor** — add, rename, toggle favorites in a clean GUI
- **All folders selector** — search, filter, and pick from your entire folder library
- **Event log** — real-time backend and frontend logging with filtering
- **Dark theme** — built-in light and dark color schemes with amber accents
- **System tray** — runs in background, keeps taskbar clean

### Smart Install

- **Zero-install deployment** — single executable, auto-registers COM server on first launch
- **Portable mode** — no admin rights required, everything runs in user space

## Vision

QuickSort is evolving into a full-featured file manager with:

- **Command line interface** — interactive text input with Everything-style search syntax for advanced file queries, filtering, and sorting
- **Batch operations** — queue-based processing for large-scale file movements with progress tracking
- **Smart file analysis** — content-based duplicate detection, file type recognition, and metadata indexing

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core | Rust (Clean Architecture + DDD) |
| GUI | Tauri 2 |
| Frontend | React 19 + TypeScript + Ant Design |
| Shell Extension | Windows COM (DLL) — independent component |
| IPC | Named Pipes |
| WinAPI | windows-rs |

## Architecture

QuickSort follows Clean Architecture with Domain-Driven Design:

```
Domain <- Application <- Infrastructure <- Adapters (src-tauri, context-menu-dll)
```

### Cargo Workspace

| Crate | Role | Description |
|-------|------|-------------|
| `quicksort-domain` | Domain | Entities, value objects, events |
| `quicksort-application` | Application | Use cases, ports, DTOs, facade |
| `quicksort-infrastructure` | Infrastructure | JSON repos, FileSystem, UUID, Clock |
| `quicksort-ipc-contract` | Contract | Named Pipe contracts |
| `src-tauri` | Adapter | Tauri app, CLI, IPC server, COM registration |
| `context-menu-dll` | Adapter | COM Shell Extension (loaded by Explorer) |

### DLL as Independent Component

The context menu DLL (`context-menu-dll`) is an **independent component** that can be built and distributed separately from the main application:

- **App works without DLL** — gracefully degrades (no context menu, all other features intact)
- **DLL is optional** — place `context_menu_dll.dll` next to `QuickSort.exe` to enable context menu
- **Separate build** — DLL has its own build process, no circular dependency with the app
- **Locked file handling** — build script handles Windows Defender locks via rename pattern

```bash
# Build app only (no DLL dependency)
npm run tauri build

# Build DLL separately
npm run build:dll

# Or use the safe build script (handles locked files)
pwsh -NoProfile -File scripts/build-dll.ps1
```

## Build

### Prerequisites

- Windows 10/11 (64-bit)
- [Rust](https://rustup.rs) (stable, `x86_64-pc-windows-msvc`)
- [Node.js](https://nodejs.org) (LTS)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++)

### Commands

```bash
git clone https://github.com/azmrv/QuickSort.git
cd QuickSort
npm install
npm run tauri dev        # development
npm run tauri build      # production build
```

The installer will be in `target/release/bundle`.

## Usage

1. Launch `QuickSort.exe` — icon appears in system tray
2. Add folders, mark favorites with a star, click Apply
3. Right-click any file in Explorer — pick a folder from the QuickSort menu
4. Files move instantly. Duplicate detection runs automatically.
5. Open the History tab to review or undo any operation.
6. Close the window — app keeps running in tray.

### Settings

Configure defaults in the Settings tab:
- **Default operation**: Move or Copy
- **Overwrite policy**: Skip, Overwrite, or Auto-rename
- **Duplicate check mode**: Name (fast), Size (medium), or Content/SHA-256 (thorough)

## Project Structure

```
QuickSort/
  src-tauri/             # Tauri adapter, CLI, IPC server
  context-menu-dll/      # COM Shell Extension (loaded by Explorer)
  crates/
    quicksort-domain/          # Entities, value objects, events
    quicksort-application/     # Use cases, ports, DTOs
    quicksort-infrastructure/  # JSON repos, FileSystem, UUID
    quicksort-ipc-contract/    # Named Pipe contracts
  src/                   # React frontend
  scripts/               # Build helpers (DLL safe build)
```

## Author

**azmrv** - [@Fib511](https://t.me/Fib511) on Telegram

[![GitHub](https://img.shields.io/badge/GitHub-azmrv-181717?style=flat&logo=github)](https://github.com/azmrv)

## Acknowledgments

This project is inspired by the work of many talented developers and projects:

- [Christian Ghisler](https://www.ghisler.com) — [Total Commander](https://www.ghisler.com), the gold standard of file managers that inspired QuickSort's plugin architecture (WCX/WDX/WFX/WLX)
- [PaulDance](https://gist.github.com/PaulDance) - Shell Extension example that became the foundation of our COM server
- [ahaoboy](https://github.com/ahaoboy) - [rcm-com](https://github.com/ahaoboy/rcm-com) and [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b)
- [ppound](https://github.com/ppound) - [xmp-reader](https://github.com/ppound/xmp-reader), another Shell Extension example
- [acdvs](https://github.com/acdvs) - [winctx-rs](https://github.com/acdvs/winctx-rs) library
- [Microsoft](https://github.com/microsoft) - [windows-rs](https://github.com/microsoft/windows-rs) WinAPI bindings
- [voidtools](https://www.voidtools.com) - Everything search engine, inspiration for our command line interface vision

## License

[MIT](LICENSE) - free to use, modify, and distribute.

---

<div align="center">

**QuickSort** — next-generation file management for Windows.

</div>



 <a target="_blank" href="https://imageban.ru/show/2026/08/25/83fc5efaa96d7d4b84317e957f628e38/png"><img src="https://i8.imageban.ru/thumbs/2026.08.25/83fc5efaa96d7d4b84317e957f628e38.png" border="0" style='border: 1px solid #000000'></a> <a target="_blank" href="https://imageban.ru/show/2026/08/25/b80ebefd51e54a070cc292b0f427fa49/png"><img src="https://i3.imageban.ru/thumbs/2026.08.25/b80ebefd51e54a070cc292b0f427fa49.png" border="0" style='border: 1px solid #000000'></a> <a target="_blank" href="https://imageban.ru/show/2026/08/25/2c0472a156b387eda9d595667fd24381/png"><img src="https://i2.imageban.ru/thumbs/2026.08.25/2c0472a156b387eda9d595667fd24381.png" border="0" style='border: 1px solid #000000'></a> <a target="_blank" href="https://imageban.ru/show/2026/08/25/f2007bd5ce1f0283353c986e2dd5d881/png"><img src="https://i7.imageban.ru/thumbs/2026.08.25/f2007bd5ce1f0283353c986e2dd5d881.png" border="0" style='border: 1px solid #000000'></a> <a target="_blank" href="https://imageban.ru/show/2026/08/25/a784d47f47a701c2f6c01cb4f20544af/png"><img src="https://i7.imageban.ru/thumbs/2026.08.25/a784d47f47a701c2f6c01cb4f20544af.png" border="0" style='border: 1px solid #000000'></a><a target="_blank" href="https://imageban.ru/show/2026/08/25/40f71dff67252d956c04edd889115666/png"><img src="https://i5.imageban.ru/thumbs/2026.08.25/40f71dff67252d956c04edd889115666.png" border="0" style='border: 1px solid #000000'></a> <a target="_blank" href="https://imageban.ru/show/2026/08/25/66a4921c12ff6afc4f030dde050b02e2/png"><img src="https://i4.imageban.ru/thumbs/2026.08.25/66a4921c12ff6afc4f030dde050b02e2.png" border="0" style='border: 1px solid #000000'></a>  
