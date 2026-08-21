# ADR-001: Architectural Style

**Status**: Accepted  
**Date**: 2026-07-07  
**Decision Makers**: Core team

## Context

We are building a platform for file operations designed to evolve for at least several years. We need an architectural foundation that allows us to easily introduce new operations, swap user interfaces, and update system integrations without rewriting core business logic.

## Problem

Which architectural style should we choose to ensure the system remains highly flexible, testable, and completely independent of specific technological implementations?

## Constraints

- The core language for the application workspace is Rust.
- The project includes native Windows Shell Extensions (via COM).
- Future additions plan for a CLI tool and potentially a web-based management interface.
- High testability (especially unit testing) is a strict requirement.

## Decision

We are adopting **Clean Architecture** combined with **Domain-Driven Design (DDD)** and **Hexagonal Architecture (Ports & Adapters)**. This enforces:

- A strict separation of concerns across four distinct boundaries: Domain, Application, Infrastructure, and Adapters.
- **Domain** remains entirely pure and has no architectural dependencies on other project layers. It may depend on a small set of well-known crates (`serde`, `uuid`, `chrono`, `thiserror`) that do not impose architectural constraints (see ADR-002).
- **Application** depends solely on the Domain layer and abstract ports (interfaces).
- **Infrastructure** provides concrete implementations for the outbound ports.
- **Adapters** (such as the GUI, Windows Shell Extension, and CLI) drive the system by using Infrastructure and Application services.

All source code dependencies must point strictly inward: Adapters → Infrastructure → Application → Domain.

## Alternatives Considered

- **Traditional Layered Architecture:** Rejected because it naturally leads to tight coupling between business logic and database/OS layers.
- **Microservices:** Rejected as highly over-engineered and unnecessary for a desktop-centric utility application.
- **Basic MVC (Model-View-Controller):** Rejected because it fails to provide the required clean separation of technical concerns from domain logic.

## Consequences

**Positive:**
- Seamlessly swap or update the GUI, Windows Explorer Shell integration, database engines, or file-system handlers.
- The core business domain can be thoroughly tested without setting up external system dependencies.
- Code readability and long-term maintainability are drastically improved.

**Negative:**
- An increase in the total number of files, folders, and boilerplate structures.
- Requires strict code discipline and monitoring when introducing new software dependencies.

## Migration Strategy

The legacy modules (`folder`, `move_engine`, `activity_log`, `models`) that were originally in `src-tauri/src/main.rs` have been fully removed (see ADR-009). All business logic now lives in `quicksort-application` and `quicksort-domain`.

## References

- Clean Architecture by Robert C. Martin
- Domain-Driven Design by Eric Evans
- Hexagonal Architecture (Ports & Adapters) by Alistair Cockburn
