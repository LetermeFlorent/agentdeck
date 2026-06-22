// Commandes d'authentification (token OAuth Claude).

use crate::auth;
use crate::provider::{self, Provider};
use tauri_plugin_opener::OpenerExt;

/// Provider depuis l'arg frontend (absent → Claude, back-compat).
fn prov(p: Option<String>) -> Provider {
    Provider::from_str(p.as_deref().unwrap_or("claude_code"))
}

#[tauri::command]
pub fn auth_status(provider: Option<String>) -> bool {
    // Connexion agentdeck INDÉPENDANTE : connecté selon le provider (token coffre pour Claude,
    // fichiers de creds CLI pour opencode/Gemini).
    auth::is_connected(prov(provider))
}

#[tauri::command]
pub fn auth_set_token(token: String, provider: Option<String>) -> Result<(), String> {
    let p = prov(provider);
    // Gemini : on écrit AUSSI la clé dans la config native du CLI (~/.gemini/.env) pour que
    // `gemini` soit réellement connecté (pas seulement via notre injection d'env au spawn).
    if p == Provider::Gemini {
        auth::write_gemini_env(token.trim())?;
    }
    auth::set_token(p, &token)
}

#[tauri::command]
pub fn auth_clear(provider: Option<String>) -> Result<(), String> {
    auth::clear_token(prov(provider))
}

/// Importe un token depuis un fichier texte. Sans chemin, cherche
/// `<Téléchargements>/claude-token.txt`.
#[tauri::command]
pub fn auth_import_from_file(path: Option<String>) -> Result<(), String> {
    let path = match path {
        Some(p) if !p.is_empty() => std::path::PathBuf::from(p),
        _ => {
            let mut p = dirs::download_dir()
                .ok_or_else(|| "Dossier Téléchargements introuvable.".to_string())?;
            p.push("claude-token.txt");
            p
        }
    };
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Lecture {} : {e}", path.display()))?;
    let token = auth::extract_token_from_text(&content)
        .ok_or_else(|| "Aucun token sk-ant-oat… trouvé dans le fichier.".to_string())?;
    auth::set_token(Provider::ClaudeCode, &token)
}

/// Connexion à un provider (dispatch). Claude : flux navigateur intégré (capte le token).
/// Gemini / opencode : on lance leur propre login (OAuth/clé) dans un terminal, puis l'UI
/// re-sonde `auth_status`. (La méthode « coller une clé » passe, elle, par `auth_set_token`.)
#[tauri::command]
pub async fn auth_login(app: tauri::AppHandle, provider: Option<String>) -> Result<(), String> {
    match prov(provider) {
        Provider::ClaudeCode => claude_login(app).await,
        Provider::Gemini => launch_terminal_login("gemini"),
        Provider::Opencode => launch_terminal_login("opencode auth login"),
    }
}

/// Ouvre un terminal exécutant la commande de login d'un CLI (Windows : `cmd /C start`).
/// Le CLI gère son propre flux OAuth/clé ; l'UI détecte ensuite la connexion via `auth_status`.
fn launch_terminal_login(cli_cmd: &str) -> Result<(), String> {
    let res = std::process::Command::new("cmd")
        .args(["/C", "start", "cmd", "/K", cli_cmd])
        .spawn();
    res.map(|_| ()).map_err(|e| format!("Ouverture du terminal de connexion : {e}"))
}

/// Ouvre un terminal sur le login NATIF du CLI choisi. Pour Claude, c'est l'option « en plus »
/// du token agentdeck : connecte le CLI `claude` lui-même (il ne partage pas notre token coffre).
#[tauri::command]
pub fn cli_terminal_login(provider: String) -> Result<(), String> {
    let cmd = match prov(Some(provider)) {
        Provider::ClaudeCode => "claude",
        Provider::Opencode => "opencode auth login",
        Provider::Gemini => "gemini",
    };
    launch_terminal_login(cmd)
}

/// Flux Claude : `claude setup-token`, ouvre l'URL, capte le token imprimé, le stocke.
async fn claude_login(app: tauri::AppHandle) -> Result<(), String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut child = tokio::process::Command::new(provider::claude_code::claude_bin())
        .arg("setup-token")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Binaire `claude` introuvable.".to_string()
            } else {
                format!("Échec de `claude setup-token` : {e}")
            }
        })?;

    // Fusionne stdout + stderr (l'URL ou le token peuvent tomber sur l'un ou l'autre).
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(64);
    if let Some(out) = child.stdout.take() {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(l)) = lines.next_line().await {
                if tx.send(l).await.is_err() {
                    break;
                }
            }
        });
    }
    if let Some(err) = child.stderr.take() {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(l)) = lines.next_line().await {
                if tx.send(l).await.is_err() {
                    break;
                }
            }
        });
    }
    drop(tx);

    let work = async {
        let mut opened = false;
        // Connexion INDÉPENDANTE : on ne valide que si `setup-token` imprime un token à nous.
        // Une session `claude` native n'est jamais considérée comme une connexion agentdeck.
        while let Some(line) = rx.recv().await {
            // Ouvre la 1ʳᵉ URL imprimée dans le navigateur.
            if !opened {
                if let Some(i) = line.find("https://") {
                    let url: String = line[i..]
                        .split(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    if url.len() > 8 {
                        let _ = app.opener().open_url(url, None::<String>);
                        opened = true;
                    }
                }
            }
            // Token imprimé (sk-ant-oat…) → on le stocke dans le coffre.
            if let Some(tok) = auth::extract_token_from_text(&line) {
                return auth::set_token(Provider::ClaudeCode, &tok);
            }
        }
        Err("Aucun token reçu. Réessaie ou colle un token manuellement.".to_string())
    };

    let res = match tokio::time::timeout(std::time::Duration::from_secs(240), work).await {
        Ok(r) => r,
        Err(_) => Err("Délai dépassé — réessaie ou colle un token manuellement.".to_string()),
    };
    let _ = child.start_kill();
    res
}
