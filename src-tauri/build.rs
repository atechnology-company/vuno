fn main() {
    if cfg!(target_os = "windows") {
        // Set WiX source files
        println!("cargo:rustc-env=WIX_SOURCES=wix/file-associations.wxs");
    }
    
    // Build the Tauri application
    tauri_build::build();
}
