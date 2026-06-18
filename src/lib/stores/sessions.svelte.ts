// Store des sessions : état de chat par session + abonnement aux events backend.

import * as ipc from "$lib/ipc";
import type { SessionEvent } from "$lib/ipc";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { usage } from "./usage.svelte";

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
}

export interface PersistedSession {
  id: string;
  title: string;
  model: string | null;
  effort: string | null;
  messages: Msg[];
}

class SessionsStore {
  map = $state<Record<string, SessionState>>({});
  /** Incrémenté à chaque changement à persister (sauvegarde déclenchée côté +page). */
  persistRev = $state(0);
  private unlisteners: Record<string, UnlistenFn> = {};

  private touch() {
    this.persistRev++;
  }

  async create(opts: { title?: string; cwd?: string; model?: string } = {}): Promise<string> {
    const id = await ipc.sessionCreate(opts);
    this.map[id] = {
      id,
      title: opts.title ?? "Claude",
      model: opts.model ?? null,
      effort: null,
      messages: [],
      streaming: false,
      error: null,
    };
    this.unlisteners[id] = await ipc.onSessionEvent(id, (e) => this.handle(id, e));
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
        model: p.model,
        effort: p.effort ?? null,
        messages: p.messages,
        streaming: false,
        error: null,
      };
      this.unlisteners[p.id] = await ipc.onSessionEvent(p.id, (e) => this.handle(p.id, e));
    }
  }

  serialize(): PersistedSession[] {
    return Object.values(this.map).map((s) => ({
      id: s.id,
      title: s.title,
      model: s.model,
      effort: s.effort,
      messages: s.messages,
    }));
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
    if (!s || s.streaming) return;
    s.error = null;
    s.messages.push({ role: "user", text, tools: [] });
    this.touch();
    try {
      await ipc.sessionSend(id, text, s.model, s.effort);
    } catch (err) {
      s.error = String(err);
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
        s.streaming = true;
        s.error = null;
        s.messages.push({ role: "assistant", text: "", tools: [] });
        break;
      case "assistant_delta":
        this.lastAssistant(s).text += e.text;
        break;
      case "tool_use":
        this.lastAssistant(s).tools.push(e.name);
        break;
      case "tool_result":
        break;
      case "turn_done":
        s.streaming = false;
        usage.refresh();
        this.touch();
        break;
      case "error":
        s.error = e.message;
        s.streaming = false;
        this.touch();
        break;
      case "exited":
        s.streaming = false;
        break;
    }
  }
}

export const sessions = new SessionsStore();
