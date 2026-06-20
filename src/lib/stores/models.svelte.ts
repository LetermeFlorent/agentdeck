// Liste des modèles sélectionnables. Par défaut = liste de secours (chat-config),
// remplacée au boot par les modèles réels du compte (API Models via le coffre).

import * as ipc from "$lib/ipc";
import { MODELS } from "$lib/components/chat/chat-config";

class ModelStore {
  /** Modèles affichés dans les sélecteurs. */
  available = $state<{ v: string; l: string }[]>(MODELS);

  /** Récupère la vraie liste ; en cas d'échec on garde la liste de secours. */
  async load() {
    try {
      const list = await ipc.claudeModels();
      if (list.length) this.available = list;
    } catch {
      /* offline / token sans accès → fallback conservé */
    }
  }
}

export const modelStore = new ModelStore();
