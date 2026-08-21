# ADR-006: Stable Ports

**Status**: Accepted  
**Date**: 2026-07-07  
**Decision Makers**: Core team

## Context

Ports are the abstract interfaces through which the Application layer communicates with the external world. They serve as strict, formalized contracts between the Application and Infrastructure layers.

## Problem

How can we guarantee that technical modifications within the Infrastructure layer do not break the Application layer, and vice versa?

## Constraints

- The application is engineered to evolve continuously over several years.
- The codebase must accommodate multiple driving adapters simultaneously (such as a React GUI, a CLI shell, Windows Explorer extensions, or potential REST APIs).

## Decision

**All ports are treated as stable, immutable contracts.** Modifying these interfaces is prohibited unless releasing a new major version of the platform.

The core portfolio of primary ports includes:
- `ConfigurationRepository` — Handles reading and writing folder metadata, user preferences, and custom sorting rules.
- `OperationRepository` — Persists and manages the historical log of executed operations.
- `FileSystem` — Abstracts physical file-system activities (such as atomic moves, copies, deletions, renames, and metadata extraction).
- `IdGenerator` — Generates cryptographically safe unique identifiers (UUIDs).
- `Clock` — Captures the system time (abstracted to enable predictable deterministic testing).
- `ConflictResolver` — Evaluates and mitigates file-system execution conflicts (such as handling pre-existing target filenames).

These interfaces are explicitly owned and declared by the Application layer. They must remain stable and free from arbitrary modifications.

## Alternatives Considered

- **Modifying port signatures during every feature iteration:** Rejected because it triggers a cascade of breaking compilation errors across all independent adapters.
- **Leaking concrete infrastructure structs directly into the Application layer:** Rejected because it completely violates the Dependency Inversion Principle (DIP).

## Consequences

**Positive:**
- Adapters (such as the desktop GUI or the native Explorer Shell extension) can be engineered, refactored, and scaled in total isolation.
- Injecting lightweight mock structures for automated unit testing becomes completely trivial.
- Establishes crisp, indisputable architectural boundaries between technical components and business code.

**Negative:**
- Introduces minor friction when planning new features, as appending methods to an existing contract requires backward-compatibility strategies or releasing a major version.

## Migration Strategy

We will boot the platform with a minimal, essential footprint of core ports. If future requirements demand additional interface methods, we will introduce them safely using standard Rust default trait implementations or by deploying specialized, independent extension ports.

## References

- Ports & Adapters (Hexagonal) Architecture
- Open/Closed Principle (OCP)
