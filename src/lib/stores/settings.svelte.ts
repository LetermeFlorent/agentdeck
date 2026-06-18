// Réglages persistés : réouverture des onglets, modèle/effort par défaut (override utilisateur).

const KEY = "agentdeck.settings.v1";

interface Persisted {
  restoreOnLaunch: boolean;
  defaultModel: string | null;
  defaultEffort: string | null;
  unavailableModels: string[];
  privateAfterMin: number | null;
  defaultZoom: number;
}

class SettingsStore {
  /** Réouvrir les mêmes onglets + discussions au prochain lancement. */
  restoreOnLaunch = $state(true);
  /** Override utilisateur du modèle par défaut (sinon = modèle Claude Code courant). */
  defaultModel = $state<string | null>(null);
  /** Override utilisateur de l'effort par défaut. */
  defaultEffort = $state<string | null>(null);
  /** Modèles détectés indisponibles (retirés des listes automatiquement). */
  unavailableModels = $state<string[]>([]);
  /** Minutes d'inactivité avant passage auto en mode privé (0 / null = désactivé). */
  privateAfterMin = $state<number | null>(null);
  /** Zoom par défaut des nouveaux chats (1 = 100%). */
  defaultZoom = $state(1);

  load() {
    try {
      const raw = localStorage.getItem(KEY);
      if (!raw) return;
      const p = JSON.parse(raw) as Partial<Persisted>;
      this.restoreOnLaunch = p.restoreOnLaunch ?? true;
      this.defaultModel = p.defaultModel ?? null;
      this.defaultEffort = p.defaultEffort ?? null;
      this.unavailableModels = p.unavailableModels ?? [];
      this.privateAfterMin = p.privateAfterMin ?? null;
      this.defaultZoom = p.defaultZoom ?? 1;
    } catch {
      /* ignore */
    }
  }

  private save() {
    const p: Persisted = {
      restoreOnLaunch: this.restoreOnLaunch,
      defaultModel: this.defaultModel,
      defaultEffort: this.defaultEffort,
      unavailableModels: this.unavailableModels,
      privateAfterMin: this.privateAfterMin,
      defaultZoom: this.defaultZoom,
    };
    try {
      localStorage.setItem(KEY, JSON.stringify(p));
    } catch {
      /* ignore */
    }
  }

  setRestore(v: boolean) {
    this.restoreOnLaunch = v;
    this.save();
  }
  setDefaultModel(v: string) {
    this.defaultModel = v || null;
    this.save();
  }
  setDefaultEffort(v: string) {
    this.defaultEffort = v || null;
    this.save();
  }
  /** Délai d'inactivité (min) avant mode privé auto. <= 0 → désactivé (null). */
  setPrivateAfterMin(v: number | null) {
    this.privateAfterMin = v && v > 0 ? Math.round(v) : null;
    this.save();
  }
  /** Zoom par défaut des nouveaux chats, borné à [0.6, 1.8]. */
  setDefaultZoom(v: number) {
    this.defaultZoom = Math.min(1.8, Math.max(0.6, Math.round(v * 100) / 100));
    this.save();
  }

  /** Marque un modèle comme indisponible (détecté à l'usage) → retiré des listes. */
  markModelUnavailable(alias: string) {
    if (!alias || this.unavailableModels.includes(alias)) return;
    this.unavailableModels = [...this.unavailableModels, alias];
    this.save();
  }
}

export const settings = new SettingsStore();
