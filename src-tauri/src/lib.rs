use clipboard_master::{CallbackResult, ClipboardHandler, Master};
use clipboard_win::{get_clipboard, formats};
use arboard::Clipboard;
use tauri::Manager;
use image::RgbaImage;

use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

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
    let mut clipboard = Clipboard::new().unwrap();
    if let Ok(image) = clipboard.get_image() {
        log::info!("Got image from clipboard: {}x{}", image.width, image.height);
        let app_data_dir = APP_DATA_DIR.get().expect("App data directory not set");
        std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
        let image_path = app_data_dir.join("clipboard_image.png");
        log::info!("Saving image to {:?}", image_path);
        
        let rgba_image = RgbaImage::from_raw(
            image.width as u32,
            image.height as u32,
            image.bytes.to_vec(),
        ).expect("Failed to create image");
        
        rgba_image.save(&image_path).expect("Failed to save image");
        log::info!("Image saved to clipboard_image.png");
    } else {
        log::info!("Clipboard does not contain an image.");
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    log::info!("234");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::thread::spawn(|| {
        let mut master = Master::new(Handler).expect("Failed to create clipboard master");
        master.run().expect("Failed to run clipboard master");
    });

    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir().expect("Failed to get app data directory");
            APP_DATA_DIR.set(dir).expect("Failed to set app data directory");
            Ok(())
        })
        .plugin(
            tauri_plugin_log::Builder::new()
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
