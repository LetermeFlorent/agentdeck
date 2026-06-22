// Sources « réelles » d'usage : endpoint OAuth (api.anthropic.com) + cache claude-statusbar.

use super::{bar_pct, now, Bar, RealUsage, UsageStore};

const OAUTH_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
// User-Agent claude-code obligatoire, sinon l'endpoint renvoie des 429 persistants.
const CLIENT_UA: &str = "claude-code/2.1.178";
const POLL_INTERVAL_S: u64 = 185; // l'endpoint limite agressivement < 180s

fn iso_to_epoch(s: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(s).map(|d| d.timestamp()).unwrap_or(0)
}

/// Vraies limites d'abonnement (5h / 7j) lues depuis le cache de claude-statusbar
/// (`~/.cache/claude-statusbar/last_stdin.json`), mis à jour par Claude Code à chaque tick.
/// Retourne (five_hour, seven_day) si disponible et non périmé.
pub fn real_rate_limits() -> Option<(Bar, Bar)> {
    let mut p = dirs::home_dir()?;
    p.push(".cache");
    p.push("claude-statusbar");
    p.push("last_stdin.json");
    let raw = std::fs::read_to_string(p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let rl = v.get("rate_limits")?;

    let read = |key: &str| -> Option<Bar> {
        let w = rl.get(key)?;
        let used = w.get("used_percentage").and_then(serde_json::Value::as_f64)?;
        let resets = w.get("resets_at").and_then(serde_json::Value::as_i64).unwrap_or(0);
        // Fenêtre déjà réinitialisée → la conso réelle est repartie à 0.
        let used = if resets > 0 && (now() as i64) >= resets { 0.0 } else { used };
        Some(bar_pct(used, resets))
    };

    Some((read("five_hour")?, read("seven_day")?))
}

/// Un appel à l'endpoint usage. Renvoie le vrai 5h/7j si succès.
async fn fetch_real(client: &reqwest::Client, token: &str) -> Option<RealUsage> {
    let resp = client
        .get(OAUTH_USAGE_URL)
        .header("Authorization", format!("Bearer {token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", CLIENT_UA)
        .header("Content-Type", "application/json")
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().await.ok()?;
    let win = |key: &str| -> Option<(f64, i64)> {
        let w = v.get(key)?;
        let util = w.get("utilization").and_then(serde_json::Value::as_f64)?;
        let reset = w
            .get("resets_at")
            .and_then(serde_json::Value::as_str)
            .map(iso_to_epoch)
            .unwrap_or(0);
        Some((util, reset))
    };
    let (fh_pct, fh_reset) = win("five_hour")?;
    let (wk_pct, wk_reset) = win("seven_day")?;
    Some(RealUsage {
        five_h_pct: fh_pct,
        five_h_reset: fh_reset,
        week_pct: wk_pct,
        week_reset: wk_reset,
    })
}

/// Boucle de fond : interroge l'endpoint toutes les ~185s avec le token courant et met en cache.
pub async fn run_poller(app: tauri::AppHandle) {
    use tauri::Manager;
    let client = reqwest::Client::new();
    // Token mis en cache (accès keyring potentiellement bloquant ; valeur quasi stable).
    let mut cached_token: Option<String> = None;
    loop {
        if cached_token.is_none() {
            cached_token = crate::auth::get_token(crate::provider::Provider::ClaudeCode);
        }
        if let Some(token) = cached_token.as_deref() {
            match fetch_real(&client, token).await {
                Some(r) => app.state::<UsageStore>().set_real(r),
                None => cached_token = None, // token révoqué/expiré → relit le keyring au prochain tick
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_S)).await;
    }
}
