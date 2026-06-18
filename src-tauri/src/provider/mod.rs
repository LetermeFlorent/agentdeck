// Abstraction provider (multi-IA). Claude Code est le premier adapter ; d'autres IA
// se brancheront en implémentant ce trait, sans changement côté frontend.

pub mod claude_code;

pub use claude_code::ClaudeCodeProvider;

use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::Mutex;

/// Handle partagé sur le process en cours d'un tour, pour pouvoir l'arrêter.
pub type SharedChild = Arc<Mutex<Option<Child>>>;

/// Config d'un tour de conversation.
#[derive(Clone)]
pub struct TurnConfig {
    /// Id interne de la session (== id de session Claude, un UUID).
    pub id: String,
    /// Texte envoyé par l'utilisateur.
    pub prompt: String,
    /// true si la session a déjà eu un tour (on reprend via --resume).
    pub resume: bool,
    /// Répertoire de travail optionnel.
    pub cwd: Option<String>,
    /// Modèle optionnel (alias sonnet/opus/haiku/fable ou id complet).
    pub model: Option<String>,
    /// Effort de raisonnement optionnel (low/medium/high/xhigh/max).
    pub effort: Option<String>,
    /// Token OAuth injecté en env au spawn.
    pub token: String,
}

/// Un provider sait démarrer un tour qui émet ses events de façon asynchrone
/// sur le canal `session://{id}`.
pub trait Provider: Send + Sync {
    fn start_turn(&self, app: tauri::AppHandle, cfg: TurnConfig, running: SharedChild);
}
