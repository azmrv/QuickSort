//! Ports (interfaces) for the Application Layer.
//!
//! - Inbound ports: called by adapters (GUI, CLI, Shell)
//! - Outbound ports: implemented by infrastructure (file system, repositories, etc.)

pub mod inbound;
pub mod outbound;