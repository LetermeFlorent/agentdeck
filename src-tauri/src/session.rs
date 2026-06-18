// Gestion des sessions : une session = une conversation Claude Code identifiée par un UUID
// (réutilisé comme --session-id / --resume). Chaque session garde un handle sur le process
// du tour en cours pour pouvoir l'arrêter.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

use crate::provider::SharedChild;

pub struct SessionMeta {
    pub id: String,
    pub title: String,
    pub started: bool,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub running: SharedChild,
}

#[derive(Serialize, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub started: bool,
    pub cwd: Option<String>,
    pub model: Option<String>,
}

#[derive(Default)]
pub struct SessionManager {
    sessions: Mutex<HashMap<String, SessionMeta>>,
}

/// Données nécessaires pour lancer un tour (extraites sous lock, utilisées hors lock).
pub struct TurnHandles {
    pub started: bool,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub running: SharedChild,
}

impl SessionManager {
    pub fn create(&self, title: Option<String>, cwd: Option<String>, model: Option<String>) -> String {
        let id = Uuid::new_v4().to_string();
        let meta = SessionMeta {
            id: id.clone(),
            title: title.unwrap_or_else(|| "Claude".to_string()),
            started: false,
            cwd,
            model,
            running: Arc::new(TokioMutex::new(None)),
        };
        self.sessions.lock().unwrap().insert(id.clone(), meta);
        id
    }

    /// Réinsère une session existante (au redémarrage de l'app). L'UUID est conservé
    /// pour pouvoir reprendre la conversation Claude via --resume.
    pub fn restore(
        &self,
        id: String,
        title: Option<String>,
        started: bool,
        cwd: Option<String>,
        model: Option<String>,
    ) {
        let meta = SessionMeta {
            id: id.clone(),
            title: title.unwrap_or_else(|| "Claude".to_string()),
            started,
            cwd,
            model,
            running: Arc::new(TokioMutex::new(None)),
        };
        self.sessions.lock().unwrap().insert(id, meta);
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        let map = self.sessions.lock().unwrap();
        let mut v: Vec<SessionInfo> = map
            .values()
            .map(|m| SessionInfo {
                id: m.id.clone(),
                title: m.title.clone(),
                started: m.started,
                cwd: m.cwd.clone(),
                model: m.model.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }

    /// Récupère les handles d'un tour et marque la session comme démarrée.
    pub fn begin_turn(&self, id: &str) -> Option<TurnHandles> {
        let mut map = self.sessions.lock().unwrap();
        let meta = map.get_mut(id)?;
        let h = TurnHandles {
            started: meta.started,
            cwd: meta.cwd.clone(),
            model: meta.model.clone(),
            running: meta.running.clone(),
        };
        meta.started = true;
        Some(h)
    }

    pub fn running_handle(&self, id: &str) -> Option<SharedChild> {
        let map = self.sessions.lock().unwrap();
        map.get(id).map(|m| m.running.clone())
    }

    pub fn remove(&self, id: &str) -> Option<SharedChild> {
        let mut map = self.sessions.lock().unwrap();
        map.remove(id).map(|m| m.running)
    }
}
