//! Initialization of the structured logging system.
//!
//! This module configures `tracing-subscriber` as the global log subscriber
//! for the entire application (Tauri adapter, IPC server, CLI commands).
//! Once `init()` is called, all `tracing::info!`, `tracing::error!`, etc.
//! macros will produce output according to the configured format and filters.
//!
//! # Why we use `tracing` instead of `log`
//! `tracing` provides:
//! - **Structured, async-aware spans** – ideal for tracking operations across
//!   multiple threads and async tasks.
//! - **Scoped filtering** – we can enable/disable logging per module at runtime
//!   via the `RUST_LOG` environment variable.
//! - **Integration with Tauri** – Tauri's internal logging also uses `tracing`,
//!   so we get a unified log stream.
//!
//! # How to control the log level
//! Set the `RUST_LOG` environment variable before starting the application:
//! ```powershell
//! # Show only errors from our crate, but all info+ from Tauri internals
//! $env:RUST_LOG="quicksort=error,tauri=info"
//! cargo run
//! ```
//!
//! # When to call this function
//! `init()` must be called exactly once, before any `tracing` macros are used.
//! In `main.rs` it is called right after the CLI arguments are parsed, so
//! both the CLI and GUI parts benefit from logging.

/// Initializes the global tracing subscriber.
///
/// The subscriber is configured with:
/// - An `EnvFilter` that reads the `RUST_LOG` environment variable.
///   If the variable is not set, the default is `warn` for all crates.
/// - A human-readable, compact output format suitable for terminal display.
///
/// # Panics
/// Panics if called more than once – `tracing_subscriber` forbids multiple
/// global subscribers.  If you need to reconfigure logging at runtime, use
/// `EnvFilter::reload` instead of calling `init()` again.
pub fn init() {
    tracing_subscriber::fmt()
        // The `EnvFilter` enables/disables log events based on their target
        // (crate/module name) and level.  It reads the RUST_LOG env var.
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
}