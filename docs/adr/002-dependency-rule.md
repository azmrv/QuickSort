# ADR-002: Dependency Rule

**Status**: Accepted  
**Date**: 2026-07-07  
**Decision Makers**: Core team

## Context

We have established four architectural layers: Domain, Application, Infrastructure, and Adapters. We need to clearly define the allowed directions of dependencies between these layers to preserve the integrity and purity of our architecture.

## Problem

Without a strict dependency rule, developers can easily bypass encapsulation boundaries, leading to tightly coupled spaghetti code over time.

## Constraints

- Rust does not natively restrict dependencies within a `Cargo.toml` workspace layout, meaning we must monitor and enforce these boundaries ourselves.
- The Domain layer must remain entirely decoupled from and unaware of the external world.

## Decision

**Dependencies must point strictly inward:**

- **Domain** – Must not depend on any other project crate. It may depend on a small, well-known set of crates that do not impose architectural constraints: `std`, `serde`, `uuid`, `chrono`, `thiserror`.
- **Application** – Depends solely on the Domain layer and the ports (interfaces) defined inside the Application layer itself. It has zero dependencies on infrastructure-level technicalities (such as JSON parsing, networks, or specific OS APIs).
- **Infrastructure** – Depends on both Domain and Application layers. It provides the concrete implementations for the outbound ports defined in the Application layer.
- **Adapters** (GUI, Shell, CLI) – May depend on the Infrastructure and Application layers. They must not interact with or depend on the Domain layer directly, communicating instead exclusively through Application layer use cases.

## Alternatives Considered

- **Allowing open dependencies across all layers:** Rejected because it inevitably results in architectural chaos and unmaintainable code.
- **Using a shared dependency layer:** Rejected because it violates the Dependency Inversion Principle (DIP) and causes leaky abstractions.

## Consequences

**Positive:**
- The core Domain remains clean, isolated, and highly straightforward to test.
- Replacing or upgrading infrastructure components has absolutely zero impact on core business logic.
- Code readability, comprehension, and modifications become much simpler.

**Negative:**
- Dependencies must be explicitly wired and passed through abstract ports.
- Requires writing a small amount of extra boilerplate code (such as data mapping structures).

## Migration Strategy

We will integrate automated checks into our CI/CD pipeline to analyze `Cargo.toml` files. This ensures that the Domain crate does not accidentally introduce external dependencies, and that the Application layer remains completely free of Windows-specific or Tauri-specific libraries.

## References

- Clean Architecture, Dependency Inversion Principle (DIP)
- Hexagonal Architecture (Ports & Adapters)
