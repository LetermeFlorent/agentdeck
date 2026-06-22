// Store des sessions : état de chat par session + abonnement aux events backend.

import * as ipc from "$lib/ipc";
import type { SessionEvent } from "$lib/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { usage } from "./data/usage.svelte";
import { settings } from "./settings.svelte";
import { activity } from "./data/activity.svelte";
import { STORAGE_KEYS } from "./keys";
import { PERM_MODES, PROVIDERS, effortsForProvider, autoPickPrompt, tierOf, providerInfo, providerOfModel } from "$lib/components/chat/chat-config";
import { modelStore } from "./data/models.svelte";
import { auth } from "./auth.svelte";

export interface ToolCall {
  name: string;
  /** Résumé de l'entrée : commande, fichier, motif… (affichage terminal). */
  input: string;
}

export interface Msg {
  role: "user" | "assistant";
  text: string;
  /** Réflexion (thinking) de l'assistant, streamée — affichée en mode terminal. */
  thinking: string;
  /** Outils appelés par l'assistant (nom + commande). */
  toolCalls: ToolCall[];
  /** Vignettes (data URLs) des images jointes à un message utilisateur — affichage seul. */
  images?: string[];
  /** Modèle réellement utilisé pour cette réponse (rempli à la fin du tour). */
  model?: string;
  /** Effort réellement utilisé pour cette réponse (snapshot du choix auto/manuel au moment du tour). */
  effort?: string;
  /** Passe de réflexion multi-tour : numéro et total (ex. {n:2, total:3}). */
  pass?: { n: number; total: number };
  /** Message auto-envoyé par le système (passe de vérification) — affiché de façon compacte. */
  autoPass?: boolean;
}

export interface SessionState {
  id: string;
  title: string;
  /** IA pilotée par ce chat : "claude_code" (défaut), "opencode" ou "gemini". */
  provider: string;
  model: string | null;
  effort: string | null;
  messages: Msg[];
  streaming: boolean;
  error: string | null;
  collapsed: boolean;
  priv: boolean;
  /** En veille : le process claude en fond est tué (éco RAM) ; réveil au clic/focus. */
  asleep: boolean;
  /** Messages tapés pendant que Claude travaille : envoyés l'un après l'autre. */
  queue: string[];
  /** Début du tour en cours (ms) pour l'indicateur de réflexion ; null si inactif. */
  turnStart: number | null;
  /** Tokens de sortie cumulés du tour en cours. */
  turnTokens: number;
  /** Tokens totaux générés par ce chat (cumul des tours). */
  totalTokens: number;
  /** Coût cumulé du chat (USD). */
  costUsd: number;
  /** Remplissage du contexte au dernier tour (entrée + cache) = jauge de contexte. */
  contextTokens: number;
  /** Fenêtre de contexte réelle du modèle (rapportée par Claude Code) ; 0 si inconnue. */
  contextWindow: number;
  /** Dernière activité (ms) — sert au passage auto en mode privé après inactivité. */
  lastActivity: number;
  /** Zoom du chat (1 = 100%) — boutons +/− de l'entête. */
  zoom: number;
  /** Mode de permission Claude Code (bypassPermissions par défaut). */
  permMode: string;
  /** Outils décochés (refusés) → passés en --disallowedTools. */
  disabledTools: string[];
  /** Règles avancées d'outils autorisés (motifs, ex. "Bash(git *)"). */
  allowRules: string;
  /** Règles avancées d'outils refusés (motifs). */
  denyRules: string;
  /** Bulles transitoires près des contrôles après un cycle clavier (Ctrl+Tab/M/E). */
  permFlash: string;
  modelFlash: string;
  effortFlash: string;
  /** Désactive l'auto-modèle / auto-effort uniquement pour ce chat (override du global). */
  autoModelOff: boolean;
  autoEffortOff: boolean;
  /** Brouillon en cours (non envoyé) — survit au repli/dépli du pane. */
  draft: string;
  /** Dossier de travail (où est le projet) ; Claude y est lancé (current_dir). */
  cwd: string;
  /** Passe de réflexion multi-tour en cours (1-based). 0 = inactif. */
  reflectPass: number;
  /** Nombre total de passes de réflexion demandées. 0 = inactif. */
  reflectTotal: number;
}

export interface PersistedSession {
  id: string;
  title: string;
  provider?: string;
  model: string | null;
  effort: string | null;
  messages: Msg[];
  collapsed?: boolean;
  priv?: boolean;
  /** Cumuls conservés entre deux lancements (coût/tokens affichés dans l'entête). */
  totalTokens?: number;
  costUsd?: number;
  contextTokens?: number;
  contextWindow?: number;
  zoom?: number;
  permMode?: string;
  disabledTools?: string[];
  allowRules?: string;
  denyRules?: string;
  autoModelOff?: boolean;
  autoEffortOff?: boolean;
  cwd?: string;
}

