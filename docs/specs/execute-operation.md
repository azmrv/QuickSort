# ExecuteOperationUseCase Specification

## Purpose
Execute a specific file-system operation (such as move, copy, delete, or rename) based on an incoming command payload.

## Preconditions
- The application configuration data (including favorite folders metadata) is fully loaded.
- The target destination folder (if required by the operation) exists within the active configuration repository.
- The source files or directories specified in the command parameters physically exist on disk (if required by the operation type).

## Inputs
- `command: OperationCommand` — A Data Transfer Object (DTO) encapsulating the operation type along with all necessary routing parameters (source paths, destination folder unique IDs, formatting options, etc.).

## Outputs
- **Success:** Returns an `OperationResult` struct aggregating execution statistics, alongside a collection of generated `DomainEvent` objects.
- **Failure:** Returns a structured error payload (`UseCaseError`) packing a distinct error code and a localized debug message.

## Postconditions (Business Invariants)
- Upon a successful execution thread, the physical state of the underlying file system is modified strictly according to the intent of the command.
- The lifecycle pipeline successfully generates and registers appropriate business facts (`OperationCompleted`, `FilesMoved`, etc.).
- In the event of a partial execution failure, the system halts cleanly, prevents data loss, avoids creating inconsistent drive states, and emits an `OperationFailed` event enriched with contextual diagnostics.

## Error Handling Matrix
- **Target Folder Missing:** If the destination ID cannot be resolved within the repository → returns a `FolderNotFound` error.
- **Source Missing:** If the targeted source file or folder does not exist on disk → returns a `FileNotFound` error.
- **Access Restrictions:** If the operating system rejects the underlying execution handle → returns a `PermissionDenied` error.
- **Target Collision:** If a filename conflict occurs (e.g., target file already exists) → the execution control is handed over to the `ConflictResolver` port. The resolver evaluates whether to cancel, auto-rename, or overwrite the item based on the user preferences captured via the GUI adapter.

## Triggered Events
- `OperationStarted` — Dispatched immediately prior to processing the physical execution pipeline.
- `OperationCompleted` — Dispatched upon flawless execution of the underlying operation.
- `OperationFailed` — Dispatched the moment a critical error aborts the execution routine.
- `FilesMoved` — Specialized event dispatched specifically upon successful `Move` operation types.

## Undo / Redo Mechanics
- The engineered operation block must be intrinsically reversible (where technically feasible). For instance, the exact inverse of a `Move` operation is a corresponding `Move` operation routed in reverse.

## Telemetry Metrics
- Total processing duration (tracked via the abstract `Clock` port).
- Net count of successfully processed file batches.
- Concrete operation type signature.

## Security Controls
- Standardized path verification routines are applied to sanitize paths and prevent unauthorized directory traversal attacks (ensuring operations do not break out of restricted system boundaries).
