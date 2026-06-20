// Liste des modèles sélectionnables. Par défaut = liste de secours (chat-config),
// remplacée au boot par les modèles réels du compte (API Models via le coffre).

import * as ipc from "$lib/ipc";
import { MODELS, tierOf } from "$lib/components/chat/chat-config";

class ModelStore {
  /** Modèles affichés dans les sélecteurs. */
  available = $state<{ v: string; l: string }[]>(MODELS);

  /** Modèle choisisseur par défaut (mode Auto) = dernier Haiku de la liste, sinon alias "haiku". */
  get pickerDefault(): string {
    const haikus = this.available.filter((m) => tierOf(m.v) === "haiku");
    if (!haikus.length) return "haiku";
    return [...haikus].sort((a, b) => b.v.localeCompare(a.v))[0].v;
  }

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
