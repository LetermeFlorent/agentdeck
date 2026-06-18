// agentdeck — backend Tauri.
// Pilote plusieurs sessions Claude Code (multi-IA à terme) et expose les commandes au frontend.

mod auth;
mod events;
mod provider;
mod session;
mod usage;

use session::{SessionInfo, SessionManager};
use usage::{UsageSnapshot, UsageStore};

// ---------- Auth ----------

#[tauri::command]
fn auth_status() -> bool {
    auth::get_token().is_some()
}

#[tauri::command]
fn auth_set_token(token: String) -> Result<(), String> {
    auth::set_token(&token)
}

#[tauri::command]
fn auth_clear() -> Result<(), String> {
    auth::clear_token()
}

/// Importe un token depuis un fichier texte. Sans chemin, cherche
/// `<Téléchargements>/claude-token.txt`.
#[tauri::command]
fn auth_import_from_file(path: Option<String>) -> Result<(), String> {
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

/// Voie A : connexion Anthropic de base — lance `claude setup-token` (ouvre le navigateur),
/// récupère le token OAuth émis et le stocke. Expérimental (dépend de l'interactivité du CLI).
#[tauri::command]
async fn auth_login() -> Result<(), String> {
    let output = tokio::process::Command::new(provider::claude_code::claude_bin())
        .arg("setup-token")
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "Binaire `claude` introuvable dans le PATH.".to_string()
            } else {
                format!("Échec de `claude setup-token` : {e}")
            }
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let token = auth::extract_token_from_text(&stdout)
        .or_else(|| auth::extract_token_from_text(&stderr))
        .ok_or_else(|| {
            "Pas de token récupéré. Utilise plutôt l'import du fichier token.".to_string()
        })?;
    auth::set_token(&token)
}

/// Récupère la liste des commandes slash de Claude Code en lisant le message `init`
/// d'un process `claude` (aucun message envoyé → 0 token), puis tue le process.
#[tauri::command]
async fn slash_commands() -> Vec<String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let token = match auth::get_token() {
        Some(t) => t,
        None => return vec![],
    };
    let mut cmd = tokio::process::Command::new(provider::claude_code::claude_bin());
    cmd.arg("-p")
        .arg("--input-format")
        .arg("stream-json")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .env("CLAUDE_CODE_OAUTH_TOKEN", &token)
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let _stdin = child.stdin.take(); // garde stdin ouvert (n'envoie rien)
    let stdout = child.stdout.take();

    let read = async {
        if let Some(out) = stdout {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                    if v.get("type").and_then(|x| x.as_str()) == Some("system")
                        && v.get("subtype").and_then(|x| x.as_str()) == Some("init")
                    {
                        if let Some(arr) = v.get("slash_commands").and_then(|x| x.as_array()) {
                            return arr
                                .iter()
                                .filter_map(|c| c.as_str().map(String::from))
                                .collect::<Vec<_>>();
                        }
                    }
                }
            }
        }
        Vec::new()
    };
    let cmds = tokio::time::timeout(std::time::Duration::from_secs(20), read)
        .await
        .unwrap_or_default();
    let _ = child.start_kill();
    cmds
}

// ---------- Dépendance : Claude Code CLI ----------

