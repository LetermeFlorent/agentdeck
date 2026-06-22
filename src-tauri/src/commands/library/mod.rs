// Bibliothèque : skills (disque ~/.claude ou projet) + plugins + serveurs MCP.
// Chaque sous-module traite un domaine ; les commandes sont réexportées à plat
// pour rester accessibles via `commands::library::<commande>`.

pub mod mcp;
pub mod mcp_add;
pub mod mcp_config;
pub mod mcp_plugins;
pub mod plugins;
pub mod skills;
pub mod skills_cmd;