class SessionsStore {
  map = $state<Record<string, SessionState>>({});
  /** Incrémenté à chaque changement à persister (sauvegarde déclenchée côté +page). */
  persistRev = $state(0);
  /** Modèle / effort par défaut (Claude Code courant), pour pré-remplir les nouveaux panes. */
  defaultModel = $state<string | null>(null);
  defaultEffort = $state<string | null>(null);
  /** Dossier personnel (cwd par défaut si non configuré). */
  homePath = $state("");
  /** Derniers dossiers de travail utilisés (récents), pour le sélecteur. */
  cwdRecents = $state<string[]>([]);
  /** Commandes slash par provider (nom + description, récupérées dynamiquement). */
  slashCommandsByProvider = $state<Record<string, ipc.SlashCmd[]>>({});
  /** Outils disponibles exposés par Claude Code (init), pour le panneau Permissions. */
  tools = $state<string[]>([]);
  /** Chat actuellement focus (composer) — cible des raccourcis clavier (Ctrl+Tab). */
  focusedSid = $state("");
  /** Niveaux d'effort valides (lus dynamiquement du CLI), pour le mode auto. */
  effortLevels = $state<string[]>([]);
  private unlisteners: Record<string, UnlistenFn> = {};
  private privacyTimer: number | null = null;
  private slashFetchedFor = new Set<string>();

  /** Retourne les commandes slash du provider courant. */
  slashCommandsFor(provider: string): ipc.SlashCmd[] {
    return this.slashCommandsByProvider[provider] ?? [];
  }

  private touch() {
    this.persistRev++;
  }

  /** (Ré)attache l'écouteur d'events d'une session sans jamais en laisser deux (évite les réponses dupliquées). */
  private async attach(id: string) {
    const old = this.unlisteners[id];
    if (old) old();
    this.unlisteners[id] = await ipc.onSessionEvent(id, (e) => this.handle(id, e));
  }

  async loadDefaults() {
    try {
      this.homePath = await ipc.homeDir();
    } catch {
      /* ignore */
    }
    try {
      const cached = JSON.parse(localStorage.getItem("agentdeck.cwd.recents") || "[]");
      if (Array.isArray(cached)) this.cwdRecents = cached;
    } catch {
      /* ignore */
    }
    try {
      const d = await ipc.claudeDefaults();
      this.defaultModel = d.model || null;
      this.defaultEffort = d.effort || null;
    } catch {
      this.defaultModel = "opus";
      this.defaultEffort = "medium";
    }
  }

  /** Charge les commandes slash du provider donné (cache localStorage + fetch backend). */
  async loadSlashCommands(provider: string) {
    const cacheKey = `${STORAGE_KEYS.slash}:${provider}`;
    if (!this.slashCommandsByProvider[provider]?.length) {
      try {
        const cached = JSON.parse(localStorage.getItem(cacheKey) || "[]");
        if (Array.isArray(cached) && cached.length)
          this.slashCommandsByProvider[provider] = cached;
      } catch {
        /* ignore */
      }
    }
    if (this.slashFetchedFor.has(provider)) return;
    this.slashFetchedFor.add(provider);
    try {
      const list = await ipc.slashCommandsFetch(provider);
      if (list.length) {
        this.slashCommandsByProvider[provider] = list;
        localStorage.setItem(cacheKey, JSON.stringify(list));
      }
    } catch {
      this.slashFetchedFor.delete(provider);
    }
  }

  /** Résout un modèle (alias/ID) vers un ID présent dans la liste réelle (sinon dernier du tier). */
  resolveModel(m: string | null | undefined): string | null {
    if (!m) return m ?? null;
    const list = modelStore.available;
    if (list.some((x) => x.v === m)) return m; // déjà un ID valide
    const t = tierOf(m);
    if (!t) return m;
    const same = list.filter((x) => tierOf(x.v) === t).sort((a, b) => b.v.localeCompare(a.v));
    return same.length ? same[0].v : m; // dernier du même tier, sinon tel quel
  }

  /** Défaut effectif = override utilisateur (réglages) sinon modèle/effort Claude Code courant. */
  get effModel(): string | null {
    return this.resolveModel(settings.defaultModelFor("claude_code") ?? this.defaultModel);
  }
  get effEffort(): string | null {
    return settings.defaultEffortFor("claude_code") ?? this.defaultEffort;
  }
  /** Dossier de travail par défaut = réglage utilisateur, sinon dossier personnel. */
  get effCwd(): string {
    return settings.defaultCwd || this.homePath;
  }

