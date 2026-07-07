# Error Handling Strategy

We utilize distinct error handling patterns tailored specifically to the requirements of each architectural layer.

## Domain Layer

Within the Domain layer, we rely exclusively on our own custom `DomainError` type. This is a pure Rust enum containing variants that explicitly describe business rule violations. Examples include:
- `InvalidFolderName`
- `FolderNotFound`
- `OperationNotAllowed`

This enum is completely self-contained and must not depend on any third-party external crates (relying solely on the Rust standard library). We strictly prohibit the use of helper crates like `anyhow` or `thiserror` within the Domain boundary.

## Application Layer

In the Application layer, we leverage the `thiserror` crate to define granular, domain-meaningful error structures for each specific Use Case (e.g., `ExecuteOperationError`). Typical enum variants include:
- `RepositoryError` (abstract wrapper)
- `FileSystemError`
- `ValidationError`
- `ConflictError`

**Crucial Constraint:** The Application layer must remain completely unaware of the technical, concrete error types originated by underlying infrastructure components (such as a raw `std::io::Error`). We gracefully encapsulate and translate these low-level faults into abstract Application-level variants within the boundaries of the Infrastructure layer.

## Infrastructure Layer

The Infrastructure layer implements its own technical error types (e.g., `JsonRepositoryError`, `StdFileSystemError`). However, when fulfilling port contracts, it is mandatory to catch these native infrastructure exceptions and map them directly to the abstract error variants defined by the port contracts (such as `RepositoryError` or `FileSystemError`).

## Adapters Layer

Within our delivery mechanisms and consumer interfaces (such as the React GUI and Windows Shell Extensions), we utilize the `anyhow` crate for convenience. Because this layer sits at the outermost boundary of the system, using generic error formatting here poses zero risk of contaminating our core business rules.

## The Universal Rule

Under no circumstances should code inside the Domain or Application layers invoke unrecoverable panic states (such as `panic!`, `unwrap()`, or `expect()`). Every single edge case or operational failure must be handled explicitly and returned up the execution stack wrapped inside a standard Rust `Result` type. Panics are tolerated within the Infrastructure and Adapters layers only under exceptional, fatal scenarios (such as an absolute inability to read critical platform configurations during the initial application boot routine).
