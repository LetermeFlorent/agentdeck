// Store des sessions : état de chat par session + abonnement aux events backend.

import * as ipc from "$lib/ipc";
import type { SessionEvent } from "$lib/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { usage } from "./usage.svelte";
import { settings } from "./settings.svelte";

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
}

class SessionsStore {
  map = $state<Record<string, SessionState>>({});
  /** Incrémenté à chaque changement à persister (sauvegarde déclenchée côté +page). */
  persistRev = $state(0);
  /** Modèle / effort par défaut (Claude Code courant), pour pré-remplir les nouveaux panes. */
  defaultModel = $state<string | null>(null);
  defaultEffort = $state<string | null>(null);
  /** Commandes slash exposées par Claude Code (nom + description, récupérées dynamiquement). */
  slashCommands = $state<ipc.SlashCmd[]>([]);
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
        const cached = JSON.parse(localStorage.getItem("agentdeck.slash.v2") || "[]");
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
        localStorage.setItem("agentdeck.slash.v2", JSON.stringify(list));
      }
    } catch {
      this.slashFetched = false; // échec → autorise une nouvelle tentative
    }
  }

  /** Défaut effectif = override utilisateur (réglages) sinon modèle/effort Claude Code courant. */
  get effModel(): string | null {
    return settings.defaultModel ?? this.defaultModel;
  }
  get effEffort(): string | null {
    return settings.defaultEffort ?? this.defaultEffort;
  }

  async create(opts: { title?: string; cwd?: string; model?: string } = {}): Promise<string> {
    const id = await ipc.sessionCreate(opts);
    this.map[id] = {
      id,
      title: opts.title ?? "Claude",
      model: opts.model ?? this.effModel,
      effort: this.effEffort,
      messages: [],
      streaming: false,
      error: null,
      collapsed: false,
      priv: false,
      queue: [],
      turnStart: null,
      turnTokens: 0,
      totalTokens: 0,
      costUsd: 0,
      contextTokens: 0,
      contextWindow: 0,
      lastActivity: Date.now(),
      zoom: settings.defaultZoom ?? 1,
    };
    await this.attach(id);
    this.touch();
    return id;
  }

  /** Restaure des sessions persistées au démarrage (réenregistre côté backend pour --resume). */
  async hydrate(list: PersistedSession[]) {
    for (const p of list) {
      const started = p.messages.some((m) => m.role === "user");
      await ipc.sessionRestore({
        id: p.id,
        title: p.title,
        started,
        model: p.model ?? undefined,
      });
      this.map[p.id] = {
        id: p.id,
        title: p.title,
        model: p.model ?? this.effModel,
        effort: p.effort ?? this.effEffort,
        messages: p.messages,
        streaming: false,
        error: null,
        collapsed: p.collapsed ?? false,
        priv: p.priv ?? false,
        queue: [],
        turnStart: null,
        turnTokens: 0,
        totalTokens: p.totalTokens ?? 0,
        costUsd: p.costUsd ?? 0,
        contextTokens: p.contextTokens ?? 0,
        contextWindow: p.contextWindow ?? 0,
        lastActivity: Date.now(),
        zoom: p.zoom ?? 1,
      };
      await this.attach(p.id);
    }
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

  async send(
    id: string,
    text: string,
    images: { dataUrl: string; media_type: string; data: string }[] = [],
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
      await ipc.sessionSend(
        id,
        text,
        s.model,
        s.effort,
        images.map((i) => ({ media_type: i.media_type, data: i.data })),
      );
    } catch (err) {
      s.error = String(err);
      s.streaming = false;
    }
  }

  /** Marque une activité sur le chat (saisie, focus…) → repousse la veille privée. */
  touchActivity(id: string) {
    const s = this.map[id];
    if (s) s.lastActivity = Date.now();
  }

  /** Surveille l'inactivité : passe un chat en mode privé après le délai réglé (Paramètres). */
  startPrivacyWatch() {
    if (this.privacyTimer !== null) return;
    this.privacyTimer = window.setInterval(() => this.checkPrivacy(), 15_000);
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
              localStorage.setItem("agentdeck.slash.v2", JSON.stringify(merged));
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
        this.touch();
        break;
      case "error":
        s.error = e.message;
        s.streaming = false;
        // Modèle indisponible → on le retire des listes et on bascule sur un modèle dispo.
        if (s.model && /selected model|may not exist|access to it|model.*not.*available/i.test(e.message)) {
          settings.markModelUnavailable(s.model);
          const fallback = ["opus", "sonnet", "haiku", "fable"].find(
            (m) => m !== s.model && !settings.unavailableModels.includes(m),
          );
          s.model = fallback ?? null;
        }
        this.touch();
        break;
      case "exited":
        s.streaming = false;
        s.turnStart = null;
        break;
    }
  }
}

export const sessions = new SessionsStore();
