# ADR-008: DTO Design — OperationCommand and OperationResult

**Status**: Accepted  
**Date**: 2026-08-20  
**Decision Makers**: Core team

## Context

The Application Layer needs to exchange data with adapters (Tauri commands, IPC server, CLI). Using domain entities directly would leak domain internals to adapters. Using anemic DTOs (only strings) would require excessive mapping.

## Problem

How should we design the data contracts between adapters and the Application Layer to be type-safe, forward-compatible, and minimal?

## Constraints

- DTOs must be serializable (JSON for IPC, Tauri invoke).
- DTOs must use domain types (`OperationId`, `WindowsPath`, `FolderId`) where appropriate — no anemic wrappers.
- DTOs must be backward-compatible when adding new fields.
- The same DTO must work for all operation types (Move, Copy, Delete, Rename).

## Decision

### OperationCommand (Input DTO)

```rust
pub struct OperationCommand {
    pub operation_type: OperationType,      // Move | Copy | Delete | Rename
    pub source_paths: Vec<WindowsPath>,     // At least one
    pub target_folder_id: Option<FolderId>, // Required for Move/Copy
    pub target_paths: Option<Vec<WindowsPath>>, // Required for Rename
    pub overwrite_policy: OverwritePolicy,  // Skip | Overwrite | AutoRename | Ask
}
```

**Design choices:**
- `operation_type` is an enum, not a string — prevents typos at compile time.
- `target_folder_id` and `target_paths` are mutually exclusive depending on operation type — validated at Use Case boundary, not in the DTO.
- `overwrite_policy` is always present (defaults to `Skip` for Delete/Rename) — simplifies the interface.
- Factory methods (`new_move`, `new_copy`, `new_delete`, `new_rename`) enforce invariants at construction.

### OperationResult (Output DTO)

```rust
pub struct OperationResult {
    pub operation_id: OperationId,
    pub state: OperationState,        // Completed { processed_files, bytes_processed } | Failed { reason } | Undone
    pub processed_files: u32,
    pub bytes_moved: u64,             // Named for Move, but used for all types
}
```

**Design choices:**
- `state` carries the full lifecycle state — adapters can pattern-match on it.
- `processed_files` and `bytes_moved` are summary statistics — detailed per-file results are a future extension.
- `bytes_moved` is named for backward compatibility but represents bytes processed for all operation types.

## Alternatives Considered

### Separate DTOs per operation type
```rust
struct MoveCommand { source: Vec<WindowsPath>, target: FolderId, ... }
struct RenameCommand { source: Vec<WindowsPath>, target: Vec<WindowsPath>, ... }
```
- Rejected: Creates N types for N operations. Adapters need to match on type anyway.

### Using `serde_json::Value` for flexibility
- Rejected: Loses type safety. Compile-time errors are better than runtime errors.

### Flat string-based DTOs
```rust
struct OperationCommand { command_type: String, source: String, target: String }
```
- Rejected: Requires parsing at Use Case boundary. Loses domain type safety.

## Consequences

**Positive:**
- Type-safe: invalid states are unrepresentable (e.g., Move without target_folder_id is caught at construction).
- Forward-compatible: new fields can be added as `Option<T>` without breaking existing adapters.
- Single DTO per direction (input/output) — simple mental model.

**Negative:**
- `bytes_moved` name is misleading for Copy/Delete operations. A future breaking change may rename it to `bytes_processed`.
- The mutual exclusivity of `target_folder_id` and `target_paths` is enforced at Use Case level, not in the type system.

## References

- DTO Pattern (Martin Fowler)
- ADR-004: Operations Are Primary Citizens
- ADR-006: Stable Ports
