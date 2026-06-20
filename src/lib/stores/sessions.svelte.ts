// Store des sessions : état de chat par session + abonnement aux events backend.

import * as ipc from "$lib/ipc";
import type { SessionEvent } from "$lib/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { usage } from "./usage.svelte";
import { settings } from "./settings.svelte";
import { activity } from "./activity.svelte";
import { STORAGE_KEYS } from "./keys";
import { PERM_MODES, effortsFor, autoPickPrompt, tierOf } from "$lib/components/chat/chat-config";
import { modelStore } from "./models.svelte";

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
}

export interface SessionState {
  id: string;
  title: string;
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
}

export interface PersistedSession {
  id: string;
  title: string;
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
  /** Commandes slash exposées par Claude Code (nom + description, récupérées dynamiquement). */
  slashCommands = $state<ipc.SlashCmd[]>([]);
  /** Outils disponibles exposés par Claude Code (init), pour le panneau Permissions. */
  tools = $state<string[]>([]);
  /** Chat actuellement focus (composer) — cible des raccourcis clavier (Ctrl+Tab). */
  focusedSid = $state("");
  /** Niveaux d'effort valides (lus dynamiquement du CLI), pour le mode auto. */
  effortLevels = $state<string[]>([]);
  private unlisteners: Record<string, UnlistenFn> = {};
  private privacyTimer: number | null = null;
  private slashFetched = false;

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

