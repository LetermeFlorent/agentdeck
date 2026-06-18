// agentdeck — backend Tauri.
// Pilote plusieurs sessions Claude Code (multi-IA à terme) et expose les commandes au frontend.

mod auth;
mod events;
mod provider;
mod session;
mod usage;

use provider::{ClaudeCodeProvider, Provider, TurnConfig};
use session::{SessionInfo, SessionManager};
use usage::{UsageSnapshot, UsageStore};

// ---------- Auth ----------

#[tauri::command]
fn auth_status() -> bool {
    auth::get_token().is_some()
}

#[tauri::command]
fn auth_set_token(token: String) -> Result<(), String> {
    auth::set_token(&token)
}

#[tauri::command]
fn auth_clear() -> Result<(), String> {
    auth::clear_token()
}

/// Importe un token depuis un fichier texte. Sans chemin, cherche
/// `<Téléchargements>/claude-token.txt`.
#[tauri::command]
fn auth_import_from_file(path: Option<String>) -> Result<(), String> {
    let path = match path {
        Some(p) if !p.is_empty() => std::path::PathBuf::from(p),
        _ => {
            let mut p = dirs::download_dir()
                .ok_or_else(|| "Dossier Téléchargements introuvable.".to_string())?;
            p.push("claude-token.txt");
            p
        }
    };
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Lecture {} : {e}", path.display()))?;
    let token = auth::extract_token_from_text(&content)
        .ok_or_else(|| "Aucun token sk-ant-oat… trouvé dans le fichier.".to_string())?;
    auth::set_token(&token)
}

/// Voie A : connexion Anthropic de base — lance `claude setup-token` (ouvre le navigateur),
/// récupère le token OAuth émis et le stocke. Expérimental (dépend de l'interactivité du CLI).
#[tauri::command]
async fn auth_login() -> Result<(), String> {
    let output = tokio::process::Command::new("claude")
        .arg("setup-token")
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Binaire `claude` introuvable dans le PATH.".to_string()
            } else {
                format!("Échec de `claude setup-token` : {e}")
            }
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let token = auth::extract_token_from_text(&stdout)
        .or_else(|| auth::extract_token_from_text(&stderr))
        .ok_or_else(|| {
            "Pas de token récupéré. Utilise plutôt l'import du fichier token.".to_string()
        })?;
    auth::set_token(&token)
}

// ---------- Sessions ----------

#[tauri::command]
fn session_create(
    mgr: tauri::State<'_, SessionManager>,
    title: Option<String>,
    cwd: Option<String>,
    model: Option<String>,
) -> String {
    mgr.create(title, cwd, model)
}

#[tauri::command]
fn session_list(mgr: tauri::State<'_, SessionManager>) -> Vec<SessionInfo> {
    mgr.list()
}

#[tauri::command]
fn session_restore(
    mgr: tauri::State<'_, SessionManager>,
    id: String,
    title: Option<String>,
    started: bool,
    cwd: Option<String>,
    model: Option<String>,
) {
    mgr.restore(id, title, started, cwd, model);
}

#[tauri::command]
fn session_send(
    app: tauri::AppHandle,
    mgr: tauri::State<'_, SessionManager>,
    id: String,
    text: String,
) -> Result<(), String> {
    let token = auth::get_token().ok_or_else(|| "Non connecté (aucun token).".to_string())?;
    let handles = mgr
        .begin_turn(&id)
        .ok_or_else(|| "Session inconnue.".to_string())?;

    let cfg = TurnConfig {
        id: id.clone(),
        prompt: text,
        resume: handles.started,
        cwd: handles.cwd,
        model: handles.model,
        token,
    };
    ClaudeCodeProvider.start_turn(app, cfg, handles.running);
    Ok(())
}

#[tauri::command]
async fn session_stop(
    mgr: tauri::State<'_, SessionManager>,
    id: String,
) -> Result<(), String> {
    if let Some(running) = mgr.running_handle(&id) {
        let mut slot = running.lock().await;
        if let Some(child) = slot.as_mut() {
            let _ = child.start_kill();
        }
    }
    Ok(())
}

#[tauri::command]
async fn session_close(
    mgr: tauri::State<'_, SessionManager>,
    id: String,
) -> Result<(), String> {
    if let Some(running) = mgr.remove(&id) {
        let mut slot = running.lock().await;
        if let Some(child) = slot.as_mut() {
            let _ = child.start_kill();
        }
    }
    Ok(())
}

// ---------- Usage ----------

#[tauri::command]
fn usage_get(store: tauri::State<'_, UsageStore>) -> UsageSnapshot {
    usage::snapshot(&store)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SessionManager::default())
        .manage(UsageStore::load())
        .invoke_handler(tauri::generate_handler![
            auth_status,
            auth_set_token,
            auth_clear,
            auth_import_from_file,
            auth_login,
            session_create,
            session_list,
            session_restore,
            session_send,
            session_stop,
            session_close,
            usage_get,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
