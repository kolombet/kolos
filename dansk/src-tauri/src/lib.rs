mod ai;
mod commands;
mod db;
mod fsrs_engine;
mod types;

use commands::{
    add_word, ai_fill_word, delete_word, get_due_queue, get_stats, list_words, preview_review, submit_review,
    update_word,
};
use db::Db;
use fsrs_engine::Engine;
use tauri::Manager;

pub struct AppState {
    pub db: Db,
    pub fsrs: Engine,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("resolve app data dir");
            let db = Db::open(&data_dir.join("dansk.db")).expect("open database");
            app.manage(AppState {
                db,
                fsrs: Engine::new(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_word,
            update_word,
            delete_word,
            list_words,
            get_due_queue,
            preview_review,
            submit_review,
            get_stats,
            ai_fill_word,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
