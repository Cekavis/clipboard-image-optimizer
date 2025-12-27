use clipboard_master::{CallbackResult, ClipboardHandler, Master};

use std::io;

struct Handler;

impl ClipboardHandler for Handler {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        log::info!("Clipboard change happened!");
        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        log::error!("Error: {}", error);
        CallbackResult::Next
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
        .plugin(
            tauri_plugin_log::Builder::new()
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
