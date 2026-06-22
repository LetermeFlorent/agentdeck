// Parsing du flux d'events JSON d'opencode (`run --format json`) → SessionEvent.
// Schéma RÉEL (capturé) : chaque ligne = un event dont `type` est le type de "part" :
//   {"type":"step_start","sessionID":"ses_…","part":{…,"type":"step-start"}}
//   {"type":"text","sessionID":"…","part":{"id":"prt_…","type":"text","text":"bonjour",…}}
//   {"type":"reasoning","part":{"type":"reasoning","text":"…"}}
//   {"type":"tool","part":{"type":"tool","tool":"bash","callID":"…","state":{"status":"completed","input":{…}}}}
//   {"type":"step_finish","part":{"type":"step-finish","reason":"stop",
//        "tokens":{"total":…,"input":…,"output":…,"reasoning":…,"cache":{"write":…,"read":…}},"cost":0}}
// Pas d'event `message.updated`/`session.idle` : le tour se termine quand le process meurt (EOF).
// → le TurnDone est émis à l'EOF par `opencode.rs` à partir des accumulateurs ici.

use std::collections::{HashMap, HashSet};

use serde_json::Value;
use tauri::Manager;

use crate::provider::claude_code::emit;
use crate::provider::common::{is_rate_limited, RATE_LIMIT_PREFIX};
use crate::events::SessionEvent;
use crate::usage::{self, UsageStore};

/// État/accumulateurs d'un tour opencode (one-shot).
#[derive(Default)]
pub(super) struct OcState {
    started: bool,
    /// Longueur de texte déjà émise par part id (les events `text` portent le texte cumulé).
    text_len: HashMap<String, usize>,
    /// callID des outils déjà annoncés.
    tools_open: HashSet<String>,
    /// Id de session natif opencode (à mémoriser pour reprendre).
    pub(super) session_id: Option<String>,
    // Accumulateurs tokens/coût (sommés sur les step_finish) → TurnDone à l'EOF.
    input: u64,
    output: u64,
    reasoning: u64,
    cache: u64,
    cost: f64,
    any_finish: bool,
}

impl OcState {
    pub(super) fn tour_done(&self) -> bool {
        self.any_finish
    }
}

fn summarize(v: &Value) -> String {
    for key in ["command", "filePath", "file_path", "path", "pattern", "query", "url", "prompt", "description"] {
        if let Some(s) = v.get(key).and_then(Value::as_str) {
            return s.chars().take(220).collect();
        }
    }
    v.to_string().chars().take(180).collect()
}

/// Texte incrémental d'un part `text`/`reasoning` : `part.text` est cumulé → on diffe sur la longueur.
fn text_increment(st: &mut OcState, part: &Value) -> String {
    let pid = part.get("id").and_then(Value::as_str).unwrap_or("");
    let full = part.get("text").and_then(Value::as_str).unwrap_or("");
    let chars: Vec<char> = full.chars().collect();
    let seen = st.text_len.get(pid).copied().unwrap_or(0);
    if chars.len() <= seen {
        return String::new();
    }
    let new: String = chars[seen..].iter().collect();
    st.text_len.insert(pid.to_string(), chars.len());
    new
}

/// Traduit une ligne d'event opencode en SessionEvent(s). Renvoie true si un rate-limit est détecté.
pub(super) fn handle_line(app: &tauri::AppHandle, id: &str, v: &Value, st: &mut OcState) -> bool {
    // Id de session natif (présent sur chaque event).
    if st.session_id.is_none() {
        if let Some(sid) = v.get("sessionID").and_then(Value::as_str) {
            st.session_id = Some(sid.to_string());
        }
    }
    let part = v.get("part").unwrap_or(&Value::Null);

    match v.get("type").and_then(Value::as_str) {
        Some("text") => {
            let text = text_increment(st, part);
            if !text.is_empty() {
                if !st.started {
                    st.started = true;
                    emit(app, id, SessionEvent::AssistantStart);
                }
                emit(app, id, SessionEvent::AssistantDelta { text });
            }
        }
        Some("reasoning") => {
            let text = text_increment(st, part);
            if !text.is_empty() {
                emit(app, id, SessionEvent::Thinking { text });
            }
        }
        Some("tool") => {
            let call = part.get("callID").and_then(Value::as_str).unwrap_or("").to_string();
            let status = part
                .get("state")
                .and_then(|s| s.get("status"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if !call.is_empty() && !st.tools_open.contains(&call) {
                st.tools_open.insert(call.clone());
                let name = part.get("tool").and_then(Value::as_str).unwrap_or("tool").to_string();
                let input = part
                    .get("state")
                    .and_then(|s| s.get("input"))
                    .map(summarize)
                    .unwrap_or_default();
                emit(app, id, SessionEvent::ToolUse { name, input, id: call.clone() });
            }
            if matches!(status, "completed" | "error") {
                emit(app, id, SessionEvent::ToolDone { id: call });
            }
        }
        Some("step_finish") => {
            st.any_finish = true;
            let tok = part.get("tokens");
            let g = |k: &str| tok.and_then(|t| t.get(k)).and_then(Value::as_u64).unwrap_or(0);
            // input/cache = contexte (dernier gagne) ; output/reasoning = cumulés sur les étapes.
            st.input = g("input");
            st.output += g("output");
            st.reasoning += g("reasoning");
            st.cache = tok
                .and_then(|t| t.get("cache"))
                .map(|c| {
                    c.get("read").and_then(Value::as_u64).unwrap_or(0)
                        + c.get("write").and_then(Value::as_u64).unwrap_or(0)
                })
                .unwrap_or(0);
            st.cost += part.get("cost").and_then(Value::as_f64).unwrap_or(0.0);
        }
        Some("error") | Some("session.error") => {
            let msg = v
                .get("error")
                .or_else(|| part.get("error"))
                .map(|e| e.to_string())
                .unwrap_or_else(|| v.to_string());
            if is_rate_limited("", None, Some(&msg)) {
                emit(app, id, SessionEvent::Error { message: format!("{RATE_LIMIT_PREFIX}{msg}") });
                return true;
            }
            emit(app, id, SessionEvent::Error { message: msg });
        }
        _ => {} // step_start, etc. : ignorés
    }
    false
}

/// Émet le TurnDone final (appelé à l'EOF par `opencode.rs`) depuis les accumulateurs.
pub(super) fn emit_turn_done(app: &tauri::AppHandle, id: &str, st: &OcState, model: &str) {
    if !st.any_finish && !st.started {
        return;
    }
    let total = st.input + st.output + st.reasoning + st.cache;
    usage::record(app.state::<UsageStore>().inner(), "opencode", total, st.cost);
    emit(
        app,
        id,
        SessionEvent::TurnDone {
            input_tokens: st.input,
            output_tokens: st.output,
            total_tokens: total,
            cost_usd: st.cost,
            context_tokens: st.input + st.cache,
            context_window: 0,
            is_error: false,
            model: model.to_string(),
        },
    );
}
