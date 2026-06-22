// Parsing du flux NDJSON de Gemini CLI (`--output-format stream-json`) → SessionEvent.
// Types d'events documentés : init / message / tool_use / tool_result / error / result.
// ⚠️ Les noms de champs internes ne sont pas figés par la doc → extraction défensive
// (plusieurs clés candidates). À ré-aligner sur une capture réelle (clé API Gemini valide).

use serde_json::Value;

use crate::provider::claude_code::emit;
use crate::provider::common::{is_rate_limited, RATE_LIMIT_PREFIX};
use crate::events::SessionEvent;
use crate::usage::{self, UsageStore};
use tauri::Manager;

/// État d'un tour Gemini (one-shot) : bulle assistant ouverte ? tour terminé (result reçu) ?
#[derive(Default)]
pub(super) struct GemState {
    started: bool,
    done: bool,
}

impl GemState {
    /// Vrai si un event `result` (fin de tour) a été reçu → pas une erreur silencieuse.
    pub(super) fn tour_done(&self) -> bool {
        self.done
    }
}

/// Premier champ texte non vide parmi une liste de clés candidates.
fn first_str<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a str> {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(Value::as_str) {
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

/// Texte d'un message : champ direct (text/content) ou concat des blocs `content[].text`.
fn message_text(v: &Value) -> String {
    if let Some(s) = first_str(v, &["text", "content", "delta", "chunk"]) {
        return s.to_string();
    }
    if let Some(arr) = v.get("content").and_then(Value::as_array) {
        return arr
            .iter()
            .filter_map(|b| b.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("");
    }
    // Certains formats imbriquent sous message.{content|text}.
    if let Some(m) = v.get("message") {
        return message_text(m);
    }
    String::new()
}

/// Traduit une ligne NDJSON Gemini en SessionEvent(s). Renvoie true si un rate-limit a été détecté.
pub(super) fn handle_line(
    app: &tauri::AppHandle,
    id: &str,
    v: &Value,
    st: &mut GemState,
) -> bool {
    match v.get("type").and_then(Value::as_str) {
        Some("init") => {
            emit(app, id, SessionEvent::Started);
            emit(app, id, SessionEvent::Init { slash_commands: vec![], tools: vec![] });
        }
        Some("message") => {
            // On ignore les messages user renvoyés en écho ; seul l'assistant nous intéresse.
            let role = v.get("role").and_then(Value::as_str).unwrap_or("assistant");
            if role == "user" {
                return false;
            }
            let text = message_text(v);
            if !text.is_empty() {
                if !st.started {
                    st.started = true;
                    emit(app, id, SessionEvent::AssistantStart);
                }
                emit(app, id, SessionEvent::AssistantDelta { text });
            }
        }
        Some("thought") | Some("thinking") => {
            let text = message_text(v);
            if !text.is_empty() {
                emit(app, id, SessionEvent::Thinking { text });
            }
        }
        Some("tool_use") => {
            let name = first_str(v, &["name", "tool", "tool_name"]).unwrap_or("tool").to_string();
            let tid = first_str(v, &["id", "tool_use_id", "call_id"]).unwrap_or("").to_string();
            let input = v
                .get("args")
                .or_else(|| v.get("input"))
                .or_else(|| v.get("arguments"))
                .map(summarize)
                .unwrap_or_default();
            emit(app, id, SessionEvent::ToolUse { name, input, id: tid });
        }
        Some("tool_result") => {
            let tid = first_str(v, &["tool_use_id", "id", "call_id"]).unwrap_or("").to_string();
            emit(app, id, SessionEvent::ToolDone { id: tid });
        }
        Some("error") => {
            let msg = first_str(v, &["message"])
                .map(String::from)
                .or_else(|| v.get("error").map(|e| e.to_string()))
                .unwrap_or_else(|| v.to_string());
            if is_rate_limited("", None, Some(&msg)) {
                emit(app, id, SessionEvent::Error { message: format!("{RATE_LIMIT_PREFIX}{msg}") });
                return true;
            }
            emit(app, id, SessionEvent::Error { message: msg });
        }
        Some("result") => {
            st.done = true;
            let stats = v.get("stats").unwrap_or(v);
            // Tokens : agrège modelUsage/models si présent, sinon champs plats.
            let (input, output, total) = read_tokens(stats);
            let context_window = stats
                .get("models")
                .or_else(|| stats.get("modelUsage"))
                .and_then(Value::as_object)
                .map(|m| {
                    m.values()
                        .filter_map(|mu| mu.get("contextWindow").and_then(Value::as_u64))
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            let model = stats
                .get("models")
                .or_else(|| stats.get("modelUsage"))
                .and_then(Value::as_object)
                .map(|m| m.keys().cloned().collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            // Free tier : pas de coût.
            usage::record(app.state::<UsageStore>().inner(), "gemini", total, 0.0);
            emit(
                app,
                id,
                SessionEvent::TurnDone {
                    input_tokens: input,
                    output_tokens: output,
                    total_tokens: total,
                    cost_usd: 0.0,
                    context_tokens: input,
                    context_window,
                    is_error: v.get("error").is_some(),
                    model,
                },
            );
        }
        _ => {}
    }
    false
}

/// Résume une entrée d'outil en une ligne pour l'affichage terminal.
fn summarize(v: &Value) -> String {
    for key in ["command", "file_path", "path", "pattern", "query", "url", "prompt", "description"] {
        if let Some(s) = v.get(key).and_then(Value::as_str) {
            return s.chars().take(220).collect();
        }
    }
    v.to_string().chars().take(180).collect()
}

/// Lit (input, output, total) depuis un objet stats Gemini (clés tolérantes).
fn read_tokens(stats: &Value) -> (u64, u64, u64) {
    let g = |keys: &[&str]| -> u64 {
        for k in keys {
            if let Some(n) = stats.get(*k).and_then(Value::as_u64) {
                return n;
            }
        }
        // Parfois imbriqué sous stats.tokens.{...}.
        if let Some(t) = stats.get("tokens") {
            for k in keys {
                if let Some(n) = t.get(*k).and_then(Value::as_u64) {
                    return n;
                }
            }
        }
        0
    };
    let input = g(&["input_tokens", "promptTokenCount", "prompt_tokens", "input"]);
    let output = g(&["output_tokens", "candidatesTokenCount", "completion_tokens", "output"]);
    let total = {
        let t = g(&["total_tokens", "totalTokenCount", "total"]);
        if t > 0 { t } else { input + output }
    };
    (input, output, total)
}
