<div align="center">

# QuickSort

**Right-click. Pick folder. Done.**

A Windows 10/11 shell extension that adds your favorite folders to the Explorer context menu.
No more dragging windows or hunting for directories.

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2-FFC131?style=flat&logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB?style=flat&logo=react)](https://react.dev)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

## Features

- **Cascading context menu** - favorite folders appear directly in Explorer (no UAC)
- **Instant move** - one click moves files via atomic `rename`
- **Duplicate detection** - pre-operation checks (by name, size, or SHA-256 hash)
- **Configurable defaults** - default operation type, overwrite policy, and duplicate check mode
- **All folders access** - "All folders..." opens the full folder list
- **Folder editor** - add, rename, toggle favorites in a clean GUI
- **Event log** - local history of all operations
- **Dark theme** - built-in light and dark color schemes
- **System tray** - runs in background, keeps taskbar clean
- **Smart install** - auto-registers COM server on first launch

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core | Rust |
| GUI | Tauri 2 |
| Frontend | React 19 + TypeScript + Ant Design |
| Shell Extension | Windows COM (DLL) |
| IPC | Named Pipes |
| WinAPI | windows-rs |

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

The installer will be in `src-tauri/target/release/bundle`.

## Usage

1. Launch `QuickSort.exe` - icon appears in system tray
2. Add folders, mark favorites with a star, click Apply
3. Right-click any file in Explorer - pick a folder from the QuickSort menu
4. Files move instantly. Duplicate detection runs automatically.
5. Close the window - app keeps running in tray.

### Settings

Configure defaults in the Settings tab:
- **Default operation**: Move or Copy
- **Overwrite policy**: Skip, Overwrite, or Auto-rename
- **Duplicate check mode**: Name (fast), Size (medium), or Content/SHA-256 (thorough)

## Project Structure

```
QuickSort/
  src-tauri/           # Tauri adapter, CLI, IPC server
  context-menu-dll/    # COM Shell Extension (loaded by Explorer)
  crates/
    quicksort-domain/        # Entities, value objects, events
    quicksort-application/   # Use cases, ports, DTOs
    quicksort-infrastructure/# JSON repos, FileSystem, UUID
    quicksort-ipc-contract/  # Named Pipe contracts
  src/                 # React frontend
```

## Author

**azmrv** - [@Fib511](https://t.me/Fib511) on Telegram

[![GitHub](https://img.shields.io/badge/GitHub-azmrv-181717?style=flat&logo=github)](https://github.com/azmrv)

## Acknowledgments

This project is inspired by the work of many talented Rust developers:

- [PaulDance](https://gist.github.com/PaulDance) - Shell Extension example that became the foundation of our COM server
- [ahaoboy](https://github.com/ahaoboy) - [rcm-com](https://github.com/ahaoboy/rcm-com) and [windows-contextmenu-manager](https://dev.to/ahaoboy/windows-contextmenu-manager-tauri-and-rust-3l9b)
- [ppound](https://github.com/ppound) - [xmp-reader](https://github.com/ppound/xmp-reader), another Shell Extension example
- [acdvs](https://github.com/acdvs) - [winctx-rs](https://github.com/acdvs/winctx-rs) library
- [Microsoft](https://github.com/microsoft) - [windows-rs](https://github.com/microsoft/windows-rs) WinAPI bindings

## License

[MIT](LICENSE) - free to use, modify, and distribute.

---

<div align="center">

**QuickSort** - tidy up your files as fast as you name a folder.

</div>
