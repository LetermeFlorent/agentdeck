// Helpers partagés par les adapters « one-shot » (opencode, Gemini) : un process par message,
// sortie NDJSON sur stdout, mappée vers `events::SessionEvent`. Détection de limite de débit
// (free tier) factorisée ici. + résolution des binaires CLI installés via npm.

use std::path::PathBuf;
use tokio::process::Command;

/// Racine npm global (Windows : %APPDATA%\npm = `dirs::data_dir()/npm`).
pub fn npm_root() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("npm"))
}

/// Commande Gemini : `node <bundle/gemini.js>` (le shim npm `.cmd` n'est pas exécutable
/// directement par CreateProcess). Repli : `gemini` sur le PATH.
pub fn gemini_command() -> Command {
    if let Some(js) = npm_root()
        .map(|r| {
            r.join("node_modules")
                .join("@google")
                .join("gemini-cli")
                .join("bundle")
                .join("gemini.js")
        })
        .filter(|p| p.exists())
    {
        let mut c = Command::new("node");
        c.arg(js);
        return c;
    }
    Command::new("gemini")
}

/// Commande opencode : binaire natif `opencode-ai/bin/opencode.exe`. Repli : `opencode` sur le PATH.
pub fn opencode_command() -> Command {
    if let Some(exe) = npm_root()
        .map(|r| r.join("node_modules").join("opencode-ai").join("bin").join("opencode.exe"))
        .filter(|p| p.exists())
    {
        return Command::new(exe);
    }
    Command::new("opencode")
}

/// Vrai si la sortie/erreur/exit indique une limite de débit temporaire (free tier).
/// Sert à exclure le modèle des candidats (auto) avec un cooldown, sans nom de modèle codé en dur.
pub fn is_rate_limited(stderr_tail: &str, exit_code: Option<i32>, json_err: Option<&str>) -> bool {
    let hay = format!("{stderr_tail}\n{}", json_err.unwrap_or("")).to_ascii_lowercase();
    // Signaux Gemini (RESOURCE_EXHAUSTED / 429) + opencode (propagé du provider sous-jacent).
    let textual = hay.contains("resource_exhausted")
        || hay.contains("resource exhausted")
        || hay.contains("ratelimitexceeded")
        || hay.contains("rate limit")
        || hay.contains("rate_limit")
        || hay.contains("too many requests")
        || hay.contains(" 429")
        || hay.contains("\"429\"")
        || hay.contains("quota");
    // Gemini : exit 1 = erreur API (429 atterrit ici), mais on n'en fait un rate-limit que si le
    // texte le confirme (exit 1 seul est trop ambigu).
    let _ = exit_code;
    textual
}

/// Préfixe stable posé sur `SessionEvent::Error` pour qu'un rate-limit soit reconnu côté frontend
/// (→ marque le modèle indisponible avec cooldown + bascule auto). Réutilise l'event Error existant
/// (pas de nouveau variant dans `events.rs`).
pub const RATE_LIMIT_PREFIX: &str = "RATE_LIMIT: ";
