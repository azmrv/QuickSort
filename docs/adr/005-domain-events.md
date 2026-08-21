# ADR-005: Domain Events

**Status**: Accepted  
**Date**: 2026-07-07  
**Decision Makers**: Core team

## Context

Important changes occur within the system: operations complete, folders are added, and errors arise. Other decoupled components of the application need to be notified about these state changes.

## Problem

How can we notify the GUI, logs, telemetry, and execution history about core business facts without violating the purity of the Domain layer?

## Constraints

- The Domain layer must remain completely unaware of its external subscribers.
- Events must be immutable and entirely self-contained.

## Decision

**The Domain communicates business facts via Domain Events.** All events are aggregated into a single Rust enum called `DomainEvent`. The current variants are:
- `OperationStarted` — an operation has begun executing.
- `OperationCompleted` — an operation finished successfully (carries `files` count and `bytes` total).
- `OperationFailed` — an operation failed (carries a `reason` string).
- `OperationUndone` — a previously completed operation has been reversed.
- `FolderAdded` — a new folder was added to the configuration.
- `FolderRemoved` — a folder was removed from the configuration.
- `FolderRenamed` — a folder's display name was changed.

**Domain aggregates publish events directly.** When an aggregate method changes state (e.g., `Operation::start()`, `Operation::complete()`, `Operation::fail()`), it pushes the corresponding `DomainEvent` into an internal `events` vector. The Application layer pulls these events via `Operation::pull_events()` after each use case execution and forwards them to subscribers (logging, telemetry, etc.).

This approach cleanly separates the core business logic from the notification infrastructure.

## Alternatives Considered

- **Full Event Sourcing:** Rejected because it introduces unnecessary architectural complexity that is not currently required.
- **Direct GUI Invocations from the Domain:** Rejected because it directly violates the Dependency Rule.
- **An Internal Observer Pattern inside the Domain:** Rejected because it leaks external execution contexts into the pure core.

## Consequences

**Positive:**
- The Domain layer remains completely pure and decoupled.
- Adding new asynchronous subscribers (such as system loggers, telemetry trackers, or history viewers) becomes trivial.
- Events naturally provide a perfect foundation for Undo/Redo mechanisms and state synchronization.

**Negative:**
- Requires explicitly gathering and returning events from Application layer Use Cases.
- Event structures must be designed with extreme care to ensure they contain all necessary metadata up front.

## Migration Strategy

We will begin by introducing basic core events (`OperationCompleted`, `FolderAdded`) and incrementally expand the system to capture all remaining lifecycle triggers.

## References

- Domain Events pattern by Martin Fowler
- Domain-Driven Design (DDD), Event-Driven Architecture (EDA)
