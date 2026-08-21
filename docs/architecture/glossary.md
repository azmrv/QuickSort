# Glossary (Ubiquitous Language)

This document aggregates the core terminology utilized across the project codebase, documentation, and team communications. Every term defined below carries a strict, unambiguous architectural meaning.

*   **Operation** — An atomic action executed upon files or directories. Examples include: `Move`, `Copy`, `Delete`, `Rename`.
*   **OperationCommand** — A Data Transfer Object (DTO) that encapsulates all necessary parameters required to execute an operation, including the operation type, source paths, destination targets, and runtime flags.
*   **OperationPipeline** — A unified execution chain through which every command must sequentially progress: validation → conflict resolution → execution → event collection.
*   **Folder** — A core domain entity representing a file-system directory. It contains an identifier, a display name, an absolute system path, a favorite flag, usage statistics, and timestamps. It forms a central component of the user configuration.
*   **ConfigurationRepository** — An outbound port responsible for persisting and retrieving folders, application preferences, favorite lists, and custom sorting rules. In the default implementation, this data is stored within a single flat JSON file.
*   **OperationRepository** — An outbound port responsible for persisting the chronological transaction history of all executed operations. This ledger provides the foundational state engine required for Undo/Redo mechanics.
*   **Domain Event** — An immutable record of a business fact that has occurred within the system. Examples include: `OperationCompleted`, `OperationFailed`, `OperationUndone`.
*   **Port** — An abstract interface declared by the Application layer to communicate with the external world without leaking technical contexts. Examples include: `FileSystem`, `IdGenerator`, `Clock`.
*   **Adapter** — A concrete infrastructure implementation or an external driving trigger bound to a port contract. Examples include: `JsonConfigurationRepository`, `StdFileSystem`.
*   **Use Case** — A formalized, specific business interaction scenario orchestrating domain entities. Examples include: `ExecuteOperation`, `GetFoldersQuery`.
