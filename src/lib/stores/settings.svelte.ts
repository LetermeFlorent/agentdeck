// Réglages persistés : réouverture des onglets, modèle/effort par défaut (override utilisateur).

import { debounce } from "../util/debounce";
import { STORAGE_KEYS } from "./keys";

const KEY = STORAGE_KEYS.settings;

interface Persisted {
  restoreOnLaunch: boolean;
  /** Modèle / effort par défaut des nouveaux chats, PAR provider. */
  defaultModel: Record<string, string | null>;
  defaultEffort: Record<string, string | null>;
  /** Indispo par "provider:model" → échéance ms (0 = permanent, sinon cooldown rate-limit). */
  unavailableModels: Record<string, number>;
  privateAfterMin: number | null;
  defaultZoom: number;
  hermesMode: boolean;
  hermesModel: string;
  hermesReflectPasses: number;
  defaultPermMode: string;
  autoEffort: boolean;
  autoModel: boolean;
  // Par-IA : candidats modèles / efforts + modèle choisisseur, par provider.
  autoModels: Record<string, string[]>;
  autoEfforts: Record<string, string[]>;
  autoPickModel: Record<string, string>;
  /** Modèle « choisisseur » GLOBAL (cross-IA) : décide l'IA + le modèle parmi tous les candidats. */
  autoPicker: string;
  /** Effort utilisé par le choisisseur global (low par défaut = rapide + peu coûteux). */
  autoPickerEffort: string;
  historyLimit: number;
  defaultCwd: string;
  chatSleepEnabled: boolean;
  chatSleepMin: number;
}

class SettingsStore {
  /** Réouvrir les mêmes onglets + discussions au prochain lancement. */
  restoreOnLaunch = $state(true);
  /** Modèle par défaut des nouveaux chats, PAR provider (sinon = défaut du provider). */
  defaultModel = $state<Record<string, string | null>>({});
  /** Effort par défaut des nouveaux chats, PAR provider. */
  defaultEffort = $state<Record<string, string | null>>({});
  /** Modèles indisponibles par "provider:model" → échéance ms (0 = permanent, sinon cooldown). */
  unavailableModels = $state<Record<string, number>>({});
  /** Minutes d'inactivité avant passage auto en mode privé (0 / null = désactivé). */
  privateAfterMin = $state<number | null>(null);
  /** Zoom par défaut des nouveaux chats (1 = 100%). */
  defaultZoom = $state(1);
  /** Mode Hermes : l'agent consulte/crée des skills et apprend de ses erreurs (auto). */
  hermesMode = $state(false);
  /** Modèle utilisé par Hermes pour la réflexion (vide = haiku par défaut). */
  hermesModel = $state("");
  /** Nombre de passes de réflexion avant exécution (1 = désactivé). */
  hermesReflectPasses = $state(1);
  /** Mode de permission par défaut des nouveaux chats (acceptEdits = édite, demande le reste). */
  defaultPermMode = $state("acceptEdits");
  /** Auto-effort : choisit l'effort adapté à chaque demande (appel Haiku). */
  autoEffort = $state(false);
  /** Auto-modèle : choisit aussi le modèle (seulement si autoEffort actif). */
  autoModel = $state(false);
  /** Modèles candidats de l'auto, PAR provider (défaut Claude : tous sauf Fable). */
  autoModels = $state<Record<string, string[]>>({ claude_code: ["opus", "sonnet", "haiku"] });
  /** Efforts candidats de l'auto, PAR provider. */
  autoEfforts = $state<Record<string, string[]>>({
    claude_code: ["low", "medium", "high", "xhigh", "max"],
  });
  /** Modèle choisisseur de l'auto, PAR provider (legacy, conservé). */
  autoPickModel = $state<Record<string, string>>({});
  /** Modèle choisisseur GLOBAL (cross-IA) : un seul décideur parmi toutes les IA connectées. */
  autoPicker = $state("");
  /** Effort du choisisseur global (low par défaut). */
  autoPickerEffort = $state("low");
  /** Nombre de conversations affichées dans l'historique (défaut 30). */
  historyLimit = $state(30);
  /** Dossier de travail par défaut des nouveaux chats (vide = dossier personnel). */
  defaultCwd = $state("");
  /** Veille des chats : suspend (tue le process claude) un chat inactif pour économiser la RAM. */
  chatSleepEnabled = $state(false);
  /** Minutes d'inactivité avant mise en veille d'un chat. */
  chatSleepMin = $state(15);

