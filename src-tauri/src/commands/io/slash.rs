// Commande : liste dynamique des commandes slash par provider.
// Claude Code : init event (source primaire) + frontmatter disque (enrichissement).
// opencode    : liste fixe des commandes connues (protocole sans découverte dynamique).

use crate::auth;
use crate::provider;
use std::collections::HashMap;

/// Une commande slash : nom + description + indice d'arguments (ex. "[message]") + CLI source.
#[derive(serde::Serialize, Default, Clone)]
pub struct SlashCmd {
    pub name: String,
    pub description: String,
    pub args: String,
    pub cli: String,
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

/// Parcourt récursivement (profondeur bornée) et associe nom → (description, args).
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
        let name: Option<&str> = if p.file_name().and_then(|n| n.to_str()) == Some("SKILL.md") {
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

/// Retourne les commandes slash selon le provider.
#[tauri::command]
pub async fn slash_commands(provider: String) -> Vec<SlashCmd> {
    match provider.as_str() {
        "opencode" => opencode_commands(),
        _ => claude_commands().await,
    }
}

/// Commandes built-in opencode (source : opencode.ai/docs/commands/).
/// TUI uniquement — en mode one-shot agentdeck elles sont reçues comme texte par l'IA.
fn opencode_commands() -> Vec<SlashCmd> {
    vec![
        SlashCmd { name: "new".into(),      description: "Démarre une nouvelle conversation".into(),      args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "compact".into(),  description: "Compacte et résume la session".into(),          args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "models".into(),   description: "Affiche les modèles disponibles".into(),        args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "sessions".into(), description: "Liste les conversations".into(),                args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "share".into(),    description: "Crée un lien partageable".into(),               args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "unshare".into(),  description: "Révoque l'accès partagé".into(),               args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "export".into(),   description: "Exporte la conversation en Markdown".into(),    args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "undo".into(),     description: "Annule le message précédent (Git)".into(),      args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "redo".into(),     description: "Restaure un message annulé (Git)".into(),       args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "connect".into(),  description: "Configure un nouveau provider AI".into(),       args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "themes".into(),   description: "Voir les thèmes disponibles".into(),            args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "editor".into(),   description: "Lance l'éditeur externe".into(),               args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "init".into(),     description: "Crée/met à jour AGENTS.md".into(),             args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "details".into(),  description: "Affiche/cache les détails d'exécution".into(),  args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "thinking".into(), description: "Affiche le raisonnement de l'IA".into(),       args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "help".into(),     description: "Affiche l'aide".into(),                        args: String::new(), cli: "opencode".into() },
        SlashCmd { name: "exit".into(),     description: "Quitte l'application".into(),                   args: String::new(), cli: "opencode".into() },
    ]
}

/// Commandes Claude Code : init event (source de vérité) + frontmatter disque (enrichissement).
/// Seules les commandes retournées par l'init sont réellement exécutables.
async fn claude_commands() -> Vec<SlashCmd> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let token = auth::get_token(crate::provider::Provider::ClaudeCode);
    if token.is_none() {
        return vec![];
    }
    // Métadonnées depuis le disque (description + args) — enrichissement seul, pas source de noms.
    let mut meta: HashMap<String, (String, String)> = HashMap::new();
    if let Some(home) = dirs::home_dir() {
        let base = home.join(".claude");
        collect_meta(&base.join("skills"), 4, &mut meta);
        collect_meta(&base.join("commands"), 4, &mut meta);
        collect_meta(&base.join("plugins"), 8, &mut meta);
    }
    let mut cmd = tokio::process::Command::new(provider::claude_code::claude_bin());
    cmd.arg("-p")
        .arg("--input-format").arg("stream-json")
        .arg("--output-format").arg("stream-json")
        .arg("--verbose")
        .arg("--permission-mode").arg("bypassPermissions")
        .arg("--model").arg("haiku")
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
                            return arr.iter().filter_map(|c| c.as_str().map(String::from)).collect::<Vec<_>>();
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

    names.into_iter().map(|name| {
        let (description, args) = meta.get(&name).cloned().unwrap_or_default();
        SlashCmd { name, description, args, cli: "Claude".to_string() }
    }).collect()
}
