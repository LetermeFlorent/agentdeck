// Lancement du process `claude` persistant + lecture du stdout (NDJSON → events).

use std::collections::HashMap;
use std::process::Stdio;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::claude_code::{claude_bin, emit};
use super::claude_stream::{handle_line, ToolAcc};
use crate::events::SessionEvent;
use crate::session::SessionProc;

/// Lance le process persistant et démarre la lecture du stdout.
pub(super) async fn spawn(
    app: &tauri::AppHandle,
    id: &str,
    cwd: &Option<String>,
    model: &Option<String>,
    effort: &Option<String>,
    token: &Option<String>,
    resume: bool,
) -> Result<SessionProc, String> {
    let mut cmd = Command::new(claude_bin());
    cmd.arg("-p")
        .arg("--input-format")
        .arg("stream-json")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--include-partial-messages")
        .arg("--permission-mode")
        .arg("bypassPermissions");

    // Persistance de la conversation : on (ré)utilise l'UUID agentdeck comme session Claude.
    // 1ʳᵉ fois → on crée la session ; ensuite → on la reprend (mémoire conservée).
    if resume {
        cmd.arg("--resume").arg(id);
    } else {
        cmd.arg("--session-id").arg(id);
    }

    if let Some(m) = model {
        if !m.is_empty() {
            cmd.arg("--model").arg(m);
        }
    }
    // Haiku ne supporte pas l'effort → on n'envoie pas le flag.
    if model.as_deref() != Some("haiku") {
        if let Some(e) = effort {
            if !e.is_empty() {
                // "ultracode" (libellé Opus) n'est pas un --effort valide → mappé sur xhigh.
                let level = if e == "ultracode" { "xhigh" } else { e.as_str() };
                cmd.arg("--effort").arg(level);
            }
        }
    }
    if let Some(dir) = cwd {
        if !dir.is_empty() {
            cmd.current_dir(dir);
        }
    }

    // Token du coffre s'il existe ; sinon `claude` utilise ses propres credentials (connexion native).
    if let Some(t) = token {
        if !t.is_empty() {
            cmd.env("CLAUDE_CODE_OAUTH_TOKEN", t);
        }
    }
    cmd.env_remove("ANTHROPIC_API_KEY");
    cmd.env_remove("ANTHROPIC_AUTH_TOKEN");

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "Binaire `claude` introuvable. Installe Claude Code.".to_string()
        } else {
            format!("Échec du lancement de claude : {e}")
        }
    })?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "stdin indisponible".to_string())?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Lecture stderr (best-effort).
    if let Some(err) = stderr {
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(_)) = lines.next_line().await {}
        });
    }

    // Lecture stdout : NDJSON → events. Se termine quand le process meurt.
    let app2 = app.clone();
    let id2 = id.to_string();
    tauri::async_runtime::spawn(async move {
        if let Some(out) = stdout {
            let mut lines = BufReader::new(out).lines();
            let mut blocks: HashMap<u64, ToolAcc> = HashMap::new();
            let mut streamed = false; // un texte a-t-il été streamé pour le tour courant ?
            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<Value>(line) {
                    handle_line(&app2, &id2, &v, &mut blocks, &mut streamed);
                }
            }
        }
        emit(&app2, &id2, SessionEvent::Exited { code: None });
    });

    emit(app, id, SessionEvent::Started);

    Ok(SessionProc {
        child,
        stdin,
        model: model.clone(),
        effort: effort.clone(),
    })
}
