// Adapter Claude Code : UN process `claude` persistant par session.
//
//   claude -p --input-format stream-json --output-format stream-json --verbose
//          --include-partial-messages --permission-mode bypassPermissions [--model --effort]
//
// On garde stdin ouvert et on y écrit chaque message utilisateur en stream-json — même
// pendant que Claude travaille (envoi « en cours de route » / steering, comme l'app interactive).
// Le stdout NDJSON est lu en continu et traduit en SessionEvent émis sur session://{id}.

use std::process::Stdio;

use serde_json::{json, Value};
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use crate::events::{channel, SessionEvent};
use crate::session::{SessionProc, SharedProc};
use crate::usage;

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

fn emit(app: &tauri::AppHandle, id: &str, ev: SessionEvent) {
    let _ = app.emit(&channel(id), ev);
}

/// Lance le process persistant et démarre la lecture du stdout.
async fn spawn(
    app: &tauri::AppHandle,
    id: &str,
    cwd: &Option<String>,
    model: &Option<String>,
    effort: &Option<String>,
    token: &str,
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

    if let Some(m) = model {
        if !m.is_empty() {
            cmd.arg("--model").arg(m);
        }
    }
    if let Some(e) = effort {
        if !e.is_empty() {
            // "ultracode" (libellé Opus) n'est pas un --effort valide → mappé sur xhigh.
            let level = if e == "ultracode" { "xhigh" } else { e.as_str() };
            cmd.arg("--effort").arg(level);
        }
    }
    if let Some(dir) = cwd {
        if !dir.is_empty() {
            cmd.current_dir(dir);
        }
    }

    cmd.env("CLAUDE_CODE_OAUTH_TOKEN", token);
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
            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<Value>(line) {
                    handle_line(&app2, &id2, &v);
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

/// Envoie un message à la session : (re)lance le process si besoin, puis écrit sur stdin.
/// Fonctionne même si Claude est déjà en train de répondre (le message est pris en cours de route).
pub async fn send(
    app: tauri::AppHandle,
    id: String,
    proc: SharedProc,
    cwd: Option<String>,
    model: Option<String>,
    effort: Option<String>,
    token: String,
    text: String,
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
        match spawn(&app, &id, &cwd, &model, &effort, &token).await {
            Ok(p) => *guard = Some(p),
            Err(e) => {
                emit(&app, &id, SessionEvent::Error { message: e });
                emit(&app, &id, SessionEvent::Exited { code: None });
                return;
            }
        }
    }

    if let Some(p) = guard.as_mut() {
        let msg = json!({
            "type": "user",
            "message": { "role": "user", "content": [{ "type": "text", "text": text }] }
        })
        .to_string();
        let write = async {
            p.stdin.write_all(msg.as_bytes()).await?;
            p.stdin.write_all(b"\n").await?;
            p.stdin.flush().await
        };
        if let Err(e) = write.await {
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

/// Traduit une ligne NDJSON en SessionEvent(s).
fn handle_line(app: &tauri::AppHandle, id: &str, v: &Value) {
    match v.get("type").and_then(Value::as_str) {
        Some("system") if v.get("subtype").and_then(Value::as_str) == Some("init") => {
            let cmds: Vec<String> = v
                .get("slash_commands")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|c| c.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            emit(app, id, SessionEvent::Init { slash_commands: cmds });
        }
        Some("stream_event") => {
            let ev = match v.get("event") {
                Some(e) => e,
                None => return,
            };
            match ev.get("type").and_then(Value::as_str) {
                Some("message_start") => emit(app, id, SessionEvent::AssistantStart),
                Some("message_delta") => {
                    if let Some(t) = ev
                        .get("usage")
                        .and_then(|u| u.get("output_tokens"))
                        .and_then(Value::as_u64)
                    {
                        emit(app, id, SessionEvent::Progress { output_tokens: t });
                    }
                }
                Some("content_block_delta") => {
                    let delta = ev.get("delta");
                    if delta.and_then(|d| d.get("type")).and_then(Value::as_str) == Some("text_delta")
                    {
                        if let Some(t) = delta.and_then(|d| d.get("text")).and_then(Value::as_str) {
                            emit(
                                app,
                                id,
                                SessionEvent::AssistantDelta {
                                    text: t.to_string(),
                                },
                            );
                        }
                    }
                }
                Some("content_block_start") => {
                    let cb = ev.get("content_block");
                    if cb.and_then(|c| c.get("type")).and_then(Value::as_str) == Some("tool_use") {
                        let name = cb
                            .and_then(|c| c.get("name"))
                            .and_then(Value::as_str)
                            .unwrap_or("tool")
                            .to_string();
                        emit(app, id, SessionEvent::ToolUse { name });
                    }
                }
                _ => {}
            }
        }
        Some("result") => {
            let usage_obj = v.get("usage");
            let tok = |k: &str| {
                usage_obj
                    .and_then(|u| u.get(k))
                    .and_then(Value::as_u64)
                    .unwrap_or(0)
            };
            let input = tok("input_tokens");
            let output = tok("output_tokens");
            let cache = tok("cache_creation_input_tokens") + tok("cache_read_input_tokens");
            // Contexte courant = prompt du dernier tour (entrée + cache), sans la sortie.
            let context = input + cache;
            let total = input + output + cache;
            let cost = v
                .get("total_cost_usd")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            // Fenêtre de contexte réelle du modèle (dynamique) : max des contextWindow
            // rapportés dans modelUsage (un tour peut toucher plusieurs modèles).
            let context_window = v
                .get("modelUsage")
                .and_then(Value::as_object)
                .map(|m| {
                    m.values()
                        .filter_map(|mu| mu.get("contextWindow").and_then(Value::as_u64))
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            usage::record(app.state::<usage::UsageStore>().inner(), total, cost);
            emit(
                app,
                id,
                SessionEvent::TurnDone {
                    input_tokens: input,
                    output_tokens: output,
                    total_tokens: total,
                    cost_usd: cost,
                    context_tokens: context,
                    context_window,
                },
            );
        }
        _ => {}
    }
}
