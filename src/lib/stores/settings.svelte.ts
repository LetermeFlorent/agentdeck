// Réglages persistés : réouverture des onglets, modèle/effort par défaut (override utilisateur).

import { debounce } from "../util/debounce";
import { STORAGE_KEYS } from "./keys";

const KEY = STORAGE_KEYS.settings;

interface Persisted {
  restoreOnLaunch: boolean;
  defaultModel: string | null;
  defaultEffort: string | null;
  unavailableModels: string[];
  privateAfterMin: number | null;
  defaultZoom: number;
  hermesMode: boolean;
  defaultPermMode: string;
  autoEffort: boolean;
  autoModel: boolean;
  autoModels: string[];
  autoEfforts: string[];
  historyLimit: number;
  defaultCwd: string;
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
  /** Mode Hermes : l'agent consulte/crée des skills et apprend de ses erreurs (auto). */
  hermesMode = $state(false);
  /** Mode de permission par défaut des nouveaux chats. */
  defaultPermMode = $state("bypassPermissions");
  /** Auto-effort : choisit l'effort adapté à chaque demande (appel Haiku). */
  autoEffort = $state(false);
  /** Auto-modèle : choisit aussi le modèle (seulement si autoEffort actif). */
  autoModel = $state(false);
  /** Modèles parmi lesquels l'auto peut choisir (défaut : tous sauf Fable). */
  autoModels = $state<string[]>(["opus", "sonnet", "haiku"]);
  /** Efforts parmi lesquels l'auto peut choisir (défaut : tous). */
  autoEfforts = $state<string[]>(["low", "medium", "high", "xhigh", "max"]);
  /** Nombre de conversations affichées dans l'historique (défaut 30). */
  historyLimit = $state(30);
  /** Dossier de travail par défaut des nouveaux chats (vide = dossier personnel). */
  defaultCwd = $state("");

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
      this.hermesMode = p.hermesMode ?? false;
      this.defaultPermMode = p.defaultPermMode ?? "bypassPermissions";
      this.autoEffort = p.autoEffort ?? false;
      this.autoModel = p.autoModel ?? false;
      this.autoModels = p.autoModels ?? ["opus", "sonnet", "haiku"];
      this.autoEfforts = p.autoEfforts ?? ["low", "medium", "high", "xhigh", "max"];
      this.historyLimit = p.historyLimit ?? 30;
      this.defaultCwd = p.defaultCwd ?? "";
    } catch {
      /* ignore */
    }
  }

  // Écriture debouncée : un setter peut être appelé en rafale (sliders, toggles) ; on ne
  // sérialise/écrit qu'après une courte inactivité au lieu d'une fois par changement.
  private save = debounce(() => this.saveNow(), 400);

  private saveNow() {
    const p: Persisted = {
      restoreOnLaunch: this.restoreOnLaunch,
      defaultModel: this.defaultModel,
      defaultEffort: this.defaultEffort,
      unavailableModels: this.unavailableModels,
      privateAfterMin: this.privateAfterMin,
      defaultZoom: this.defaultZoom,
      hermesMode: this.hermesMode,
      defaultPermMode: this.defaultPermMode,
      autoEffort: this.autoEffort,
      autoModel: this.autoModel,
      autoModels: this.autoModels,
      autoEfforts: this.autoEfforts,
      historyLimit: this.historyLimit,
      defaultCwd: this.defaultCwd,
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

  setHermesMode(v: boolean) {
    this.hermesMode = v;
    this.save();
  }
  setDefaultPermMode(v: string) {
    this.defaultPermMode = v || "bypassPermissions";
    this.save();
  }
  setAutoEffort(v: boolean) {
    this.autoEffort = v;
    if (!v) this.autoModel = false; // auto-modèle dépend d'auto-effort
    this.save();
  }
  setAutoModel(v: boolean) {
    this.autoModel = v;
    this.save();
  }
  /** Coche/décoche un modèle de la liste auto. */
  toggleAutoModelChoice(model: string) {
    this.autoModels = this.autoModels.includes(model)
      ? this.autoModels.filter((m) => m !== model)
      : [...this.autoModels, model];
    this.save();
  }
  setDefaultCwd(v: string) {
    this.defaultCwd = v || "";
    this.save();
  }
  setHistoryLimit(v: number) {
    this.historyLimit = Math.min(200, Math.max(1, Math.round(v) || 30));
    this.save();
  }
  /** Coche/décoche un effort de la liste auto. */
  toggleAutoEffortChoice(effort: string) {
    this.autoEfforts = this.autoEfforts.includes(effort)
      ? this.autoEfforts.filter((e) => e !== effort)
      : [...this.autoEfforts, effort];
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
