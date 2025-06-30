fn main() {
    // Skip icon requirements for development
    std::env::set_var("TAURI_SKIP_DEVTOOLS_CHECK", "true");
    tauri_build::build()
}
