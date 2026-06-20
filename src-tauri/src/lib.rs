// agentdeck — backend Tauri. (relaunch : chip dossier visible)
// Pilote plusieurs sessions Claude Code (multi-IA à terme). Les commandes exposées au
// frontend vivent dans le module `commands` (regroupées par domaine).

mod auth;
mod commands;
mod events;
mod provider;
mod session;
mod usage;

use session::SessionManager;
use usage::UsageStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir { file_name: Some("agentdeck".into()) },
                ))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init());
    // Auto-update : disponible uniquement sur desktop.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }
    builder
        .manage(SessionManager::default())
        .manage(UsageStore::load())
        .setup(|app| {
            // Poller de fond : vrai usage d'abonnement via l'endpoint OAuth.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(usage::run_poller(handle));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::auth_status,
            commands::auth::auth_set_token,
            commands::auth::auth_clear,
            commands::auth::auth_import_from_file,
            commands::auth::auth_login,
            commands::sessions::session_create,
            commands::sessions::session_list,
            commands::sessions::session_restore,
            commands::sessions::session_send,
            commands::sessions::session_stop,
            commands::sessions::session_close,
            commands::sessions::session_set_cwd,
            commands::sessions::os_username,
            commands::fs::home_dir,
            commands::fs::list_dirs,
            commands::fs::pick_folder,
            commands::slash::slash_commands,
            commands::deps::check_claude,
            commands::deps::install_claude,
            commands::meta::usage_get,
            commands::meta::claude_defaults,
            commands::meta::subscription_plan,
            commands::library::skills_installed,
            commands::library::skill_write,
            commands::library::skill_delete,
            commands::library::mcp_installed,
            commands::library::mcp_add,
            commands::library::mcp_add_json,
            commands::library::mcp_remove,
            commands::learn::reflect_and_learn,
            commands::auto::effort_levels,
            commands::auto::auto_pick,
            commands::history::recent_sessions,
            commands::history::search_sessions,
            commands::history::load_messages,
            commands::net::net_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
