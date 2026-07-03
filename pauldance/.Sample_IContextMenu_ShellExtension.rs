// This shows an example of how to implement the `IContextMenu` interface
// in Rust for Windows Shell context menu integration.

// The build results in a DLL that should be placed somewhere on the file
// system and then referenced in the registry. `activation.reg` shows the
// registry keys and values to set; can be used with `reg.exe import`.

// The Rust files should be moved into place as such:
//  * `Cargo.toml`
//  * `build.rs`
//  * `src/lib.rs`
//  * `src/shellext.rs`
//  * `src/icon.rs`
