# ADR-007: ApplicationFacade as Single Entry Point

**Status**: Accepted  
**Date**: 2026-08-20  
**Decision Makers**: Core team

## Context

Adapters (Tauri commands, IPC server, CLI) need to call Use Cases. Without a unified entry point, each adapter would need to hold references to individual use cases, leading to:
- Duplicated wiring code in every adapter
- Tight coupling between adapters and concrete use case types
- Difficulty testing adapters in isolation

## Problem

How should adapters access Application Layer functionality without violating the Dependency Rule or creating excessive boilerplate?

## Constraints

- Adapters must not depend on Domain directly (ADR-002).
- Adapters must not depend on concrete Use Case types — only on abstractions.
- The facade must be testable (injectable via trait objects or concrete types).
- Multiple adapters (Tauri GUI, IPC server, CLI) must share the same entry point.

## Decision

**`ApplicationFacadeImpl` is the single concrete entry point for all adapters.** It:
- Implements all inbound port traits (`ExecuteOperation`, `UndoOperation`, `GetFolders`, `ManageFolders`)
- Holds `Arc<UseCaseType>` for each use case (concrete types, not trait objects — enables inlining)
- Contains **zero business logic** — pure delegation
- Is constructed once at startup and shared via `Arc<ApplicationFacadeImpl>`

Adapters receive `State<'_, AppState>` (Tauri) or `Arc<ApplicationFacadeImpl>` (IPC) and call methods directly on the facade.

## Alternatives Considered

### Separate trait objects per port
```rust
// Adapters would hold multiple references
struct TauriState {
    execute: Arc<dyn ExecuteOperation>,
    undo: Arc<dyn UndoOperation>,
    get_folders: Arc<dyn GetFolders>,
    manage: Arc<dyn ManageFolders>,
}
```
- Rejected: Duplicates wiring in every adapter. Increases cognitive load.

### Generic facade trait
```rust
#[async_trait]
trait ApplicationFacade: ExecuteOperation + UndoOperation + GetFolders + ManageFolders {}
```
- Rejected: Adds abstraction layer with no practical benefit. Concrete type is sufficient.

### Module-level functions
```rust
async fn execute_operation(cmd: OperationCommand) -> Result<OperationResult, UseCaseError> { ... }
```
- Rejected: Makes dependency injection impossible. Cannot test without global state.

## Consequences

**Positive:**
- Adapters depend on a single type, not four trait objects.
- Wiring is centralized in `main.rs` — one place to change.
- Testing adapters is trivial: construct `ApplicationFacadeImpl` with mock use cases.

**Negative:**
- The facade is a "god object" that knows about all ports (though it contains no logic).
- Adding a new port requires updating the facade struct and its trait implementations.

## Implementation Notes

The facade fields use short names (`execute`, `undo`, `get_folders`, `manage_folders`) to minimize verbosity at call sites.

> **Note:** An alternative `ApplicationFacade` struct (using `Arc<dyn Trait>` trait objects) exists in `facade.rs` but is not used by any adapter. It may be removed in a future cleanup.

## References

- Facade Pattern (GoF)
- ADR-006: Stable Ports
- Clean Architecture: "Use cases orchestrate the flow of data"
