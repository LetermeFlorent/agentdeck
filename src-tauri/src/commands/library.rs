// Gestion de la « bibliothèque » : skills (sur disque, même emplacement que le CLI :
// ~/.claude/skills) et serveurs MCP (via le CLI `claude mcp`). Tout est dynamique :
// les listes installées sont relues du disque / du CLI à chaque appel.

use crate::provider;

#[derive(serde::Serialize, Clone)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
}

#[derive(serde::Serialize, Clone)]
pub struct McpItem {
    pub name: String,
    pub target: String,
    pub status: String,
    /// Scope de config : local/user/project (supprimables ici) ou claudeai (géré sur le web).
    pub scope: String,
    /// Faux pour les connecteurs claude.ai (non supprimables via le CLI).
    pub removable: bool,
}

/// Nom sûr : empêche toute traversée de chemin (slash, backslash, "..").
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

/// Dossier des skills du CLI : ~/.claude/skills
fn skills_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("skills"))
}

/// Lit `description:` du frontmatter YAML d'un SKILL.md.
fn read_description(path: &std::path::Path) -> String {
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    let mut lines = raw.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return String::new();
    }
    for line in lines {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Some(rest) = t.strip_prefix("description:") {
            return rest.trim().trim_matches(['"', '\'']).trim().to_string();
        }
    }
    String::new()
}

/// Liste les skills installés (~/.claude/skills/*/SKILL.md), nom + description.
#[tauri::command]
pub fn skills_installed() -> Vec<SkillItem> {
    let dir = match skills_dir() {
        Some(d) => d,
        None => return vec![],
    };
    let rd = match std::fs::read_dir(&dir) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let mut out: Vec<SkillItem> = rd
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let skill_md = e.path().join("SKILL.md");
            if !skill_md.exists() {
                return None;
            }
            let name = e.file_name().to_string_lossy().to_string();
            Some(SkillItem { description: read_description(&skill_md), name })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Écrit un SKILL.md (création depuis le catalogue OU « ajouter le mien »).
/// Le frontend fournit le contenu complet du fichier.
#[tauri::command]
pub fn skill_write(name: String, content: String) -> Result<(), String> {
    let name = safe_name(&name).ok_or("Nom de skill invalide")?;
    let dir = skills_dir().ok_or("Dossier ~/.claude introuvable")?.join(&name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Création dossier : {e}"))?;
    std::fs::write(dir.join("SKILL.md"), content).map_err(|e| format!("Écriture : {e}"))?;
    Ok(())
}

/// Supprime un skill installé (dossier complet ~/.claude/skills/<name>).
#[tauri::command]
pub fn skill_delete(name: String) -> Result<(), String> {
    let name = safe_name(&name).ok_or("Nom de skill invalide")?;
    let dir = skills_dir().ok_or("Dossier ~/.claude introuvable")?.join(&name);
    if !dir.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(&dir).map_err(|e| format!("Suppression : {e}"))
}

/// Liste les serveurs MCP configurés via `claude mcp list`.
#[tauri::command]
pub async fn mcp_installed() -> Vec<McpItem> {
    let out = tokio::process::Command::new(provider::claude_code::claude_bin())
        .arg("mcp")
        .arg("list")
        .output()
        .await;
    let out = match out {
        Ok(o) => o,
        Err(_) => return vec![],
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut items = vec![];
    for line in text.lines() {
        let line = line.trim();
        // Format : "Nom: cible - statut"  (le nom peut contenir des espaces)
        let Some((name, rest)) = line.split_once(": ") else {
            continue;
        };
        if name.is_empty() || rest.is_empty() {
            continue;
        }
        let (target, status) = match rest.rsplit_once(" - ") {
            Some((t, s)) => (t.trim().to_string(), s.trim().to_string()),
            None => (rest.trim().to_string(), String::new()),
        };
        let name = name.trim().to_string();
        // Le scope détermine si on peut supprimer depuis l'app (claudeai = non, géré sur le web).
        let scope = mcp_scope(&name).await.unwrap_or_default();
        let removable = scope != "claudeai";
        items.push(McpItem { name, target, status, scope, removable });
    }
    items
}

/// Ajoute un serveur MCP. `target` = URL (http/https) → transport http ; sinon commande stdio.
#[tauri::command]
pub async fn mcp_add(name: String, target: String, transport: Option<String>) -> Result<(), String> {
    let name = name.trim();
    let target = target.trim();
    if name.is_empty() || target.is_empty() {
        return Err("Nom et cible requis".into());
    }
    let mut cmd = tokio::process::Command::new(provider::claude_code::claude_bin());
    cmd.arg("mcp").arg("add");
    let is_url = target.starts_with("http://") || target.starts_with("https://");
    let tr = transport.as_deref().unwrap_or(if is_url { "http" } else { "stdio" });
    if is_url {
        cmd.arg("--transport").arg(tr).arg(name).arg(target);
    } else {
        // stdio : "claude mcp add <name> -- <command> [args...]"
        cmd.arg(name).arg("--");
        for part in target.split_whitespace() {
            cmd.arg(part);
        }
    }
    let out = cmd.output().await.map_err(|e| format!("Lancement : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Ajoute un serveur MCP via une config JSON complète (`claude mcp add-json`).
/// Format attendu : `{"command":"npx","args":[...],"env":{...}}` ou `{"type":"http","url":"…"}`.
#[tauri::command]
pub async fn mcp_add_json(name: String, json: String) -> Result<(), String> {
    let name = name.trim();
    let json = json.trim();
    if name.is_empty() || json.is_empty() {
        return Err("Nom et config JSON requis".into());
    }
    // Validation locale pour un message d'erreur clair avant d'appeler le CLI.
    serde_json::from_str::<serde_json::Value>(json).map_err(|e| format!("JSON invalide : {e}"))?;
    let out = tokio::process::Command::new(provider::claude_code::claude_bin())
        .arg("mcp")
        .arg("add-json")
        .arg(name)
        .arg(json)
        .output()
        .await
        .map_err(|e| format!("Lancement : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Détecte le scope d'un serveur MCP via `claude mcp get` (local/user/project/claudeai).
/// Nécessaire car `mcp remove` cherche dans le scope par défaut, alors que les serveurs
/// peuvent vivre ailleurs (ex. connecteurs « claude.ai config » → scope `claudeai`).
async fn mcp_scope(name: &str) -> Option<String> {
    let out = tokio::process::Command::new(provider::claude_code::claude_bin())
        .arg("mcp")
        .arg("get")
        .arg(name)
        .output()
        .await
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().find(|l| l.trim_start().starts_with("Scope:"))?;
    let v = line.split(':').nth(1)?.to_lowercase();
    if v.contains("claude.ai") || v.contains("claudeai") {
        Some("claudeai".into())
    } else if v.contains("user") || v.contains("global") {
        Some("user".into())
    } else if v.contains("project") {
        Some("project".into())
    } else if v.contains("local") {
        Some("local".into())
    } else {
        None
    }
}

/// Retire un serveur MCP via `claude mcp remove` (avec le bon scope détecté).
#[tauri::command]
pub async fn mcp_remove(name: String) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Nom requis".into());
    }
    let scope = mcp_scope(name).await;
    let mut cmd = tokio::process::Command::new(provider::claude_code::claude_bin());
    cmd.arg("mcp").arg("remove").arg(name);
    if let Some(s) = &scope {
        cmd.arg("-s").arg(s);
    }
    let out = cmd.output().await.map_err(|e| format!("Lancement : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
