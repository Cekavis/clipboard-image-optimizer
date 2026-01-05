use arboard::Clipboard;
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use clipboard_win::raw;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, PhysicalPosition,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
static PROCESSING_LOCK: Mutex<()> = Mutex::new(());
static ORIGINAL_IMAGE: Mutex<Option<arboard::ImageData<'static>>> = Mutex::new(None);
static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

fn show_progress_window() {
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("progress") {
            // Get the primary monitor to position the window
            if let Ok(Some(monitor)) = window.primary_monitor() {
                let monitor_size = monitor.size();
                let monitor_position = monitor.position();
                let window_size = window.outer_size().unwrap_or_default();

                // Position at lower right corner with some margin
                let margin = 20;
                let x = monitor_position.x as i32 + monitor_size.width as i32
                    - window_size.width as i32
                    - margin;
                let y = monitor_position.y as i32 + monitor_size.height as i32
                    - window_size.height as i32
                    - margin
                    - 48; // Account for taskbar

                let _ = window.set_position(PhysicalPosition::new(x, y));
            }
            let _ = window.show();
        }
    }
}

fn emit_optimization_start() {
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit("optimization-start", ());
    }
}

fn emit_optimization_complete(original_size: u64, new_size: u64) {
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit(
            "optimization-complete",
            serde_json::json!({ "original_size": original_size, "new_size": new_size }),
        );
    }
}

struct Handler;

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        log::info!("Clipboard change happened!");
        process_clipboard();
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        log::error!("Error: {}", error);
        CallbackResult::Next
    }
}

fn path_is_image(path: &PathBuf) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lc = ext.to_string_lossy().to_lowercase();
        return ext_lc == "png"
            || ext_lc == "jpg"
            || ext_lc == "jpeg"
            || ext_lc == "bmp"
            || ext_lc == "tiff"
            || ext_lc == "webp";
    }
    false
}

fn process_clipboard() {
    // Get directory
    let app_data_dir = APP_DATA_DIR.get().expect("App data directory not set");
    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    // Get clipboard contents
    let file_list;
    let image;
    {
        let mut clipboard = CLIPBOARD.lock().unwrap();
        file_list = clipboard.as_mut().unwrap().get().file_list();
        image = clipboard.as_mut().unwrap().get_image();
    }

    // Get original size
    let original_size = if let Ok(list) = &file_list {
        if list.len() == 1 && path_is_image(&list[0]) {
            // Clipboard contains a single file path
            std::fs::metadata(&list[0]).unwrap().len()
        } else {
            0
        }
    } else {
        // Find the "image/jpeg" format in clipboard
        let mut size: usize = 0;
        let mut buf = [0u8; 16];
        raw::open().expect("Failed to open clipboard");
        let iter = raw::EnumFormats::new();
        iter.for_each(|id| {
            if raw::format_name(id, buf.as_mut_slice().into()) == Some("image/jpeg") {
                size = raw::size(id).unwrap().get();
            }
        });
        size as u64
    };

    // Skip if already optimized
    if let Ok(file_list) = &file_list {
        if file_list.len() == 1 && file_list[0].parent() == Some(app_data_dir) {
            log::info!("Skipping optimized image.");
            return;
        }
    }

    // Allow only one processing at a time
    let guard = PROCESSING_LOCK.try_lock();
    if guard.is_err() {
        log::info!("Image processing is already in progress, skipping.");
        return;
    }

    let mut width: usize = 0;
    let mut height: usize = 0;
    let mut bytes: Vec<u8> = Vec::new();

    if let Ok(image) = image {
        width = image.width;
        height = image.height;
        bytes = image
            .bytes
            .chunks(4)
            .flat_map(|pixel| pixel[..3].to_vec())
            .collect();
        log::info!("Got image from clipboard: {}x{}", width, height);
    } else {
        if let Ok(file_list) = &file_list {
            if file_list.len() == 1 && path_is_image(&file_list[0]) {
                let img = image::open(&file_list[0]).expect("Failed to open image file");
                let rgb_img = img.to_rgb8();
                width = rgb_img.width() as usize;
                height = rgb_img.height() as usize;
                bytes = rgb_img.into_raw();
                log::info!("Got image from file clipboard: {}x{}", width, height);
            }
        }
    };

    if width == 0 {
        log::info!("No valid image found in clipboard, skipping.");
        return;
    }

    // Show progress window and emit start event
    show_progress_window();
    emit_optimization_start();

    let image_path = app_data_dir.join("optimized.jpg");
    let new_size = save_image(width, height, bytes, &image_path);

    let mut clipboard = CLIPBOARD.lock().unwrap();
    clipboard
        .as_mut()
        .unwrap()
        .clear()
        .expect("Failed to clear clipboard");
    clipboard
        .as_mut()
        .unwrap()
        .set()
        .file_list(&[image_path.clone()])
        .expect("Failed to set clipboard file list");

    // Emit completion event with file size
    emit_optimization_complete(original_size, new_size);
}

