// Adapter Claude Code : UN process `claude` persistant par session.
//
//   claude -p --input-format stream-json --output-format stream-json --verbose
//          --include-partial-messages --permission-mode bypassPermissions [--model --effort]
//
// On garde stdin ouvert et on y écrit chaque message utilisateur en stream-json — même
// pendant que Claude travaille (envoi « en cours de route » / steering, comme l'app interactive).
// Le lancement vit dans `claude_spawn`, le parsing du stdout dans `claude_stream`.

use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;

use super::claude_spawn::spawn;
use crate::events::{channel, SessionEvent};
use crate::session::SharedProc;

/// Résout le binaire `claude` : chemin de l'installeur natif (~/.local/bin) s'il existe
/// (PATH pas forcément rafraîchi après installation), sinon "claude" depuis le PATH.
pub fn claude_bin() -> std::ffi::OsString {
    if let Some(home) = dirs::home_dir() {
        for name in ["claude.exe", "claude"] {
            let p = home.join(".local").join("bin").join(name);
            if p.exists() {
                return p.into_os_string();
            }
        }
    }
    std::ffi::OsString::from("claude")
}

/// Émet un SessionEvent sur le canal de la session (partagé avec spawn/stream).
pub(super) fn emit(app: &tauri::AppHandle, id: &str, ev: SessionEvent) {
    let _ = app.emit(&channel(id), ev);
}

/// Image jointe par l'utilisateur (base64 + type MIME).
#[derive(Debug, Clone, Deserialize)]
pub struct ImageInput {
    pub media_type: String,
    /// base64 brut (sans préfixe `data:…`).
    pub data: String,
}

/// Écrit les images en fichiers temporaires et renvoie leurs chemins absolus.
/// Claude Code ignore les blocs image base64 en stream-json → on passe par des
/// fichiers que le modèle lit avec l'outil Read (vérifié).
fn write_images(images: &[ImageInput]) -> Vec<String> {
    if images.is_empty() {
        return Vec::new();
    }
    let dir = std::env::temp_dir().join("agentdeck-img");
    let _ = std::fs::create_dir_all(&dir);
    let mut paths = Vec::new();
    for img in images {
        let bytes = match base64::engine::general_purpose::STANDARD.decode(img.data.trim()) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let ext = match img.media_type.as_str() {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "png",
        };
        let p = dir.join(format!("{}.{}", uuid::Uuid::new_v4(), ext));
        if std::fs::write(&p, &bytes).is_ok() {
            paths.push(p.display().to_string());
        }
    }
    paths
}

/// Écrit une ligne (message stream-json) sur le stdin du process.
async fn write_line(stdin: &mut tokio::process::ChildStdin, msg: &str) -> std::io::Result<()> {
    stdin.write_all(msg.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await
}

/// Envoie un message à la session : (re)lance le process si besoin, puis écrit sur stdin.
/// Fonctionne même si Claude est déjà en train de répondre (le message est pris en cours de route).
#[allow(clippy::too_many_arguments)]
pub async fn send(
    app: tauri::AppHandle,
    id: String,
    proc: SharedProc,
    cwd: Option<String>,
    model: Option<String>,
    effort: Option<String>,
    token: Option<String>,
    resume: bool,
    text: String,
    images: Vec<ImageInput>,
) {
    let mut guard = proc.lock().await;

    // (Re)lance si pas de process, ou si le modèle/effort a changé.
    let need_spawn = match &*guard {
        None => true,
        Some(p) => p.model != model || p.effort != effort,
    };
    if need_spawn {
        if let Some(old) = guard.take() {
            let mut c = old.child;
            let _ = c.start_kill();
        }
        match spawn(&app, &id, &cwd, &model, &effort, &token, resume).await {
            Ok(p) => *guard = Some(p),
            Err(e) => {
                emit(&app, &id, SessionEvent::Error { message: e });
                emit(&app, &id, SessionEvent::Exited { code: None });
                return;
            }
        }
    }

    // Images → fichiers temp + chemins ajoutés au texte (Claude les lit via Read).
    let mut full_text = text;
    let paths = write_images(&images);
    if !paths.is_empty() {
        full_text.push_str("\n\n[Images jointes par l'utilisateur — lis-les avec l'outil Read :]");
        for path in &paths {
            full_text.push('\n');
            full_text.push_str(path);
        }
    }
    let msg = json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": full_text }] }
    })
    .to_string();

    // 1ʳᵉ écriture. Si le pipe est fermé (process mort, ex. `--resume` sur une session inexistante
    // pour un vieux chat), on relance en INVERSANT la stratégie session-id/resume et on réessaie.
    let mut ok = false;
    if let Some(p) = guard.as_mut() {
        ok = write_line(&mut p.stdin, &msg).await.is_ok();
    }
    if !ok {
        if let Some(old) = guard.take() {
            let mut c = old.child;
            let _ = c.start_kill();
        }
        let retry_resume = if need_spawn { !resume } else { true };
        match spawn(&app, &id, &cwd, &model, &effort, &token, retry_resume).await {
            Ok(p) => *guard = Some(p),
            Err(e) => {
                emit(&app, &id, SessionEvent::Error { message: e });
                emit(&app, &id, SessionEvent::Exited { code: None });
                return;
            }
        }
        if let Some(p) = guard.as_mut() {
            if let Err(e) = write_line(&mut p.stdin, &msg).await {
                emit(
                    &app,
                    &id,
                    SessionEvent::Error {
                        message: format!("Écriture stdin : {e}"),
                    },
                );
            }
        }
    }
}