  load() {
    try {
      const raw = localStorage.getItem(KEY);
      if (!raw) return;
      const p = JSON.parse(raw) as Partial<Persisted>;
      this.restoreOnLaunch = p.restoreOnLaunch ?? true;
      // Migration ancien format (string|null = Claude only) → map par provider.
      this.defaultModel =
        p.defaultModel && typeof p.defaultModel === "object"
          ? p.defaultModel
          : p.defaultModel != null
            ? { claude_code: p.defaultModel as unknown as string }
            : {};
      this.defaultEffort =
        p.defaultEffort && typeof p.defaultEffort === "object"
          ? p.defaultEffort
          : p.defaultEffort != null
            ? { claude_code: p.defaultEffort as unknown as string }
            : {};
      // Migration : ancien format string[] (aliases Claude) → map "claude_code:<m>" = permanent.
      const um = p.unavailableModels;
      if (Array.isArray(um)) {
        this.unavailableModels = Object.fromEntries(um.map((m) => [`claude_code:${m}`, 0]));
      } else {
        this.unavailableModels = um ?? {};
      }
      this.privateAfterMin = p.privateAfterMin ?? null;
      this.defaultZoom = p.defaultZoom ?? 1;
      this.hermesMode = p.hermesMode ?? false;
      this.hermesModel = p.hermesModel ?? "";
      this.hermesReflectPasses = p.hermesReflectPasses ?? 1;
      this.defaultPermMode = p.defaultPermMode ?? "bypassPermissions";
      this.autoEffort = p.autoEffort ?? false;
      this.autoModel = p.autoModel ?? false;
      // Migration ancien format (string[]/string = Claude only) → map par provider.
      this.autoModels = Array.isArray(p.autoModels)
        ? { claude_code: p.autoModels }
        : p.autoModels ?? { claude_code: ["opus", "sonnet", "haiku"] };
      this.autoEfforts = Array.isArray(p.autoEfforts)
        ? { claude_code: p.autoEfforts }
        : p.autoEfforts ?? { claude_code: ["low", "medium", "high", "xhigh", "max"] };
      this.autoPickModel =
        typeof p.autoPickModel === "string"
          ? { claude_code: p.autoPickModel }
          : p.autoPickModel ?? {};
      this.autoPicker = p.autoPicker ?? this.autoPickModel.claude_code ?? "";
      this.autoPickerEffort = p.autoPickerEffort ?? "low";
      this.historyLimit = p.historyLimit ?? 30;
      this.defaultCwd = p.defaultCwd ?? "";
      this.chatSleepEnabled = p.chatSleepEnabled ?? false;
      this.chatSleepMin = p.chatSleepMin ?? 15;
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
      hermesModel: this.hermesModel,
      hermesReflectPasses: this.hermesReflectPasses,
      defaultPermMode: this.defaultPermMode,
      autoEffort: this.autoEffort,
      autoModel: this.autoModel,
      autoModels: this.autoModels,
      autoEfforts: this.autoEfforts,
      autoPickModel: this.autoPickModel,
      autoPicker: this.autoPicker,
      autoPickerEffort: this.autoPickerEffort,
      historyLimit: this.historyLimit,
      defaultCwd: this.defaultCwd,
      chatSleepEnabled: this.chatSleepEnabled,
      chatSleepMin: this.chatSleepMin,
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
  defaultModelFor(provider: string): string | null {
    return this.defaultModel[provider] ?? null;
  }
  defaultEffortFor(provider: string): string | null {
    return this.defaultEffort[provider] ?? null;
  }
  setDefaultModel(provider: string, v: string) {
    this.defaultModel = { ...this.defaultModel, [provider]: v || null };
    this.save();
  }
  setDefaultEffort(provider: string, v: string) {
    this.defaultEffort = { ...this.defaultEffort, [provider]: v || null };
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
  setHermesModel(v: string) {
    this.hermesModel = v;
    this.save();
  }
  setHermesReflectPasses(v: number) {
    this.hermesReflectPasses = Math.max(1, Math.min(10, v));
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
  // --- Auto par provider (candidats modèles/efforts + picker) ---
  autoModelsFor(provider: string): string[] {
    return this.autoModels[provider] ?? [];
  }
  autoEffortsFor(provider: string): string[] {
    return this.autoEfforts[provider] ?? [];
  }
  autoPickModelFor(provider: string): string {
    return this.autoPickModel[provider] ?? "";
  }

  /** Coche/décoche un modèle candidat de l'auto pour un provider. */
  toggleAutoModelChoice(provider: string, model: string) {
    const cur = this.autoModels[provider] ?? [];
    const next = cur.includes(model) ? cur.filter((m) => m !== model) : [...cur, model];
    this.autoModels = { ...this.autoModels, [provider]: next };
    this.save();
  }
  setDefaultCwd(v: string) {
    this.defaultCwd = v || "";
    this.save();
  }
  /** Remplace la liste des modèles candidats de l'auto pour un provider. */
  setAutoModels(provider: string, list: string[]) {
    this.autoModels = { ...this.autoModels, [provider]: [...list] };
    this.save();
  }
  /** Remplace la liste des efforts candidats de l'auto pour un provider. */
  setAutoEfforts(provider: string, list: string[]) {
    this.autoEfforts = { ...this.autoEfforts, [provider]: [...list] };
    this.save();
  }
  /** Vrai si le provider n'a jamais été configuré (→ on pré-coche tous les candidats). */
  autoModelsUnset(provider: string): boolean {
    return this.autoModels[provider] === undefined;
  }
  autoEffortsUnset(provider: string): boolean {
    return this.autoEfforts[provider] === undefined;
  }
  setAutoPickModel(provider: string, v: string) {
    this.autoPickModel = { ...this.autoPickModel, [provider]: v || "" };
    this.save();
  }
  /** Choisisseur global (cross-IA). */
  setAutoPicker(v: string) {
    this.autoPicker = v || "";
    this.save();
  }
  setAutoPickerEffort(v: string) {
    this.autoPickerEffort = v || "low";
    this.save();
  }
  setHistoryLimit(v: number) {
    this.historyLimit = Math.min(200, Math.max(1, Math.round(v) || 30));
    this.save();
  }
  setChatSleepEnabled(v: boolean) {
    this.chatSleepEnabled = v;
    this.save();
  }
  /** Délai d'inactivité (min) avant veille, borné à [1, 240]. */
  setChatSleepMin(v: number) {
    this.chatSleepMin = Math.min(240, Math.max(1, Math.round(v) || 15));
    this.save();
  }
  /** Coche/décoche un effort candidat de l'auto pour un provider. */
  toggleAutoEffortChoice(provider: string, effort: string) {
    const cur = this.autoEfforts[provider] ?? [];
    const next = cur.includes(effort) ? cur.filter((e) => e !== effort) : [...cur, effort];
    this.autoEfforts = { ...this.autoEfforts, [provider]: next };
    this.save();
  }

  private key(provider: string, model: string) {
    return `${provider}:${model}`;
  }

  /** Marque un modèle indisponible. cooldownMs > 0 = temporaire (rate-limit), sinon permanent. */
  markModelUnavailable(model: string, provider = "claude_code", cooldownMs = 0) {
    if (!model) return;
    this.unavailableModels = this.purgeExpired({
      ...this.unavailableModels,
      [this.key(provider, model)]: cooldownMs > 0 ? Date.now() + cooldownMs : 0,
    });
    this.save();
  }

  /** Modèle indisponible ? PUR (aucune mutation d'état — appelé dans des $derived).
   *  Un cooldown expiré redevient simplement « dispo » ; l'entrée résiduelle est nettoyée
   *  à la prochaine écriture via `markModelUnavailable` (purgeExpired). */
  isUnavailable(provider: string, model: string): boolean {
    const exp = this.unavailableModels[this.key(provider, model)];
    if (exp === undefined) return false;
    if (exp === 0) return true; // permanent
    return Date.now() <= exp; // cooldown encore actif
  }

  /** Retire les cooldowns expirés (appelé hors rendu, sur écriture). */
  private purgeExpired(map: Record<string, number>): Record<string, number> {
    const now = Date.now();
    return Object.fromEntries(Object.entries(map).filter(([, exp]) => exp === 0 || now <= exp));
  }
}

export const settings = new SettingsStore();
