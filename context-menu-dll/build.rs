fn main() {
    // Embed the version resource (icon, version info, manifest)
    embed_resource::compile("resources.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();

    // Suppress LNK4104 warnings about DLL exports missing PRIVATE attribute.
    // Rust's #[no_mangle] extern "system" exports do not set PRIVATE, but this
    // is harmless for COM DLLs where these symbols are the intended public API.
    println!("cargo:rustc-link-arg=/ignore:4104");
}
