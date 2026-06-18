// Suivi d'usage 5h / semaine.
//
// ⚠️ Aucune API publique ne donne les vrais % d'abonnement (5h/hebdo) ; `/usage` est interactif
// seulement. On fournit donc un comptage LOCAL fiable : on additionne les tokens consommés par
// les tours lancés depuis l'app, sur des fenêtres glissantes de 5h et 7j. Source = "estimé".
// (Un futur adapter pourra remplacer ça par les vrais chiffres si un endpoint devient accessible.)

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const FIVE_H_SECS: u64 = 5 * 60 * 60;
const WEEK_SECS: u64 = 7 * 24 * 60 * 60;

// Plafonds par défaut (ajustables) — estimations, faute de chiffres officiels exposés.
const DEFAULT_FIVE_H_CAP: u64 = 5_000_000;
const DEFAULT_WEEK_CAP: u64 = 50_000_000;

#[derive(Clone, Serialize, Deserialize)]
struct Entry {
    ts: u64, // epoch secondes
    tokens: u64,
    #[serde(default)]
    cost: f64,
}

#[derive(Serialize, Deserialize)]
struct UsageData {
    entries: Vec<Entry>,
    five_h_cap: u64,
    week_cap: u64,
}

impl Default for UsageData {
    fn default() -> Self {
        UsageData {
            entries: Vec::new(),
            five_h_cap: DEFAULT_FIVE_H_CAP,
            week_cap: DEFAULT_WEEK_CAP,
        }
    }
}

/// Vrai usage d'abonnement récupéré via l'endpoint OAuth (token-driven, portable).
#[derive(Clone, Copy)]
struct RealUsage {
    five_h_pct: f64,
    five_h_reset: i64,
    week_pct: f64,
    week_reset: i64,
}

pub struct UsageStore {
    inner: Mutex<UsageData>,
    real: Mutex<Option<RealUsage>>,
}

#[derive(Serialize)]
pub struct Bar {
    pub tokens: u64,
    pub cap: u64,
    pub pct: u32,
    /// Epoch (s) de réinitialisation de la fenêtre, si connu (source réelle).
    pub resets_at: Option<i64>,
}

#[derive(Serialize)]
pub struct UsageSnapshot {
    pub five_h: Bar,
    pub week: Bar,
    /// Coût cumulé (USD) sur les fenêtres, d'après total_cost_usd renvoyé par Claude.
    pub five_h_cost: f64,
    pub week_cost: f64,
    /// "estimated" (comptage local) ou "real" (si un jour branché sur un vrai endpoint).
    pub source: String,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn store_path() -> Option<std::path::PathBuf> {
    let mut p = dirs::data_dir()?;
    p.push("agentdeck");
    let _ = std::fs::create_dir_all(&p);
    p.push("usage.json");
    Some(p)
}

impl UsageStore {
    pub fn load() -> Self {
        let data = store_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<UsageData>(&s).ok())
            .unwrap_or_default();
        UsageStore {
            inner: Mutex::new(data),
            real: Mutex::new(None),
        }
    }

    fn set_real(&self, r: RealUsage) {
        if let Ok(mut slot) = self.real.lock() {
            *slot = Some(r);
        }
    }

    fn persist(data: &UsageData) {
        if let Some(p) = store_path() {
            if let Ok(s) = serde_json::to_string(data) {
                let _ = std::fs::write(p, s);
            }
        }
    }
}

fn bar(tokens: u64, cap: u64) -> Bar {
    let pct = if cap == 0 {
        0
    } else {
        ((tokens as f64 / cap as f64) * 100.0).round().min(100.0) as u32
    };
    Bar {
        tokens,
        cap,
        pct,
        resets_at: None,
    }
}

fn bar_pct(used_pct: f64, resets_at: i64) -> Bar {
    Bar {
        tokens: 0,
        cap: 0,
        pct: used_pct.round().clamp(0.0, 100.0) as u32,
        resets_at: Some(resets_at),
    }
}

