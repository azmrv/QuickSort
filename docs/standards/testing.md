# Testing Strategy

We enforce a tiered testing paradigm structured across three distinct testing levels, supplemented by automated architectural verification.

## 1. Unit Tests

Unit tests validate individual software components in total isolation.

*   **Domain Layer:** We test value objects, entities, and core domain services. These tests run completely pure and require zero external mock injection or system dependencies.
*   **Application Layer:** We test individual Use Cases by injecting lightweight mocks of our abstract ports. These verify orchestration logic, pipeline flow, and correct `DomainEvent` generation.
*   **Infrastructure Layer:** We test concrete port implementations in sandboxed scopes (for instance, validating the `JsonConfigurationRepository` against ephemeral, short-lived temporary files).

All unit tests reside directly alongside the source code in local `mod tests` blocks and are executed using the native `cargo test` command.

## 2. Contract Tests

Contract tests verify that a concrete Infrastructure adapter adheres strictly to the behavioral specification defined by its abstract Application port. 

For instance, we maintain a `ConfigurationRepositoryContractTest` suite that runs uniformly against both the production-ready `JsonConfigurationRepository` and the automated test-focused `InMemoryConfigurationRepository`. This guarantees that all current and future implementations behave identically under identical constraints.

Contract testing configurations are located in the `tests/contract/` directory.

## 3. Integration Tests

Integration tests validate the end-to-end orchestration and interaction across multiple architectural layers (e.g., executing a Use Case coupled with a real Repository adapter and a real `FileSystem` port driver). 

These tests interact with actual disk operations, safely locked within isolated, temporary runtime directories that are automatically wiped clean upon test completion.

Integration testing suites are located in the `tests/integration/` directory.

## Architectural Enforcement

To programmatically enforce the dependency boundaries defined in our [Dependency Rule](./../adr/002-dependency-rule.md), we integrate automated linting checks into our CI/CD pipeline. 

A dedicated workspace scanning script (written in Python or Bash) parses all internal `Cargo.toml` configurations during build time. It will immediately trigger a CI failure if forbidden dependencies cross layer thresholds (e.g., if the `domain` crate attempts to import metadata serialization libraries like `serde`).

## Coverage Expectations

We explicitly avoid chasing an arbitrary 100% test coverage metric. Instead, we prioritize deep, comprehensive code coverage across our mission-critical business boundaries: the **Domain layer** and **Application Use Cases**. We utilize `cargo tarpaulin` to automatically generate coverage metrics and tracking reports.
