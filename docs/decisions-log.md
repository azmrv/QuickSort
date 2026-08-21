# Decisions Log

This document serves as a high-level historical ledger tracking important architectural, technical, and strategic engineering decisions. It captures foundational milestones that guide the development of QuickSort.

## 2026-08-20

*   **Decision**: Adopt `ApplicationFacadeImpl` as the single entry point for all adapters (ADR-007).
*   **Rationale**: Centralizes wiring, eliminates duplicated trait object management, simplifies testing. Adapters depend on one type, not four.
*   **Author**: Core team

## 2026-08-20

*   **Decision**: Define `OperationCommand` and `OperationResult` as the canonical DTOs for all file operations (ADR-008).
*   **Rationale**: Type-safe input/output contracts that use domain types directly. Factory methods enforce invariants at construction. Single DTO per direction (input/output) keeps the interface simple.
*   **Author**: Core team

## 2026-08-20

*   **Decision**: Migrate legacy modules incrementally via Parallel Operation Phase (ADR-009).
*   **Rationale**: Avoids Big Bang rewrite. Application remains functional at every step. Each migration step is small and verifiable.
*   **Author**: Core team

## 2026-07-07

*   **Decision**: Initiate the project with "Phase 0 — Engineering Memory," documenting all key architectural blueprints, invariants, and constraints before writing any production source code.
*   **Rationale**: To solidify non-negotiable architectural boundaries early, align the development team (and AI context windows), and completely mitigate the risk of costly future structural refactoring cycles.
*   **Author**: Core team

## 2026-07-07

*   **Decision**: Adopt the `Operation` entity as the foundational root construct of the system instead of using a `Folder`-centric approach.
*   **Rationale**: This shift ensures maximum extensibility, providing a generic design pattern capable of supporting all future system actions (such as copying, archiving, renaming, and checksum hashing) without altering core application layers.
*   **Author**: Core team
