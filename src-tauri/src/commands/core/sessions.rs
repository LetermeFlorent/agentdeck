// Commandes de gestion des sessions (création, envoi, arrêt…).

use crate::auth;
use crate::provider::{self, Provider};
use crate::session::{SessionInfo, SessionManager};

#[tauri::command]
pub fn session_create(
    mgr: tauri::State<'_, SessionManager>,
    title: Option<String>,
    cwd: Option<String>,
    model: Option<String>,
    provider: Option<String>,
) -> String {
    let provider = Provider::from_str(provider.as_deref().unwrap_or("claude_code"));
    mgr.create(title, cwd, model, provider)
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
    provider: Option<String>,
) {
    let provider = Provider::from_str(provider.as_deref().unwrap_or("claude_code"));
    mgr.restore(id, title, started, cwd, model, provider);
}

#[tauri::command]
#[allow(clippy::too_many_arguments)] // commande Tauri : paramètres mappés depuis le front
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
    provider: Option<String>,
) -> Result<(), String> {
    // L'IA peut changer par chat (choix manuel ou auto cross-IA) → on met à jour la session.
    if let Some(p) = provider.as_deref() {
        mgr.set_provider(&id, Provider::from_str(p));
    }
    let (prov, proc, cwd) = mgr
        .send_ctx(&id)
        .ok_or_else(|| "Session inconnue.".to_string())?;
    // Connexion agentdeck indépendante : il faut être connecté pour ce provider.
    if !auth::is_connected(prov) {
        return Err("Non connecté.".to_string());
    }
    // Dispatch vers le bon adapter (Claude = process persistant ; opencode/Gemini = one-shot).
    tauri::async_runtime::spawn(provider::send(
        app,
        prov,
        id,
        proc,
        cwd,
        model,
        effort,
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
