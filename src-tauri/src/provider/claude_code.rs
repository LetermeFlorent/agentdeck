// Adapter Claude Code : UN process `claude` persistant par session.
//
//   claude -p --input-format stream-json --output-format stream-json --verbose
//          --include-partial-messages --permission-mode bypassPermissions [--model --effort]
//
// On garde stdin ouvert et on y écrit chaque message utilisateur en stream-json — même
// pendant que Claude travaille (envoi « en cours de route » / steering, comme l'app interactive).
// Le lancement vit dans `claude_spawn`, le parsing du stdout dans `claude_stream`.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

/// Fichier joint par l'utilisateur (base64 + type MIME + nom d'origine). Tous types.
#[derive(Debug, Clone, Deserialize)]
pub struct ImageInput {
    pub media_type: String,
    /// base64 brut (sans préfixe `data:…`).
    pub data: String,
    /// Nom d'origine (sert à garder l'extension pour que Claude sache le type).
    #[serde(default)]
    pub name: Option<String>,
}

/// Extension à partir du nom d'origine, sinon du type MIME, sinon "bin".
fn pick_ext(file: &ImageInput) -> String {
    if let Some(n) = &file.name {
        if let Some(e) = std::path::Path::new(n).extension().and_then(|e| e.to_str()) {
            if !e.is_empty() {
                return e.to_lowercase();
            }
        }
    }
    match file.media_type.as_str() {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        _ => "bin",
    }
    .to_string()
}

/// Écrit les fichiers joints en temporaires et renvoie leurs chemins absolus.
/// Claude Code ignore les blocs base64 en stream-json → on passe par des fichiers
/// que le modèle lit avec l'outil Read (vérifié).
/// Dossier temporaire des fichiers joints, créé une seule fois (au lieu de `create_dir_all`
/// à chaque envoi). `None` tant qu'aucune création n'a réussi.
fn files_dir() -> Option<&'static Path> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::temp_dir().join("agentdeck-files");
        std::fs::create_dir_all(&dir).ok().map(|_| dir)
    })
    .as_deref()
}

fn write_images(images: &[ImageInput]) -> Vec<String> {
    if images.is_empty() {
        return Vec::new();
    }
    let dir = match files_dir() {
        Some(d) => d,
        None => return Vec::new(),
    };
    let mut paths = Vec::new();
    for file in images {
        let bytes = match base64::engine::general_purpose::STANDARD.decode(file.data.trim()) {
            Ok(b) => b,
            Err(_) => continue,
        };
        let p = dir.join(format!("{}.{}", uuid::Uuid::new_v4(), pick_ext(file)));
        if std::fs::write(&p, &bytes).is_ok() {
            paths.push(p.display().to_string());
        }
    }
    paths
}

/// Vrai si Claude Code a déjà une session persistée pour cet id (fichier
/// `~/.claude/projects/<projet>/<id>.jsonl`). Décide `--resume` vs `--session-id` de façon
/// déterministe (pas de course, pas de --resume sur une session inexistante).
fn session_exists(id: &str) -> bool {
    let mut dir = match dirs::home_dir() {
        Some(d) => d,
        None => return false,
    };
    dir.push(".claude");
    dir.push("projects");
    let file = format!("{id}.jsonl");
    match std::fs::read_dir(&dir) {
        Ok(rd) => rd.flatten().any(|e| {
            let p = e.path();
            p.is_dir() && p.join(&file).exists()
        }),
        Err(_) => false,
    }
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
    text: String,
    images: Vec<ImageInput>,
    hermes: bool,
    perm_mode: Option<String>,
    allowed: Option<String>,
    disallowed: Option<String>,
) {
    let mut guard = proc.lock().await;
    // Reprend la conversation si Claude a déjà persisté cette session ; sinon la crée.
    let resume = session_exists(&id);

    // (Re)lance si pas de process, ou si le modèle/effort/permissions ont changé.
    let need_spawn = match &*guard {
        None => true,
        Some(p) => {
            p.model != model
                || p.effort != effort
                || p.perm_mode != perm_mode
                || p.allowed != allowed
                || p.disallowed != disallowed
        }
    };
    if need_spawn {
        if let Some(old) = guard.take() {
            let mut c = old.child;
            let _ = c.start_kill();
        }
        match spawn(&app, &id, &cwd, &model, &effort, &token, resume, hermes, &perm_mode, &allowed, &disallowed).await {
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
        full_text.push_str("\n\n[Fichiers joints par l'utilisateur — lis-les avec l'outil Read :]");
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
        match spawn(&app, &id, &cwd, &model, &effort, &token, retry_resume, hermes, &perm_mode, &allowed, &disallowed).await {
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
