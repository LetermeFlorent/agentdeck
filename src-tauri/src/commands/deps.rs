// Commandes liées à la dépendance Claude Code CLI.

use crate::provider;

/// true si le binaire `claude` répond (`--version`).
#[tauri::command]
pub async fn check_claude() -> bool {
    tokio::process::Command::new(provider::claude_code::claude_bin())
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Installe Claude Code via l'installeur natif officiel (Windows PowerShell).
#[tauri::command]
pub async fn install_claude() -> Result<(), String> {
    let out = tokio::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "irm https://claude.ai/install.ps1 | iex",
        ])
        .output()
        .await
        .map_err(|e| format!("Lancement installeur : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
