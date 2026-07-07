# Executable Specification

## What Is an Executable Specification?

An **Executable Specification** is a set of automated tests that describe the expected behaviour of the system **before any implementation exists**. These tests are written in the language of the domain and serve as:

- Formal, living documentation of business rules.
- A safety net that prevents regression.
- A precise contract for developers and infrastructure implementers.

In QuickSort, we treat executable specifications as **first‑class architectural artefacts**, not as after‑thought verification. They are the bridge between the **Domain Model** and the **Application Layer**.

---

## Why Executable Specification?

### 1. It Prevents Infrastructure Leakage

When tests are written after code, they tend to mirror the implementation details (e.g., “the repository saves JSON”, “the file system uses `std::fs`”). By writing tests first, we force ourselves to think in terms of **what** the system does, not **how** it does it. This keeps the Application Layer decoupled from Infrastructure.

### 2. It Documents Behaviour Naturally

BDD‑style tests (`Given‑When‑Then`) are readable by product owners, QA, and developers alike. They become the **ubiquitous language** of the system.

### 3. It Accelerates Development

With a complete set of red tests, developers know exactly what to implement. There is no ambiguity about edge cases or error handling. The feedback loop is instant: run the tests, see what fails, fix it, repeat.

### 4. It Enables Safe Refactoring

When the code evolves, the tests remain the same. If a change breaks a business rule, the test fails immediately. This gives confidence to refactor the Application or Infrastructure layers without rewriting the specification.

---

## The Test Pyramid for QuickSort

In our project, we organise tests into several layers, each addressing a different concern.

```
┌─────────────────────────────────────────────┐
│          End‑to‑End Tests                   │  (GUI + Shell + Core)
├─────────────────────────────────────────────┤
│          Integration Tests                  │  (Use Cases + real Infrastructure)
├─────────────────────────────────────────────┤
│          Contract Tests                     │  (Port implementations)
├─────────────────────────────────────────────┤
│          Executable Specification           │  (Use Cases with mocks)
├─────────────────────────────────────────────┤
│          Property / Fuzz Tests              │  (Domain invariants)
├─────────────────────────────────────────────┤
│          Unit Tests                         │  (Domain entities)
└─────────────────────────────────────────────┘
```

### Executable Specification (Application Layer)

These tests verify the behaviour of **Use Cases** in isolation. They use **test doubles** (mocks, stubs) for all outbound ports (repositories, file system, clock, etc.). They do not touch real infrastructure.

**Characteristics:**
- Run in milliseconds.
- Deterministic – no randomness, no file I/O.
- Cover all scenarios: success, errors, edge cases, undo, conflicts.
- Use a **Given‑When‑Then** structure.

### Contract Tests

These tests ensure that any implementation of an outbound port adheres to the expected contract. For example, `JsonConfigurationRepository` and `InMemoryConfigurationRepository` must both pass the same set of tests.

### Architecture Tests

These tests enforce **dependency rules** (e.g., Domain cannot import `tokio` or `serde`). They are often implemented as compile‑time checks or CI scripts.

### Property Tests

These tests generate random inputs to verify **invariants** (e.g., `WindowsPath` never panics on any valid string, or `ConflictResolver` always produces a unique name).

### Fuzz Tests

Fuzz tests feed malformed or random bytes into the system to find crashes or panics. They are especially useful for parsing and validation logic.

---

## How to Write Executable Specifications

### Structure

Place each Use Case in its own module under `crates/quicksort-application/tests/scenarios/`.

Example:

```
crates/quicksort-application/tests/scenarios/
├── mod.rs
├── execute_operation/
│   ├── mod.rs
│   ├── move.rs
│   ├── copy.rs
│   ├── delete.rs
│   ├── rename.rs
│   ├── conflicts.rs
│   ├── undo.rs
│   └── errors.rs
```

### Given‑When‑Then Pattern

- **Given** – set up the initial state (prepare mocks, create test data).
- **When** – execute the Use Case with specific input.
- **Then** – verify the outcome (result, state changes, events, mock calls).

