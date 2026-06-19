// Historique des conversations : liste les sessions Claude Code (~/.claude/projects/*/*.jsonl),
// les plus récentes d'abord, et reconstruit les messages d'une session pour réaffichage.

use serde_json::Value;
use std::path::PathBuf;

#[derive(serde::Serialize, Clone)]
pub struct SessionHist {
    pub id: String,
    pub title: String,
    pub cwd: String,
    /// Date de dernière modif (epoch secondes) — pour le tri et l'affichage.
    pub ts: u64,
}

#[derive(serde::Serialize, Clone)]
pub struct HistMsg {
    pub role: String, // "user" | "assistant"
    pub text: String,
}

fn projects_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("projects"))
}

/// Vrai si un texte utilisateur est du « bruit » (caveat, commande locale, system-reminder…).
fn is_noise(t: &str) -> bool {
    let t = t.trim_start();
    t.is_empty()
        || t.starts_with("<local-command")
        || t.starts_with("<command-")
        || t.starts_with("<system-reminder")
        || t.starts_with("Caveat:")
        || t.starts_with("[Request interrupted")
        || t.starts_with("<bash-")
}

/// Extrait le texte d'un `message.content` (string OU tableau de blocs).
fn extract_text(content: &Value) -> String {
    if let Some(s) = content.as_str() {
        return s.to_string();
    }
    if let Some(arr) = content.as_array() {
        return arr
            .iter()
            .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
            .filter_map(|b| b.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("");
    }
    String::new()
}

/// Lit (titre, cwd) d'une session : 1ᵉʳ vrai message user + cwd rencontré.
fn read_meta(path: &std::path::Path) -> (String, String) {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut title = String::new();
    let mut cwd = String::new();
    for line in content.lines().take(60) {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        if cwd.is_empty() {
            if let Some(c) = v.get("cwd").and_then(Value::as_str) {
                cwd = c.to_string();
            }
        }
        if title.is_empty()
            && v.get("type").and_then(Value::as_str) == Some("user")
        {
            if let Some(msg) = v.get("message") {
                let t = extract_text(msg.get("content").unwrap_or(&Value::Null));
                if !is_noise(&t) {
                    title = t.chars().take(80).collect::<String>().replace('\n', " ");
                }
            }
        }
        if !title.is_empty() && !cwd.is_empty() {
            break;
        }
    }
    if title.is_empty() {
        title = "(sans titre)".into();
    }
    (title, cwd)
}

/// Liste les `limit` sessions les plus récentes (toutes confondues), titre + date.
#[tauri::command]
pub fn recent_sessions(limit: usize) -> Vec<SessionHist> {
    let dir = match projects_dir() {
        Some(d) => d,
        None => return vec![],
    };
    // Récupère tous les .jsonl avec leur mtime.
    let mut files: Vec<(PathBuf, u64)> = vec![];
    let walk = |d: &PathBuf, files: &mut Vec<(PathBuf, u64)>| {
        if let Ok(rd) = std::fs::read_dir(d) {
            for proj in rd.flatten().filter(|e| e.path().is_dir()) {
                if let Ok(inner) = std::fs::read_dir(proj.path()) {
                    for f in inner.flatten() {
                        let p = f.path();
                        if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                            let ts = f
                                .metadata()
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            files.push((p, ts));
                        }
                    }
                }
            }
        }
    };
    walk(&dir, &mut files);
    files.sort_by(|a, b| b.1.cmp(&a.1));
    files.truncate(limit.clamp(1, 200));
    files
        .into_iter()
        .map(|(p, ts)| {
            let id = p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let (title, cwd) = read_meta(&p);
            SessionHist { id, title, cwd, ts }
        })
        .collect()
}

/// Trouve le chemin du .jsonl d'une session par son id (recherche dans tous les projets).
fn find_session(id: &str) -> Option<PathBuf> {
    let dir = projects_dir()?;
    let file = format!("{id}.jsonl");
    std::fs::read_dir(&dir).ok()?.flatten().find_map(|proj| {
        let p = proj.path().join(&file);
        p.exists().then_some(p)
    })
}

/// Reconstruit les messages (user/assistant, texte) d'une session pour réaffichage.
/// Borné aux derniers messages pour éviter de charger des Mo dans l'UI.
#[tauri::command]
pub fn load_messages(id: String) -> Vec<HistMsg> {
    let path = match find_session(&id) {
        Some(p) => p,
        None => return vec![],
    };
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let mut msgs: Vec<HistMsg> = vec![];
    for line in content.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        let role = match v.get("type").and_then(Value::as_str) {
            Some("user") => "user",
            Some("assistant") => "assistant",
            _ => continue,
        };
        let Some(msg) = v.get("message") else { continue };
        let text = extract_text(msg.get("content").unwrap_or(&Value::Null));
        let text = text.trim().to_string();
        if text.is_empty() || (role == "user" && is_noise(&text)) {
            continue;
        }
        msgs.push(HistMsg { role: role.to_string(), text });
    }
    // Ne garde que les derniers (UI), pour les très longues sessions.
    let n = msgs.len();
    if n > 300 {
        msgs.drain(0..n - 300);
    }
    msgs
}
