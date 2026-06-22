// Commandes Tauri des skills (global ou projet selon `cwd`).

use super::skills::{
    base_collect, collect_skills, plugin_skills, project_dir, resolve_skill_md, safe_name,
    skills_dir, write_skill_at, SkillItem,
};
use std::collections::HashSet;
use std::path::PathBuf;

/// Résout le dossier cible : cwd non vide = projet, sinon global.
fn base_dir(cwd: Option<&str>) -> Option<PathBuf> {
    match cwd {
        Some(c) if !c.trim().is_empty() => project_dir(c),
        _ => skills_dir(),
    }
}

/// Liste les skills globaux (~/.claude/skills/*/SKILL.md, supprimables) + ceux
/// fournis par les plugins (lecture seule). Les skills user priment en cas de doublon.
#[tauri::command]
pub fn skills_installed() -> Vec<SkillItem> {
    let mut out = vec![];
    let mut seen = HashSet::new();
    if let Some(dir) = skills_dir() {
        collect_skills(&dir, "user", true, &mut seen, &mut out);
    }
    plugin_skills(&mut seen, &mut out);
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Liste les skills d'un projet (<cwd>/.claude/skills/*/SKILL.md).
#[tauri::command]
pub fn project_skills(cwd: String) -> Vec<SkillItem> {
    base_collect(project_dir(&cwd), "project")
}

/// Écrit un SKILL.md (global si `cwd` absent, sinon dans le projet).
#[tauri::command]
pub fn skill_write(name: String, content: String, cwd: Option<String>) -> Result<(), String> {
    let name = safe_name(&name).ok_or("Nom de skill invalide")?;
    let base = base_dir(cwd.as_deref()).ok_or("Dossier de skills introuvable")?;
    write_skill_at(&base, &name, &content)
}

/// Lit le SKILL.md brut d'un skill (global ou projet).
#[tauri::command]
pub fn skill_read(name: String, cwd: Option<String>) -> Result<String, String> {
    let name = safe_name(&name).ok_or("Nom de skill invalide")?;
    let base = base_dir(cwd.as_deref());
    let path = resolve_skill_md(base.as_deref(), &name).ok_or("Skill introuvable")?;
    std::fs::read_to_string(&path).map_err(|e| format!("Lecture : {e}"))
}

/// Supprime un skill (dossier complet, global ou projet).
#[tauri::command]
pub fn skill_delete(name: String, cwd: Option<String>) -> Result<(), String> {
    let name = safe_name(&name).ok_or("Nom de skill invalide")?;
    let dir = base_dir(cwd.as_deref())
        .ok_or("Dossier de skills introuvable")?
        .join(&name);
    if !dir.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(&dir).map_err(|e| format!("Suppression : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_write_read_delete() {
        let name = "test-dev-skill-agentdeck".to_string();
        let content = "---\nname: test-dev-skill-agentdeck\ndescription: skill de test dev\n---\n\nContenu test.".to_string();

        skill_write(name.clone(), content.clone(), None).expect("skill_write failed");
        let got = skill_read(name.clone(), None).expect("skill_read failed");
        assert_eq!(got.trim(), content.trim());
        skill_delete(name.clone(), None).expect("skill_delete failed");
        assert!(skill_read(name.clone(), None).is_err(), "skill should be deleted");
    }

    #[test]
    fn test_safe_name_rejects_traversal() {
        assert!(safe_name("../evil").is_none());
        assert!(safe_name("a/b").is_none());
        assert!(safe_name(".hidden").is_none());
        assert_eq!(safe_name(" ok-name ").as_deref(), Some("ok-name"));
    }
}