#[tauri::command]
fn set_auto_start(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autostart_manager = app.autolaunch();
    if enabled {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_auto_start(app: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app.autolaunch();
    autostart_manager.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_progress() {
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("progress") {
            let _ = window.hide();
        }
    }
}

#[tauri::command]
fn revert_clipboard() {
    log::info!("Reverting clipboard to original image");
    let image = ORIGINAL_IMAGE.lock().unwrap();
    let mut clipboard = CLIPBOARD.lock().unwrap();
    clipboard
        .as_mut()
        .unwrap()
        .clear()
        .expect("Failed to clear clipboard");
    clipboard
        .as_mut()
        .unwrap()
        .set()
        .image(image.as_ref().unwrap().clone())
        .expect("Failed to set clipboard image");
    log::info!("Clipboard reverted to original image");
    hide_progress();
}

/// Encode the image and return the JPEG size
fn save_image(width: usize, height: usize, image_data: Vec<u8>, path: &PathBuf) -> u64 {
    assert_eq!(image_data.len(), width * height * 3);

    let result = std::panic::catch_unwind(|| -> std::io::Result<Vec<u8>> {
        let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        comp.set_quality(60.0);

        comp.set_size(width, height);
        let mut comp = comp.start_compress(Vec::new())?;

        comp.write_scanlines(&&image_data[..])?;

        let writer = comp.finish()?;
        Ok(writer)
    });

    match result {
        Ok(Ok(jpeg_data)) => {
            let size = jpeg_data.len() as u64;
            std::fs::write(path, jpeg_data).expect("Failed to write JPEG file");
            log::info!("Optimized image saved to {:?}", path);
            size
        }
        _ => {
            log::error!("Failed to compress image");
            0
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize clipboard
    {
        let mut clipboard = CLIPBOARD.lock().unwrap();
        *clipboard = Some(Clipboard::new().expect("Failed to initialize clipboard"));
    }

    // Start clipboard monitoring in a separate thread
    std::thread::spawn(|| {
        let mut master = Master::new(Handler).expect("Failed to create clipboard master");
        master.run().expect("Failed to run clipboard master");
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .invoke_handler(tauri::generate_handler![
            set_auto_start,
            get_auto_start,
            hide_progress,
            revert_clipboard
        ])
        .setup(|app| {
            // Store app handle for use in clipboard handler
            APP_HANDLE
                .set(app.handle().clone())
                .expect("Failed to set app handle");

            // Get app data directory
            let dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");
            APP_DATA_DIR
                .set(dir)
                .expect("Failed to set app data directory");

            // Initialize tray
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        log::info!("quit menu item was clicked");
                        app.exit(0);
                    }
                    "settings" => {
                        log::info!("settings menu item was clicked");
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {
                        log::error!("menu item {:?} not handled", event.id);
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        log::info!("left click pressed and released");
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                log::info!("Window close requested, hiding window instead.");
                api.prevent_close();
                window.hide().unwrap();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
