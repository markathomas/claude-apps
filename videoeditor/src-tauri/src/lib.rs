pub mod commands;
pub mod error;
pub mod ffmpeg;
pub mod model;
pub mod paths;
pub mod project_io;
pub mod recent;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::new_project,
            commands::open_project,
            commands::save_project_cmd,
            commands::get_recent_projects,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
