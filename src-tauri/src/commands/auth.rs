// Commandes d'authentification (token OAuth Claude).

use crate::auth;
use crate::provider;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub fn auth_status() -> bool {
    // Connecté si on a un token dans le coffre OU si Claude Code est déjà connecté nativement.
    auth::get_token().is_some() || auth::claude_logged_in()
}

#[tauri::command]
pub fn auth_set_token(token: String) -> Result<(), String> {
    auth::set_token(&token)
}

#[tauri::command]
pub fn auth_clear() -> Result<(), String> {
    auth::clear_token()
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
    auth::set_token(&token)
}

/// Connexion navigateur, sans terminal : lance `claude setup-token`, **ouvre automatiquement**
/// l'URL d'autorisation qu'il imprime dans le navigateur, puis **capte le token** renvoyé au retour
/// (callback) et le stocke (chiffré, coffre Windows). Timeout pour ne jamais geler.
#[tauri::command]
pub async fn auth_login(app: tauri::AppHandle) -> Result<(), String> {
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
        let mut stream_open = true;
        loop {
            if stream_open {
                tokio::select! {
                    line = rx.recv() => match line {
                        Some(line) => {
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
                                return auth::set_token(&tok);
                            }
                        }
                        None => stream_open = false,
                    },
                    _ = tokio::time::sleep(std::time::Duration::from_millis(800)) => {}
                }
            } else {
                tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            }
            // Connexion native de Claude Code détectée (credentials écrits par le callback) → OK.
            if auth::claude_logged_in() {
                return Ok(());
            }
        }
    };

    let res = match tokio::time::timeout(std::time::Duration::from_secs(240), work).await {
        Ok(r) => r,
        Err(_) => Err("Délai dépassé — réessaie ou colle un token manuellement.".to_string()),
    };
    let _ = child.start_kill();
    res
}
