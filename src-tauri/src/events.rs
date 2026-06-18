// Events normalisés émis vers le frontend, indépendants du provider (multi-IA).
// Chaque session émet sur le canal Tauri "session://{id}".

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SessionEvent {
    /// Le tour a démarré (process lancé / init reçu).
    Started,
    /// Fragment de texte de la réponse de l'assistant (streaming).
    AssistantDelta { text: String },
    /// L'assistant utilise un outil.
    ToolUse { name: String },
    /// Résultat d'un outil (résumé court).
    ToolResult { ok: bool },
    /// Fin du tour, avec compteurs de tokens si disponibles.
    TurnDone {
        input_tokens: u64,
        output_tokens: u64,
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
