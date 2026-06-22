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
            commands::core::auth::auth_status,
            commands::core::auth::auth_set_token,
            commands::core::auth::auth_clear,
            commands::core::auth::auth_import_from_file,
            commands::core::auth::auth_login,
            commands::core::auth::cli_terminal_login,
            commands::core::sessions::session_create,
            commands::core::sessions::session_list,
            commands::core::sessions::session_restore,
            commands::core::sessions::session_send,
            commands::core::sessions::session_stop,
            commands::core::sessions::session_close,
            commands::core::sessions::session_set_cwd,
            commands::core::sessions::os_username,
            commands::io::fs::home_dir,
            commands::io::fs::list_dirs,
            commands::io::fs::pick_folder,
            commands::io::slash::slash_commands,
            commands::core::deps::check_claude,
            commands::core::deps::install_claude,
            commands::core::deps::provider_installed,
            commands::core::deps::provider_install_cmd,
            commands::meta::usage_get,
            commands::meta::usage_get_provider,
            commands::meta::claude_models,
            commands::meta::provider_models,
            commands::meta::claude_defaults,
            commands::meta::subscription_plan,
            commands::library::skills_cmd::skills_installed,
            commands::library::skills_cmd::project_skills,
            commands::library::skills_cmd::skill_write,
            commands::library::skills_cmd::skill_read,
            commands::library::skills_cmd::skill_delete,
            commands::library::plugins::plugins_installed,
            commands::library::plugins::plugin_uninstall,
            commands::library::mcp::mcp_installed,
            commands::library::mcp_add::mcp_add,
            commands::library::mcp_add::mcp_add_json,
            commands::library::mcp::mcp_remove,
            commands::library::mcp_config::mcp_read_raw,
            commands::library::mcp_config::mcp_write_raw,
            commands::learn::reflect_and_learn,
            commands::auto::effort_levels,
            commands::auto::auto_pick,
            commands::io::history::recent_sessions,
            commands::io::history::search_sessions,
            commands::io::history::load_messages,
            commands::io::net::net_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
