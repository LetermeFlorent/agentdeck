// Auto-config : choisit modèle + effort adaptés à une demande (mode « Auto »).
// Niveaux d'effort lus dynamiquement du CLI ; décision par un appel Haiku jetable.

use crate::provider::{self, headless, Provider};

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

/// Choisit modèle + effort pour une demande via un appel jetable au « choisisseur » de l'IA active.
/// Le choisisseur tourne sur le MÊME provider que le chat (Claude / opencode / Gemini).
#[tauri::command]
pub async fn auto_pick(
    provider: Option<String>,
    prompt: String,
    models: Vec<String>,
    efforts: Vec<String>,
    picker: Option<String>,
    picker_effort: Option<String>,
) -> AutoPick {
    let prov = Provider::from_str(provider.as_deref().unwrap_or("claude_code"));
    let picker = picker.unwrap_or_default();
    let effort = picker_effort.unwrap_or_else(|| "low".into());
    // Le choisisseur tourne en headless sur le même provider que le chat.
    let text = match prov {
        Provider::ClaudeCode => headless::claude_oneshot(&prompt, &picker, Some(&effort), 30).await,
        Provider::Opencode => headless::opencode_oneshot(&prompt, &picker, &effort, 40).await,
        Provider::Gemini => headless::gemini_oneshot(&prompt, &picker, 40).await,
    };

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
