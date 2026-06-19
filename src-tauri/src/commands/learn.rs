// Mode Hermes — réflexion automatique sur échec : un appel Haiku headless résume l'erreur
// en une leçon réutilisable, décide la portée (global / projet), et écrit le SKILL.md.

use crate::auth;
use crate::provider;

/// Nom sûr (anti-traversée de chemin) — même règle que `library::safe_name`.
fn safe_name(name: &str) -> Option<String> {
    let n = name.trim();
    if n.is_empty()
        || n.len() > 100
        || n.contains('/')
        || n.contains('\\')
        || n.contains("..")
        || n.starts_with('.')
    {
        return None;
    }
    Some(n.to_string())
}

/// Slugifie un nom libre en kebab-case sûr.
fn slugify(name: &str) -> String {
    let s: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    s.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-")
}

/// Réfléchit sur une erreur via Haiku et écrit un skill (global ou projet).
/// Renvoie le chemin du SKILL.md écrit, ou une erreur. Best-effort : ne bloque jamais l'app.
#[tauri::command]
pub async fn reflect_and_learn(
    cwd: Option<String>,
    request: String,
    summary: String,
    error: String,
) -> Result<String, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    // Connexion agentdeck indépendante : il faut notre propre token dans le coffre
    // (une session `claude` native ne suffit pas).
    let token = auth::get_token();
    if token.is_none() {
        return Err("Non connecté.".into());
    }

    let prompt = format!(
        "Un agent de code vient d'échouer sur une tâche. Déduis-en UNE leçon réutilisable pour \
ne pas répéter l'erreur, formulée comme un skill. Réponds STRICTEMENT en JSON sur une ligne, \
sans texte autour, au format : \
{{\"scope\":\"global|project\",\"name\":\"nom-kebab-case\",\"description\":\"quand utiliser ce skill\",\"body\":\"instructions en Markdown\"}}. \
Mets scope=\"project\" si la leçon est spécifique à ce dépôt, sinon \"global\". \
Demande de l'utilisateur : {request}\nActions / réponse de l'agent : {summary}\nErreur rencontrée : {error}"
    );
    let msg = serde_json::json!({
        "type": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": prompt }] }
    })
    .to_string();

    let mut cmd = tokio::process::Command::new(provider::claude_code::claude_bin());
    cmd.arg("-p")
        .arg("--input-format")
        .arg("stream-json")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--permission-mode")
        .arg("bypassPermissions")
        .arg("--model")
        .arg("haiku")
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    if let Some(t) = &token {
        cmd.env("CLAUDE_CODE_OAUTH_TOKEN", t);
    }

    let mut child = cmd.spawn().map_err(|e| format!("Lancement : {e}"))?;
    if let Some(mut si) = child.stdin.take() {
        let _ = si.write_all(msg.as_bytes()).await;
        let _ = si.write_all(b"\n").await;
        let _ = si.flush().await;
    }
    let stdout = child.stdout.take();

    // Lit jusqu'à la ligne `result` et récupère le texte final.
    let read = async {
        let mut text = String::new();
        if let Some(out) = stdout {
            let mut lines = BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                    if v.get("type").and_then(|x| x.as_str()) == Some("result") {
                        if let Some(r) = v.get("result").and_then(|x| x.as_str()) {
                            text = r.to_string();
                        }
                        break;
                    }
                }
            }
        }
        text
    };
    let text = tokio::time::timeout(std::time::Duration::from_secs(45), read)
        .await
        .unwrap_or_default();
    let _ = child.start_kill();

    // Extrait le bloc JSON {...} de la réponse.
    let json_str = {
        let start = text.find('{');
        let end = text.rfind('}');
        match (start, end) {
            (Some(s), Some(e)) if e > s => &text[s..=e],
            _ => return Err("Réflexion : pas de JSON exploitable".into()),
        }
    };
    let v: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("JSON réflexion invalide : {e}"))?;

    let raw_name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").trim();
    let name = safe_name(&slugify(raw_name)).ok_or("Nom de skill invalide")?;
    let description = v.get("description").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    let body = v.get("body").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    let scope = v.get("scope").and_then(|x| x.as_str()).unwrap_or("global");

    // Dossier cible : projet (<cwd>/.claude/skills) ou global (~/.claude/skills).
    let base = if scope == "project" {
        match cwd.as_deref().filter(|c| !c.is_empty()) {
            Some(c) => std::path::PathBuf::from(c).join(".claude").join("skills"),
            None => dirs::home_dir().ok_or("home introuvable")?.join(".claude").join("skills"),
        }
    } else {
        dirs::home_dir().ok_or("home introuvable")?.join(".claude").join("skills")
    };

    let dir = base.join(&name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Création dossier : {e}"))?;
    let content = format!("---\nname: {name}\ndescription: {description}\n---\n\n{body}\n");
    let path = dir.join("SKILL.md");
    std::fs::write(&path, content).map_err(|e| format!("Écriture : {e}"))?;
    Ok(path.display().to_string())
}
