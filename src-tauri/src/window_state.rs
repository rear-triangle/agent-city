use std::{path::PathBuf, sync::Mutex};
use tauri::{Emitter, Manager};
pub struct Placement {
    path: PathBuf,
    state: Mutex<serde_json::Value>,
    initialized: std::sync::atomic::AtomicBool,
}
impl Placement {
    pub fn load(path: PathBuf) -> Self {
        let state = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_else(|| serde_json::json!({"mode":"command"}));
        Self {
            path,
            state: Mutex::new(state),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }
    pub fn save(&self, w: &tauri::WebviewWindow) {
        if let (Ok(p), Ok(s)) = (w.outer_position(), w.inner_size()) {
            let mut state = self.state.lock().unwrap();
            let mode = state["mode"].as_str().unwrap_or("command").to_string();
            state[&mode] = serde_json::json!({"x":p.x,"y":p.y,"width":s.width,"height":s.height});
            if let Ok(bytes) = serde_json::to_vec(&*state) {
                let _ = city_core::files::atomic_write(&self.path, &bytes);
            }
        }
    }
    pub fn set_mode(&self, w: &tauri::WebviewWindow, mode: &str) -> Result<(), String> {
        if self
            .initialized
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            self.save(w);
        }
        let mut state = self.state.lock().unwrap();
        let saved = state[mode].clone();
        state["mode"] = mode.into();
        drop(state);
        if let (Some(width), Some(height)) = (saved["width"].as_u64(), saved["height"].as_u64()) {
            w.set_size(tauri::PhysicalSize::new(width as u32, height as u32))
                .map_err(|e| e.to_string())?;
            if let (Some(x), Some(y)) = (saved["x"].as_i64(), saved["y"].as_i64()) {
                w.set_position(tauri::PhysicalPosition::new(x as i32, y as i32))
                    .map_err(|e| e.to_string())?;
            }
        } else {
            w.set_size(tauri::LogicalSize::new(
                if mode == "ambient" { 420.0 } else { 1280.0 },
                if mode == "ambient" { 280.0 } else { 800.0 },
            ))
            .map_err(|e| e.to_string())?;
        }
        clamp(w);
        Ok(())
    }
}
pub fn clamp(w: &tauri::WebviewWindow) {
    let (Ok(monitors), Ok(position), Ok(size)) =
        (w.available_monitors(), w.outer_position(), w.outer_size())
    else {
        return;
    };
    let selected = monitors
        .iter()
        .find(|m| {
            let a = m.work_area();
            position.x >= a.position.x
                && position.y >= a.position.y
                && position.x < a.position.x + a.size.width as i32
                && position.y < a.position.y + a.size.height as i32
        })
        .or(monitors.first());
    if let Some(m) = selected {
        let a = m.work_area();
        let width = size.width.min(a.size.width);
        let height = size.height.min(a.size.height);
        let x = position
            .x
            .clamp(a.position.x, a.position.x + (a.size.width - width) as i32);
        let y = position
            .y
            .clamp(a.position.y, a.position.y + (a.size.height - height) as i32);
        if width < size.width || height < size.height {
            let _ = w.set_size(tauri::PhysicalSize::new(width, height));
        }
        if x != position.x || y != position.y {
            let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
        }
    }
}
pub fn mode(w: &tauri::WebviewWindow, mode: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    DESIRED_MODE.store(
        if mode == "ambient" {
            1
        } else if mode == "hide" {
            2
        } else {
            0
        },
        std::sync::atomic::Ordering::SeqCst,
    );
    if mode == "hide" {
        w.state::<Placement>().save(w);
        let _ = w.emit("city:visibility", false);
        return w.hide().map_err(|e| e.to_string());
    }
    let ambient = mode == "ambient";
    w.state::<Placement>()
        .set_mode(w, if ambient { "ambient" } else { "command" })?;
    w.set_ignore_cursor_events(ambient)
        .map_err(|e| e.to_string())?;
    w.set_focusable(!ambient).map_err(|e| e.to_string())?;
    w.set_always_on_top(ambient).map_err(|e| e.to_string())?;
    w.set_visible_on_all_workspaces(ambient)
        .map_err(|e| e.to_string())?;
    w.set_decorations(!ambient).map_err(|e| e.to_string())?;
    if ambient {
        show_ambient(w)?;
    } else {
        w.show().map_err(|e| e.to_string())?;
    }
    if !ambient {
        w.set_focus().map_err(|e| e.to_string())?;
    }
    let _ = w.emit("city:mode", if ambient { "ambient" } else { "command" });
    let _ = w.emit("city:visibility", true);
    Ok(())
}

