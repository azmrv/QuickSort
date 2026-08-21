# ADR-004: Operations Are Primary Citizens

**Status**: Accepted  
**Date**: 2026-07-07  
**Decision Makers**: Core team

## Context

In the current version of QuickSort, the core action is moving files to selected favorite folders. However, we anticipate that the available functionality will expand significantly in future releases.

## Problem

Which entity should be treated as the root component of our design to ensure the system can effortlessly scale to accommodate entirely new types of actions?

## Constraints

- We want to avoid rewriting or restructuring our architectural foundation when introducing new features.
- The system must natively support Undo/Redo capabilities, execution history logging, and automation workflows.

## Decision

**The central, foundational entity of the system is the Operation.** Folders are treated as configuration metadata and resources that operations consume.

Examples of future operations include:
- `MoveFiles`
- `CopyFiles`
- `DeleteFiles`
- `RenameFiles`
- `Archive` (compression)
- `Extract` (decompression)
- `Hash` (checksum calculation)
- `Sync` (directory synchronization)

Every operation passes through a unified **Pipeline**: validation → conflict resolution → execution → event generation.

This approach ensures:
- New operations can be added seamlessly simply by implementing a standard interface.
- Logging, telemetry, performance tracking, and Undo mechanics are handled centrally by the pipeline.
- Future automation engines and custom user rules can be built dynamically on top of these structured operations.

## Alternatives Considered

- **Folder-Centric Design (Folder as Root):** Rejected because certain operations may not be bound to a target directory at all (such as calculating a file checksum).
- **File-Centric Design:** Rejected because it is too narrow; operations frequently process batches containing multiple files and directories simultaneously.

## Consequences

**Positive:**
- The entire system is engineered for future expansion without requiring structural refactoring.
- Introducing new utility features becomes highly streamlined and predictable.
- Operations can be easily combined, scheduled, queued, or re-run.

**Negative:**
- Requires designing a robust, highly generic abstract interface capable of representing all possible operations.
- Building and maintaining the central execution Pipeline requires precise engineering up front.

## Migration Strategy

We will encapsulate the existing file-moving functionality into a concrete `MoveFiles` operation block. The rest of the codebase (the React GUI and the Explorer Shell DLL) will interact with this logic by invoking a unified `ExecuteOperationUseCase` instead of executing direct file-system moves.

## References

- Command Pattern
- Operation Pipeline pattern
- DDD patterns, treating "Operations as Aggregates"
