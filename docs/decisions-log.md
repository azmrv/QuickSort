# Decisions Log

This document serves as a high-level historical ledger tracking important architectural, technical, and strategic engineering decisions. It captures foundational milestones that guide the development of QuickSort.

## 2026-07-07

*   **Decision**: Initiate the project with "Phase 0 — Engineering Memory," documenting all key architectural blueprints, invariants, and constraints before writing any production source code.
*   **Rationale**: To solidify non-negotiable architectural boundaries early, align the development team (and AI context windows), and completely mitigate the risk of costly future structural refactoring cycles.
*   **Author**: Core team

## 2026-07-07

*   **Decision**: Adopt the `Operation` entity as the foundational root construct of the system instead of using a `Folder`-centric approach.
*   **Rationale**: This shift ensures maximum extensibility, providing a generic design pattern capable of supporting all future system actions (such as copying, archiving, renaming, and checksum hashing) without altering core application layers.
*   **Author**: Core team
