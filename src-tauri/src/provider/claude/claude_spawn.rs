// Lancement du process `claude` persistant + lecture du stdout (NDJSON → events).

use std::collections::HashMap;
use std::process::Stdio;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::claude_code::{claude_bin, emit};
use super::claude_stream::{handle_line, ToolAcc};
use crate::events::SessionEvent;
use crate::session::SessionProc;

/// Directive « mode Hermes » injectée dans le system prompt quand l'auto-apprentissage est actif :
/// l'agent consulte ses skills avant d'agir et capitalise ses erreurs en skills réutilisables.
const HERMES_DIRECTIVE: &str = "Tu es un agent qui s'améliore en continu (façon Hermes). \
Avant de traiter une tâche, examine tes skills disponibles et utilise ceux qui s'appliquent. \
Quand tu fais une erreur, rencontres un échec, ou découvres une technique réutilisable, \
capitalise-la en créant ou en affinant un skill (un dossier avec un SKILL.md : frontmatter \
`name` + `description`, puis les instructions) pour ne pas répéter l'erreur. \
Vérifie d'abord s'il existe déjà un skill proche → affine-le plutôt que d'en dupliquer un. \
Place-le en global (~/.claude/skills/<nom>/SKILL.md) s'il est généralement utile, \
sinon dans le projet (.claude/skills/<nom>/SKILL.md) s'il est spécifique à ce dépôt.";

/// Lance le process persistant et démarre la lecture du stdout.
#[allow(clippy::too_many_arguments)]
pub(super) async fn spawn(
    app: &tauri::AppHandle,
    id: &str,
    cwd: &Option<String>,
    model: &Option<String>,
    effort: &Option<String>,
    token: &Option<String>,
    resume: bool,
    hermes: bool,
    perm_mode: &Option<String>,
    allowed: &Option<String>,
    disallowed: &Option<String>,
) -> Result<SessionProc, String> {
    let mut cmd = Command::new(claude_bin());
    cmd.arg("-p")
        .arg("--input-format")
        .arg("stream-json")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--include-partial-messages");

    // Mode de permission (défaut : bypassPermissions = comportement historique).
    let mode = perm_mode.as_deref().filter(|m| !m.is_empty()).unwrap_or("bypassPermissions");
    cmd.arg("--permission-mode").arg(mode);
    // Règles d'outils (listes séparées par virgules, ex. "Bash(git *),Edit").
    if let Some(a) = allowed {
        if !a.is_empty() {
            cmd.arg("--allowedTools").arg(a);
        }
    }
    if let Some(d) = disallowed {
        if !d.is_empty() {
            cmd.arg("--disallowedTools").arg(d);
        }
    }

    // Mode Hermes : injecte la directive d'auto-apprentissage dans le system prompt.
    if hermes {
        cmd.arg("--append-system-prompt").arg(HERMES_DIRECTIVE);
    }

    // Persistance de la conversation : on (ré)utilise l'UUID agentdeck comme session Claude.
    // 1ʳᵉ fois → on crée la session ; ensuite → on la reprend (mémoire conservée).
    if resume {
        cmd.arg("--resume").arg(id);
    } else {
        cmd.arg("--session-id").arg(id);
    }

    if let Some(m) = model {
        if !m.is_empty() {
            cmd.arg("--model").arg(m);
        }
    }
    // Haiku ne supporte pas l'effort → on n'envoie pas le flag.
    if model.as_deref() != Some("haiku") {
        if let Some(e) = effort {
            if !e.is_empty() {
                // "ultracode" (libellé Opus) n'est pas un --effort valide → mappé sur xhigh.
                let level = if e == "ultracode" { "xhigh" } else { e.as_str() };
                cmd.arg("--effort").arg(level);
            }
        }
    }
    if let Some(dir) = cwd {
        if !dir.is_empty() {
            cmd.current_dir(dir);
        }
    }

    // Token agentdeck (coffre) injecté pour cette session : connexion indépendante de la session
    // `claude` native éventuelle de l'utilisateur.
    if let Some(t) = token {
        if !t.is_empty() {
            cmd.env("CLAUDE_CODE_OAUTH_TOKEN", t);
        }
    }
    cmd.env_remove("ANTHROPIC_API_KEY");
    cmd.env_remove("ANTHROPIC_AUTH_TOKEN");

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "Binaire `claude` introuvable. Installe Claude Code.".to_string()
        } else {
            format!("Échec du lancement de claude : {e}")
        }
    })?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "stdin indisponible".to_string())?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Lecture stderr : on conserve les dernières lignes (au lieu de les jeter) pour pouvoir
    // diagnostiquer un crash de `claude`. La tâche se termine quand le pipe se ferme (process mort).
    let stderr_task = stderr.map(|err| {
        tauri::async_runtime::spawn(async move {
            const MAX_LINES: usize = 30;
            let mut tail: Vec<String> = Vec::new();
            let mut lines = BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                log::warn!("[claude stderr] {line}");
                if tail.len() == MAX_LINES {
                    tail.remove(0);
                }
                tail.push(line.to_string());
            }
            tail
        })
    });

    // Lecture stdout : NDJSON → events. Se termine quand le process meurt.
    let app2 = app.clone();
    let id2 = id.to_string();
    tauri::async_runtime::spawn(async move {
        if let Some(out) = stdout {
            let mut lines = BufReader::new(out).lines();
            let mut blocks: HashMap<u64, ToolAcc> = HashMap::new();
            let mut streamed = false; // un texte a-t-il été streamé pour le tour courant ?
            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<Value>(line) {
                    handle_line(&app2, &id2, &v, &mut blocks, &mut streamed);
                }
            }
        }
        // Le process est mort : on récupère ce qui a été écrit sur stderr. Si non vide, c'est un
        // crash/erreur → on le remonte à l'UI (sinon le diagnostic serait silencieusement perdu).
        if let Some(task) = stderr_task {
            if let Ok(tail) = task.await {
                if !tail.is_empty() {
                    emit(
                        &app2,
                        &id2,
                        SessionEvent::Error {
                            message: tail.join("\n"),
                        },
                    );
                }
            }
        }
        emit(&app2, &id2, SessionEvent::Exited { code: None });
    });

    emit(app, id, SessionEvent::Started);

    Ok(SessionProc {
        provider: crate::provider::Provider::ClaudeCode,
        child,
        stdin: Some(stdin),
        ext_session: None,
        model: model.clone(),
        effort: effort.clone(),
        perm_mode: perm_mode.clone(),
        allowed: allowed.clone(),
        disallowed: disallowed.clone(),
    })
}
