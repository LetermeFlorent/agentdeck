// Commande : liste dynamique des commandes slash de Claude Code (+ description et arguments).
// Sources : init (noms) ; frontmatter des skills/commandes sur disque (description + argument-hint) ;
// table de référence pour les commandes built-in du CLI (que le protocole n'expose pas).

use crate::auth;
use crate::provider;
use std::collections::HashMap;

/// Une commande slash : nom + description + indice d'arguments (ex. "[message]").
#[derive(serde::Serialize, Default, Clone)]
pub struct SlashCmd {
    pub name: String,
    pub description: String,
    pub args: String,
}

/// Lit `description:` et `argument-hint:` du frontmatter YAML d'un fichier markdown.
fn read_meta(path: &std::path::Path) -> (String, String) {
    let raw = match std::fs::read_to_string(path) {
        Ok(r) => r,
        Err(_) => return (String::new(), String::new()),
    };
    let mut lines = raw.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return (String::new(), String::new());
    }
    let (mut desc, mut args) = (String::new(), String::new());
    for line in lines {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Some(rest) = t.strip_prefix("description:") {
            desc = rest.trim().trim_matches(['"', '\'']).trim().to_string();
        } else if let Some(rest) = t.strip_prefix("argument-hint:") {
            args = rest.trim().trim_matches(['"', '\'']).trim().to_string();
        }
    }
    (desc, args)
}

/// Parcourt récursivement (profondeur bornée) et associe nom → (description, args) :
/// `SKILL.md` → nom du dossier parent ; `*.md` sous `commands/` → nom du fichier.
fn collect_meta(dir: &std::path::Path, depth: usize, map: &mut HashMap<String, (String, String)>) {
    if depth == 0 {
        return;
    }
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_meta(&p, depth - 1, map);
            continue;
        }
        let name = if p.file_name().and_then(|n| n.to_str()) == Some("SKILL.md") {
            p.parent().and_then(|pp| pp.file_name()).and_then(|n| n.to_str())
        } else if p.extension().and_then(|e| e.to_str()) == Some("md")
            && p.parent().and_then(|pp| pp.file_name()).and_then(|n| n.to_str()) == Some("commands")
        {
            p.file_stem().and_then(|n| n.to_str())
        } else {
            None
        };
        if let Some(name) = name {
            let (d, a) = read_meta(&p);
            if !d.is_empty() || !a.is_empty() {
                map.entry(name.to_string()).or_insert((d, a));
            }
        }
    }
}

/// Métadonnées des commandes built-in du CLI (non exposées par le protocole stream-json).
fn builtin_meta(name: &str) -> (&'static str, &'static str) {
    match name {
        "clear" => ("Effacer l'historique de la conversation", ""),
        "compact" => ("Résumer et compacter le contexte", "[instructions]"),
        "config" => ("Ouvrir les réglages de Claude Code", ""),
        "context" => ("Afficher l'usage de la fenêtre de contexte", ""),
        "init" => ("Générer / mettre à jour le CLAUDE.md du projet", ""),
        "review" => ("Revue de code d'une pull request", "[PR]"),
        "security-review" => ("Revue de sécurité des changements en cours", ""),
        "usage" => ("Afficher l'usage et les limites d'abonnement", ""),
        "usage-credits" => ("Afficher les crédits d'usage restants", ""),
        "extra-usage" => ("Activer / gérer l'usage supplémentaire payant", ""),
        "insights" => ("Statistiques d'utilisation", ""),
        "heapdump" => ("Dump mémoire du process (debug)", ""),
        "reload-skills" => ("Recharger les skills depuis le disque", ""),
        "goal" => ("Définir l'objectif de la session", "[objectif]"),
        "team-onboarding" => ("Onboarding de l'équipe", ""),
        "batch" => ("Lancer plusieurs tâches en lot", ""),
        "fewer-permission-prompts" => ("Réduire les demandes de permission", ""),
        "run-skill-generator" => ("Générer un nouveau skill", ""),
        "model" => ("Changer de modèle", "[modèle]"),
        "agents" => ("Gérer les sous-agents", ""),
        "mcp" => ("Gérer les serveurs MCP", ""),
        _ => ("", ""),
    }
}

/// Récupère la liste des commandes slash en lisant l'`init` (déclencheur minimal → coût nul),
/// puis enrichit chaque nom avec sa description + ses arguments.
#[tauri::command]
pub async fn slash_commands() -> Vec<SlashCmd> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let token = auth::get_token();
    if token.is_none() && !auth::claude_logged_in() {
        return vec![];
    }
    // Map nom → (description, args), lue sur disque (skills + commandes). Dynamique.
    let mut meta: HashMap<String, (String, String)> = HashMap::new();
    if let Some(home) = dirs::home_dir() {
        let base = home.join(".claude");
        collect_meta(&base.join("skills"), 4, &mut meta);
        collect_meta(&base.join("commands"), 4, &mut meta);
        collect_meta(&base.join("plugins"), 8, &mut meta);
    }
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
        .arg("haiku") // au cas où un tour partirait : le moins cher
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
    if let Some(t) = &token {
        cmd.env("CLAUDE_CODE_OAUTH_TOKEN", t);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    // Déclencheur minimal pour forcer l'émission de l'init.
    if let Some(mut si) = child.stdin.take() {
        let trigger = "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":[{\"type\":\"text\",\"text\":\".\"}]}}\n";
        let _ = si.write_all(trigger.as_bytes()).await;
        let _ = si.flush().await;
    }
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
    let names: Vec<String> = tokio::time::timeout(std::time::Duration::from_secs(20), read)
        .await
        .unwrap_or_default();
    let _ = child.start_kill();
    names
        .into_iter()
        .map(|name| {
            if let Some((d, a)) = meta.get(&name) {
                SlashCmd { name, description: d.clone(), args: a.clone() }
            } else {
                let (d, a) = builtin_meta(&name);
                SlashCmd { name, description: d.to_string(), args: a.to_string() }
            }
        })
        .collect()
}
