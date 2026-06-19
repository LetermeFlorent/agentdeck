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
pub async fn auto_pick(prompt: String, models: Vec<String>, efforts: Vec<String>) -> AutoPick {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    // Connexion agentdeck indépendante : il faut notre propre token dans le coffre
    // (une session `claude` native ne suffit pas, sinon l'UI montre « connecté » à tort).
    let token = auth::get_token();
    if token.is_none() {
        return AutoPick::default();
    }
    let models_s = models.join(", ");
    let efforts_s = efforts.join(", ");
    let ask = format!(
        "Choisis la config optimale pour traiter cette demande d'un utilisateur à un agent de code. \
Modèles possibles : [{models_s}]. Efforts possibles : [{efforts_s}]. \
Règle : demande simple / question courte → effort bas + modèle léger ; \
tâche complexe (code, architecture, debug, raisonnement long) → effort élevé + modèle puissant. \
Réponds STRICTEMENT en JSON une ligne, sans texte autour : \
{{\"model\":\"<un des modèles ou vide>\",\"effort\":\"<un des efforts ou vide>\"}}. \
Demande : {prompt}"
    );
    let msg = serde_json::json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": ask }] }
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
        .arg("haiku")
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
    let pick = AutoPick {
        model: v.get("model").and_then(|x| x.as_str()).unwrap_or("").trim().to_string(),
        effort: v.get("effort").and_then(|x| x.as_str()).unwrap_or("").trim().to_string(),
    };
    // Valide contre les listes fournies (sinon vide → on garde la valeur courante côté UI).
    AutoPick {
        model: if models.contains(&pick.model) { pick.model } else { String::new() },
        effort: if efforts.contains(&pick.effort) { pick.effort } else { String::new() },
    }
}
