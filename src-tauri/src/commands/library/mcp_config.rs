// Section `mcpServers` brute du fichier ~/.claude/settings.json (lecture / écriture JSON).

fn claude_settings_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Répertoire home introuvable".to_string())?;
    Ok(std::path::PathBuf::from(home).join(".claude").join("settings.json"))
}

/// Lit la section `mcpServers` du fichier `~/.claude/settings.json` en JSON pretty.
#[tauri::command]
pub fn mcp_read_raw() -> Result<String, String> {
    let path = claude_settings_path()?;
    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "{}".into());
    let val: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::json!({}));
    let mcp = val.get("mcpServers").cloned().unwrap_or(serde_json::json!({}));
    serde_json::to_string_pretty(&mcp).map_err(|e| format!("Sérialisation : {e}"))
}

/// Écrit la section `mcpServers` dans `~/.claude/settings.json` sans toucher aux autres clés.
#[tauri::command]
pub fn mcp_write_raw(json: String) -> Result<(), String> {
    let new_mcp: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| format!("JSON invalide : {e}"))?;
    let path = claude_settings_path()?;
    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "{}".into());
    let mut val: serde_json::Value =
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}));
    val["mcpServers"] = new_mcp;
    let out = serde_json::to_string_pretty(&val).map_err(|e| format!("Sérialisation : {e}"))?;
    std::fs::write(&path, out).map_err(|e| format!("Écriture : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_write_read() {
        let test_mcp = r#"{"test-dev-server": {"command": "echo", "args": ["agentdeck-test"]}}"#;
        mcp_write_raw(test_mcp.into()).expect("mcp_write_raw failed");

        let got = mcp_read_raw().expect("mcp_read_raw failed");
        let v: serde_json::Value = serde_json::from_str(&got).unwrap();
        assert!(v.get("test-dev-server").is_some(), "MCP server should be present");

        // cleanup — remove only the test key
        let mut val = v.clone();
        val.as_object_mut().unwrap().remove("test-dev-server");
        mcp_write_raw(serde_json::to_string(&val).unwrap()).expect("cleanup failed");

        let after = mcp_read_raw().expect("mcp_read_raw failed");
        let v2: serde_json::Value = serde_json::from_str(&after).unwrap();
        assert!(v2.get("test-dev-server").is_none(), "MCP server should be removed");
    }
}
