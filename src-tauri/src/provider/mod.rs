// Abstraction provider (multi-IA). Claude Code = premier adapter (process persistant,
// entrée/sortie stream-json). opencode et Gemini CLI s'ajoutent comme adapters « one-shot »
// (un process par message, sortie NDJSON), tous mappés sur le même `events::SessionEvent`.

pub mod claude;
pub mod common;
pub mod gemini;
pub mod headless;
pub mod opencode;

// Chemin stable `provider::claude_code::…` (claude_bin, emit, write_images, ImageInput),
// réutilisé par les autres adapters et de nombreuses commandes.
pub use claude::claude_code;

use serde::{Deserialize, Serialize};

use crate::session::SharedProc;

/// IA pilotée pour une session. Sérialisé en snake_case pour le frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    #[default]
    ClaudeCode,
    Opencode,
    Gemini,
}

impl Provider {
    /// Parse depuis l'identifiant frontend ; inconnu / absent → ClaudeCode (back-compat).
    pub fn from_str(s: &str) -> Provider {
        match s {
            "opencode" => Provider::Opencode,
            "gemini" => Provider::Gemini,
            _ => Provider::ClaudeCode,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Provider::ClaudeCode => "claude_code",
            Provider::Opencode => "opencode",
            Provider::Gemini => "gemini",
        }
    }
}

/// Envoie un message à une session via le bon adapter selon le provider.
/// Le token/authentification est résolu à l'intérieur de chaque arm (chaque IA s'authentifie
/// différemment). `claude_code::send` garde sa signature historique → aucune régression Claude.
#[allow(clippy::too_many_arguments)]
pub async fn send(
    app: tauri::AppHandle,
    provider: Provider,
    id: String,
    proc: SharedProc,
    cwd: Option<String>,
    model: Option<String>,
    effort: Option<String>,
    text: String,
    images: Vec<claude_code::ImageInput>,
    hermes: bool,
    perm_mode: Option<String>,
    allowed: Option<String>,
    disallowed: Option<String>,
) {
    match provider {
        Provider::ClaudeCode => {
            let token = crate::auth::get_token(Provider::ClaudeCode);
            claude_code::send(
                app, id, proc, cwd, model, effort, token, text, images, hermes, perm_mode,
                allowed, disallowed,
            )
            .await;
        }
        Provider::Gemini => {
            // Gemini ignore effort/permissions/hermes (pas exposés par son CLI headless).
            let _ = (effort, hermes, perm_mode, allowed, disallowed);
            gemini::gemini::send(app, id, proc, cwd, model, text, images).await;
        }
        Provider::Opencode => {
            // opencode : effort via --variant ; permissions/hermes ignorés.
            let _ = (hermes, perm_mode, allowed, disallowed);
            opencode::opencode::send(app, id, proc, cwd, model, effort, text, images).await;
        }
    }
}