/// Tauri's macOS `show` calls makeKeyAndOrderFront. Ambient must neither take
/// focus nor keep the key window left over from Command mode.
#[cfg(target_os = "macos")]
fn show_ambient(w: &tauri::WebviewWindow) -> Result<(), String> {
    let window = w.clone();
    w.run_on_main_thread(move || {
        let Ok(pointer) = window.ns_window() else {
            return; // The window was closed before the queued transition ran.
        };
        let mtm = objc2::MainThreadMarker::new().expect("Tauri main-thread callback");
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        // SAFETY: Tauri owns this NSWindow; access stays within its main-thread callback.
        let native = unsafe { &*pointer.cast::<objc2_app_kit::NSWindow>() };
        native.setIgnoresMouseEvents(true);
        native.resignKeyWindow();
        native.resignMainWindow();
        if app.isActive() {
            // AppKit's hide completes asynchronously. The notification observer
            // restores the overlay only after the focus handoff has finished.
            RESTORE_AFTER_HIDE.store(true, std::sync::atomic::Ordering::SeqCst);
            app.hide(None);
        } else {
            if app.isHidden() {
                app.unhideWithoutActivation();
            }
            native.orderFrontRegardless();
        }
    })
    .map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
static RESTORE_AFTER_HIDE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(target_os = "macos")]
static DESIRED_MODE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

#[cfg(target_os = "macos")]
pub fn initialize_native(w: &tauri::WebviewWindow) -> Result<(), String> {
    let window = w.clone();
    w.run_on_main_thread(move || {
        use objc2_app_kit::{NSApplication, NSApplicationDidHideNotification, NSWindow};
        use objc2_foundation::{NSNotificationCenter, NSOperationQueue};
        let callback = block2::RcBlock::new(
            move |_: std::ptr::NonNull<objc2_foundation::NSNotification>| {
                if !RESTORE_AFTER_HIDE.swap(false, std::sync::atomic::Ordering::SeqCst) {
                    return;
                }
                let desired = DESIRED_MODE.load(std::sync::atomic::Ordering::SeqCst);
                if desired == 2 {
                    return;
                }
                let Ok(pointer) = window.ns_window() else {
                    return;
                };
                let mtm = objc2::MainThreadMarker::new().expect("main notification queue");
                let app = NSApplication::sharedApplication(mtm);
                app.unhideWithoutActivation();
                // SAFETY: Tauri owns the window and this observer uses the main queue.
                let native = unsafe { &*pointer.cast::<NSWindow>() };
                if desired == 1 {
                    native.orderFrontRegardless();
                } else {
                    // An explicit Command request may arrive while AppKit is hiding.
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            },
        );
        // SAFETY: the callback captures only a Send window handle and always runs
        // on the main queue. NotificationCenter retains this app-lifetime observer.
        let _observer = unsafe {
            NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
                Some(NSApplicationDidHideNotification),
                None,
                Some(&NSOperationQueue::mainQueue()),
                &callback,
            )
        };
    })
    .map_err(|e| e.to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn initialize_native(_: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn show_ambient(w: &tauri::WebviewWindow) -> Result<(), String> {
    w.show().map_err(|e| e.to_string())
}
