mod commands;
mod discord_log;
mod ledger;
mod models;
mod placement;
mod runner;
mod services;
mod system;

use commands::GameIndex;
use runner::Runner;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .setup(|app| {
            let resource_dir = app
                .path()
                .resource_dir()
                .unwrap_or_else(|_| std::env::current_exe().unwrap().parent().unwrap().to_path_buf());
            let runner = Runner::new(
                resource_dir.join("engine").join("dummy.exe"),
                services::storage::runtime_dir(),
                services::storage::ledger_path(),
            );
            runner.remove_leftovers(&services::storage::legacy_local_dir());
            app.manage(runner);
            app.manage(GameIndex::default());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                window.state::<Runner>().stop_all();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_games,
            commands::refresh_games,
            commands::start_game,
            commands::stop_game,
            commands::stop_all,
            commands::get_running,
            commands::is_discord_running,
            commands::steam_restart_needed,
            commands::open_steam_store,
            commands::reveal_dummy,
            commands::load_settings,
            commands::save_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                app_handle.state::<Runner>().stop_all();
            }
        });
}
