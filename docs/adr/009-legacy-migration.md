# ADR-009: Legacy Module Migration Strategy

**Status**: Superseded (migration completed 2026-08-21)  
**Date**: 2026-08-20  
**Decision Makers**: Core team

> **This ADR is kept for historical reference.** The migration it describes was completed on 2026-08-21. All legacy modules (`folder`, `move_engine`, `activity_log`, `models`) have been deleted. All commands use `ApplicationFacadeImpl`. The only operation repository remains `InMemoryOperationRepository` (JSON persistence is planned separately).

## References

- ADR-001: Architectural Style
- ADR-002: Dependency Rule
- Strangler Fig Pattern (Martin Fowler)
