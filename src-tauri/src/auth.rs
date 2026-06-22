// Auth: stockage sécurisé du token OAuth Claude Code dans le coffre OS (Windows Credential Manager via keyring).
// Le token n'est jamais écrit en clair sur disque ; il est injecté en env CLAUDE_CODE_OAUTH_TOKEN au spawn.

use keyring::Entry;

use crate::provider::Provider;

const SERVICE: &str = "agentdeck";

/// Compte keyring par provider. Seul Claude stocke réellement un token ici ; opencode et Gemini
/// s'authentifient via leur propre CLI (fichiers de creds) — voir `is_connected`.
fn account(p: Provider) -> &'static str {
    match p {
        Provider::ClaudeCode => "claude_code_oauth_token",
        Provider::Opencode => "opencode_oauth_token",
        Provider::Gemini => "gemini_oauth_token",
    }
}

fn entry(p: Provider) -> Result<Entry, String> {
    Entry::new(SERVICE, account(p)).map_err(|e| format!("keyring: {e}"))
}

/// Récupère le token stocké pour ce provider, s'il existe (Claude uniquement en pratique).
/// Pour Claude : coffre agentdeck → sinon fallback sur `~/.claude/.credentials.json` (token CLI).
pub fn get_token(p: Provider) -> Option<String> {
    let keyring_token = match entry(p) {
        Ok(e) => e.get_password().ok(),
        Err(_) => None,
    };
    if keyring_token.is_some() {
        return keyring_token;
    }
    if p == Provider::ClaudeCode {
        return claude_cli_token();
    }
    None
}

/// Lit le token OAuth depuis `~/.claude/.credentials.json` (créé par le CLI Claude Code).
fn claude_cli_token() -> Option<String> {
    let path = dirs::home_dir()?.join(".claude").join(".credentials.json");
    let raw = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let token = v["claudeAiOauth"]["accessToken"].as_str()?;
    if looks_like_oauth(token) { Some(token.to_string()) } else { None }
}

/// Stocke le token (écrase l'existant). Validation de format réservée à Claude.
pub fn set_token(p: Provider, token: &str) -> Result<(), String> {
    let token = token.trim();
    if p == Provider::ClaudeCode && !looks_like_oauth(token) {
        return Err("Format de token inattendu (attendu: sk-ant-oat01-…).".into());
    }
    entry(p)?
        .set_password(token)
        .map_err(|e| format!("keyring set: {e}"))
}

/// Supprime le token (déconnexion).
pub fn clear_token(p: Provider) -> Result<(), String> {
    match entry(p)?.delete_credential() {
        Ok(_) => Ok(()),
        // Pas d'entrée à supprimer = déjà déconnecté, on considère OK.
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("keyring delete: {e}")),
    }
}

/// Connecté pour ce provider ?
/// - Claude : token dans le coffre (connexion agentdeck indépendante).
/// - Gemini : creds OAuth `~/.gemini/oauth_creds.json` OU `$GEMINI_API_KEY`.
/// - opencode : `~/.local/share/opencode/auth.json` non vide.
pub fn is_connected(p: Provider) -> bool {
    match p {
        Provider::ClaudeCode => get_token(p).is_some(),
        Provider::Gemini => gemini_creds_exist(),
        Provider::Opencode => opencode_auth_exists(),
    }
}

/// Connecté à Gemini = une clé API RÉELLEMENT utilisable.
/// ⚠️ On NE se fie PAS à `~/.gemini/oauth_creds.json` : ce fichier peut exister mais être
/// inutilisable (le tier OAuth « gratuit individuel » de gemini-cli est arrêté → IneligibleTier).
/// → on vérifie : coffre agentdeck, `$GEMINI_API_KEY`, ou clé écrite dans `~/.gemini/.env`.
fn gemini_creds_exist() -> bool {
    if get_token(Provider::Gemini).is_some() {
        return true;
    }
    if std::env::var("GEMINI_API_KEY").map(|v| !v.is_empty()).unwrap_or(false) {
        return true;
    }
    gemini_env_has_key()
}

/// Lit `~/.gemini/.env` et renvoie vrai si une `GEMINI_API_KEY` non vide y est définie.
fn gemini_env_has_key() -> bool {
    dirs::home_dir()
        .and_then(|mut p| {
            p.push(".gemini");
            p.push(".env");
            std::fs::read_to_string(p).ok()
        })
        .map(|s| {
            s.lines().any(|l| {
                let l = l.trim();
                (l.starts_with("GEMINI_API_KEY") || l.starts_with("GOOGLE_API_KEY"))
                    && l.split('=').nth(1).map(|v| !v.trim().is_empty()).unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Écrit la clé API Gemini dans la config NATIVE du CLI (`~/.gemini/.env`) pour que `gemini`
/// l'utilise lui-même (pas seulement via notre injection d'env). Met à jour la ligne si présente.
pub fn write_gemini_env(key: &str) -> Result<(), String> {
    let mut dir = dirs::home_dir().ok_or("Dossier home introuvable.")?;
    dir.push(".gemini");
    std::fs::create_dir_all(&dir).map_err(|e| format!("création ~/.gemini : {e}"))?;
    let path = dir.join(".env");
    let mut lines: Vec<String> = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim_start().starts_with("GEMINI_API_KEY"))
        .map(String::from)
        .collect();
    lines.push(format!("GEMINI_API_KEY={key}"));
    std::fs::write(&path, lines.join("\n") + "\n").map_err(|e| format!("écriture ~/.gemini/.env : {e}"))
}

fn opencode_auth_exists() -> bool {
    dirs::home_dir()
        .map(|mut p| {
            p.push(".local");
            p.push("share");
            p.push("opencode");
            p.push("auth.json");
            // Non vide = au moins un provider configuré.
            std::fs::read_to_string(&p)
                .map(|s| s.trim().len() > 2)
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// Heuristique de validation de format d'un token OAuth Claude Code.
/// Préfixe `sk-ant-oat`, longueur dans une plage réaliste, et seulement des caractères
/// attendus (base64url + tirets) — évite d'accepter une ligne de texte arbitraire qui
/// commencerait par le préfixe.
pub fn looks_like_oauth(token: &str) -> bool {
    token.starts_with("sk-ant-oat")
        && (40..=500).contains(&token.len())
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Extrait un token OAuth depuis le contenu d'un fichier texte
/// (ex. le fichier claude-token.txt du dossier Téléchargements).
pub fn extract_token_from_text(content: &str) -> Option<String> {
    for raw in content.split(|c: char| c.is_whitespace() || c == '"' || c == '\'') {
        let t = raw.trim();
        if looks_like_oauth(t) {
            return Some(t.to_string());
        }
    }
    None
}
