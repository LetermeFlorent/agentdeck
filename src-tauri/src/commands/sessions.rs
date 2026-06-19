// Commandes de gestion des sessions (création, envoi, arrêt…).

use crate::auth;
use crate::provider;
use crate::session::{SessionInfo, SessionManager};

#[tauri::command]
pub fn session_create(
    mgr: tauri::State<'_, SessionManager>,
    title: Option<String>,
    cwd: Option<String>,
    model: Option<String>,
) -> String {
    mgr.create(title, cwd, model)
}

#[tauri::command]
pub fn session_list(mgr: tauri::State<'_, SessionManager>) -> Vec<SessionInfo> {
    mgr.list()
}

#[tauri::command]
pub fn session_restore(
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
pub fn session_send(
    app: tauri::AppHandle,
    mgr: tauri::State<'_, SessionManager>,
    id: String,
    text: String,
    model: Option<String>,
    effort: Option<String>,
    images: Option<Vec<provider::claude_code::ImageInput>>,
    hermes: Option<bool>,
    perm_mode: Option<String>,
    allowed: Option<String>,
    disallowed: Option<String>,
) -> Result<(), String> {
    // Token du coffre si présent ; sinon on s'appuie sur la connexion native de Claude Code.
    let token = auth::get_token();
    if token.is_none() && !auth::claude_logged_in() {
        return Err("Non connecté.".to_string());
    }
    let (proc, cwd) = mgr
        .send_ctx(&id)
        .ok_or_else(|| "Session inconnue.".to_string())?;
    // Écrit le message sur le stdin du process persistant (pris en cours de route si Claude bosse).
    tauri::async_runtime::spawn(provider::claude_code::send(
        app,
        id,
        proc,
        cwd,
        model,
        effort,
        token,
        text,
        images.unwrap_or_default(),
        hermes.unwrap_or(false),
        perm_mode,
        allowed,
        disallowed,
    ));
    Ok(())
}

/// Change le dossier de travail (cwd) d'un chat : le process est relancé dedans au prochain envoi.
#[tauri::command]
pub async fn session_set_cwd(
    mgr: tauri::State<'_, SessionManager>,
    id: String,
    cwd: Option<String>,
) -> Result<(), String> {
    if let Some(proc) = mgr.set_cwd(&id, cwd) {
        let mut slot = proc.lock().await;
        if let Some(p) = slot.take() {
            let mut c = p.child;
            let _ = c.start_kill();
        }
    }
    Ok(())
}

/// Nom d'utilisateur du PC (pour l'accueil « Bonjour … » au démarrage).
#[tauri::command]
pub fn os_username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_default()
}

#[tauri::command]
pub async fn session_stop(mgr: tauri::State<'_, SessionManager>, id: String) -> Result<(), String> {
    if let Some(proc) = mgr.proc_handle(&id) {
        let mut slot = proc.lock().await;
        if let Some(p) = slot.take() {
            let mut c = p.child;
            let _ = c.start_kill();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn session_close(mgr: tauri::State<'_, SessionManager>, id: String) -> Result<(), String> {
    if let Some(proc) = mgr.remove(&id) {
        let mut slot = proc.lock().await;
        if let Some(p) = slot.take() {
            let mut c = p.child;
            let _ = c.start_kill();
        }
    }
    Ok(())
}
