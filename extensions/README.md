# QuickSort Linux Integrations

File manager integrations for Linux desktop environments.

## Supported Desktop Environments

| DE | File Manager | Integration Type | Status |
|----|--------------|------------------|--------|
| GNOME | Nautilus | Python extension | ✅ Ready |
| KDE | Dolphin | Service Menu | ✅ Ready |
| XFCE | Thunar | Custom Actions | ⚠️ Manual |
| LXQt | PCManFM-Qt | Custom Actions | ✅ Ready |
| Cinnamon | Nemo | Python extension | ✅ Ready |
| MATE | Caja | Python extension | ✅ Ready |

## Supported Distributions

- Debian 12+
- Ubuntu 22.04+
- Astra Linux
- ALT Linux
- Rosa Linux
- Lionux
- Red OS

## Installation

### Automatic (Recommended)

```bash
./scripts/install-linux.sh
```

The installer automatically detects your desktop environment and installs the
appropriate integration.

### Manual Installation

#### Nautilus (GNOME/Nemo/MATE)

1. Install `python3-nautilus`:

   ```bash
   sudo apt-get install python3-nautilus
   ```

2. Copy the extension and the shared IPC module:

   ```bash
   mkdir -p ~/.local/share/nautilus-python/extensions/
   cp extensions/nautilus/quicksort-nautilus.py ~/.local/share/nautilus-python/extensions/
   cp -r extensions/shared ~/.local/share/nautilus-python/
   ```

3. Restart Nautilus:

   ```bash
   nautilus -q
   ```

#### Dolphin (KDE)

1. Copy the Service Menu:

   ```bash
   mkdir -p ~/.local/share/kservices5/
   cp extensions/dolphin/quicksort.desktop ~/.local/share/kservices5/
   ```

2. Restart Dolphin.

#### Thunar (XFCE)

1. Open Thunar → Edit → Configure Custom Actions
2. Add a new action with command: `quicksort select-folder --file %f`

#### PCManFM-Qt (LXQt)

1. Copy the custom actions:

   ```bash
   mkdir -p ~/.config/libfm/actions/
   cp extensions/thunar/quicksort.desktop ~/.config/libfm/actions/
   ```

## Configuration

All integrations read folder configuration from `~/.config/QuickSort/folders.json`.

Example:

```json
{
  "folders": [
    {
      "id": "1",
      "name": "Documents",
      "path": "/home/user/Documents",
      "color": "#4CAF50"
    }
  ]
}
```

## IPC Protocol

All integrations communicate with QuickSort via a Unix Domain Socket:

- Socket path: `/run/user/{uid}/quicksort.sock`
- Protocol: `[u32 LE length][JSON payload]`
- Commands: `select_folder`, `ping`

## Troubleshooting

### Extension not appearing

1. Restart your file manager after installation.
2. Check that the required packages are installed:
   - GNOME/Nemo: `python3-nautilus`, `nemo-python`
   - KDE: `kiowidgets`

### "QuickSort not running" error

1. Ensure the QuickSort Tauri app is running.
2. Check that the socket exists:

   ```bash
   ls -la /run/user/$(id -u)/quicksort.sock
   ```

### Check logs

- Nautilus: `NAUTILUS_DEBUG=1 nautilus`
- Dolphin: check `~/.xsession-errors`

## Architecture

```
File Manager (Nautilus/Dolphin/Thunar)
        │
        │  Context menu integration
        ▼
  IPC client (extensions/shared/ipc_client.py)
        │
        │  Unix Domain Socket
        ▼
  QuickSort Tauri App
        ├── UnixSocketTransport (src-tauri/src/ipc/unix_socket.rs)
        └── IPC server (src-tauri/src/ipc/server.rs)
```
