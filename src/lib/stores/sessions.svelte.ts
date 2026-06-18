// Store des sessions : état de chat par session + abonnement aux events backend.

import * as ipc from "$lib/ipc";
import type { SessionEvent } from "$lib/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { usage } from "./usage.svelte";
import { settings } from "./settings.svelte";

export interface Msg {
  role: "user" | "assistant";
  text: string;
  tools: string[];
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
}

export interface PersistedSession {
  id: string;
  title: string;
  model: string | null;
  effort: string | null;
  messages: Msg[];
  collapsed?: boolean;
  priv?: boolean;
}

class SessionsStore {
  map = $state<Record<string, SessionState>>({});
  /** Incrémenté à chaque changement à persister (sauvegarde déclenchée côté +page). */
  persistRev = $state(0);
  /** Modèle / effort par défaut (Claude Code courant), pour pré-remplir les nouveaux panes. */
  defaultModel = $state<string | null>(null);
  defaultEffort = $state<string | null>(null);
  /** Commandes slash exposées par Claude Code (récupérées dynamiquement à l'init). */
  slashCommands = $state<string[]>([]);
  private unlisteners: Record<string, UnlistenFn> = {};

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
    if (this.slashCommands.length > 0) return;
    try {
      const cached = JSON.parse(localStorage.getItem("agentdeck.slash.v1") || "[]");
      if (Array.isArray(cached) && cached.length) this.slashCommands = cached;
    } catch {
      /* ignore */
    }
    try {
      const list = await ipc.slashCommandsFetch();
      if (list.length) {
        this.slashCommands = list;
        localStorage.setItem("agentdeck.slash.v1", JSON.stringify(list));
      }
    } catch {
      /* ignore */
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
        totalTokens: 0,
        costUsd: 0,
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
      messages: s.messages,
      collapsed: s.collapsed,
      priv: s.priv,
    }));
  }

  /** Replie / déplie un pane (minimise sur le côté). */
  setCollapsed(id: string, collapsed: boolean) {
    const s = this.map[id];
    if (!s) return;
    s.collapsed = collapsed;
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

  async send(id: string, text: string) {
    const s = this.map[id];
    if (!s) return;
    s.error = null;
    s.messages.push({ role: "user", text, tools: [] });
    // Le process est persistant : on écrit toujours, même si Claude travaille (pris en cours de route).
    s.streaming = true;
    if (s.turnStart === null) s.turnStart = Date.now();
    s.turnTokens = 0;
    this.touch();
    try {
      await ipc.sessionSend(id, text, s.model, s.effort);
    } catch (err) {
      s.error = String(err);
      s.streaming = false;
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
    const m: Msg = { role: "assistant", text: "", tools: [] };
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
          this.slashCommands = e.slash_commands;
          try {
            localStorage.setItem("agentdeck.slash.v1", JSON.stringify(e.slash_commands));
          } catch {
            /* ignore */
          }
        }
        break;
      case "assistant_start": {
        // Nouveau tour → nouvelle bulle assistant (sauf si une bulle vide attend déjà).
        s.streaming = true;
        s.error = null;
        if (s.turnStart === null) s.turnStart = Date.now();
        const last = s.messages[s.messages.length - 1];
        if (!(last && last.role === "assistant" && !last.text && last.tools.length === 0)) {
          s.messages.push({ role: "assistant", text: "", tools: [] });
        }
        break;
      }
      case "assistant_delta":
        this.lastAssistant(s).text += e.text;
        break;
      case "tool_use":
        this.lastAssistant(s).tools.push(e.name);
        break;
      case "progress":
        s.turnTokens = e.output_tokens;
        break;
      case "turn_done":
        s.streaming = false;
        s.turnStart = null;
        s.totalTokens += e.total_tokens;
        // total_cost_usd est cumulatif par process → on prend la dernière valeur.
        if (e.cost_usd >= s.costUsd || e.cost_usd === 0) s.costUsd = e.cost_usd || s.costUsd;
        else s.costUsd = e.cost_usd; // process relancé (modèle changé) → réinitialisé
        usage.refresh();
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
