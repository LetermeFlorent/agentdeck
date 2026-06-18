// Events normalisés émis vers le frontend, indépendants du provider (multi-IA).
// Chaque session émet sur le canal Tauri "session://{id}".

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionEvent {
    /// Le process de session est prêt (init reçu) + commandes slash disponibles.
    Started,
    /// Init : liste des commandes slash exposées par Claude Code (dynamique).
    Init { slash_commands: Vec<String> },
    /// Début d'un nouveau message assistant (nouveau tour, nouvelle bulle).
    AssistantStart,
    /// Fragment de texte de la réponse de l'assistant (streaming).
    AssistantDelta { text: String },
    /// L'assistant utilise un outil.
    ToolUse { name: String },
    /// Progression du tour : tokens de sortie cumulés (pour l'indicateur live).
    Progress { output_tokens: u64 },
    /// Fin du tour : tokens du tour + coût cumulé (USD) + taille du contexte courant.
    TurnDone {
        input_tokens: u64,
        output_tokens: u64,
        total_tokens: u64,
        cost_usd: f64,
        /// Tokens de prompt du dernier tour (input + cache) = remplissage du contexte.
        context_tokens: u64,
        /// Taille de la fenêtre de contexte du modèle, rapportée par Claude Code
        /// (`modelUsage.<model>.contextWindow`). Dynamique : 200k, 1M… ; 0 si inconnue.
        context_window: u64,
    },
    /// Erreur (spawn, parse, auth…).
    Error { message: String },
    /// Le process s'est terminé.
    Exited { code: Option<i32> },
}

/// Nom du canal d'event pour une session donnée.
pub fn channel(id: &str) -> String {
    format!("session://{id}")
}
