//! Single source of truth for all application metadata.
//!
//! This module contains the complete application profile: version, description,
//! authors, license, dependencies, credits, donation links, and external resources.
//!
//! The frontend accesses this data via the `get_app_metadata` Tauri command.

use serde::Serialize;

// ---------------------------------------------------------------------------
// Application info
// ---------------------------------------------------------------------------

pub const APP_NAME: &str = "QuickSort";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_DESCRIPTION: &str = "Next-generation file manager for Windows — fast, keyboard-driven, with smart sorting and context menu integration.";
pub const APP_LICENSE: &str = "MIT";
pub const APP_REPOSITORY: &str = "https://github.com/azmrv/quicksort";
pub const APP_HOMEPAGE: &str = "https://github.com/azmrv/quicksort";

// ---------------------------------------------------------------------------
// Authors & Contributors
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct Person {
    pub name: &'static str,
    pub role: &'static str,
    pub url: Option<&'static str>,
}

pub const AUTHORS: &[Person] = &[
    Person {
        name: "pr0math3us",
        role: "Author & Lead Developer",
        url: Some("https://github.com/azmrv"),
    },
];

pub const CONTRIBUTORS: &[Person] = &[
    // Add contributors here as the project grows.
    // Person { name: "...", role: "...", url: Some("...") },
];

// ---------------------------------------------------------------------------
// Credits & Acknowledgments
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct Credit {
    pub name: &'static str,
    pub url: &'static str,
    pub reason: &'static str,
}

pub const CREDITS: &[Credit] = &[
    Credit {
        name: "Tauri",
        url: "https://tauri.app",
        reason: "Cross-platform application framework",
    },
    Credit {
        name: "React",
        url: "https://react.dev",
        reason: "Frontend UI library",
    },
    Credit {
        name: "Ant Design",
        url: "https://ant.design",
        reason: "UI component library",
    },
    Credit {
        name: "Vite",
        url: "https://vitejs.dev",
        reason: "Frontend build tool",
    },
];

// ---------------------------------------------------------------------------
// Dependencies (external crates & libraries)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct Dependency {
    pub name: &'static str,
    pub version: &'static str,
    pub license: &'static str,
    pub url: &'static str,
    pub description: &'static str,
}

pub const DEPENDENCIES: &[Dependency] = &[
    // === Framework ===
    Dependency {
        name: "tauri",
        version: "2",
        license: "MIT/Apache-2.0",
        url: "https://tauri.app",
        description: "Cross-platform desktop application framework",
    },
    Dependency {
        name: "tauri-plugin-dialog",
        version: "2",
        license: "MIT/Apache-2.0",
        url: "https://github.com/tauri-apps/plugins-workspace",
        description: "Native file/folder picker dialogs",
    },
    Dependency {
        name: "tauri-plugin-opener",
        version: "2",
        license: "MIT/Apache-2.0",
        url: "https://github.com/tauri-apps/plugins-workspace",
        description: "Open URLs and files in the default application",
    },
    // === Serialization ===
    Dependency {
        name: "serde",
        version: "1",
        license: "MIT/Apache-2.0",
        url: "https://github.com/serde-rs/serde",
        description: "Generic serialization/deserialization framework",
    },
    Dependency {
        name: "serde_json",
        version: "1",
        license: "MIT/Apache-2.0",
        url: "https://github.com/serde-rs/json",
        description: "JSON serialization/deserialization",
    },
    // === Windows ===
    Dependency {
        name: "windows",
        version: "0.62.2",
        license: "MIT/Apache-2.0",
        url: "https://github.com/microsoft/windows-rs",
        description: "Windows API bindings for Rust",
    },
    Dependency {
        name: "winreg",
        version: "0.56",
        license: "MIT/Apache-2.0",
        url: "https://github.com/gentoo90/windows-rs",
        description: "Windows registry access",
    },
    Dependency {
        name: "win-ctx",
        version: "1.4.1",
        license: "MIT",
        url: "https://github.com/nicmcd/win-ctx",
        description: "Windows context menu integration",
    },
    // === Utilities ===
    Dependency {
        name: "clap",
        version: "4",
        license: "MIT/Apache-2.0",
        url: "https://github.com/clap-rs/clap",
        description: "CLI argument parsing",
    },
    Dependency {
        name: "anyhow",
        version: "1",
        license: "MIT/Apache-2.0",
        url: "https://github.com/dtolnay/anyhow",
        description: "Flexible error handling",
    },
    Dependency {
        name: "chrono",
        version: "0.4",
        license: "MIT/Apache-2.0",
        url: "https://github.com/chronotope/chrono",
        description: "Date and time library",
    },
    Dependency {
        name: "uuid",
        version: "1",
        license: "MIT/Apache-2.0",
        url: "https://github.com/uuid-rs/uuid",
        description: "UUID generation (v4)",
    },
    Dependency {
        name: "tokio",
        version: "1",
        license: "MIT",
        url: "https://github.com/tokio-rs/tokio",
        description: "Async runtime",
    },
    Dependency {
        name: "tracing",
        version: "0.1",
        license: "MIT",
        url: "https://github.com/tokio-rs/tracing",
        description: "Structured logging",
    },
    Dependency {
        name: "tracing-subscriber",
        version: "0.3",
        license: "MIT",
        url: "https://github.com/tokio-rs/tracing",
        description: "Log output formatting and filtering",
    },
    Dependency {
        name: "parking_lot",
        version: "0.12",
        license: "MIT/Apache-2.0",
        url: "https://github.com/Amanieu/parking_lot",
        description: "Efficient mutex and rwlock implementations",
    },
    Dependency {
        name: "directories",
        version: "6",
        license: "MIT/Apache-2.0",
        url: "https://github.com/xdg-rs/directories",
        description: "Standard platform-specific directories",
    },
    Dependency {
        name: "async-trait",
        version: "0.1",
        license: "MIT/Apache-2.0",
        url: "https://github.com/dtolnay/async-trait",
        description: "Async methods in traits",
    },
    // === Frontend ===
    Dependency {
        name: "react",
        version: "19",
        license: "MIT",
        url: "https://react.dev",
        description: "Frontend UI library",
    },
    Dependency {
        name: "ant-design",
        version: "6.5.0",
        license: "MIT",
        url: "https://ant.design",
        description: "UI component library",
    },
    Dependency {
        name: "vite",
        version: "7",
        license: "MIT",
        url: "https://vitejs.dev",
        description: "Frontend build tool and dev server",
    },
];

