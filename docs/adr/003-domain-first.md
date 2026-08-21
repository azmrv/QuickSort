# ADR-003: Domain First

**Status**: Accepted  
**Date**: 2026-07-07  
**Decision Makers**: Core team

## Context

Many software projects begin their development cycle by designing the GUI or database schema first. This pattern inevitably forces core business logic to adapt to specific technological limitations rather than driving the technical implementation itself.

## Problem

How can we ensure that business logic remains the central pillar of the codebase and remains completely decoupled from external implementation details?

## Constraints

- The development team consists of engineers with varying levels of experience.
- New team members must be able to onboard quickly and comprehend the core purpose of the project without getting lost in boilerplate technical code.

## Decision

**Development always begins with the Domain layer.** We model the core domain first: business operations, folders, domain rules, and internal lifecycle events. Application use cases, infrastructure wrappers, and graphical user interfaces are built only after the domain layer is established.

This approach means:
- All data structure and behavioral engineering decisions are made exclusively on the basis of business requirements.
- Technologies (such as Tauri, Windows WinAPI, or JSON serialization formats) are treated as implementation details that are integrated later in the cycle.
- The Domain layer must remain completely unaware that the application runs on a Windows OS environment, relies on Tauri bindings, or stores state inside JSON flat files.

## Alternatives Considered

- **UI-First:** Rejected because it frequently results in leaky abstractions where business constraints become tightly bound to presentation elements.
- **Database-First:** Rejected because it creates a rigid, brittle data model that is highly resistant to core business process changes.

## Consequences

**Positive:**
- Business logic remains highly stable even when underlying tech stacks are upgraded or completely replaced.
- The domain model can be comprehensively unit-tested without mock-injecting active GUIs or real file-system operations.
- Adding additional entry channels (such as a CLI wrapper or an automated REST agent) becomes trivial.

**Negative:**
- Requires a higher upfront time investment before writing user-facing GUI modules.
- Engineers accustomed to a traditional UI-centric workflow may experience initial friction adapting to an abstract domain-driven process.

## Migration Strategy

Legacy business calculations and validation routines from the old Tauri prototype and early context menu DLL have been fully harvested into the Domain layer (completed 2026-08-21).

## References

- Domain-Driven Design by Eric Evans
- "Domain First" implementation patterns in DDD
