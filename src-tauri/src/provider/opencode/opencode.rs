// Adapter opencode : un process `opencode run` par message (« one-shot »), sortie
// `--format json` (events du bus) mappée vers SessionEvent. Pas de stdin persistant.
//
//   opencode run --format json --dangerously-skip-permissions [-m provider/model]
//                [--variant <effort>] [-s <session natif>] "<texte>"
//
// Continuité : opencode crée sa propre session (id `ses_…`) au 1er message ; on la mémorise
// (`ext_session`) pour reprendre avec `-s` aux messages suivants.

use std::path::PathBuf;
use std::process::Stdio;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};

use super::opencode_stream::{emit_turn_done, handle_line, OcState};
use crate::provider::claude_code::{emit, write_images, ImageInput};
use crate::provider::common::{is_rate_limited, opencode_command, RATE_LIMIT_PREFIX};
use crate::events::SessionEvent;
use crate::session::{SessionProc, SharedProc};

fn oc_sessions_path() -> Option<PathBuf> {
    let mut p = dirs::data_dir()?;
    p.push("agentdeck");
    let _ = std::fs::create_dir_all(&p);
    p.push("oc_sessions.json");
    Some(p)
}

/// Lit le session ID opencode (`ses_…`) associé à un UUID agentdeck depuis le disque.
fn load_oc_session(agentdeck_id: &str) -> Option<String> {
    let p = oc_sessions_path()?;
    let raw = std::fs::read_to_string(p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get(agentdeck_id)?.as_str().map(String::from)
}

/// Persiste le mapping agentdeck UUID → opencode `ses_…` sur disque.
fn save_oc_session(agentdeck_id: &str, oc_session: &str) {
    let Some(p) = oc_sessions_path() else { return };
    let mut map: serde_json::Map<String, serde_json::Value> = std::fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    map.insert(agentdeck_id.to_string(), serde_json::Value::String(oc_session.to_string()));
    if let Ok(s) = serde_json::to_string(&serde_json::Value::Object(map)) {
        let _ = std::fs::write(p, s);
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn send(
    app: tauri::AppHandle,
    id: String,
    proc: SharedProc,
    cwd: Option<String>,
    model: Option<String>,
    effort: Option<String>,
    text: String,
    images: Vec<ImageInput>,
) {
    // Id de session natif d'opencode : d'abord en mémoire (même run), sinon depuis le fichier persisté.
    let prev_session = {
        let guard = proc.lock().await;
        guard.as_ref().and_then(|p| p.ext_session.clone())
    }.or_else(|| load_oc_session(&id));

    let mut full = text;
    let paths = write_images(&images);
    if !paths.is_empty() {
        full.push_str("\n\n[Fichiers joints par l'utilisateur — lis-les :]");
        for p in &paths {
            full.push('\n');
            full.push_str(p);
        }
    }

    let mut cmd = opencode_command();
    cmd.arg("run").arg("--format").arg("json").arg("--dangerously-skip-permissions");
    if let Some(m) = &model {
        if !m.is_empty() {
            cmd.arg("-m").arg(m);
        }
    }
    // Effort → `--variant` (high/max/minimal selon le provider sous-jacent).
    if let Some(e) = &effort {
        if !e.is_empty() {
            let variant = if e == "ultracode" { "max" } else { e.as_str() };
            cmd.arg("--variant").arg(variant);
        }
    }
    if let Some(sid) = &prev_session {
        cmd.arg("-s").arg(sid);
    }
    cmd.arg(&full);
    if let Some(dir) = &cwd {
        if !dir.is_empty() {
            cmd.current_dir(dir);
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
                "Binaire `opencode` introuvable. Installe opencode (npm i -g opencode-ai).".to_string()
            } else {
                format!("Échec du lancement d'opencode : {e}")
            };
            emit(&app, &id, SessionEvent::Error { message: msg });
            emit(&app, &id, SessionEvent::Exited { code: None });
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    // Marqueur (slot conservé) — ext_session sera mis à jour à la fin avec l'id capturé.
    {
        let mut guard = proc.lock().await;
        if let Some(old) = guard.take() {
            let mut c = old.child;
            let _ = c.start_kill();
        }
        *guard = Some(SessionProc {
            provider: crate::provider::Provider::Opencode,
            child,
            stdin: None,
            ext_session: prev_session.clone(),
            model: model.clone(),
            effort: effort.clone(),
            perm_mode: None,
            allowed: None,
            disallowed: None,
        });
    }

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

    let mut rate_limited = false;
    let mut st = OcState::default();
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

    // Fin de tour : opencode n'a pas d'event `session.idle` → on émet le TurnDone à l'EOF.
    emit_turn_done(&app, &id, &st, model.as_deref().unwrap_or(""));

    // Mémorise l'id de session natif capturé (en mémoire + sur disque pour survie au redémarrage).
    if let Some(sid) = st.session_id.clone() {
        save_oc_session(&id, &sid);
        let mut guard = proc.lock().await;
        if let Some(p) = guard.as_mut() {
            p.ext_session = Some(sid);
        }
    }

    let tail = match stderr_task {
        Some(t) => t.await.unwrap_or_default(),
        None => String::new(),
    };
    if !rate_limited && !tail.is_empty() && is_rate_limited(&tail, None, None) {
        emit(&app, &id, SessionEvent::Error { message: format!("{RATE_LIMIT_PREFIX}{tail}") });
    } else if !tail.is_empty() && !st.tour_done() {
        emit(&app, &id, SessionEvent::Error { message: tail });
    }
    emit(&app, &id, SessionEvent::Exited { code: None });
}