// ---------------------------------------------------------------------------
// Donation & Support
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct DonationLink {
    pub platform: &'static str,
    pub url: &'static str,
    pub label: &'static str,
}

pub const DONATION_LINKS: &[DonationLink] = &[
    DonationLink {
        platform: "donationalerts.com",
        url: "https://dalink.to/pr0math3us",
        label: "donationalerts.com",
    },
    // DonationLink {
    //     platform: "Buy Me a Coffee",
    //     url: "https://buymeacoffee.com/pr0math3us",
    //     label: "Buy Me a Coffee",
    // },
    // DonationLink {
    //     platform: "Buy Me a Coffee",
    //     url: "https://buymeacoffee.com/pr0math3us",
    //     label: "Buy Me a Coffee",
    // },
];

// ---------------------------------------------------------------------------
// External Resources
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ExternalResource {
    pub name: &'static str,
    pub url: &'static str,
    pub description: &'static str,
    pub resource_type: &'static str, // "source", "docs", "community", "ad"
}

pub const EXTERNAL_RESOURCES: &[ExternalResource] = &[
    ExternalResource {
        name: "GitHub Repository",
        url: "https://github.com/azmrv/quicksort",
        description: "Source code, issues, and releases",
        resource_type: "source",
    },
    ExternalResource {
        name: "Documentation",
        url: "https://github.com/azmrv/quicksort#readme",
        description: "Project README and setup guide",
        resource_type: "docs",
    },
    ExternalResource {
        name: "Tauri Documentation",
        url: "https://tauri.app/start/",
        description: "Tauri framework documentation",
        resource_type: "docs",
    },
    ExternalResource {
        name: "Rust Book",
        url: "https://doc.rust-lang.org/book/",
        description: "The Rust Programming Language",
        resource_type: "docs",
    },
];

// ---------------------------------------------------------------------------
// Structured metadata for the frontend (serializable)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AppMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub license: &'static str,
    pub repository: &'static str,
    pub homepage: &'static str,
    pub authors: &'static [Person],
    pub contributors: &'static [Person],
    pub credits: &'static [Credit],
    pub dependencies: &'static [Dependency],
    pub donation_links: &'static [DonationLink],
    pub external_resources: &'static [ExternalResource],
}

/// Returns the complete application metadata for the frontend.
pub fn get_metadata() -> AppMetadata {
    AppMetadata {
        name: APP_NAME,
        version: APP_VERSION,
        description: APP_DESCRIPTION,
        license: APP_LICENSE,
        repository: APP_REPOSITORY,
        homepage: APP_HOMEPAGE,
        authors: AUTHORS,
        contributors: CONTRIBUTORS,
        credits: CREDITS,
        dependencies: DEPENDENCIES,
        donation_links: DONATION_LINKS,
        external_resources: EXTERNAL_RESOURCES,
    }
}
