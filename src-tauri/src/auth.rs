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
