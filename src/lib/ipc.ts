// Wrappers typés autour des commandes Tauri et du canal d'events par session.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface SessionInfo {
  id: string;
  title: string;
  started: boolean;
  cwd: string | null;
  model: string | null;
}

export interface Bar {
  tokens: number;
  cap: number;
  pct: number;
  resets_at: number | null;
}

export interface UsageSnapshot {
  five_h: Bar;
  week: Bar;
  five_h_cost: number;
  week_cost: number;
  source: string;
}

// Events émis par le backend sur session://{id} (tag "kind", snake_case).
export type SessionEvent =
  | { kind: "started" }
  | { kind: "assistant_delta"; text: string }
  | { kind: "tool_use"; name: string }
  | { kind: "tool_result"; ok: boolean }
  | { kind: "turn_done"; input_tokens: number; output_tokens: number }
  | { kind: "error"; message: string }
  | { kind: "exited"; code: number | null };

// ---- Auth ----
export const authStatus = () => invoke<boolean>("auth_status");
export const authSetToken = (token: string) => invoke<void>("auth_set_token", { token });
export const authClear = () => invoke<void>("auth_clear");
export const authImportFromFile = (path?: string) =>
  invoke<void>("auth_import_from_file", { path: path ?? null });
export const authLogin = () => invoke<void>("auth_login");

// ---- Sessions ----
export const sessionCreate = (opts: { title?: string; cwd?: string; model?: string } = {}) =>
  invoke<string>("session_create", {
    title: opts.title ?? null,
    cwd: opts.cwd ?? null,
    model: opts.model ?? null,
  });
export const sessionList = () => invoke<SessionInfo[]>("session_list");
export const sessionRestore = (s: {
  id: string;
  title?: string;
  started: boolean;
  cwd?: string;
  model?: string;
}) =>
  invoke<void>("session_restore", {
    id: s.id,
    title: s.title ?? null,
    started: s.started,
    cwd: s.cwd ?? null,
    model: s.model ?? null,
  });
export const sessionSend = (id: string, text: string) =>
  invoke<void>("session_send", { id, text });
export const sessionStop = (id: string) => invoke<void>("session_stop", { id });
export const sessionClose = (id: string) => invoke<void>("session_close", { id });

// ---- Usage ----
export const usageGet = () => invoke<UsageSnapshot>("usage_get");

// ---- Events ----
export const onSessionEvent = (id: string, cb: (e: SessionEvent) => void): Promise<UnlistenFn> =>
  listen<SessionEvent>(`session://${id}`, (evt) => cb(evt.payload));
