# Architecture Vision

## What is QuickSort

QuickSort is not just another utility for moving files around. It is a comprehensive platform designed to execute any file and folder operations within the Windows environment. The platform is engineered around a core philosophical pivot: **"The Operation is the Primary Entity."** Directories, favorite lists, and preferences are merely configuration building blocks that enable operations to perform their work.

Currently, the system is capable of moving files. In the near future, it will expand to copy, rename, archive, calculate checksum hashes, synchronize directories, and apply complex automated rules. Every single one of these capabilities is treated uniformly as an **Operation**.

## Core Architectural Principles

*   **Clean Architecture + DDD + Hexagonal Patterns:** The codebase is divided by strict, non-negotiable boundaries into four clear layers: the core business domain (**Domain**), user scenario coordinators (**Application**), technical system mechanisms (**Infrastructure**), and driving input/output triggers (**Adapters** including the GUI, shell extensions, and CLI wrappers).
*   **Strict Inward Dependencies:** The code directions follow a one-way path. The Domain layer is entirely self-contained and unaware of external technical frameworks. The Application layer depends exclusively on the Domain and abstract ports. The Infrastructure layer implements those ports.
*   **Domain-First Methodology:** Engineering pipelines prioritize modeling business constraints and rules over writing technical code. If tomorrow we decide to completely overhaul the desktop GUI framework or port the underlying system to a non-Windows OS, the core Domain layer will remain completely untouched and unmodified.
*   **Operations as the Core Focus:** We architect the entire software lifecycle around the execution flow of actions, rather than focusing on target storage directories or folder hierarchies.
*   **Decoupling via Domain Events:** Every structural change or operational lifecycle state transition births an immutable business fact. The Application layer orchestrates and emits these events, leaving it up to the specialized Infrastructure adapters (such as active loggers, telemetry trackers, live GUIs, or history ledgers) to decide how to respond.

## High-Level Component Mapping

*   **Domain:** Contains concrete entity models (`Operation`, `Folder`, `OperationHistory`), semantic Value Objects, `Domain Events`, and pure Domain Services.
*   **Application:** Exposes executable Use Cases (`ExecuteOperation`, `GetConfiguration`), declares stable abstract Ports (repositories, file-system interfaces, ID generators, internal system clocks), and manages the unified execution `Pipeline` (validation → conflict mitigation → execution).
*   **Infrastructure:** Packs the physical adapters fulfilling the port agreements (flat-file JSON storage engines, OS operations routed via `std::fs`, background IPC channels, and system loggers).
*   **Adapters:** Houses the consumer interfaces driving or consuming the app core: the React-based Tauri GUI, native Windows Explorer COM Shell Extensions, a command-line interface (CLI), and potential future REST APIs.

## Practical Engineering Advantages

Adhering strictly to this vision guarantees:
- **Scalability:** Introducing entirely new specialized file operations requires zero structural modifications.
- **Technology Independence:** Total decoupling from third-party vendor frameworks and operating system variations.
- **Frictionless Testing:** The entire business core can be thoroughly unit-tested in isolation without making real, destructive calls to physical file systems or starting heavy UI windows.
- **Future Readiness:** The code establishes a pristine foundation for hosting complex user-defined automation rules, cron schedules, and third-party developer plugin ecosystems.

## Explicit Architectural Anti-Patterns (What We Do Not Do)

We do not bind our business logic to specific serialization formats or databases. We never invoke native Windows WinAPI methods directly within the Domain or Application boundaries. We do not allow presentation layers, UI-state properties, or graphical layout logic to contaminate our backend business processes.
