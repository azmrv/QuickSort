# ADR-010: User Settings and Preferences

## Status
Proposed

## Context
QuickSort currently has no persistent user settings. The only persisted data is the folder list in `folders.json`. UI state (theme, log level, default action) is lost on restart. Users need to configure:
- Default operation type (Move vs Copy)
- Default overwrite policy when duplicate found
- Duplicate detection quality level

## Decision
Introduce a `settings.json` file alongside `folders.json` at `%LOCALAPPDATA%/QuickSort/settings.json`.

### Settings Schema
```json
{
  "version": 1,
  "default_operation": "Move",
  "default_overwrite_policy": "Skip",
  "duplicate_check": {
    "enabled": true,
    "mode": "name"
  }
}
```

### Fields
| Field | Type | Values | Default |
|-------|------|--------|---------|
| `default_operation` | enum | `Move`, `Copy` | `Move` |
| `default_overwrite_policy` | enum | `Skip`, `Overwrite`, `AutoRename` | `Skip` |
| `duplicate_check.enabled` | bool | `true`, `false` | `true` |
| `duplicate_check.mode` | enum | `name`, `size`, `content` | `name` |

### Duplicate Detection Modes
- **name**: Quick check — file with same name exists at destination
- **size**: Medium check — same name AND same file size
- **content**: Deep check — SHA-256 hash comparison (slowest, most accurate)

### Architecture
```
Domain: Settings entity (value object)
Application: SettingsRepository port, LoadSettings/SaveSettings use cases
Infrastructure: JsonSettingsRepository
Adapters: Tauri commands (get_settings, save_settings)
Frontend: SettingsPage expanded with new sections
```

## Consequences
- Settings persist across restarts
- DLL reads settings.json for context menu behavior (overwrite policy)
- Frontend reads/writes via Tauri commands
- Backward compatible: missing file = defaults
