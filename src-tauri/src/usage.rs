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
}

#[derive(Serialize)]
pub struct UsageSnapshot {
    pub five_h: Bar,
    pub week: Bar,
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
    Bar { tokens, cap, pct }
}

/// Enregistre les tokens d'un tour terminé et purge les entrées > 7j.
pub fn record(store: &UsageStore, input: u64, output: u64) {
    let total = input + output;
    if total == 0 {
        return;
    }
    let mut data = match store.inner.lock() {
        Ok(d) => d,
        Err(_) => return,
    };
    let t = now();
    data.entries.push(Entry {
        ts: t,
        tokens: total,
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
                source: "estimated".into(),
            }
        }
    };
    let t = now();
    let five_h_cut = t.saturating_sub(FIVE_H_SECS);
    let week_cut = t.saturating_sub(WEEK_SECS);
    let five_h_tokens: u64 = data
        .entries
        .iter()
        .filter(|e| e.ts >= five_h_cut)
        .map(|e| e.tokens)
        .sum();
    let week_tokens: u64 = data
        .entries
        .iter()
        .filter(|e| e.ts >= week_cut)
        .map(|e| e.tokens)
        .sum();
    UsageSnapshot {
        five_h: bar(five_h_tokens, data.five_h_cap),
        week: bar(week_tokens, data.week_cap),
        source: "estimated".into(),
    }
}