/// Vraies limites d'abonnement (5h / 7j) lues depuis le cache de claude-statusbar
/// (`~/.cache/claude-statusbar/last_stdin.json`), que Claude Code met à jour à chaque
/// tick de statusline. C'est la donnée officielle (used_percentage + resets_at).
/// Retourne (five_hour, seven_day) en (used_pct, resets_at) si disponible et non périmé.
fn real_rate_limits() -> Option<(Bar, Bar)> {
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
        let resets = w
            .get("resets_at")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        // Fenêtre déjà réinitialisée → la conso réelle est repartie à 0.
        let used = if resets > 0 && (now() as i64) >= resets {
            0.0
        } else {
            used
        };
        Some(bar_pct(used, resets))
    };

    Some((read("five_hour")?, read("seven_day")?))
}

/// Enregistre la conso d'un tour terminé (tous types de tokens + coût) et purge les entrées > 7j.
pub fn record(store: &UsageStore, tokens: u64, cost: f64) {
    if tokens == 0 && cost == 0.0 {
        return;
    }
    let mut data = match store.inner.lock() {
        Ok(d) => d,
        Err(_) => return,
    };
    let t = now();
    data.entries.push(Entry {
        ts: t,
        tokens,
        cost,
    });
    let cutoff = t.saturating_sub(WEEK_SECS);
    data.entries.retain(|e| e.ts >= cutoff);
    UsageStore::persist(&data);
}

// ---- Vrai usage via l'endpoint OAuth (api.anthropic.com/api/oauth/usage) ----

const OAUTH_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
// User-Agent claude-code obligatoire, sinon l'endpoint renvoie des 429 persistants.
const CLIENT_UA: &str = "claude-code/2.1.178";
const POLL_INTERVAL_S: u64 = 185; // l'endpoint limite agressivement < 180s

fn iso_to_epoch(s: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| d.timestamp())
        .unwrap_or(0)
}

/// Un appel à l'endpoint usage. Renvoie le vrai 5h/7j si succès.
async fn fetch_real(token: &str) -> Option<RealUsage> {
    let client = reqwest::Client::new();
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

/// Boucle de fond : interroge l'endpoint toutes les ~185s avec le token courant
/// et met en cache le vrai usage. Démarre dès qu'un token est présent.
pub async fn run_poller(app: tauri::AppHandle) {
    use tauri::Manager;
    loop {
        if let Some(token) = crate::auth::get_token() {
            if let Some(r) = fetch_real(&token).await {
                app.state::<UsageStore>().set_real(r);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_S)).await;
    }
}

/// Calcule les barres 5h / semaine.
pub fn snapshot(store: &UsageStore) -> UsageSnapshot {
    let data = match store.inner.lock() {
        Ok(d) => d,
        Err(_) => {
            return UsageSnapshot {
                five_h: bar(0, DEFAULT_FIVE_H_CAP),
                week: bar(0, DEFAULT_WEEK_CAP),
                five_h_cost: 0.0,
                week_cost: 0.0,
                source: "estimated".into(),
            }
        }
    };
    let t = now();
    let five_h_cut = t.saturating_sub(FIVE_H_SECS);
    let week_cut = t.saturating_sub(WEEK_SECS);
    let in_window = |cut: u64| data.entries.iter().filter(move |e| e.ts >= cut);
    let five_h_cost: f64 = in_window(five_h_cut).map(|e| e.cost).sum();
    let week_cost: f64 = in_window(week_cut).map(|e| e.cost).sum();

    // Priorité des sources de "réel" :
    //  1. endpoint OAuth interrogé par agentdeck (token-driven, portable) ;
    //  2. cache claude-statusbar (si présent) ;
    //  3. repli : comptage local "estimé".
    let real_api = store.real.lock().ok().and_then(|g| *g);
    if let Some(r) = real_api {
        return UsageSnapshot {
            five_h: bar_pct(r.five_h_pct, r.five_h_reset),
            week: bar_pct(r.week_pct, r.week_reset),
            five_h_cost,
            week_cost,
            source: "real".into(),
        };
    }
    if let Some((five_h, week)) = real_rate_limits() {
        return UsageSnapshot {
            five_h,
            week,
            five_h_cost,
            week_cost,
            source: "real".into(),
        };
    }
    let five_h_tokens: u64 = in_window(five_h_cut).map(|e| e.tokens).sum();
    let week_tokens: u64 = in_window(week_cut).map(|e| e.tokens).sum();
    UsageSnapshot {
        five_h: bar(five_h_tokens, data.five_h_cap),
        week: bar(week_tokens, data.week_cap),
        five_h_cost,
        week_cost,
        source: "estimated".into(),
    }
}