  async create(opts: { title?: string; cwd?: string; model?: string; provider?: string } = {}): Promise<string> {
    const cwd = opts.cwd ?? this.effCwd;
    const provider = opts.provider ?? (auth.anyConnected ? (PROVIDERS.find((p) => auth.isConnected(p.id))?.id ?? "claude_code") : "claude_code");
    const id = await ipc.sessionCreate({ title: opts.title, cwd, model: opts.model, provider });
    // Charge la liste réelle du provider avant de choisir le modèle.
    if (provider !== "claude_code") await modelStore.loadFor(provider);
    // Modèle par défaut : Claude → défaut utilisateur ; autres → défaut réglé sinon 1er modèle.
    const model =
      provider === "claude_code"
        ? opts.model
          ? this.resolveModel(opts.model)
          : this.effModel
        : opts.model ?? settings.defaultModelFor(provider) ?? modelStore.visibleFor(provider).find((m) => m.v.includes("-free"))?.v ?? modelStore.visibleFor(provider)[0]?.v ?? null;
    this.map[id] = {
      id,
      title: opts.title ?? PROVIDERS.find((p) => p.id === provider)?.label ?? "Claude",
      provider,
      model,
      effort: providerInfo(provider).hasEffort
        ? (settings.defaultEffortFor(provider) ?? (provider === "claude_code" ? this.effEffort : "medium"))
        : null,
      cwd,
      messages: [],
      streaming: false,
      error: null,
      collapsed: false,
      priv: false,
      asleep: false,
      queue: [],
      turnStart: null,
      turnTokens: 0,
      totalTokens: 0,
      costUsd: 0,
      contextTokens: 0,
      contextWindow: 0,
      lastActivity: Date.now(),
      zoom: settings.defaultZoom ?? 1,
      permMode: settings.defaultPermMode ?? "bypassPermissions",
      disabledTools: [],
      allowRules: "",
      denyRules: "",
      permFlash: "",
      modelFlash: "",
      effortFlash: "",
      autoModelOff: false,
      autoEffortOff: false,
      draft: "",
      reflectPass: 0,
      reflectTotal: 0,
    };
    this.focusedSid = id;
    await this.attach(id);
    this.touch();
    return id;
  }

  /** Ouvre une conversation de l'historique : enregistre la session (resume) + messages reconstruits. */
  async openExisting(
    id: string,
    title: string,
    cwd: string,
    msgs: { role: "user" | "assistant"; text: string }[],
  ): Promise<string> {
    if (this.map[id]) {
      this.focusedSid = id;
      return id;
    }
    try {
      await ipc.sessionRestore({ id, title, started: true, cwd: cwd || undefined });
    } catch {
      /* ignore */
    }
    this.map[id] = {
      id,
      title: title || "Claude",
      provider: "claude_code",
      model: this.effModel,
      effort: this.effEffort,
      messages: msgs.map((m) => ({ role: m.role, text: m.text, thinking: "", toolCalls: [] })),
      streaming: false,
      error: null,
      collapsed: false,
      priv: false,
      asleep: false,
      queue: [],
      turnStart: null,
      turnTokens: 0,
      totalTokens: 0,
      costUsd: 0,
      contextTokens: 0,
      contextWindow: 0,
      lastActivity: Date.now(),
      zoom: settings.defaultZoom ?? 1,
      permMode: settings.defaultPermMode ?? "bypassPermissions",
      disabledTools: [],
      allowRules: "",
      denyRules: "",
      permFlash: "",
      modelFlash: "",
      effortFlash: "",
      autoModelOff: false,
      autoEffortOff: false,
      draft: "",
      cwd: cwd || this.effCwd,
      reflectPass: 0,
      reflectTotal: 0,
    };
    this.focusedSid = id;
    await this.attach(id);
    this.touch();
    return id;
  }

  /** Restaure des sessions persistées au démarrage (réenregistre côté backend pour --resume). */
  async hydrate(list: PersistedSession[]) {
    // Sessions indépendantes → on les restaure en parallèle (au lieu d'enchaîner les
    // allers-retours IPC séquentiellement, ce qui ralentit la réouverture du deck).
    await Promise.all(list.map((p) => this.hydrateOne(p)));
  }

  private async hydrateOne(p: PersistedSession) {
    const started = p.messages.some((m) => m.role === "user");
    const provider = p.provider ?? "claude_code";
    await ipc.sessionRestore({
      id: p.id,
      title: p.title,
      started,
      cwd: p.cwd ?? this.effCwd,
      model: p.model ?? undefined,
      provider,
    });
    if (provider !== "claude_code") modelStore.loadFor(provider);
    this.map[p.id] = {
      id: p.id,
      title: p.title,
      provider,
      model: provider === "claude_code" ? (p.model ? this.resolveModel(p.model) : this.effModel) : p.model,
      effort: p.effort ?? this.effEffort,
      messages: p.messages,
      streaming: false,
      error: null,
      collapsed: p.collapsed ?? false,
      priv: p.priv ?? false,
      asleep: false,
      queue: [],
      turnStart: null,
      turnTokens: 0,
      totalTokens: p.totalTokens ?? 0,
      costUsd: p.costUsd ?? 0,
      contextTokens: p.contextTokens ?? 0,
      contextWindow: p.contextWindow ?? 0,
      lastActivity: Date.now(),
      zoom: p.zoom ?? 1,
      permMode: p.permMode ?? "bypassPermissions",
      disabledTools: p.disabledTools ?? [],
      allowRules: p.allowRules ?? "",
      denyRules: p.denyRules ?? "",
      permFlash: "",
      modelFlash: "",
      effortFlash: "",
      autoModelOff: p.autoModelOff ?? false,
      autoEffortOff: p.autoEffortOff ?? false,
      draft: "",
      cwd: p.cwd ?? this.effCwd,
      reflectPass: 0,
      reflectTotal: 0,
    };
    await this.attach(p.id);
  }

