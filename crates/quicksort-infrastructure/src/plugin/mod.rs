//! Plugin system implementations.
//!
//! This module contains adapters for various plugin types:
//! - WCX: Total Commander packer plugins (archives)

pub mod wcx_adapter;

pub use wcx_adapter::{WcxPluginAdapter, WcxPluginLoader};
