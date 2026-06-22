// Vérif de connectivité : l'app exige internet (login OAuth + CLI claude). Au lancement on
// confirme qu'on peut joindre le réseau, sinon le frontend affiche un écran bloquant.

/// Vrai s'il y a une connexion internet (réponse d'un endpoint léger, timeout court).
/// Essaie un endpoint « generate_204 » (réponse vide, rapide), repli sur l'API Anthropic.
#[tauri::command]
pub async fn net_check() -> bool {
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };
    const PROBES: [&str; 2] = [
        "https://www.gstatic.com/generate_204",
        "https://api.anthropic.com",
    ];
    for url in PROBES {
        if client.get(url).send().await.is_ok() {
            return true;
        }
    }
    false
}