### Use Test Doubles

All outbound ports must be replaced with test doubles. We provide ready‑to‑use mocks (e.g., `MockFileSystem`, `MockOperationRepository`, `StubIdGenerator`). These mocks allow us to control errors, inspect calls, and emulate file system behaviour without touching the disk.

### Example

```rust
#[tokio::test]
async fn move_single_file_to_existing_folder() {
    // Given
    let fs = MockFileSystem::new();
    let src = WindowsPath::new("C:\\src\\file.txt").unwrap();
    let dst = WindowsPath::new("D:\\dst\\file.txt").unwrap();
    fs.create_file(&src, 1024);

    let repo = MockOperationRepository::default();
    let clock = FrozenClock::new();
    let id_gen = StubIdGenerator::new("op-123");

    let use_case = ExecuteOperationUseCase::new(
        Box::new(StubConfigRepository),
        Box::new(repo.clone()),
        Box::new(fs.clone()),
        Box::new(id_gen),
        Box::new(clock),
        Box::new(SuffixConflictResolver),
    );

    let command = OperationCommand::Move {
        source_paths: vec![src.clone()],
        target_folder_path: Some(dst.clone()),
        overwrite_policy: OverwritePolicy::Skip,
    };

    // When
    let result = use_case.execute(command).await.unwrap();

    // Then
    assert_eq!(result.processed_files, 1);
    assert_eq!(result.bytes_moved, 1024);
    assert!(!fs.exists(&src).await.unwrap());
    assert!(fs.exists(&dst).await.unwrap());

    let saved = repo.find_by_id(&result.operation_id).await.unwrap().unwrap();
    assert!(matches!(saved.state, OperationState::Completed { .. }));
}
```

---

## Coverage Goals

| Layer | Covered By | Target |
|-------|------------|--------|
| Domain (entities, value objects) | Unit tests + Property tests | 90%+ |
| Application (Use Cases) | Executable Specification | 100% of scenarios |
| Port interfaces | Contract tests | 100% of methods |
| Infrastructure (adapters) | Contract tests + Integration tests | 80%+ |
| Architecture rules | Architecture tests | All rules |

---

## Integration with CI

The test suite is executed on every pull request and merge to main. The CI pipeline includes:

1. `cargo test` – all unit, specification, and contract tests.
2. `cargo test --features fuzz` – property and fuzz tests (optional, time‑consuming).
3. `python docs/standards/architecture_guard.py` – architecture dependency check.
4. `cargo clippy` with strict warnings.

The pipeline fails if any test fails or if any architecture rule is violated.

---

## Frequently Asked Questions

### Q: Isn’t this just TDD?

It is an evolution of TDD. While TDD focuses on the **red‑green‑refactor** cycle for individual functions, Executable Specification focuses on **behaviour first**. The specifications are written at the Use Case level, not at the method level, and they serve as the primary source of truth.

### Q: Do we need to maintain the specs when the domain changes?

Yes, but changes to the domain should be reflected first in the specifications. This ensures that the business rules remain the source of truth.

### Q: How do we handle configuration or secrets in tests?

All test doubles are configured in‑memory. No external files, environment variables, or network calls are allowed in the specification tests. If configuration is needed, it should be provided through a stub.

### Q: What about GUI or Shell extension tests?

Those belong to the adapter layer and should use **integration** or **end‑to‑end** tests. They are written separately and may run with real infrastructure or in a sandboxed environment.

---

## Next Steps

- Write the executable specifications for all Use Cases (`ExecuteOperation`, `UndoOperation`, `GetFolders`).
- Implement the Application layer to make them pass.
- Use the specifications as the acceptance criteria for the Infrastructure layer.

---

## References

- [BDD in DDD projects – Martin Fowler](https://martinfowler.com/bliki/GivenWhenThen.html)
- [Executable Specifications – Dan North](https://dannorth.net/introducing-bdd/)
- [Clean Architecture – Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)


---
