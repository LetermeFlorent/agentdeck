// Auto-config : choisit modèle + effort adaptés à une demande (mode « Auto »).
// Niveaux d'effort lus dynamiquement du CLI ; décision par un appel Haiku jetable.

use crate::auth;
use crate::provider;

/// Niveaux d'effort valides, lus dynamiquement depuis `claude --help` (--effort <level>).
#[tauri::command]
pub async fn effort_levels() -> Vec<String> {
    let out = tokio::process::Command::new(provider::claude_code::claude_bin())
        .arg("--help")
        .output()
        .await;
    let text = match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => return vec![],
    };
    // Cherche la ligne "--effort <level> ... (low, medium, high, xhigh, max)".
    for line in text.lines() {
        if line.contains("--effort") || line.contains("low, medium") {
            if let (Some(a), Some(b)) = (line.find('('), line.find(')')) {
                if b > a {
                    return line[a + 1..b]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
    }
    vec![]
}

/// Résultat du classifieur : modèle + effort recommandés (l'un ou l'autre peut être vide).
#[derive(serde::Serialize, Default)]
pub struct AutoPick {
    pub model: String,
    pub effort: String,
}

/// Choisit modèle + effort pour une demande via un appel Haiku jetable (cheap).
#[tauri::command]
pub async fn auto_pick(
    prompt: String,
    models: Vec<String>,
    efforts: Vec<String>,
    picker: Option<String>,
) -> AutoPick {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    // Connexion agentdeck indépendante : il faut notre propre token dans le coffre
    // (une session `claude` native ne suffit pas, sinon l'UI montre « connecté » à tort).
    let token = auth::get_token();
    if token.is_none() {
        return AutoPick::default();
    }
    // L'instruction complète (avec prix/récence) est construite côté frontend et passée telle
    // quelle dans `prompt` — ce qui permet aussi de l'afficher en aperçu dans les réglages.
    let msg = serde_json::json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": prompt }] }
    })
    .to_string();

    let mut cmd = tokio::process::Command::new(provider::claude_code::claude_bin());
    cmd.arg("-p")
        .arg("--input-format")
        .arg("stream-json")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .arg("--model")
        .arg(picker.as_deref().filter(|p| !p.is_empty()).unwrap_or("haiku"))
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    if let Some(t) = &token {
        cmd.env("CLAUDE_CODE_OAUTH_TOKEN", t);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return AutoPick::default(),
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
                        if let Some(r) = v.get("result").and_then(|x| x.as_str()) {
                            text = r.to_string();
                        }
                        break;
                    }
                }
            }
        }
        text
    };
    let text = tokio::time::timeout(std::time::Duration::from_secs(30), read)
        .await
        .unwrap_or_default();
    let _ = child.start_kill();

    // Extrait le JSON {...}.
    let json = match (text.find('{'), text.rfind('}')) {
        (Some(a), Some(b)) if b > a => &text[a..=b],
        _ => return AutoPick::default(),
    };
    let v: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return AutoPick::default(),
    };
    let raw_model = v.get("model").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    let raw_effort = v.get("effort").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    // Résolution tolérante : le choisisseur peut renvoyer un label ("Haiku 4.5") ou un alias
    // ("haiku") au lieu de l'ID exact → on rapproche par normalisation alphanumérique.
    let model = resolve(&raw_model, &models);
    let effort = resolve(&raw_effort, &efforts);
    log::info!(
        "auto_pick: brut model={raw_model:?} effort={raw_effort:?} -> retenu model={model:?} effort={effort:?}"
    );
    AutoPick { model, effort }
}

/// Normalise (minuscules + alphanumériques) pour un rapprochement tolérant.
fn norm(s: &str) -> String {
    s.to_lowercase().chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Rapproche `pick` d'une entrée de `list` (exact, sinon par normalisation/inclusion).
fn resolve(pick: &str, list: &[String]) -> String {
    if pick.is_empty() {
        return String::new();
    }
    if let Some(m) = list.iter().find(|m| m.as_str() == pick) {
        return m.clone();
    }
    let p = norm(pick);
    if p.is_empty() {
        return String::new();
    }
    list.iter()
        .find(|m| {
            let n = norm(m);
            n == p || n.contains(&p) || p.contains(&n)
        })
        .cloned()
        .unwrap_or_default()
}
