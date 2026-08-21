# ADR-011: Duplicate Detection Strategy

## Status
Accepted (implemented 2026-08-21)

## Context
When moving/copying files, conflicts occur when a file with the same name already exists at the destination. Current behavior is hardcoded `OverwritePolicy::Skip` in both DLL and frontend. Users need:
- Pre-operation duplicate detection (before any file is moved)
- Choice of detection quality (name only vs content hash)
- Configurable default action per duplicate found

## Decision
Add a duplicate detection phase to the operation pipeline, before the execute phase.

### Pipeline Addition
```
validate → detect_duplicates → resolve_conflicts → execute → log
```

### Detection Modes
| Mode | Speed | Accuracy | Use Case |
|------|-------|----------|----------|
| `name` | Fast | Low | Default, catches obvious conflicts |
| `size` | Medium | Medium | Name + size reduces false positives |
| `content` | Slow | High | SHA-256 hash, definitive but requires reading files |

### Conflict Resolution Actions
When a duplicate is found, the configured default action is applied:
1. **Skip** — do not move this file, log as skipped
2. **Overwrite** — replace existing file
3. **AutoRename** — add timestamp suffix: `file (2025-01-15 14-30-00).ext`
4. **Ask** — in interactive mode (frontend), show dialog; in DLL context, fall back to Skip

### Domain Changes
```rust
// New entity
pub struct DuplicateCheckResult {
    pub source: WindowsPath,
    pub destination: WindowsPath,
    pub exists: bool,
    pub check_mode: DuplicateCheckMode,
}

pub enum DuplicateCheckMode {
    Name,
    Size,
    Content,
}
```

### Port Addition
```rust
#[async_trait]
pub trait DuplicateChecker: Send + Sync {
    async fn check(&self, source: &WindowsPath, destination: &WindowsPath, mode: DuplicateCheckMode) -> Result<DuplicateCheckResult, UseCaseError>;
}
```

## Consequences
- Users see duplicates before any destructive action
- Detection quality is configurable per user preference
- DLL sends configured overwrite policy from settings
- Performance impact only when `content` mode is selected
