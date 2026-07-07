# Rust Style Guide

We utilize the standard `rustfmt` formatting utility configured with default settings. The following project-specific guidelines must be strictly adhered to across the entire workspace:

## Naming Conventions
- **Types (Structs, Enums, Traits):** Must use `PascalCase`.
- **Variables and Functions:** Must use `snake_case`.
- **Constants and Statics:** Must use `SCREAMING_SNAKE_CASE`.
- **Modules and File Names:** Must use `snake_case`.

## Code Formatting
- **Line Width:** The maximum code line length is capped at 100 characters (aligned with default `rustfmt` rules).
- **Automation:** It is mandatory to execute `cargo fmt` before submitting every code commit.

## Clippy & Safety Invariants
- **Strict Prohibition:** Invoking unrecoverable panic panaceas—specifically `.unwrap()` or `.expect()`—is strictly forbidden within the **Domain** and **Application** layers. Developers must utilize the `?` operator alongside explicit `Result`-based error handling mechanisms as defined in our [Error Handling Strategy](./error-handling.md).
- **Infrastructure & Adapters Exception:** Utilizing `.unwrap()` is tolerated within the Infrastructure and Adapters layers exclusively under scenarios where an operational error is mathematically or structurally impossible (for instance, reading a hardcoded static configuration asset that is guaranteed to be embedded at compile time).

## Documentation and Comments
- **Public API:** Detailed Rustdoc comments (`///`) are required for all public functions, structs, enums, and traits across the codebase.
- **Language Uniformity:** All in-line code comments and documentation headers must be written exclusively in English to ensure workspace-wide uniformity.
- **Intent-Driven Comments:** Comments must prioritize explaining **why** a specific engineering path was taken, rather than describing **what** the code line is doing.

## Import Formatting (`use` blocks)
- **Grouping Order:** Imports inside modules must be structured sequentially in three distinct blocks separated by whitespace:
  1. Standard library components (`std::...`).
  2. External third-party dependencies from crates.io.
  3. Internal workspace crate dependencies (`crate::...` or path-based references).
- **Wildcards:** Using glob imports—specifically `use crate::*`—is strictly prohibited, except when importing common testing utilities within automated `mod tests` blocks.
