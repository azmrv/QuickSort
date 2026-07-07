# QuickSort Project Documentation Index

Welcome to the official technical documentation for the **QuickSort** utility. This directory is organized as a structured knowledge base tracking our engineering standards, architecture, and design decisions.

## 🗺️ Documentation Map

### 📐 [Architecture](./architecture/)
Core engineering vision and language definitions.
*   **[Vision](./architecture/vision.md):** High-level product overview, system context, and core goals.
*   **[Glossary](./architecture/glossary.md):** Unified Ubiquitous Language (DDD terms, Windows COM specifics, and domain boundaries).
*   **[Domain Models](./architecture/domain-models.md):** Core aggregates, entities, and value objects (pure business logic).
### 🏛️ [Architectural Decision Records (ADR)](./adr/)
Immutable log of strategic technical choices.
*   **[001: Architectural Style](./adr/001-architectural-style.md):** Hexagonal + Clean + DDD pattern layout.
*   **[002: Dependency Rule](./adr/002-dependency-rule.md):** Enforcement of inward-pointing dependencies.
*   **[003: Domain First](./adr/003-domain-first.md):** Designing logic independently of system primitives.
*   **[004: Operations Primary](./adr/004-operations-primary.md):** Use-case orchestration rules.
*   **[005: Domain Events](./adr/005-domain-events.md):** Decoupling side effects via internal publish-subscribe.
*   **[006: Stable Ports](./adr/006-stable-ports.md):** Design principles for inbound and outbound abstractions.
*   **[Decisions Log](../decisions-log.md):** High-level chronological index of all approved ADRs.

### 📋 [Specifications](./specs/)
Detailed functional requirements and technical execution flows.
*   **[Execute Operation](./specs/execute-operation.md):** Step-by-step lifecycles of file sorting, atomic movements, and logging execution.

### 🛡️ [Standards & Guidelines](./standards/)
Coding rules and constraints enforced across the workspace.
*   **[Rust Style](./standards/rust-style.md):** Idiomatic Rust, memory management boundaries, and workspace crate structure.
*   **[Error Handling](./standards/error-handling.md):** Standardized usage of `anyhow` vs domain-specific error enums across layers.
*   **[Testing](./standards/testing.md):** Unit testing rules for domain models and architecture validation (ArchUnit-like patterns).
*   **[Architecture Guard](./standards/architecture_guard.py):** Automated Python script to enforce dependency rules in CI.

### 🤖 [AI & Automation Context](./ai/)
Prompts, instructions, and rules dedicated to AI agents working on this codebase.
*   **[AI Guidelines](./ai/ai.md):** How LLMs must interact with this repository.
*   **[Dev Rules](./ai/devrules/):** Automated rules injected into context windows for code generation.

---

## 🧭 Translation / Локализация
*   For the Russian version of the entry points, please refer to **[Русский README](./readmeru.md)**.
