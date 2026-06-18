// Auth: stockage sécurisé du token OAuth Claude Code dans le coffre OS (Windows Credential Manager via keyring).
// Le token n'est jamais écrit en clair sur disque ; il est injecté en env CLAUDE_CODE_OAUTH_TOKEN au spawn.

use keyring::Entry;

const SERVICE: &str = "agentdeck";
const ACCOUNT: &str = "claude_code_oauth_token";

fn entry() -> Result<Entry, String> {
    Entry::new(SERVICE, ACCOUNT).map_err(|e| format!("keyring: {e}"))
}

/// Récupère le token stocké, s'il existe.
pub fn get_token() -> Option<String> {
    match entry() {
        Ok(e) => e.get_password().ok(),
        Err(_) => None,
    }
}

/// Vrai si Claude Code a déjà des credentials valides (connexion native via navigateur),
/// même si agentdeck n'a pas de token dans son coffre. Dans ce cas, `claude` utilise ses
/// propres credentials et on n'a pas besoin d'injecter de token.
pub fn claude_logged_in() -> bool {
    let mut p = match dirs::home_dir() {
        Some(p) => p,
        None => return false,
    };
    p.push(".claude");
    p.push(".credentials.json");
    let raw = match std::fs::read_to_string(p) {
        Ok(r) => r,
        Err(_) => return false,
    };
    serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| {
            v.get("claudeAiOauth")
                .and_then(|o| o.get("accessToken"))
                .and_then(|x| x.as_str())
                .map(|s| !s.is_empty())
        })
        .unwrap_or(false)
}

/// Stocke le token (écrase l'existant).
pub fn set_token(token: &str) -> Result<(), String> {
    let token = token.trim();
    if !looks_like_oauth(token) {
        return Err("Format de token inattendu (attendu: sk-ant-oat01-…).".into());
    }
    entry()?
        .set_password(token)
        .map_err(|e| format!("keyring set: {e}"))
}

/// Supprime le token (déconnexion).
pub fn clear_token() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(_) => Ok(()),
        // Pas d'entrée à supprimer = déjà déconnecté, on considère OK.
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("keyring delete: {e}")),
    }
}

/// Heuristique de validation de format d'un token OAuth Claude Code.
pub fn looks_like_oauth(token: &str) -> bool {
    token.starts_with("sk-ant-oat") && token.len() > 30
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
