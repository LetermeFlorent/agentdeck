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

/// Commande PowerShell de l'installeur officiel.
/// On répare `PSModulePath` (valeur machine) avant `irm | iex` : dans le sous-processus
/// lancé par Tauri, l'auto-chargement de cmdlets comme `Get-FileHash` (module Utility)
/// échoue si ce chemin est incomplet.
const INSTALL_CMD: &str = "$env:PSModulePath = [Environment]::GetEnvironmentVariable('PSModulePath','Machine') + ';' + $env:PSModulePath; irm https://claude.ai/install.ps1 | iex";

async fn run_installer(shell: &str) -> std::io::Result<std::process::Output> {
    tokio::process::Command::new(shell)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            INSTALL_CMD,
        ])
        .output()
        .await
}

/// Installe Claude Code via l'installeur natif officiel.
/// Préfère `pwsh` (PowerShell 7) puis retombe sur `powershell` (5.1) s'il est absent.
#[tauri::command]
pub async fn install_claude() -> Result<(), String> {
    let out = match run_installer("pwsh").await {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => run_installer("powershell").await,
        other => other,
    }
    .map_err(|e| format!("Lancement installeur : {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