  /** Charge la liste des commandes slash (cache localStorage + fetch backend), pour le « / » immédiat. */
  async loadSlashCommands() {
    // Cache (instantané) pour l'affichage, puis fetch backend pour rafraîchir/compléter.
    if (this.slashCommands.length === 0) {
      try {
        const cached = JSON.parse(localStorage.getItem(STORAGE_KEYS.slash) || "[]");
        if (Array.isArray(cached) && cached.length) this.slashCommands = cached;
      } catch {
        /* ignore */
      }
    }
    if (this.slashFetched) return; // un seul fetch backend par session (spawn process)
    this.slashFetched = true;
    try {
      const list = await ipc.slashCommandsFetch();
      if (list.length) {
        this.slashCommands = list;
        localStorage.setItem(STORAGE_KEYS.slash, JSON.stringify(list));
      }
    } catch {
      this.slashFetched = false; // échec → autorise une nouvelle tentative
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
    return this.resolveModel(settings.defaultModel ?? this.defaultModel);
  }
  get effEffort(): string | null {
    return settings.defaultEffort ?? this.defaultEffort;
  }
  /** Dossier de travail par défaut = réglage utilisateur, sinon dossier personnel. */
  get effCwd(): string {
    return settings.defaultCwd || this.homePath;
  }

  async create(opts: { title?: string; cwd?: string; model?: string } = {}): Promise<string> {
    const cwd = opts.cwd ?? this.effCwd;
    const id = await ipc.sessionCreate({ title: opts.title, cwd, model: opts.model });
    this.map[id] = {
      id,
      title: opts.title ?? "Claude",
      model: opts.model ? this.resolveModel(opts.model) : this.effModel,
      effort: this.effEffort,
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
    await ipc.sessionRestore({
      id: p.id,
      title: p.title,
      started,
      cwd: p.cwd ?? this.effCwd,
      model: p.model ?? undefined,
    });
    this.map[p.id] = {
      id: p.id,
      title: p.title,
      model: p.model ? this.resolveModel(p.model) : this.effModel,
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
    };
    await this.attach(p.id);
  }

  serialize(): PersistedSession[] {
    return Object.values(this.map).map((s) => ({
      id: s.id,
      title: s.title,
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

  /** Change le modèle / l'effort d'un pane (appliqué au prochain tour). */
  setModel(id: string, model: string) {
    const s = this.map[id];
    if (!s) return;
    s.model = model || null;
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
    const avail = modelStore.available.filter((m) => !settings.unavailableModels.includes(m.v));
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
    const list = effortsFor(s.model);
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
        // Modèles candidats = liste autorisée (réglages) ∩ modèles disponibles.
        const avail = settings.autoModels.filter((m) => !settings.unavailableModels.includes(m));
        // Efforts candidats = liste autorisée ∩ niveaux réellement détectés (si connus).
        const effs = this.effortLevels.length
          ? settings.autoEfforts.filter((e) => this.effortLevels.includes(e))
          : settings.autoEfforts;
        // Résout les libellés des modèles candidats (pour prix/récence dans le prompt).
        const modelInfos = avail.map((v) => modelStore.available.find((m) => m.v === v) ?? { v, l: v });
        const effInfos = effs.map((v) => ({ v, l: v }));
        const instruction = autoPickPrompt(text, autoMod ? modelInfos : [], autoEff ? effInfos : []);
        try {
          const pick = await ipc.autoPick(instruction, autoMod ? avail : [], autoEff ? effs : [], settings.autoPickModel || modelStore.pickerDefault);
          if (autoMod && pick.model) {
            s.model = pick.model;
            this.flash(s, "modelFlash", "auto: " + pick.model);
          }
          if (autoEff && pick.effort) {
            s.effort = pick.effort;
            this.flash(s, "effortFlash", "auto: " + pick.effort);
          }
        } catch { /* ignore : garde le réglage courant */ }
      }
      // Permissions : refusés = outils décochés + règles refusées ; autorisés = règles avancées.
      const deny = [...s.disabledTools, ...(s.denyRules ? [s.denyRules] : [])].join(",");
      await ipc.sessionSend(
        id,
        text,
        s.model,
        s.effort,
        images.map((i) => ({ media_type: i.media_type, data: i.data, name: i.name })),
        settings.hermesMode,
        { mode: s.permMode, allowed: s.allowRules || null, disallowed: deny || null },
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

  /** Mode Hermes : sur échec, demande au backend de réfléchir et d'en tirer un skill. */
  private maybeLearn(s: SessionState, errorText: string) {
    if (!settings.hermesMode) return;
    const rev = [...s.messages].reverse();
    const request = rev.find((m) => m.role === "user")?.text ?? "";
    const asst = rev.find((m) => m.role === "assistant");
    const tools = asst?.toolCalls.map((t) => `${t.name}: ${t.input}`).join(" ; ") ?? "";
    const summary = `${asst?.text ?? ""}\n${tools}`.trim().slice(0, 2000);
    // Best-effort, asynchrone, ne bloque pas l'UI.
    ipc.reflectAndLearn(s.cwd || null, request.slice(0, 2000), summary, errorText.slice(0, 1000)).catch(() => {});
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
          // Fusionne : garde les descriptions déjà connues, ajoute les noms manquants.
          const have = new Set(this.slashCommands.map((c) => c.name));
          const merged = [...this.slashCommands];
          for (const n of e.slash_commands) {
            if (!have.has(n)) merged.push({ name: n, description: "", args: "" });
          }
          if (merged.length !== this.slashCommands.length) {
            this.slashCommands = merged;
            try {
              localStorage.setItem(STORAGE_KEYS.slash, JSON.stringify(merged));
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
        // Chat fini pendant que la fenêtre n'a pas le focus → clignote la barre des tâches.
        if (typeof document !== "undefined" && !document.hasFocus()) ipc.requestAttention();
        // Mode Hermes : tour soldé par une erreur → apprentissage auto.
        if (e.is_error) this.maybeLearn(s, "Le tour s'est terminé en erreur (is_error).");
        activity.clear(id); // fin du tour → plus rien ne tourne
        this.touch();
        break;
      case "error": {
        s.error = e.message;
        s.streaming = false;
        // Modèle indisponible → on le retire des listes et on bascule sur un modèle dispo.
        const modelIssue = /selected model|may not exist|access to it|model.*not.*available/i.test(e.message);
        if (s.model && modelIssue) {
          settings.markModelUnavailable(s.model);
          const fallback = ["opus", "sonnet", "haiku", "fable"].find(
            (m) => m !== s.model && !settings.unavailableModels.includes(m),
          );
          s.model = fallback ?? null;
        } else {
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
