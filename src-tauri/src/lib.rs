use arboard::Clipboard;
use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
static PROCESSING_LOCK: Mutex<()> = Mutex::new(());

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

fn process_clipboard() {
    let app_data_dir = APP_DATA_DIR.get().expect("App data directory not set");
    std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

    let mut clipboard = Clipboard::new().unwrap();
    if let Ok(files) = clipboard.get().file_list() {
        if files.len() == 1 && files[0].parent() == Some(app_data_dir) {
            log::info!("Skipping optimized image.");
            return;
        }
    }
    if let Ok(image) = clipboard.get_image() {
        if let Ok(_guard) = PROCESSING_LOCK.try_lock() {
            log::info!("Got image from clipboard: {}x{}", image.width, image.height);
            let image_path = app_data_dir.join("optimized.jpg");

            save_image(
                image.width,
                image.height,
                image
                    .bytes
                    .chunks(4)
                    .flat_map(|pixel| pixel[..3].to_vec())
                    .collect(),
                &image_path,
            );

            clipboard.clear().expect("Failed to clear clipboard");
            clipboard
                .set()
                .file_list(&[image_path])
                .expect("Failed to set clipboard file list");
        } else {
            log::info!("Image processing is already in progress, skipping.");
        }
    } else {
        log::info!("Clipboard does not contain an image.");
    }
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

fn save_image(width: usize, height: usize, image_data: Vec<u8>, path: &PathBuf) {
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
            std::fs::write(path, jpeg_data).expect("Failed to write JPEG file");
            log::info!("Optimized image saved to {:?}", path);
        }
        _ => {
            log::error!("Failed to compress image");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::thread::spawn(|| {
        let mut master = Master::new(Handler).expect("Failed to create clipboard master");
        master.run().expect("Failed to run clipboard master");
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .invoke_handler(tauri::generate_handler![set_auto_start, get_auto_start])
        .setup(|app| {
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
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        log::info!("quit menu item was clicked");
                        app.exit(0);
                    }
                    "settings" => {
                        log::info!("settings menu item was clicked");
                        app.get_webview_window("main").unwrap().show().unwrap();
                        app.get_webview_window("main").unwrap().set_focus().unwrap();
                    }
                    _ => {
                        log::error!("menu item {:?} not handled", event.id);
                    }
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
