// Ajout de serveurs MCP via le CLI `claude mcp add` / `add-json`.

use crate::provider;

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
