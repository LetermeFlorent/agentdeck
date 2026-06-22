// Plugins Claude Code : unités installées (manifeste ~/.claude/plugins) gérées via
// le CLI `claude plugin`. Un plugin est désinstallé en bloc, pas skill par skill.

use crate::provider;
use std::path::PathBuf;

#[derive(serde::Serialize, Clone)]
pub struct PluginItem {
    /// Identifiant complet "nom@marketplace" (requis par `claude plugin uninstall`).
    pub id: String,
    pub name: String,
    pub marketplace: String,
    pub version: String,
    pub description: String,
    /// Nombre de skills fournis par ce plugin.
    pub skills: u32,
    pub scope: String,
}

/// Manifeste des plugins installés : ~/.claude/plugins/installed_plugins.json
fn plugins_manifest() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("plugins").join("installed_plugins.json"))
}

/// Parse le manifeste → (nom_plugin, installPath) pour chaque plugin installé.
pub fn installed_plugins() -> Vec<(String, PathBuf)> {
    let raw = plugins_manifest()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let json: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(j) => j,
        Err(_) => return vec![],
    };
    let mut out = vec![];
    if let Some(map) = json.get("plugins").and_then(|p| p.as_object()) {
        for (key, entries) in map {
            let plugin_name = key.split('@').next().unwrap_or(key).to_string();
            if let Some(arr) = entries.as_array() {
                for ent in arr {
                    if let Some(ip) = ent.get("installPath").and_then(|v| v.as_str()) {
                        out.push((plugin_name.clone(), PathBuf::from(ip)));
                    }
                }
            }
        }
    }
    out
}

/// Lit la `description` du `.claude-plugin/plugin.json` d'un plugin.
fn plugin_description(install_path: &std::path::Path) -> String {
    let pj = install_path.join(".claude-plugin").join("plugin.json");
    let raw = match std::fs::read_to_string(&pj) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|j| j.get("description").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_default()
}

/// Compte les sous-dossiers `skills/*/SKILL.md` d'un plugin.
fn count_skills(install_path: &std::path::Path) -> u32 {
    let dir = install_path.join("skills");
    let rd = match std::fs::read_dir(&dir) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    rd.flatten()
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
        .count() as u32
}

/// Liste les plugins installés (manifeste) + description et nb de skills.
#[tauri::command]
pub fn plugins_installed() -> Vec<PluginItem> {
    let raw = plugins_manifest()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let json: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(j) => j,
        Err(_) => return vec![],
    };
    let mut out = vec![];
    if let Some(map) = json.get("plugins").and_then(|p| p.as_object()) {
        for (key, entries) in map {
            let mut parts = key.splitn(2, '@');
            let name = parts.next().unwrap_or(key).to_string();
            let marketplace = parts.next().unwrap_or("").to_string();
            let ent = entries.as_array().and_then(|a| a.first());
            let install_path = ent
                .and_then(|e| e.get("installPath"))
                .and_then(|v| v.as_str())
                .map(PathBuf::from);
            let version = ent
                .and_then(|e| e.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let scope = ent
                .and_then(|e| e.get("scope"))
                .and_then(|v| v.as_str())
                .unwrap_or("user")
                .to_string();
            let (description, skills) = match &install_path {
                Some(p) => (plugin_description(p), count_skills(p)),
                None => (String::new(), 0),
            };
            out.push(PluginItem { id: key.clone(), name, marketplace, version, description, skills, scope });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Désinstalle un plugin via `claude plugin uninstall <id> -y`.
#[tauri::command]
pub async fn plugin_uninstall(id: String, scope: Option<String>) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Identifiant de plugin requis".into());
    }
    let mut cmd = tokio::process::Command::new(provider::claude_code::claude_bin());
    cmd.arg("plugin").arg("uninstall").arg(id).arg("-y");
    if let Some(s) = scope.as_deref().filter(|s| !s.is_empty()) {
        cmd.arg("-s").arg(s);
    }
    let out = cmd.output().await.map_err(|e| format!("Lancement : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