/// true si le binaire `claude` répond (`--version`).
#[tauri::command]
async fn check_claude() -> bool {
    tokio::process::Command::new(provider::claude_code::claude_bin())
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Installe Claude Code via l'installeur natif officiel (Windows PowerShell).
#[tauri::command]
async fn install_claude() -> Result<(), String> {
    let out = tokio::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "irm https://claude.ai/install.ps1 | iex",
        ])
        .output()
        .await
        .map_err(|e| format!("Lancement installeur : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

// ---------- Sessions ----------

#[tauri::command]
fn session_create(
    mgr: tauri::State<'_, SessionManager>,
    title: Option<String>,
    cwd: Option<String>,
    model: Option<String>,
) -> String {
    mgr.create(title, cwd, model)
}

#[tauri::command]
fn session_list(mgr: tauri::State<'_, SessionManager>) -> Vec<SessionInfo> {
    mgr.list()
}

#[tauri::command]
fn session_restore(
    mgr: tauri::State<'_, SessionManager>,
    id: String,
    title: Option<String>,
    started: bool,
    cwd: Option<String>,
    model: Option<String>,
) {
    mgr.restore(id, title, started, cwd, model);
}

#[tauri::command]
fn session_send(
    app: tauri::AppHandle,
    mgr: tauri::State<'_, SessionManager>,
    id: String,
    text: String,
    model: Option<String>,
    effort: Option<String>,
) -> Result<(), String> {
    let token = auth::get_token().ok_or_else(|| "Non connecté (aucun token).".to_string())?;
    let (proc, cwd) = mgr
        .send_ctx(&id)
        .ok_or_else(|| "Session inconnue.".to_string())?;
    // Écrit le message sur le stdin du process persistant (pris en cours de route si Claude bosse).
    tauri::async_runtime::spawn(provider::claude_code::send(
        app, id, proc, cwd, model, effort, token, text,
    ));
    Ok(())
}

#[tauri::command]
async fn session_stop(mgr: tauri::State<'_, SessionManager>, id: String) -> Result<(), String> {
    if let Some(proc) = mgr.proc_handle(&id) {
        let mut slot = proc.lock().await;
        if let Some(p) = slot.take() {
            let mut c = p.child;
            let _ = c.start_kill();
        }
    }
    Ok(())
}

#[tauri::command]
async fn session_close(mgr: tauri::State<'_, SessionManager>, id: String) -> Result<(), String> {
    if let Some(proc) = mgr.remove(&id) {
        let mut slot = proc.lock().await;
        if let Some(p) = slot.take() {
            let mut c = p.child;
            let _ = c.start_kill();
        }
    }
    Ok(())
}

// ---------- Usage ----------

#[tauri::command]
fn usage_get(store: tauri::State<'_, UsageStore>) -> UsageSnapshot {
    usage::snapshot(&store)
}

/// Modèle / effort par défaut pour un nouveau pane : on récupère ceux du Claude Code
/// courant (cache statusline) ; sinon valeurs par défaut (Opus / medium).
#[tauri::command]
fn claude_defaults() -> serde_json::Value {
    let read = || -> Option<(String, String)> {
        let mut p = dirs::home_dir()?;
        p.push(".cache");
        p.push("claude-statusbar");
        p.push("last_stdin.json");
        let raw = std::fs::read_to_string(p).ok()?;
        let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let model_id = v
            .get("model")
            .and_then(|m| m.get("id"))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        let alias = if model_id.contains("opus") {
            "opus"
        } else if model_id.contains("sonnet") {
            "sonnet"
        } else if model_id.contains("haiku") {
            "haiku"
        } else if model_id.contains("fable") {
            "fable"
        } else {
            ""
        };
        let effort = v
            .get("effort")
            .and_then(|e| e.get("level"))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        Some((alias.to_string(), effort.to_string()))
    };
    let (mut model, mut effort) = read().unwrap_or_default();
    if model.is_empty() {
        model = "opus".into();
    }
    if effort.is_empty() {
        effort = "medium".into();
    }
    serde_json::json!({ "model": model, "effort": effort })
}

/// Plan d'abonnement (lu dans ~/.claude/.credentials.json). `level` 0–4 pilote l'effet visuel.
#[tauri::command]
fn subscription_plan() -> serde_json::Value {
    let read = || -> Option<(String, String)> {
        let mut p = dirs::home_dir()?;
        p.push(".claude");
        p.push(".credentials.json");
        let raw = std::fs::read_to_string(p).ok()?;
        let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
        let o = v.get("claudeAiOauth")?;
        let sub = o
            .get("subscriptionType")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let tier = o
            .get("rateLimitTier")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        Some((sub, tier))
    };
    let (sub, tier) = read().unwrap_or_default();
    let (label, level): (String, u8) = if tier.contains("max_20x") {
        ("Max 20×".into(), 4)
    } else if tier.contains("max_5x") {
        ("Max 5×".into(), 3)
    } else if tier.contains("pro") {
        ("Pro".into(), 1)
    } else {
        match sub.as_str() {
            "max" => ("Max".into(), 3),
            "pro" => ("Pro".into(), 1),
            "team" => ("Team".into(), 4),
            "enterprise" => ("Enterprise".into(), 4),
            "" => (String::new(), 0),
            other => {
                let mut c = other.chars();
                let cap = c
                    .next()
                    .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                    .unwrap_or_default();
                (cap, 2)
            }
        }
    };
    serde_json::json!({ "label": label, "level": level })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(SessionManager::default())
        .manage(UsageStore::load())
        .setup(|app| {
            // Poller de fond : vrai usage d'abonnement via l'endpoint OAuth.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(usage::run_poller(handle));
            // Met à jour Claude Code au démarrage (best-effort, non bloquant).
            tauri::async_runtime::spawn(async {
                let _ = tokio::process::Command::new(provider::claude_code::claude_bin())
                    .arg("update")
                    .output()
                    .await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_status,
            auth_set_token,
            auth_clear,
            auth_import_from_file,
            auth_login,
            session_create,
            session_list,
            session_restore,
            session_send,
            session_stop,
            session_close,
            usage_get,
            claude_defaults,
            subscription_plan,
            check_claude,
            install_claude,
            slash_commands,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
