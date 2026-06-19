// Navigation de dossiers maison (sans plugin dialog) : home + listage des sous-dossiers,
// pour choisir le dossier de travail (cwd) d'un chat.

use std::path::PathBuf;

#[derive(serde::Serialize)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
}

#[derive(serde::Serialize)]
pub struct DirList {
    pub path: String,
    pub parent: Option<String>,
    pub dirs: Vec<DirEntry>,
}

/// Ouvre le vrai explorateur de fichiers du système (sélection de dossier).
/// Renvoie le chemin choisi, ou None si annulé.
#[tauri::command]
pub async fn pick_folder(start: Option<String>) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut d = rfd::FileDialog::new().set_title("Choisir le dossier de travail");
        if let Some(s) = start.filter(|s| !s.is_empty()) {
            d = d.set_directory(s);
        }
        d.pick_folder().map(|p| p.display().to_string())
    })
    .await
    .ok()
    .flatten()
}

/// Dossier personnel de l'utilisateur (cwd par défaut).
#[tauri::command]
pub fn home_dir() -> String {
    dirs::home_dir().map(|p| p.display().to_string()).unwrap_or_default()
}

/// Liste les sous-dossiers d'un chemin (ou du home si vide) pour le navigateur de dossiers.
#[tauri::command]
pub fn list_dirs(path: Option<String>) -> DirList {
    let base: PathBuf = match path.as_deref().filter(|p| !p.is_empty()) {
        Some(p) => PathBuf::from(p),
        None => dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")),
    };
    let mut dirs: Vec<DirEntry> = vec![];
    if let Ok(rd) = std::fs::read_dir(&base) {
        for e in rd.flatten() {
            let p = e.path();
            if !p.is_dir() {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue; // masque les dossiers cachés
            }
            dirs.push(DirEntry { name, path: p.display().to_string() });
        }
    }
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    DirList {
        path: base.display().to_string(),
        parent: base.parent().map(|p| p.display().to_string()),
        dirs,
    }
}
