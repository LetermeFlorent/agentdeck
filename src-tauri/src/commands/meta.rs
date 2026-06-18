// Commandes « méta » : usage, défauts modèle/effort, plan d'abonnement.

use crate::usage::{self, UsageSnapshot, UsageStore};

#[tauri::command]
pub fn usage_get(store: tauri::State<'_, UsageStore>) -> UsageSnapshot {
    usage::snapshot(&store)
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
