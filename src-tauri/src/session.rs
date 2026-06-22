// Gestion des sessions. Chaque session = une conversation Claude pilotée par UN process
// `claude` persistant (entrée stream-json) : on écrit les messages sur son stdin, même
// pendant qu'il travaille (steering / envoi en cours de route), comme l'app interactive.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::Serialize;
use tokio::process::{Child, ChildStdin};
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

use crate::provider::Provider;

/// Process d'une session + son stdin (pour injecter les messages).
/// Claude = process persistant (stdin `Some`, réutilisé). opencode/Gemini = « one-shot » :
/// un nouveau child par message, `stdin` à `None`, `child` = dernier process en vol (pour l'arrêt).
pub struct SessionProc {
    /// Provider du process (informatif ; le dispatch se fait via `SessionMeta.provider`).
    #[allow(dead_code)]
    pub provider: Provider,
    pub child: Child,
    pub stdin: Option<ChildStdin>,
    /// Id de session natif du provider one-shot (ex. opencode `ses_…`), pour reprendre la conversation.
    pub ext_session: Option<String>,
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
    pub provider: Provider,
    pub proc: SharedProc,
}

#[derive(Serialize, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub provider: String,
}

#[derive(Default)]
pub struct SessionManager {
    sessions: Mutex<HashMap<String, SessionMeta>>,
}

impl SessionManager {
    pub fn create(
        &self,
        title: Option<String>,
        cwd: Option<String>,
        model: Option<String>,
        provider: Provider,
    ) -> String {
        let id = Uuid::new_v4().to_string();
        self.insert(id.clone(), title, cwd, model, provider);
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
        provider: Provider,
    ) {
        self.insert(id, title, cwd, model, provider);
    }

    fn insert(
        &self,
        id: String,
        title: Option<String>,
        cwd: Option<String>,
        model: Option<String>,
        provider: Provider,
    ) {
        let meta = SessionMeta {
            id: id.clone(),
            title: title.unwrap_or_else(|| "Claude".to_string()),
            cwd,
            model,
            provider,
            proc: Arc::new(TokioMutex::new(None)),
        };
        self.sessions.lock().insert(id, meta);
    }

    pub fn list(&self) -> Vec<SessionInfo> {
        let map = self.sessions.lock();
        let mut v: Vec<SessionInfo> = map
            .values()
            .map(|m| SessionInfo {
                id: m.id.clone(),
                title: m.title.clone(),
                cwd: m.cwd.clone(),
                model: m.model.clone(),
                provider: m.provider.as_str().to_string(),
            })
            .collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }

    /// Contexte d'envoi : (provider, handle process partagé, cwd) si la session existe.
    pub fn send_ctx(&self, id: &str) -> Option<(Provider, SharedProc, Option<String>)> {
        let map = self.sessions.lock();
        let m = map.get(id)?;
        Some((m.provider, m.proc.clone(), m.cwd.clone()))
    }

    /// Met à jour l'IA d'une session (changement manuel ou auto cross-IA). True si elle a changé.
    pub fn set_provider(&self, id: &str, provider: Provider) -> bool {
        let mut map = self.sessions.lock();
        match map.get_mut(id) {
            Some(m) if m.provider != provider => {
                m.provider = provider;
                true
            }
            _ => false,
        }
    }

    /// Change le dossier de travail d'une session et renvoie son process (à tuer pour
    /// qu'il soit relancé dans le nouveau dossier au prochain envoi).
    pub fn set_cwd(&self, id: &str, cwd: Option<String>) -> Option<SharedProc> {
        let mut map = self.sessions.lock();
        let m = map.get_mut(id)?;
        m.cwd = cwd;
        Some(m.proc.clone())
    }

    pub fn proc_handle(&self, id: &str) -> Option<SharedProc> {
        let map = self.sessions.lock();
        map.get(id).map(|m| m.proc.clone())
    }

    pub fn remove(&self, id: &str) -> Option<SharedProc> {
        let mut map = self.sessions.lock();
        map.remove(id).map(|m| m.proc)
    }
}
