// Gestion des sessions. Chaque session = une conversation Claude pilotée par UN process
// `claude` persistant (entrée stream-json) : on écrit les messages sur son stdin, même
// pendant qu'il travaille (steering / envoi en cours de route), comme l'app interactive.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tokio::process::{Child, ChildStdin};
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

/// Process persistant d'une session + son stdin (pour injecter les messages).
pub struct SessionProc {
    pub child: Child,
    pub stdin: ChildStdin,
    pub model: Option<String>,
    pub effort: Option<String>,
    /// Permissions actives du process (pour décider d'un respawn si elles changent).
    pub perm_mode: Option<String>,
    pub allowed: Option<String>,
    pub disallowed: Option<String>,
}

pub type SharedProc = Arc<TokioMutex<Option<SessionProc>>>;

pub struct SessionMeta {
    pub id: String,
    pub title: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub proc: SharedProc,
}

#[derive(Serialize, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
}

#[derive(Default)]
pub struct SessionManager {
    sessions: Mutex<HashMap<String, SessionMeta>>,
}

impl SessionManager {
    pub fn create(&self, title: Option<String>, cwd: Option<String>, model: Option<String>) -> String {
        let id = Uuid::new_v4().to_string();
        self.insert(id.clone(), title, cwd, model);
        id
    }

    /// Réinsère une session existante au redémarrage (le process sera relancé au 1er envoi ;
    /// la reprise `--resume` est décidée selon la présence du fichier de session sur disque).
    pub fn restore(
        &self,
        id: String,
        title: Option<String>,
        _started: bool,
        cwd: Option<String>,
        model: Option<String>,
    ) {
        self.insert(id, title, cwd, model);
    }

    fn insert(&self, id: String, title: Option<String>, cwd: Option<String>, model: Option<String>) {
        let meta = SessionMeta {
            id: id.clone(),
            title: title.unwrap_or_else(|| "Claude".to_string()),
            cwd,
            model,
            proc: Arc::new(TokioMutex::new(None)),
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
                cwd: m.cwd.clone(),
                model: m.model.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }

    /// Contexte d'envoi : (handle process partagé, cwd) si la session existe.
    pub fn send_ctx(&self, id: &str) -> Option<(SharedProc, Option<String>)> {
        let map = self.sessions.lock().unwrap();
        let m = map.get(id)?;
        Some((m.proc.clone(), m.cwd.clone()))
    }

    pub fn proc_handle(&self, id: &str) -> Option<SharedProc> {
        let map = self.sessions.lock().unwrap();
        map.get(id).map(|m| m.proc.clone())
    }

    pub fn remove(&self, id: &str) -> Option<SharedProc> {
        let mut map = self.sessions.lock().unwrap();
        map.remove(id).map(|m| m.proc)
    }
}
