// Suivi d'usage 5h / semaine.
//
// ⚠️ Aucune API publique ne donne les vrais % d'abonnement (5h/hebdo) ; `/usage` est interactif
// seulement. On fournit donc un comptage LOCAL fiable (fenêtres glissantes 5h/7j, source
// "estimé"), complété si possible par les vrais chiffres (endpoint OAuth ou cache statusbar).
//
// mod.rs : types partagés + magasin. `oauth` : sources « réelles ». `report` : agrégation.

mod oauth;
mod report;

pub use oauth::run_poller;
pub use report::{record, snapshot, snapshot_for};

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const FIVE_H_SECS: u64 = 5 * 60 * 60;
pub const WEEK_SECS: u64 = 7 * 24 * 60 * 60;

// Plafonds par défaut (ajustables) — estimations, faute de chiffres officiels exposés.
pub const DEFAULT_FIVE_H_CAP: u64 = 5_000_000;
pub const DEFAULT_WEEK_CAP: u64 = 50_000_000;

#[derive(Clone, Serialize, Deserialize)]
struct Entry {
    ts: u64, // epoch secondes
    tokens: u64,
    #[serde(default)]
    cost: f64,
    #[serde(default = "default_provider")]
    provider: String,
}

fn default_provider() -> String {
    "claude_code".into()
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
    /// "estimated" (comptage local) ou "real" (endpoint OAuth / cache statusbar).
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
    Bar { tokens, cap, pct, resets_at: None }
}

fn bar_pct(used_pct: f64, resets_at: i64) -> Bar {
    Bar {
        tokens: 0,
        cap: 0,
        pct: used_pct.round().clamp(0.0, 100.0) as u32,
        resets_at: Some(resets_at),
    }
}
