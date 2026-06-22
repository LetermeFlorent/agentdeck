// Agrégation : enregistrement des tours + snapshots (Claude Code 5h/7j, providers tiers 24h).
// Priorité des sources « réelles » : endpoint OAuth > cache statusbar > comptage local estimé.

use super::oauth::real_rate_limits;
use super::{
    bar, now, UsageSnapshot, UsageStore, Entry, DEFAULT_FIVE_H_CAP, DEFAULT_WEEK_CAP, FIVE_H_SECS,
    WEEK_SECS,
};

const DAY_SECS: u64 = 24 * 60 * 60;
// Gemini 2.5 Flash free tier : ~250K tokens/requête. Cap journalier conservateur (le vrai cap
// est en RPD pas en TPD, mais c'est la meilleure approximation sans API quota).
const GEMINI_DAY_CAP: u64 = 250_000;

/// Enregistre la conso d'un tour terminé (tokens + coût) et purge les entrées > 7j.
pub fn record(store: &UsageStore, provider: &str, tokens: u64, cost: f64) {
    if tokens == 0 && cost == 0.0 {
        return;
    }
    let mut data = match store.inner.lock() {
        Ok(d) => d,
        Err(_) => return,
    };
    let t = now();
    data.entries.push(Entry { ts: t, tokens, cost, provider: provider.to_string() });
    let cutoff = t.saturating_sub(WEEK_SECS);
    data.entries.retain(|e| e.ts >= cutoff);
    UsageStore::persist(&data);
}

/// Snapshot pour un provider tiers (gemini, opencode) : fenêtre 24h uniquement.
pub fn snapshot_for(store: &UsageStore, provider: &str) -> UsageSnapshot {
    let estimated = |fh: super::Bar| UsageSnapshot {
        five_h: fh,
        week: bar(0, 0),
        five_h_cost: 0.0,
        week_cost: 0.0,
        source: "estimated".into(),
    };
    let data = match store.inner.lock() {
        Ok(d) => d,
        Err(_) => return estimated(bar(0, 0)),
    };
    let t = now();
    let day_cut = t.saturating_sub(DAY_SECS);
    let (day_tokens, day_cost) = data
        .entries
        .iter()
        .filter(|e| e.provider == provider && e.ts >= day_cut)
        .fold((0u64, 0.0f64), |(tok, cost), e| (tok + e.tokens, cost + e.cost));
    let cap = match provider {
        "gemini" => GEMINI_DAY_CAP,
        _ => 0, // opencode : informatif seulement
    };
    UsageSnapshot {
        five_h: bar(day_tokens, cap),
        week: bar(0, 0),
        five_h_cost: day_cost,
        week_cost: 0.0,
        source: "estimated".into(),
    }
}

/// Calcule les barres 5h / semaine (Claude Code).
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
    // Une seule passe sur les entrées Claude Code (les autres providers ont leur propre snapshot).
    let (five_h_cost, week_cost, five_h_tokens, week_tokens) = data
        .entries
        .iter()
        .filter(|e| e.provider == "claude_code")
        .fold((0.0_f64, 0.0_f64, 0_u64, 0_u64), |(fhc, wc, fht, wt), e| {
            let in_5h = e.ts >= five_h_cut;
            let in_week = e.ts >= week_cut;
            (
                if in_5h { fhc + e.cost } else { fhc },
                if in_week { wc + e.cost } else { wc },
                if in_5h { fht + e.tokens } else { fht },
                if in_week { wt + e.tokens } else { wt },
            )
        });

    let real_api = store.real.lock().ok().and_then(|g| *g);
    if let Some(r) = real_api {
        return UsageSnapshot {
            five_h: super::bar_pct(r.five_h_pct, r.five_h_reset),
            week: super::bar_pct(r.week_pct, r.week_reset),
            five_h_cost,
            week_cost,
            source: "real".into(),
        };
    }
    if let Some((five_h, week)) = real_rate_limits() {
        return UsageSnapshot { five_h, week, five_h_cost, week_cost, source: "real".into() };
    }
    UsageSnapshot {
        five_h: bar(five_h_tokens, data.five_h_cap),
        week: bar(week_tokens, data.week_cap),
        five_h_cost,
        week_cost,
        source: "estimated".into(),
    }
}
