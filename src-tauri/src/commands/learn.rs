// Mode Hermes — réflexion automatique sur échec : un appel headless résume l'erreur
// en une leçon réutilisable, décide la portée (global / projet), et écrit le SKILL.md.
// Supporte claude_code, opencode et gemini comme modèle de réflexion.

use crate::commands::library::skills::{self, safe_name, slugify, write_skill_at};
use crate::provider::{headless, Provider};

/// Réfléchit sur une erreur via le modèle choisi et écrit un skill (global ou projet).
/// Renvoie le chemin du SKILL.md écrit, ou une erreur. Best-effort : ne bloque jamais l'app.
#[tauri::command]
pub async fn reflect_and_learn(
    cwd: Option<String>,
    request: String,
    summary: String,
    error: String,
    model: Option<String>,
    provider: Option<String>,
) -> Result<String, String> {
    let prov = Provider::from_str(provider.as_deref().unwrap_or("claude_code"));
    let model_str = model.as_deref().unwrap_or("").trim().to_string();

    let prompt = format!(
        "Un agent de code vient d'échouer sur une tâche. Déduis-en UNE leçon réutilisable pour \
ne pas répéter l'erreur, formulée comme un skill. Réponds STRICTEMENT en JSON sur une ligne, \
sans texte autour, au format : \
{{\"scope\":\"global|project\",\"name\":\"nom-kebab-case\",\"description\":\"quand utiliser ce skill\",\"body\":\"instructions en Markdown\"}}. \
Mets scope=\"project\" si la leçon est spécifique à ce dépôt, sinon \"global\". \
Demande de l'utilisateur : {request}\nActions / réponse de l'agent : {summary}\nErreur rencontrée : {error}"
    );

    // Réflexion headless sur le même provider que le chat (Claude : effort par défaut).
    let text = match prov {
        Provider::ClaudeCode => headless::claude_oneshot(&prompt, &model_str, None, 45).await,
        Provider::Opencode => headless::opencode_oneshot(&prompt, &model_str, "", 60).await,
        Provider::Gemini => headless::gemini_oneshot(&prompt, &model_str, 60).await,
    };
    if text.trim().is_empty() {
        return Err("Réflexion : réponse vide".into());
    }

    // Extrait le bloc JSON {...} de la réponse.
    let json_str = {
        let start = text.find('{');
        let end = text.rfind('}');
        match (start, end) {
            (Some(s), Some(e)) if e > s => text[s..=e].to_string(),
            _ => return Err("Réflexion : pas de JSON exploitable".into()),
        }
    };
    let v: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("JSON réflexion invalide : {e}"))?;

    let raw_name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").trim();
    let name = safe_name(&slugify(raw_name)).ok_or("Nom de skill invalide")?;
    let description = v.get("description").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    let body = v.get("body").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
    let scope = v.get("scope").and_then(|x| x.as_str()).unwrap_or("global");

    // Dossier cible : projet (<cwd>/.claude/skills) ou global (~/.claude/skills).
    let base = match (scope, cwd.as_deref().filter(|c| !c.trim().is_empty())) {
        ("project", Some(c)) => skills::project_dir(c).ok_or("Dossier projet introuvable")?,
        _ => skills::skills_dir().ok_or("home introuvable")?,
    };

    let content = format!("---\nname: {name}\ndescription: {description}\n---\n\n{body}\n");
    write_skill_at(&base, &name, &content)?;
    Ok(base.join(&name).join("SKILL.md").display().to_string())
}
