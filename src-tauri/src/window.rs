use tauri;

#[tauri::command]
pub fn minimize_window(window: tauri::Window) {
    if let Err(e) = window.minimize() {
        println!("Error minimizing window: {}", e);
    }
}

#[tauri::command]
pub fn maximize_window(window: tauri::Window) {
    match window.is_maximized() {
        Ok(is_maximized) => {
            let result = if is_maximized {
                println!("Window is currently maximized. Unmaximizing...");
                window.unmaximize()
            } else {
                println!("Window is currently not maximized. Maximizing...");
                window.maximize()
            };
            
            if let Err(e) = result {
                eprintln!("Error toggling maximize state: {}", e);
            } else {
                println!("Window maximize state toggled successfully");
            }
        }
        Err(e) => {
            eprintln!("Error checking window state: {}", e);
            if let Err(e) = window.maximize() {
                eprintln!("Fallback maximize also failed: {}", e);
            }
        }
    }
}

#[tauri::command]
pub fn close_window(window: tauri::Window) {
    // Emit event before closing for any cleanup
    let _ = window.emit("window-will-close", {});
    
    // Small delay to ensure event is processed
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Now close the window
    if let Err(e) = window.close() {
        println!("Error closing window: {}", e);
    }
} 