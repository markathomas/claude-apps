use std::sync::Arc;

pub mod commands;
pub mod error;
pub mod ffmpeg;
pub mod media_repo;
pub mod model;
pub mod paths;
pub mod project_io;
pub mod proxy_worker;
pub mod recent;

pub fn run() {
    let (handle, rx) = proxy_worker::channel();
    let repo = Arc::new(media_repo::MediaRepo::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(repo.clone())
        .manage(handle)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let repo = repo.clone();
            tauri::async_runtime::spawn(async move {
                let emitter: Arc<dyn proxy_worker::EventEmitter> =
                    Arc::new(proxy_worker::TauriEmitter::new(app_handle));
                proxy_worker::run_worker_loop(rx, repo, emitter).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::new_project,
            commands::open_project,
            commands::save_project_cmd,
            commands::get_recent_projects,
            commands::import_media,
            commands::delete_media,
            commands::list_media,
            commands::timeline_insert_clip,
            commands::timeline_move_clip,
            commands::timeline_trim_clip,
            commands::timeline_split_clip,
            commands::timeline_delete_clip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
