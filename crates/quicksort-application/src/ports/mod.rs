//! Ports (interfaces) for the Application Layer.
//!
//! This module defines the boundary contracts that decouple the Application
//! layer from external systems (adapters, infrastructure).
//!
//! # Sub-modules
//! - `inbound`  – Interfaces that adapters (Tauri, CLI, Shell Extension) call.
//! - `outbound` – Interfaces implemented by infrastructure (repositories, file system).
//!
//! # Dependency Rule
//! Ports are owned by the Application layer and are implemented by outer layers.
//! The Domain layer never references ports.

pub mod inbound;
pub mod outbound;

// This module was originally part of the ports but has been moved to
// `outbound::OperationRepository` to follow the Hexagonal Architecture pattern.
// Kept commented out to document the historical structure.
// operation_repository is now defined in outbound/operation_repository.rs
