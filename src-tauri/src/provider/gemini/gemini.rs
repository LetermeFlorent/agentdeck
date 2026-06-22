// Adapter Gemini CLI : un process `gemini` par message (« one-shot »), sortie NDJSON
// (`--output-format stream-json`) mappée vers SessionEvent. Pas de stdin persistant.
//
//   gemini -p "<texte>" -o stream-json --skip-trust --yolo [-m <model>]
//          (--session-id <uuid> | -r <uuid>)
//
// Auth : gérée par le CLI lui-même (OAuth Google / clé API Gemini / Vertex), selon
// `~/.gemini/settings.json`. agentdeck ne stocke pas le secret (voir auth::is_connected).

use std::process::Stdio;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};

use super::gemini_stream::{handle_line, GemState};
use crate::provider::claude_code::{emit, write_images, ImageInput};
use crate::provider::common::{gemini_command, is_rate_limited, RATE_LIMIT_PREFIX};
use crate::events::SessionEvent;
use crate::session::{SessionProc, SharedProc};

/// Envoie un message à une session Gemini (one-shot). `proc` sert de marqueur « session démarrée »
/// (slot `Some` ⇒ on reprend avec `-r`, sinon on crée avec `--session-id`).
pub async fn send(
    app: tauri::AppHandle,
    id: String,
    proc: SharedProc,
    cwd: Option<String>,
    model: Option<String>,
    text: String,
    images: Vec<ImageInput>,
) {
    let mut guard = proc.lock().await;
    let resume = guard.is_some();
    // Tue un éventuel process résiduel (one-shot précédent encore en vol).
    if let Some(old) = guard.take() {
        let mut c = old.child;
        let _ = c.start_kill();
    }

    // Fichiers joints → chemins ajoutés au texte (Gemini les lit avec ses outils de lecture).
    let mut full = text;
    let paths = write_images(&images);
    if !paths.is_empty() {
        full.push_str("\n\n[Fichiers joints par l'utilisateur — lis-les :]");
        for p in &paths {
            full.push('\n');
            full.push_str(p);
        }
    }

    let mut cmd = gemini_command();
    cmd.arg("-p")
        .arg(&full)
        .arg("-o")
        .arg("stream-json")
        .arg("--skip-trust")
        .arg("--yolo");
    if let Some(m) = &model {
        if !m.is_empty() {
            cmd.arg("-m").arg(m);
        }
    }
    // Continuité : `-r <uuid>` pour reprendre, `--session-id <uuid>` pour créer.
    // ⚠️ `-r` par UUID non confirmé par la doc (qui parle d'index/latest) → à vérifier sur capture.
    if resume {
        cmd.arg("-r").arg(&id);
    } else {
        cmd.arg("--session-id").arg(&id);
    }
    if let Some(dir) = &cwd {
        if !dir.is_empty() {
            cmd.current_dir(dir);
        }
    }
    // Clé API Gemini stockée dans notre coffre (méthode « coller une clé ») → injectée en env.
    // Si absente, le CLI utilise sa propre auth (OAuth/Vertex de `~/.gemini`).
    if let Some(key) = crate::auth::get_token(crate::provider::Provider::Gemini) {
        if !key.is_empty() {
            cmd.env("GEMINI_API_KEY", key);
        }
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let msg = if e.kind() == std::io::ErrorKind::NotFound {
                "Binaire `gemini` introuvable. Installe Gemini CLI (npm i -g @google/gemini-cli).".to_string()
            } else {
                format!("Échec du lancement de gemini : {e}")
            };
            emit(&app, &id, SessionEvent::Error { message: msg });
            emit(&app, &id, SessionEvent::Exited { code: None });
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    // Marqueur de session démarrée (slot conservé pour décider resume au prochain envoi).
    *guard = Some(SessionProc {
        provider: crate::provider::Provider::Gemini,
        child,
        stdin: None,
        ext_session: None,
        model: model.clone(),
        effort: None,
        perm_mode: None,
        allowed: None,
        disallowed: None,
    });
    drop(guard);

    // stderr : on conserve la fin pour diagnostiquer un crash / détecter un rate-limit.
    let stderr_task = stderr.map(|err| {
        tauri::async_runtime::spawn(async move {
            let mut tail: Vec<String> = Vec::new();
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if tail.len() == 40 {
                    tail.remove(0);
                }
                tail.push(line.to_string());
            }
            tail.join("\n")
        })
    });

    // stdout : NDJSON → events.
    let mut rate_limited = false;
    let mut st = GemState::default();
    if let Some(out) = stdout {
        let mut lines = BufReader::new(out).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if handle_line(&app, &id, &v, &mut st) {
                    rate_limited = true;
                }
            }
        }
    }

    // Process mort : on récupère stderr. Erreur/rate-limit non déjà signalé → on le remonte.
    let tail = match stderr_task {
        Some(t) => t.await.unwrap_or_default(),
        None => String::new(),
    };
    if !rate_limited && !tail.is_empty() && is_rate_limited(&tail, None, None) {
        emit(&app, &id, SessionEvent::Error { message: format!("{RATE_LIMIT_PREFIX}{tail}") });
    } else if !tail.is_empty() && !st.tour_done() {
        // Pas de result émis + stderr non vide = échec → on le montre.
        emit(&app, &id, SessionEvent::Error { message: tail });
    }
    emit(&app, &id, SessionEvent::Exited { code: None });
}
