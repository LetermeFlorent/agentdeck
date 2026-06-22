// Serveurs MCP configurés via le CLI `claude mcp` (liste / scope / suppression).

use super::mcp_plugins::plugin_mcp_servers;
use crate::provider;

#[derive(serde::Serialize, Clone)]
pub struct McpItem {
    pub name: String,
    pub target: String,
    pub status: String,
    /// Scope de config : local/user/project (supprimables) ou claudeai (géré sur le web).
    pub scope: String,
    /// Faux pour les connecteurs claude.ai (non supprimables via le CLI).
    pub removable: bool,
}

/// Liste les serveurs MCP configurés via `claude mcp list` (+ ceux fournis par les plugins).
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
        let scope = mcp_scope(&name).await.unwrap_or_default();
        let removable = scope != "claudeai";
        items.push(McpItem { name, target, status, scope, removable });
    }
    // Serveurs MCP fournis par les plugins (absents de `claude mcp list`) — lecture seule.
    let known: std::collections::HashSet<String> = items.iter().map(|i| i.name.clone()).collect();
    for m in plugin_mcp_servers() {
        if !known.contains(&m.name) {
            items.push(m);
        }
    }
    items
}

/// Détecte le scope d'un serveur MCP via `claude mcp get` (local/user/project/claudeai).
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
