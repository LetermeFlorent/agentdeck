// Auto-update : vérifie au démarrage s'il existe une version plus récente publiée sur les
// GitHub Releases (latest.json signé). Si oui, télécharge + installe puis relance l'app.
// Tout échec (hors-ligne, pas d'endpoint, build dev) est silencieux : l'app continue normalement.

import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export async function checkForUpdates(): Promise<void> {
  try {
    const update = await check();
    if (!update?.available) return;
    await update.downloadAndInstall();
    await relaunch();
  } catch {
    /* hors-ligne / pas de release / mode dev : on ignore */
  }
}
