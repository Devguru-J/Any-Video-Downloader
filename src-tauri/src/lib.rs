mod commands;
mod jobs;
mod policy;
mod scraper;
mod sidecar;
mod ytdlp;

use jobs::JobRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,any_video_lib=debug".into()),
        )
        .init();

    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                use tauri::Manager;
                if let Some(win) = app.get_webview_window("main") {
                    win.open_devtools();
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(JobRegistry::new())
        .invoke_handler(tauri::generate_handler![
            commands::probe_url,
            commands::start_download,
            commands::cancel_download,
            commands::list_jobs,
            commands::clear_history,
            commands::open_in_finder,
            commands::check_policy,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
