//! Domain Services – stateless business logic that doesn't belong to a single
//! entity or value object.
//!
//! Domain services operate on domain objects and encapsulate rules that
//! involve multiple entities or require external computation.
//!
//! # Current Status
//! No domain services have been defined yet.  When a piece of logic doesn't
//! naturally fit into an existing entity or value object, create a new
//! sub-module here.
//!
//! # Example Candidates
//! - `ConflictResolutionService` – determines the final destination path
//!   when a file already exists, applying the chosen `OverwritePolicy`.
//! - `PathNormalisationService` – normalises user-supplied paths (already
//!   handled by `WindowsPath::new`, but more complex rules could be added).
//!
//! # Dependency Rule
//! Domain services MAY depend on domain entities, value objects, and domain
//! errors.  They MUST NOT depend on Application, Infrastructure, or Adapters.