// Cœur des skills : types + résolution de dossiers + lecture/écriture sur disque.
// Globaux dans ~/.claude/skills, projet dans <cwd>/.claude/skills. Les commandes
// Tauri vivent dans `skills_cmd` ; ces helpers sont partagés avec le mode Hermes.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(serde::Serialize, Clone)]
pub struct SkillItem {
    pub name: String,
    pub description: String,
    /// Origine : "user" (global), "project" (dépôt) ou nom du plugin (lecture seule).
    pub source: String,
    /// Faux pour les skills fournis par un plugin (non supprimables ici).
    pub removable: bool,
}

/// Nom sûr : empêche toute traversée de chemin (slash, backslash, "..").
pub fn safe_name(name: &str) -> Option<String> {
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
pub fn slugify(name: &str) -> String {
    let s: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    s.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-")
}

/// Dossier des skills globaux : ~/.claude/skills
pub fn skills_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("skills"))
}

/// Dossier des skills d'un projet : <cwd>/.claude/skills (None si cwd vide).
pub fn project_dir(cwd: &str) -> Option<PathBuf> {
    let c = cwd.trim();
    if c.is_empty() {
        None
    } else {
        Some(PathBuf::from(c).join(".claude").join("skills"))
    }
}

/// Lit `description:` du frontmatter YAML d'un SKILL.md.
pub fn read_description(path: &Path) -> String {
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

/// Collecte les skills d'un dossier `<dir>/*/SKILL.md` (sans récursion).
/// `seen` déduplique par nom (les skills user priment sur ceux des plugins).
pub fn collect_skills(
    dir: &Path,
    source: &str,
    removable: bool,
    seen: &mut HashSet<String>,
    out: &mut Vec<SkillItem>,
) {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in rd.flatten() {
        let p = e.path();
        let skill_md = p.join("SKILL.md");
        if !p.is_dir() || !skill_md.exists() {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if !seen.insert(name.clone()) {
            continue;
        }
        out.push(SkillItem {
            description: read_description(&skill_md),
            name,
            source: source.to_string(),
            removable,
        });
    }
}

/// Liste triée des skills d'un dossier optionnel (None → liste vide). `source` étiquette l'origine.
pub fn base_collect(dir: Option<PathBuf>, source: &str) -> Vec<SkillItem> {
    let mut out = vec![];
    let mut seen = HashSet::new();
    if let Some(dir) = dir {
        collect_skills(&dir, source, true, &mut seen, &mut out);
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Résout le chemin du SKILL.md d'un skill par son nom : d'abord dans le dossier
/// `base` (user ou projet), sinon parmi les skills fournis par les plugins. None si introuvable.
pub fn resolve_skill_md(base: Option<&Path>, name: &str) -> Option<PathBuf> {
    if let Some(b) = base {
        let p = b.join(name).join("SKILL.md");
        if p.exists() {
            return Some(p);
        }
    }
    for (_plugin, install_path) in super::plugins::installed_plugins() {
        let p = install_path.join("skills").join(name).join("SKILL.md");
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Skills fournis par les plugins installés : `<install_path>/skills/*/SKILL.md`.
/// Source "plugin:<nom>", non supprimables (gérés par le plugin). `seen` déduplique
/// par nom pour que les skills user/projet priment.
pub fn plugin_skills(seen: &mut HashSet<String>, out: &mut Vec<SkillItem>) {
    for (plugin, install_path) in super::plugins::installed_plugins() {
        let dir = install_path.join("skills");
        let source = format!("plugin:{plugin}");
        collect_skills(&dir, &source, false, seen, out);
    }
}

/// Écrit `<base>/<name>/SKILL.md` (helper partagé avec le mode Hermes).
pub fn write_skill_at(base: &Path, name: &str, content: &str) -> Result<(), String> {
    let dir = base.join(name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Création dossier : {e}"))?;
    std::fs::write(dir.join("SKILL.md"), content).map_err(|e| format!("Écriture : {e}"))
}
