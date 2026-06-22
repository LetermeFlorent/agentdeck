// Appels « one-shot » headless aux CLI provider : un prompt → on lit la sortie JSON → texte.
// Partagé par le mode Auto (choisisseur de modèle/effort) et le mode Hermes (réflexion).
// Factorise le schéma commun spawn → pipe → lecture JSON → timeout → kill.

use crate::auth;
use crate::provider::{self, Provider};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Claude : `claude -p` stream-json avec token coffre → texte de l'event `result`.
/// `model` vide → "haiku". `effort` None (ou vide) → pas de `--effort`.
pub async fn claude_oneshot(prompt: &str, model: &str, effort: Option<&str>, timeout_s: u64) -> String {
    let token = match auth::get_token(Provider::ClaudeCode) {
        Some(t) => t,
        None => return String::new(),
    };
    let msg = serde_json::json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": prompt }] }
    })
    .to_string();
    let mut cmd = tokio::process::Command::new(provider::claude_code::claude_bin());
    cmd.arg("-p")
        .arg("--input-format").arg("stream-json")
        .arg("--output-format").arg("stream-json")
        .arg("--verbose")
        .arg("--permission-mode").arg("bypassPermissions")
        .arg("--model").arg(if model.is_empty() { "haiku" } else { model });
    if let Some(e) = effort.filter(|e| !e.is_empty()) {
        cmd.arg("--effort").arg(e);
    }
    cmd.env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .env("CLAUDE_CODE_OAUTH_TOKEN", &token)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    if let Some(mut si) = child.stdin.take() {
        let _ = si.write_all(msg.as_bytes()).await;
        let _ = si.write_all(b"\n").await;
        let _ = si.flush().await;
    }
    let stdout = child.stdout.take();
    let read = async {
        let mut text = String::new();
        if let Some(out) = stdout {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                    if v.get("type").and_then(|x| x.as_str()) == Some("result") {
                        text = v.get("result").and_then(|x| x.as_str()).unwrap_or("").to_string();
                        break;
                    }
                }
            }
        }
        text
    };
    let text = tokio::time::timeout(Duration::from_secs(timeout_s), read).await.unwrap_or_default();
    let _ = child.start_kill();
    text
}

/// opencode : `opencode run --format json -m <model> --variant <variant>` → dernière part texte.
pub async fn opencode_oneshot(prompt: &str, model: &str, variant: &str, timeout_s: u64) -> String {
    let mut cmd = provider::common::opencode_command();
    cmd.arg("run").arg("--format").arg("json").arg("--dangerously-skip-permissions");
    if !model.is_empty() {
        cmd.arg("-m").arg(model);
    }
    if !variant.is_empty() {
        cmd.arg("--variant").arg(variant);
    }
    cmd.arg(prompt)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let stdout = child.stdout.take();
    let read = async {
        let mut text = String::new();
        if let Some(out) = stdout {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                    if v.get("type").and_then(|x| x.as_str()) == Some("text") {
                        if let Some(t) = v.get("part").and_then(|p| p.get("text")).and_then(|x| x.as_str()) {
                            text = t.to_string(); // part.text est cumulé
                        }
                    }
                }
            }
        }
        text
    };
    let text = tokio::time::timeout(Duration::from_secs(timeout_s), read).await.unwrap_or_default();
    let _ = child.start_kill();
    text
}

/// Gemini : `gemini -p "<prompt>" -o json -m <model>` → champ `response` (sinon brut).
pub async fn gemini_oneshot(prompt: &str, model: &str, timeout_s: u64) -> String {
    let mut cmd = provider::common::gemini_command();
    cmd.arg("-p").arg(prompt).arg("-o").arg("json").arg("--skip-trust").arg("--yolo");
    if !model.is_empty() {
        cmd.arg("-m").arg(model);
    }
    if let Some(key) = auth::get_token(Provider::Gemini) {
        if !key.is_empty() {
            cmd.env("GEMINI_API_KEY", key);
        }
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let out = tokio::time::timeout(Duration::from_secs(timeout_s), async { cmd.output().await.ok() })
        .await
        .ok()
        .flatten();
    let out = match out {
        Some(o) => o,
        None => return String::new(),
    };
    let raw = String::from_utf8_lossy(&out.stdout);
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw.trim()) {
        if let Some(r) = v.get("response").and_then(|x| x.as_str()) {
            return r.to_string();
        }
    }
    raw.to_string()
}
