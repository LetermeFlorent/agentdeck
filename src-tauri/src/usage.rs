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

pub struct UsageStore {
    inner: Mutex<UsageData>,
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

    // Préférence : vraies limites d'abonnement (claude-statusbar). Repli : comptage local.
    match real_rate_limits() {
        Some((five_h, week)) => UsageSnapshot {
            five_h,
            week,
            five_h_cost,
            week_cost,
            source: "real".into(),
        },
        None => {
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
    }
}