  serialize(): PersistedSession[] {
    return Object.values(this.map).map((s) => ({
      id: s.id,
      title: s.title,
      provider: s.provider,
      model: s.model,
      effort: s.effort,
      // Quota localStorage : on ne persiste ni les data URLs d'images ni la réflexion (volumineuse).
      messages: s.messages.map((m) =>
        m.images?.length || m.thinking ? { ...m, images: undefined, thinking: "" } : m,
      ),
      collapsed: s.collapsed,
      priv: s.priv,
      totalTokens: s.totalTokens,
      costUsd: s.costUsd,
      contextTokens: s.contextTokens,
      contextWindow: s.contextWindow,
      zoom: s.zoom,
      permMode: s.permMode,
      disabledTools: s.disabledTools,
      allowRules: s.allowRules,
      denyRules: s.denyRules,
      autoModelOff: s.autoModelOff,
      autoEffortOff: s.autoEffortOff,
      cwd: s.cwd,
    }));
  }

  /** Replie / déplie un pane (minimise sur le côté). */
  setCollapsed(id: string, collapsed: boolean) {
    const s = this.map[id];
    if (!s) return;
    s.collapsed = collapsed;
    this.touch();
  }

  /** Zoom du chat : incrément (±0.1), borné à [0.6, 1.8]. */
  setZoom(id: string, delta: number) {
    const s = this.map[id];
    if (!s) return;
    s.zoom = Math.min(1.8, Math.max(0.6, Math.round((s.zoom + delta) * 10) / 10));
    this.touch();
  }

  /** Mode privé : floute le contenu (veille), garde le statut visible. */
  setPrivate(id: string, priv: boolean) {
    const s = this.map[id];
    if (!s) return;
    s.priv = priv;
    this.touch();
  }

  /** Renomme un pane. */
  setTitle(id: string, title: string) {
    const s = this.map[id];
    if (!s) return;
    s.title = title;
    this.touch();
  }

  /** Change l'IA d'un pane : réinitialise modèle/effort sur ceux du nouveau provider + recharge la liste. */
  async setProvider(id: string, provider: string) {
    const s = this.map[id];
    if (!s || s.provider === provider) return;
    const oldLabel = PROVIDERS.find((p) => p.id === (s.provider ?? "claude_code"))?.label ?? "Claude";
    const newLabel = PROVIDERS.find((p) => p.id === provider)?.label ?? provider;
    if (!s.title || s.title === oldLabel) s.title = newLabel;
    s.provider = provider;
    await modelStore.loadFor(provider);
    s.model = modelStore.visibleFor(provider).find((m) => m.v.includes("-free"))?.v ?? modelStore.visibleFor(provider)[0]?.v ?? null;
    s.effort = providerInfo(provider).hasEffort
      ? (settings.defaultEffortFor(provider) ?? (provider === "claude_code" ? this.effEffort : "medium"))
      : null;
    // Coupe l'éventuel process en cours (ex. Claude persistant) — relancé au prochain envoi.
    this.stop(id);
    this.touch();
  }

  /** Change le modèle d'un pane (appliqué au prochain tour). Réinitialise l'effort si incompatible. */
  setModel(id: string, model: string) {
    const s = this.map[id];
    if (!s) return;
    s.model = model || null;
    const valid = effortsForProvider(s.provider, s.model).map((e) => e.v);
    if (valid.length && s.effort && !valid.includes(s.effort)) {
      s.effort = s.provider === "claude_code" ? this.effEffort : (settings.defaultEffortFor(s.provider) ?? "medium");
    } else if (!valid.length) {
      s.effort = null;
    }
    this.touch();
  }
  setEffort(id: string, effort: string) {
    const s = this.map[id];
    if (!s) return;
    s.effort = effort || null;
    this.touch();
  }

  /** Permissions du pane (appliquées au prochain tour → respawn du process). */
  setPermMode(id: string, mode: string) {
    const s = this.map[id];
    if (!s) return;
    s.permMode = mode || "bypassPermissions";
    this.touch();
  }
  /** Bascule un outil autorisé/refusé (refusé = présent dans disabledTools). */
  toggleTool(id: string, tool: string) {
    const s = this.map[id];
    if (!s) return;
    s.disabledTools = s.disabledTools.includes(tool)
      ? s.disabledTools.filter((t) => t !== tool)
      : [...s.disabledTools, tool];
    this.touch();
  }
  setAllowRules(id: string, v: string) {
    const s = this.map[id];
    if (s) { s.allowRules = v; this.touch(); }
  }
  setDenyRules(id: string, v: string) {
    const s = this.map[id];
    if (s) { s.denyRules = v; this.touch(); }
  }

