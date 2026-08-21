# ADR-012: Single Source of Truth for Version

## Status
Proposed

## Context
Version numbers are scattered across multiple files with inconsistencies:
- `package.json`: 0.1.0
- `src-tauri/tauri.conf.json`: 0.1.0
- `src-tauri/Cargo.toml`: 0.2.0
- `App.tsx` (hardcoded): v0.2.0
- `AboutPage.tsx` (hardcoded): v0.1.0

This leads to confusion and potential build issues.

## Decision
Use `Cargo.toml` (src-tauri) as the single source of truth for version.

### Strategy
1. **Cargo.toml** (`quicksort` crate) holds the canonical version
2. **tauri.conf.json** reads version from Cargo.toml at build time via `cargo metadata`
3. **Frontend** receives version at runtime via a Tauri command `get_app_version`
4. **package.json** stays in sync manually (or via `npm version`)

### Implementation
```rust
// Tauri command
#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

```typescript
// Frontend
const version = await invoke<string>('get_app_version');
```

### Build-time Sync
`tauri.conf.json` can use dynamic version:
```json
{
  "version": "${CARGO_PKG_VERSION}"
}
```
Or via `tauri.conf.json` script that reads Cargo.toml.

## Consequences
- Single place to bump version: `src-tauri/Cargo.toml`
- Frontend always shows correct version from backend
- No more hardcoded strings in React components
- Release workflow reads version from one place
