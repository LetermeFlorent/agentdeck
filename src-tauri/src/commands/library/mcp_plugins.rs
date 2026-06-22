// Serveurs MCP déclarés par les plugins installés (lecture seule : gérés par le plugin).

use super::mcp::McpItem;
use super::plugins::installed_plugins;

/// Scanne les plugins pour leurs serveurs MCP (clé `mcpServers` du plugin.json
/// ou fichier `.mcp.json` à la racine du plugin). Scope "plugin:<nom>", non supprimables.
pub fn plugin_mcp_servers() -> Vec<McpItem> {
    let mut out = vec![];
    for (plugin, install_path) in installed_plugins() {
        // Source 1 : plugin.json → mcpServers
        let pj = install_path.join(".claude-plugin").join("plugin.json");
        if let Ok(raw) = std::fs::read_to_string(&pj) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                collect_mcp_servers(json.get("mcpServers"), &plugin, &mut out);
            }
        }
        // Source 2 : .mcp.json à la racine → { "mcpServers": {...} } ou map directe
        let mj = install_path.join(".mcp.json");
        if let Ok(raw) = std::fs::read_to_string(&mj) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                let node = json.get("mcpServers").or(Some(&json));
                collect_mcp_servers(node, &plugin, &mut out);
            }
        }
    }
    out
}

/// Transforme un objet `mcpServers` JSON en McpItem (target = commande ou URL).
fn collect_mcp_servers(node: Option<&serde_json::Value>, plugin: &str, out: &mut Vec<McpItem>) {
    let Some(map) = node.and_then(|v| v.as_object()) else {
        return;
    };
    for (name, cfg) in map {
        let target = if let Some(url) = cfg.get("url").and_then(|v| v.as_str()) {
            url.to_string()
        } else {
            let cmd = cfg.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let args: Vec<String> = cfg
                .get("args")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
                .unwrap_or_default();
            if args.is_empty() {
                cmd.to_string()
            } else {
                format!("{cmd} {}", args.join(" "))
            }
        };
        out.push(McpItem {
            name: name.clone(),
            target,
            status: String::new(),
            scope: format!("plugin:{plugin}"),
            removable: false,
        });
    }
}
