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

#[tauri::command]
pub fn start_drag(window: tauri::Window) {
    if let Err(e) = window.start_dragging() {
        println!("Error starting drag: {}", e);
    }
}

#[tauri::command]
pub fn toggle_fullscreen(window: tauri::Window) {
    match window.is_fullscreen() {
        Ok(is_fullscreen) => {
            let result = if is_fullscreen {
                println!("Window is currently fullscreen. Exiting fullscreen...");
                window.set_fullscreen(false)
            } else {
                println!("Window is currently not fullscreen. Entering fullscreen...");
                window.set_fullscreen(true)
            };
            
            if let Err(e) = result {
                eprintln!("Error toggling fullscreen state: {}", e);
            } else {
                println!("Window fullscreen state toggled successfully");
            }
        }
        Err(e) => {
            eprintln!("Error checking window fullscreen state: {}", e);
            // Fallback to setting fullscreen
            if let Err(e) = window.set_fullscreen(true) {
                eprintln!("Fallback fullscreen toggle also failed: {}", e);
            }
        }
    }
}

#[tauri::command]
pub fn move_window(window: tauri::Window, delta_x: f64, delta_y: f64) {
  if let Ok(position) = window.inner_position() {
    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
      x: position.x as i32 + delta_x as i32,
      y: position.y as i32 + delta_y as i32,
    }));
  }
}