  /** Change le dossier de travail d'un chat (relance le process dedans au prochain envoi). */
  async setCwd(id: string, cwd: string) {
    const s = this.map[id];
    if (!s || !cwd) return;
    s.cwd = cwd;
    // Récents (dédupliqués, max 12).
    this.cwdRecents = [cwd, ...this.cwdRecents.filter((r) => r !== cwd)].slice(0, 12);
    try {
      localStorage.setItem("agentdeck.cwd.recents", JSON.stringify(this.cwdRecents));
    } catch {
      /* ignore */
    }
    this.touch();
    try {
      await ipc.sessionSetCwd(id, cwd);
    } catch {
      /* ignore */
    }
  }

  /** Mémorise le chat focus (cible des raccourcis clavier). */
  setFocused(id: string) {
    this.focusedSid = id;
  }
  /** Active/désactive l'auto-modèle pour ce chat seulement. */
  toggleAutoModelOff(id: string) {
    const s = this.map[id];
    if (s) { s.autoModelOff = !s.autoModelOff; this.touch(); }
  }
  /** Active/désactive l'auto-effort pour ce chat seulement. */
  toggleAutoEffortOff(id: string) {
    const s = this.map[id];
    if (s) { s.autoEffortOff = !s.autoEffortOff; this.touch(); }
  }
  /** Affiche une bulle transitoire (~1,8 s) sur un contrôle après un cycle clavier. */
  private flash(s: SessionState, key: "permFlash" | "modelFlash" | "effortFlash", label: string) {
    s[key] = label;
    this.touch();
    window.setTimeout(() => {
      if (s[key] === label) s[key] = "";
    }, 1800);
  }

  /** Cycle le mode de permission (Ctrl+Tab). */
  cyclePermMode(id: string) {
    const s = this.map[id];
    if (!s) return;
    const i = PERM_MODES.findIndex((m) => m.v === (s.permMode ?? "bypassPermissions"));
    const next = PERM_MODES[(i + 1) % PERM_MODES.length];
    s.permMode = next.v;
    this.flash(s, "permFlash", next.l);
  }

  /** Cycle le modèle parmi les modèles disponibles (Ctrl+M). */
  cycleModel(id: string) {
    const s = this.map[id];
    if (!s) return;
    const avail = modelStore.visibleFor(s.provider).filter((m) => !settings.isUnavailable(s.provider, m.v));
    if (!avail.length) return;
    const i = avail.findIndex((m) => m.v === s.model);
    const next = avail[(i + 1) % avail.length];
    s.model = next.v;
    this.flash(s, "modelFlash", next.l);
  }

  /** Cycle l'effort parmi ceux valides pour le modèle courant (Ctrl+E). */
  cycleEffort(id: string) {
    const s = this.map[id];
    if (!s) return;
    const list = effortsForProvider(s.provider, s.model);
    if (!list.length) {
      this.flash(s, "effortFlash", "Non réglable");
      return;
    }
    const i = list.findIndex((e) => e.v === s.effort);
    const next = list[(i + 1) % list.length];
    s.effort = next.v;
    this.flash(s, "effortFlash", next.l);
  }

