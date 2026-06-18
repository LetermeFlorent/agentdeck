// Adapter Claude Code : pilote le binaire `claude` en sous-process.
//
// Modèle « 1 process par tour » (documenté et robuste) : chaque envoi lance
//   claude -p <prompt> --output-format stream-json --verbose --include-partial-messages
//          --session-id <uuid>  (1er tour)  |  --resume <uuid>  (tours suivants)
// On lit le NDJSON sur stdout, on le traduit en SessionEvent, émis sur session://{id}.

use std::process::Stdio;

use serde_json::Value;
use tauri::{Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::events::{channel, SessionEvent};
use crate::provider::{Provider, SharedChild, TurnConfig};
use crate::usage;

pub struct ClaudeCodeProvider;

impl Provider for ClaudeCodeProvider {
    fn start_turn(&self, app: tauri::AppHandle, cfg: TurnConfig, running: SharedChild) {
        tauri::async_runtime::spawn(async move {
            run_turn(app, cfg, running).await;
        });
    }
}

fn emit(app: &tauri::AppHandle, id: &str, ev: SessionEvent) {
    let _ = app.emit(&channel(id), ev);
}

async fn run_turn(app: tauri::AppHandle, cfg: TurnConfig, running: SharedChild) {
    let id = cfg.id.clone();

    let mut cmd = Command::new("claude");
    cmd.arg("-p")
        .arg(&cfg.prompt)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--include-partial-messages")
        .arg("--permission-mode")
        .arg("acceptEdits");

    if cfg.resume {
        cmd.arg("--resume").arg(&cfg.id);
    } else {
        cmd.arg("--session-id").arg(&cfg.id);
    }
    if let Some(model) = &cfg.model {
        if !model.is_empty() {
            cmd.arg("--model").arg(model);
        }
    }
    if let Some(cwd) = &cfg.cwd {
        if !cwd.is_empty() {
            cmd.current_dir(cwd);
        }
    }

    // Auth : forcer l'OAuth, neutraliser une éventuelle clé API qui aurait priorité silencieuse.
    cmd.env("CLAUDE_CODE_OAUTH_TOKEN", &cfg.token);
    cmd.env_remove("ANTHROPIC_API_KEY");
    cmd.env_remove("ANTHROPIC_AUTH_TOKEN");

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let msg = if e.kind() == std::io::ErrorKind::NotFound {
                "Binaire `claude` introuvable dans le PATH. Installe Claude Code CLI.".to_string()
            } else {
                format!("Échec du lancement de claude : {e}")
            };
            emit(&app, &id, SessionEvent::Error { message: msg });
            emit(&app, &id, SessionEvent::Exited { code: None });
            return;
        }
    };

    emit(&app, &id, SessionEvent::Started);

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Stocke le child pour permettre l'arrêt (session_stop).
    {
        let mut slot = running.lock().await;
        *slot = Some(child);
    }

    // Collecte stderr en tâche de fond.
    let stderr_task = tauri::async_runtime::spawn(async move {
        let mut buf = String::new();
        if let Some(err) = stderr {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                buf.push_str(&line);
                buf.push('\n');
            }
        }
        buf
    });

    // Lecture/parse du NDJSON sur stdout.
    if let Some(out) = stdout {
        let mut lines = BufReader::new(out).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let v: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue, // ligne non-JSON (log), on ignore
            };
            handle_line(&app, &id, &v);
        }
    }

    // Récupère le code de sortie.
    let code = {
        let mut slot = running.lock().await;
        if let Some(mut c) = slot.take() {
            match c.wait().await {
                Ok(status) => status.code(),
                Err(_) => None,
            }
        } else {
            None
        }
    };

    // Si sortie anormale, remonter stderr.
    let stderr_text = stderr_task.await.unwrap_or_default();
    if code.unwrap_or(0) != 0 && !stderr_text.trim().is_empty() {
        emit(
            &app,
            &id,
            SessionEvent::Error {
                message: stderr_text.trim().to_string(),
            },
        );
    }

    emit(&app, &id, SessionEvent::Exited { code });
}

/// Traduit une ligne NDJSON de Claude Code en SessionEvent(s).
fn handle_line(app: &tauri::AppHandle, id: &str, v: &Value) {
    match v.get("type").and_then(Value::as_str) {
        Some("stream_event") => {
            let ev = match v.get("event") {
                Some(e) => e,
                None => return,
            };
            match ev.get("type").and_then(Value::as_str) {
                Some("content_block_delta") => {
                    let delta = ev.get("delta");
                    let dtype = delta
                        .and_then(|d| d.get("type"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if dtype == "text_delta" {
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
        Some("user") => {
            // Résultats d'outils renvoyés à l'assistant.
            emit(app, id, SessionEvent::ToolResult { ok: true });
        }
        Some("result") => {
            let usage_obj = v.get("usage");
            let input = usage_obj
                .and_then(|u| u.get("input_tokens"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let output = usage_obj
                .and_then(|u| u.get("output_tokens"))
                .and_then(Value::as_u64)
                .unwrap_or(0);
            // Comptabilise pour les barres d'usage (fenêtres 5h / 7j).
            usage::record(app.state::<usage::UsageStore>().inner(), input, output);
            emit(
                app,
                id,
                SessionEvent::TurnDone {
                    input_tokens: input,
                    output_tokens: output,
                },
            );
        }
        _ => {}
    }
}
