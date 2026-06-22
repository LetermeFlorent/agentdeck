// Parsing du flux NDJSON de Claude Code → SessionEvent. Garde l'état des blocs tool_use
// en cours pour assembler leur entrée JSON (streamée par fragments).

use std::collections::HashMap;

use serde_json::Value;
use tauri::Manager;

use super::claude_code::emit;
use crate::events::SessionEvent;
use crate::usage;

/// Accumulateur d'un bloc tool_use en cours de streaming (entrée JSON partielle).
pub(super) struct ToolAcc {
    name: String,
    buf: String,
    id: String,
}

/// Résume l'entrée d'un outil en une ligne (commande, fichier, motif…) pour l'affichage terminal.
fn summarize_tool_input(buf: &str) -> String {
    let v: Value = match serde_json::from_str(buf) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    for key in [
        "command",
        "file_path",
        "path",
        "pattern",
        "query",
        "url",
        "prompt",
        "description",
    ] {
        if let Some(s) = v.get(key).and_then(Value::as_str) {
            return s.chars().take(220).collect();
        }
    }
    v.to_string().chars().take(180).collect()
}

/// Traduit une ligne NDJSON en SessionEvent(s).
pub(super) fn handle_line(
    app: &tauri::AppHandle,
    id: &str,
    v: &Value,
    blocks: &mut HashMap<u64, ToolAcc>,
    streamed: &mut bool,
) {
    match v.get("type").and_then(Value::as_str) {
        Some("system") if v.get("subtype").and_then(Value::as_str) == Some("init") => {
            let cmds: Vec<String> = v
                .get("slash_commands")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let tools: Vec<String> = v
                .get("tools")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect())
                .unwrap_or_default();
            emit(app, id, SessionEvent::Init { slash_commands: cmds, tools });
        }
        Some("stream_event") => {
            let ev = match v.get("event") {
                Some(e) => e,
                None => return,
            };
            let index = ev.get("index").and_then(Value::as_u64).unwrap_or(0);
            match ev.get("type").and_then(Value::as_str) {
                Some("message_start") => {
                    blocks.clear(); // les index de blocs sont réinitialisés à chaque message
                    *streamed = false; // nouveau tour : aucun texte streamé pour l'instant
                    emit(app, id, SessionEvent::AssistantStart);
                }
                Some("message_delta") => {
                    if let Some(t) = ev
                        .get("usage")
                        .and_then(|u| u.get("output_tokens"))
                        .and_then(Value::as_u64)
                    {
                        emit(app, id, SessionEvent::Progress { output_tokens: t });
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
                        let tid = cb
                            .and_then(|c| c.get("id"))
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        // On attend la fin du bloc pour émettre (entrée assemblée).
                        blocks.insert(index, ToolAcc { name, buf: String::new(), id: tid });
                    }
                }
                Some("content_block_delta") => {
                    let delta = ev.get("delta");
                    match delta.and_then(|d| d.get("type")).and_then(Value::as_str) {
                        Some("text_delta") => {
                            if let Some(t) = delta.and_then(|d| d.get("text")).and_then(Value::as_str)
                            {
                                *streamed = true;
                                emit(app, id, SessionEvent::AssistantDelta { text: t.to_string() });
                            }
                        }
                        Some("thinking_delta") => {
                            if let Some(t) =
                                delta.and_then(|d| d.get("thinking")).and_then(Value::as_str)
                            {
                                emit(app, id, SessionEvent::Thinking { text: t.to_string() });
                            }
                        }
                        Some("input_json_delta") => {
                            if let (Some(acc), Some(p)) = (
                                blocks.get_mut(&index),
                                delta.and_then(|d| d.get("partial_json")).and_then(Value::as_str),
                            ) {
                                acc.buf.push_str(p);
                            }
                        }
                        _ => {}
                    }
                }
                Some("content_block_stop") => {
                    if let Some(acc) = blocks.remove(&index) {
                        let input = summarize_tool_input(&acc.buf);
                        emit(app, id, SessionEvent::ToolUse { name: acc.name, input, id: acc.id });
                    }
                }
                _ => {}
            }
        }
        // Sous-agents (Task) : cycle de vie exposé par le CLI en events system.
        Some("system") if v.get("subtype").and_then(Value::as_str) == Some("task_started") => {
            let g = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_string();
            emit(app, id, SessionEvent::TaskStarted {
                task_id: g("task_id"),
                description: g("description"),
                subagent_type: g("subagent_type"),
                prompt: g("prompt"),
            });
        }
        Some("system") if v.get("subtype").and_then(Value::as_str) == Some("task_progress") => {
            let usage = v.get("usage");
            emit(app, id, SessionEvent::TaskProgress {
                task_id: v.get("task_id").and_then(Value::as_str).unwrap_or("").to_string(),
                action: v.get("description").and_then(Value::as_str).unwrap_or("").to_string(),
                last_tool: v.get("last_tool_name").and_then(Value::as_str).unwrap_or("").to_string(),
                tokens: usage.and_then(|u| u.get("total_tokens")).and_then(Value::as_u64).unwrap_or(0),
                duration_ms: usage.and_then(|u| u.get("duration_ms")).and_then(Value::as_u64).unwrap_or(0),
            });
        }
        Some("system")
            if matches!(
                v.get("subtype").and_then(Value::as_str),
                Some("task_updated") | Some("task_notification")
            ) =>
        {
            // statut dans patch.status (task_updated) ou status (task_notification).
            let status = v
                .get("patch")
                .and_then(|p| p.get("status"))
                .and_then(Value::as_str)
                .or_else(|| v.get("status").and_then(Value::as_str))
                .unwrap_or("")
                .to_string();
            if !status.is_empty() && status != "in_progress" && status != "running" {
                emit(app, id, SessionEvent::TaskEnded {
                    task_id: v.get("task_id").and_then(Value::as_str).unwrap_or("").to_string(),
                    status,
                });
            }
        }
        // Résultat d'outil (tool_result dans un message user) → l'outil a fini.
        Some("user") => {
            if let Some(arr) = v.get("message").and_then(|m| m.get("content")).and_then(Value::as_array) {
                for b in arr {
                    if b.get("type").and_then(Value::as_str) == Some("tool_result") {
                        if let Some(tid) = b.get("tool_use_id").and_then(Value::as_str) {
                            emit(app, id, SessionEvent::ToolDone { id: tid.to_string() });
                        }
                    }
                }
            }
        }
        // Message assistant complet (non streamé via stream_event). Cas typique :
        // sortie d'une commande slash built-in (/usage, /context, /cost…) qui ne
        // produit pas de deltas. Si du texte a déjà été streamé pour ce tour, on
        // ne ré-émet pas (le message complet est alors un doublon du flux).
        Some("assistant") if !*streamed => {
            {
                let text: String = v
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                            .filter_map(|b| b.get("text").and_then(Value::as_str))
                            .collect::<Vec<_>>()
                            .join("")
                    })
                    .unwrap_or_default();
                if !text.is_empty() {
                    *streamed = true;
                    emit(app, id, SessionEvent::AssistantStart);
                    emit(app, id, SessionEvent::AssistantDelta { text });
                }
            }
        }
        Some("result") => {
            let usage_obj = v.get("usage");
            let tok = |k: &str| usage_obj.and_then(|u| u.get(k)).and_then(Value::as_u64).unwrap_or(0);
            let input = tok("input_tokens");
            let output = tok("output_tokens");
            let cache = tok("cache_creation_input_tokens") + tok("cache_read_input_tokens");
            // Contexte courant = prompt du dernier tour (entrée + cache), sans la sortie.
            let context = input + cache;
            let total = input + output + cache;
            let cost = v.get("total_cost_usd").and_then(Value::as_f64).unwrap_or(0.0);
            // Échec du tour : `is_error` ou un `subtype` autre que "success".
            let is_error = v.get("is_error").and_then(Value::as_bool).unwrap_or(false)
                || v.get("subtype").and_then(Value::as_str).map(|s| s != "success").unwrap_or(false);
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
            // Modèle(s) réellement utilisé(s) = clés de modelUsage (ex. "claude-haiku-4-5").
            let model = v
                .get("modelUsage")
                .and_then(Value::as_object)
                .map(|m| m.keys().cloned().collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            usage::record(app.state::<usage::UsageStore>().inner(), "claude_code", total, cost);
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
                    is_error,
                    model,
                },
            );
        }
        _ => {}
    }
}