  async send(
    id: string,
    text: string,
    images: { dataUrl: string; media_type: string; data: string; name?: string }[] = [],
  ) {
    const s = this.map[id];
    if (!s) return;
    s.error = null;
    s.messages.push({
      role: "user",
      text,
      thinking: "",
      toolCalls: [],
      images: images.length ? images.map((i) => i.dataUrl) : undefined,
    });
    // Le process est persistant : on écrit toujours, même si Claude travaille (pris en cours de route).
    s.streaming = true;
    if (s.turnStart === null) s.turnStart = Date.now();
    s.turnTokens = 0;
    s.lastActivity = Date.now();
    this.touch();
    try {
      // Mode auto (global ET non désactivé pour ce chat) : choisit effort/modèle avant l'envoi.
      const autoEff = settings.autoEffort && !s.autoEffortOff;
      const autoMod = settings.autoModel && !s.autoModelOff;
      if (autoEff || autoMod) {
        if (autoEff && !this.effortLevels.length) {
          try { this.effortLevels = await ipc.effortLevels(); } catch { /* ignore */ }
        }
        // Candidats MODÈLES : cross-IA si auto-modèle (toutes IA connectées), sinon l'IA du chat.
        // Le sélecteur « IA configurée » des réglages ne sert qu'à AFFICHER les listes ; ici on
        // réunit les candidats cochés de chaque IA → l'auto peut router vers Claude OU opencode OU Gemini.
        const provs = autoMod
          ? ["claude_code", "opencode", "gemini"].filter((p) => auth.isConnected(p))
          : [s.provider];
        const idProv = new Map<string, string>();
        const modelInfos: { v: string; l: string }[] = [];
        for (const p of provs) {
          const vis = modelStore.visibleFor(p);
          const liveIds = new Set(vis.map((m) => m.v));
          let sel = settings.autoModelsFor(p).filter((m) => liveIds.has(m) && !settings.isUnavailable(p, m));
          // Aucun candidat coché pour l'IA du chat seule → repli sur ses modèles dispo (sauf Fable).
          if (autoMod && !sel.length && provs.length === 1) {
            sel = vis.filter((m) => tierOf(m.v) !== "fable" && !settings.isUnavailable(p, m.v)).map((m) => m.v);
          }
          for (const mid of sel) {
            if (idProv.has(mid)) continue;
            idProv.set(mid, p);
            const info = modelStore.availableFor(p).find((m) => m.v === mid) ?? { v: mid, l: mid };
            modelInfos.push({ v: mid, l: `${providerInfo(p).label}: ${info.l}` }); // préfixe IA pour le picker
          }
        }
        const availIds = modelInfos.map((m) => m.v);
        // Efforts candidats = union des IA concernées.
        const effSet = new Set<string>();
        for (const p of provs) for (const e of settings.autoEffortsFor(p)) effSet.add(e);
        let effs = [...effSet];
        if (s.provider === "claude_code" && !autoMod && this.effortLevels.length) {
          effs = effs.filter((e) => this.effortLevels.includes(e));
        }
        const effInfos = effs.map((v) => ({ v, l: v }));
        const instruction = autoPickPrompt(text, autoMod ? modelInfos : [], autoEff ? effInfos : []);
        try {
          // Picker GLOBAL unique : un seul décideur (n'importe quelle IA connectée) choisit IA + modèle.
          const picker = settings.autoPicker || modelStore.pickerDefaultFor(s.provider);
          const pick = await ipc.autoPick(providerOfModel(picker), instruction, autoMod ? availIds : [], autoEff ? effs : [], picker, settings.autoPickerEffort || "low");
          if (autoMod && pick.model) {
            const newProv = idProv.get(pick.model) ?? providerOfModel(pick.model);
            if (newProv !== s.provider) {
              await this.stop(id); // coupe l'ancien process (ex. Claude persistant)
              s.provider = newProv;
              modelStore.loadFor(newProv);
            }
            s.model = pick.model;
            this.flash(s, "modelFlash", "auto: " + providerInfo(newProv).label + " " + pick.model);
          }
          if (autoEff && pick.effort) {
            // Valide l'effort pour l'IA finale (l'effort dépend du modèle/IA).
            const valid = effortsForProvider(s.provider, s.model).some((e) => e.v === pick.effort);
            if (valid) {
              s.effort = pick.effort;
              this.flash(s, "effortFlash", "auto: " + pick.effort);
            }
          }
        } catch { /* ignore : garde le réglage courant */ }
      }
      // Permissions : refusés = outils décochés + règles refusées ; autorisés = règles avancées.
      const deny = [...s.disabledTools, ...(s.denyRules ? [s.denyRules] : [])].join(",");
      const passes = settings.hermesReflectPasses ?? 1;
      if (passes > 1) {
        s.reflectPass = 1;
        s.reflectTotal = passes;
      } else {
        s.reflectPass = 0;
        s.reflectTotal = 0;
      }
      const finalText = text;
      await ipc.sessionSend(
        id,
        finalText,
        s.model,
        s.effort,
        images.map((i) => ({ media_type: i.media_type, data: i.data, name: i.name })),
        settings.hermesMode,
        { mode: s.permMode, allowed: s.allowRules || null, disallowed: deny || null },
        s.provider,
      );
    } catch (err) {
      s.error = String(err);
      s.streaming = false;
    }
  }

  /** Envoie automatiquement une passe de vérification multi-tour. */
  async sendReflectPass(id: string, n: number, total: number) {
    const s = this.map[id];
    if (!s || s.streaming) return;
    s.reflectPass = n;
    s.reflectTotal = total;
    const verifyPrompt = `[Passe ${n}/${total} — Vérification] Relis ta réponse précédente. Identifie les erreurs, imprécisions ou points manquants. Corrige et complète.`;
    s.messages.push({ role: "user", text: verifyPrompt, thinking: "", toolCalls: [], autoPass: true, pass: { n, total } });
    s.streaming = true;
    s.error = null;
    s.turnStart = Date.now();
    s.turnTokens = 0;
    s.messages.push({ role: "assistant", text: "", thinking: "", toolCalls: [] });
    try {
      const deny = [...s.disabledTools, ...(s.denyRules ? [s.denyRules] : [])].join(",");
      await ipc.sessionSend(
        id,
        verifyPrompt,
        s.model,
        s.effort,
        [],
        settings.hermesMode,
        { mode: s.permMode, allowed: s.allowRules || null, disallowed: deny || null },
        s.provider,
      );
    } catch (err) {
      s.error = String(err);
      s.streaming = false;
    }
  }

