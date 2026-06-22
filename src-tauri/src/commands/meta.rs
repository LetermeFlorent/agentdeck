// Commandes « méta » : usage, défauts modèle/effort, plan d'abonnement, liste de modèles par IA.

use crate::provider::Provider;
use crate::usage::{self, UsageSnapshot, UsageStore};

/// Modèles disponibles pour un provider donné (dispatch). `[{v: id, l: label}]`.
#[tauri::command]
pub async fn provider_models(provider: String) -> Vec<serde_json::Value> {
    match Provider::from_str(&provider) {
        Provider::ClaudeCode => claude_models().await,
        Provider::Opencode => opencode_models().await,
        Provider::Gemini => gemini_models(),
    }
}

/// Modèles opencode : `opencode models` (liste `provider/model`). On ne garde que les modèles
/// des providers réellement configurés (auth.json) + les modèles intégrés `opencode/*` (free/zen).
async fn opencode_models() -> Vec<serde_json::Value> {
    // Providers configurés (clés de auth.json) — sinon la liste catalogue fait des centaines d'entrées.
    let mut authed: std::collections::HashSet<String> = std::collections::HashSet::new();
    authed.insert("opencode".to_string());
    if let Some(mut p) = dirs::home_dir() {
        p.push(".local");
        p.push("share");
        p.push("opencode");
        p.push("auth.json");
        if let Ok(raw) = std::fs::read_to_string(&p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(obj) = v.as_object() {
                    for k in obj.keys() {
                        authed.insert(k.clone());
                    }
                }
            }
        }
    }
    let out = crate::provider::common::opencode_command()
        .arg("models")
        .output()
        .await;
    let out = match out {
        Ok(o) if o.status.success() => o.stdout,
        _ => return vec![],
    };
    let text = String::from_utf8_lossy(&out);
    let mut models = vec![];
    for line in text.lines() {
        let id = line.trim();
        if id.is_empty() {
            continue;
        }
        let prov = id.split('/').next().unwrap_or("");
        if !authed.contains(prov) {
            continue;
        }
        // Label = partie modèle (après le provider) pour rester lisible dans le sélecteur.
        let label = id.split_once('/').map(|x| x.1).unwrap_or(id);
        models.push(serde_json::json!({ "v": id, "l": label }));
    }
    models
}

/// Modèles Gemini : le CLI n'expose pas de liste machine fiable → liste curée (à actualiser).
fn gemini_models() -> Vec<serde_json::Value> {
    [
        ("gemini-2.5-pro", "2.5 Pro"),
        ("gemini-2.5-flash", "2.5 Flash"),
        ("gemini-2.5-flash-lite", "2.5 Flash-Lite"),
        ("gemini-2.0-flash", "2.0 Flash"),
    ]
    .iter()
    .map(|(v, l)| serde_json::json!({ "v": v, "l": l }))
    .collect()
}

#[tauri::command]
pub fn usage_get(store: tauri::State<'_, UsageStore>) -> UsageSnapshot {
    usage::snapshot(&store)
}

#[tauri::command]
pub fn usage_get_provider(provider: String, store: tauri::State<'_, UsageStore>) -> UsageSnapshot {
    usage::snapshot_for(&store, &provider)
}

/// Liste les modèles réellement disponibles pour le compte (API Models d'Anthropic),
/// avec le token OAuth du coffre. Renvoie `[{v: id, l: display_name}]`, ou vide si
/// indisponible (offline / token sans accès) → le frontend garde sa liste de secours.
#[tauri::command]
pub async fn claude_models() -> Vec<serde_json::Value> {
    let token = match crate::auth::get_token(crate::provider::Provider::ClaudeCode) {
        Some(t) => t,
        None => return vec![],
    };
    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.anthropic.com/v1/models?limit=100")
        .header("Authorization", format!("Bearer {token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("anthropic-version", "2023-06-01")
        .header("User-Agent", "claude-code/2.1.178")
        .send()
        .await;
    let resp = match resp {
        Ok(r) if r.status().is_success() => r,
        _ => return vec![],
    };
    let v: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let mut out = vec![];
    if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
        for m in arr {
            let id = m.get("id").and_then(|x| x.as_str()).unwrap_or("");
            if !id.starts_with("claude-") {
                continue;
            }
            let name = m.get("display_name").and_then(|x| x.as_str()).unwrap_or(id);
            // Pas besoin du préfixe "Claude " dans le sélecteur.
            let label = name.strip_prefix("Claude ").unwrap_or(name);
            out.push(serde_json::json!({ "v": id, "l": label }));
        }
    }
    out
}

/// Modèle / effort par défaut pour un nouveau pane : on récupère ceux du Claude Code
/// courant (cache statusline) ; sinon valeurs par défaut (Opus / medium).
#[tauri::command]
pub fn claude_defaults() -> serde_json::Value {
    let read = || -> Option<(String, String)> {
        let mut p = dirs::home_dir()?;
        p.push(".cache");
        p.push("claude-statusbar");
        p.push("last_stdin.json");
        let raw = std::fs::read_to_string(p).ok()?;
        let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let model_id = v
            .get("model")
            .and_then(|m| m.get("id"))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        let alias = if model_id.contains("opus") {
            "opus"
        } else if model_id.contains("sonnet") {
            "sonnet"
        } else if model_id.contains("haiku") {
            "haiku"
        } else if model_id.contains("fable") {
            "fable"
        } else {
            ""
        };
        let effort = v
            .get("effort")
            .and_then(|e| e.get("level"))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        Some((alias.to_string(), effort.to_string()))
    };
    let (mut model, mut effort) = read().unwrap_or_default();
    if model.is_empty() {
        model = "opus".into();
    }
    if effort.is_empty() {
        effort = "medium".into();
    }
    serde_json::json!({ "model": model, "effort": effort })
}

/// Plan d'abonnement (lu dans ~/.claude/.credentials.json). `level` 0–4 pilote l'effet visuel.
#[tauri::command]
pub fn subscription_plan() -> serde_json::Value {
    let read = || -> Option<(String, String)> {
        let mut p = dirs::home_dir()?;
        p.push(".claude");
        p.push(".credentials.json");
        let raw = std::fs::read_to_string(p).ok()?;
        let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let o = v.get("claudeAiOauth")?;
        let sub = o
            .get("subscriptionType")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let tier = o
            .get("rateLimitTier")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        Some((sub, tier))
    };
    // Compte connecté (email / nom), lu dans ~/.claude.json → oauthAccount.
    let account = (|| -> Option<String> {
        let mut p = dirs::home_dir()?;
        p.push(".claude.json");
        let raw = std::fs::read_to_string(p).ok()?;
        let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let o = v.get("oauthAccount")?;
        o.get("displayName")
            .and_then(|x| x.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| o.get("emailAddress").and_then(|x| x.as_str()))
            .map(String::from)
    })()
    .unwrap_or_default();

    let (sub, tier) = read().unwrap_or_default();
    let (label, level): (String, u8) = if tier.contains("max_20x") {
        ("Max 20×".into(), 4)
    } else if tier.contains("max_5x") {
        ("Max 5×".into(), 3)
    } else if tier.contains("pro") {
        ("Pro".into(), 1)
    } else {
        match sub.as_str() {
            "max" => ("Max".into(), 3),
            "pro" => ("Pro".into(), 1),
            "team" => ("Team".into(), 4),
            "enterprise" => ("Enterprise".into(), 4),
            "" => (String::new(), 0),
            other => {
                let mut c = other.chars();
                let cap = c
                    .next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default();
                (cap, 2)
            }
        }
    };
    serde_json::json!({ "label": label, "level": level, "account": account })
}