  /** Marque une activité sur le chat (saisie, focus…) → repousse la veille + réveille. */
  touchActivity(id: string) {
    const s = this.map[id];
    if (!s) return;
    s.lastActivity = Date.now();
    if (s.asleep) {
      s.asleep = false; // le process repart au prochain envoi via --resume
      this.touch();
    }
  }

  /** Met manuellement un chat en veille (bouton) : tue le process, garde les messages. */
  sleepNow(id: string) {
    const s = this.map[id];
    if (s && !s.asleep) {
      s.asleep = true;
      this.stop(id); // tue le process claude (RAM libérée)
      this.touch();
    }
  }

  /** Réveille un chat en veille (clic sur le pane) sans rien envoyer. */
  wake(id: string) {
    const s = this.map[id];
    if (s?.asleep) {
      s.asleep = false;
      s.lastActivity = Date.now();
      this.touch();
    }
  }

  /** Surveille l'inactivité : mode privé (floute) + veille (tue le process) après les délais réglés. */
  startPrivacyWatch() {
    if (this.privacyTimer !== null) return;
    this.privacyTimer = window.setInterval(() => {
      this.checkPrivacy();
      this.checkSleep();
    }, 15_000);
  }
  stopPrivacyWatch() {
    if (this.privacyTimer !== null) {
      window.clearInterval(this.privacyTimer);
      this.privacyTimer = null;
    }
  }
  private checkPrivacy() {
    const min = settings.privateAfterMin;
    if (!min || min <= 0) return; // 0 / null = désactivé
    const now = Date.now();
    const ms = min * 60_000;
    for (const s of Object.values(this.map)) {
      if (!s.priv && !s.streaming && now - s.lastActivity > ms) {
        s.priv = true;
        this.touch();
      }
    }
  }

  /** Veille : tue le process claude des chats inactifs (éco RAM). Réveil au clic/envoi. */
  private checkSleep() {
    if (!settings.chatSleepEnabled) return;
    const ms = settings.chatSleepMin * 60_000;
    const now = Date.now();
    for (const s of Object.values(this.map)) {
      if (!s.asleep && !s.streaming && now - s.lastActivity > ms) {
        s.asleep = true;
        this.stop(s.id); // tue le process, garde l'enregistrement + les messages
        this.touch();
      }
    }
  }

  async stop(id: string) {
    try {
      await ipc.sessionStop(id);
    } catch {
      /* ignore */
    }
  }

  async close(id: string) {
    const un = this.unlisteners[id];
    if (un) un();
    delete this.unlisteners[id];
    try {
      await ipc.sessionClose(id);
    } catch {
      /* ignore */
    }
    delete this.map[id];
    this.touch();
  }

  /** Ferme TOUTES les sessions (vrai logout : tue les process CLI + fichiers session backend). */
  async closeAll() {
    await Promise.all(Object.keys(this.map).map((id) => this.close(id)));
  }

  /** Mode Hermes : sur échec, demande au backend de réfléchir et d'en tirer un skill. */
  private maybeLearn(s: SessionState, errorText: string) {
    if (!settings.hermesMode) return;
    const rev = [...s.messages].reverse();
    const request = rev.find((m) => m.role === "user")?.text ?? "";
    const asst = rev.find((m) => m.role === "assistant");
    const tools = asst?.toolCalls.map((t) => `${t.name}: ${t.input}`).join(" ; ") ?? "";
    const summary = `${asst?.text ?? ""}\n${tools}`.trim().slice(0, 2000);
    // Best-effort, asynchrone, ne bloque pas l'UI.
    ipc.reflectAndLearn(s.cwd || null, request.slice(0, 2000), summary, errorText.slice(0, 1000), settings.hermesModel || undefined, s.provider || undefined).catch(() => {});
  }

  private lastAssistant(s: SessionState): Msg {
    const last = s.messages[s.messages.length - 1];
    if (last && last.role === "assistant") return last;
    const m: Msg = { role: "assistant", text: "", thinking: "", toolCalls: [] };
    s.messages.push(m);
    return m;
  }

  private handle(id: string, e: SessionEvent) {
    const s = this.map[id];
    if (!s) return;
    switch (e.kind) {
      case "started":
        // Process prêt (pas une nouvelle bulle).
        s.error = null;
        break;
      case "init":
        if (e.tools?.length) this.tools = e.tools;
        if (e.slash_commands.length > 0) {
          const prov = s.provider ?? "claude_code";
          const cacheKey = `${STORAGE_KEYS.slash}:${prov}`;
          const existing = this.slashCommandsByProvider[prov] ?? [];
          const have = new Set(existing.map((c) => c.name));
          const merged = [...existing];
          for (const n of e.slash_commands) {
            if (!have.has(n)) merged.push({ name: n, description: "", args: "", cli: "Claude" });
          }
          if (merged.length !== existing.length) {
            this.slashCommandsByProvider[prov] = merged;
            try {
              localStorage.setItem(cacheKey, JSON.stringify(merged));
            } catch {
              /* ignore */
            }
          }
        }
        break;
      case "assistant_start": {
        // Nouveau tour → nouvelle bulle assistant (sauf si une bulle vide attend déjà).
        s.streaming = true;
        s.error = null;
        if (s.turnStart === null) s.turnStart = Date.now();
        const last = s.messages[s.messages.length - 1];
        const emptyAssistant =
          last && last.role === "assistant" && !last.text && !last.thinking && last.toolCalls.length === 0;
        if (!emptyAssistant) {
          s.messages.push({ role: "assistant", text: "", thinking: "", toolCalls: [] });
        }
        break;
      }
      case "assistant_delta":
        this.lastAssistant(s).text += e.text;
        break;
      case "thinking":
        this.lastAssistant(s).thinking += e.text;
        break;
      case "tool_use":
        this.lastAssistant(s).toolCalls.push({ name: e.name, input: e.input });
        activity.handle(id, e); // suivi shells (Bash/PowerShell) qui tournent
        break;
      case "tool_done":
      case "task_started":
      case "task_progress":
      case "task_ended":
        activity.handle(id, e); // suivi sous-agents + fin d'outils
        break;
      case "progress":
        s.turnTokens = e.output_tokens;
        break;
      case "turn_done":
        s.streaming = false;
        s.turnStart = null;
        s.lastActivity = Date.now(); // fin de réponse → départ du compteur d'inactivité
        s.totalTokens += e.total_tokens;
        // Remplissage du contexte = prompt du dernier tour (remplace, pas de cumul).
        s.contextTokens = e.context_tokens;
        // Fenêtre réelle du modèle (dynamique) ; on garde la dernière valeur connue.
        if (e.context_window > 0) s.contextWindow = e.context_window;
        // total_cost_usd est cumulatif par process → on prend la dernière valeur.
        if (e.cost_usd >= s.costUsd || e.cost_usd === 0) s.costUsd = e.cost_usd || s.costUsd;
        else s.costUsd = e.cost_usd; // process relancé (modèle changé) → réinitialisé
        usage.refresh();
        // Modèle + effort réellement utilisés ce tour → attachés à la dernière bulle assistant.
        // L'effort = `s.effort` (déjà écrasé par le choix AUTO en amont si l'auto est actif),
        // snapshot ici pour ne pas bouger si l'utilisateur change le dropdown ensuite.
        {
          const last = s.messages[s.messages.length - 1];
          if (last && last.role === "assistant") {
            if (e.model) last.model = e.model;
            if (s.effort) last.effort = s.effort;
            if (s.reflectPass > 0 && s.reflectTotal > 0) {
              last.pass = { n: s.reflectPass, total: s.reflectTotal };
              if (s.reflectPass < s.reflectTotal) {
                const nextPass = s.reflectPass + 1;
                const total = s.reflectTotal;
                setTimeout(() => this.sendReflectPass(id, nextPass, total), 400);
              } else {
                s.reflectPass = 0;
                s.reflectTotal = 0;
              }
            }
          }
        }
        // Chat fini pendant que la fenêtre n'a pas le focus → clignote la barre des tâches.
        if (typeof document !== "undefined" && !document.hasFocus()) ipc.requestAttention();
        // Mode Hermes : tour soldé par une erreur → apprentissage auto.
        if (e.is_error) this.maybeLearn(s, "Le tour s'est terminé en erreur (is_error).");
        activity.clear(id); // fin du tour → plus rien ne tourne
        this.touch();
        break;
      case "error": {
        // Limite de débit (free tier) : exclusion TEMPORAIRE + bascule auto sur un modèle dispo.
        const rateLimited = /^RATE_LIMIT:/.test(e.message) || /resource.?exhausted|rate.?limit|quota|too many requests|\b429\b/i.test(e.message);
        // Modèle indisponible (permanent) : message d'erreur typé « modèle ».
        const modelIssue = /selected model|may not exist|access to it|model.*not.*available/i.test(e.message);
        if (s.model && rateLimited) {
          settings.markModelUnavailable(s.model, s.provider, 60_000); // cooldown ~60s
          const fb = modelStore.visibleFor(s.provider).find(
            (m) => m.v !== s.model && !settings.isUnavailable(s.provider, m.v),
          );
          if (fb) s.model = fb.v;
          s.error = "Modèle en limite de débit — bascule temporaire.";
          s.streaming = false;
        } else if (s.model && modelIssue) {
          s.error = e.message;
          s.streaming = false;
          settings.markModelUnavailable(s.model, s.provider);
          const fb = modelStore.visibleFor(s.provider).find(
            (m) => m.v !== s.model && !settings.isUnavailable(s.provider, m.v),
          );
          s.model = fb?.v ?? null;
        } else {
          s.error = e.message;
          s.streaming = false;
          // Mode Hermes : erreur réelle (hors souci de modèle) → apprentissage auto.
          this.maybeLearn(s, e.message);
        }
        this.touch();
        break;
      }
      case "exited":
        s.streaming = false;
        s.turnStart = null;
        activity.clear(id);
        break;
    }
  }
}

export const sessions = new SessionsStore();
