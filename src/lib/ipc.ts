// Wrappers typés autour des commandes Tauri et du canal d'events par session.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow, UserAttentionType } from "@tauri-apps/api/window";

export interface SessionInfo {
  id: string;
  title: string;
  started: boolean;
  cwd: string | null;
  model: string | null;
}

/** Image jointe à un message : base64 brut (sans préfixe data:) + type MIME. */
export interface ImageInput {
  media_type: string;
  data: string;
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
  | { kind: "init"; slash_commands: string[] }
  | { kind: "assistant_start" }
  | { kind: "assistant_delta"; text: string }
  | { kind: "thinking"; text: string }
  | { kind: "tool_use"; name: string; input: string }
  | { kind: "tool_result"; ok: boolean }
  | { kind: "progress"; output_tokens: number }
  | { kind: "turn_done"; input_tokens: number; output_tokens: number; total_tokens: number; cost_usd: number; context_tokens: number; context_window: number }
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
export const sessionSend = (
  id: string,
  text: string,
  model?: string | null,
  effort?: string | null,
  images?: ImageInput[],
) =>
  invoke<void>("session_send", {
    id,
    text,
    model: model ?? null,
    effort: effort ?? null,
    images: images ?? [],
  });
export const sessionStop = (id: string) => invoke<void>("session_stop", { id });
export const sessionClose = (id: string) => invoke<void>("session_close", { id });

// ---- Usage ----
export const usageGet = () => invoke<UsageSnapshot>("usage_get");

// ---- Défauts modèle/effort (Claude Code courant) ----
export const claudeDefaults = () =>
  invoke<{ model: string; effort: string }>("claude_defaults");

// ---- Plan d'abonnement ----
export const subscriptionPlan = () =>
  invoke<{ label: string; level: number; account: string }>("subscription_plan");

// ---- Dépendance Claude Code ----
export const checkClaude = () => invoke<boolean>("check_claude");
export const installClaude = () => invoke<void>("install_claude");

// ---- Commandes slash (liste dynamique) ----
export interface SlashCmd {
  name: string;
  description: string;
  /** Indice d'arguments (ex. "[message]") — vide si aucun. */
  args: string;
}
export const slashCommandsFetch = () => invoke<SlashCmd[]>("slash_commands");

// ---- Nom d'utilisateur du PC (accueil démarrage) ----
export const osUsername = () => invoke<string>("os_username");

// ---- Fenêtre : faire clignoter l'icône dans la barre des tâches (attention) ----
export const requestAttention = async () => {
  try {
    await getCurrentWindow().requestUserAttention(UserAttentionType.Critical);
  } catch {
    /* ignore (permission / plateforme) */
  }
};
export const clearAttention = async () => {
  try {
    await getCurrentWindow().requestUserAttention(null);
  } catch {
    /* ignore */
  }
};

// ---- Contrôles de fenêtre (titlebar custom) ----
export const winMinimize = () => getCurrentWindow().minimize();
export const winToggleMaximize = () => getCurrentWindow().toggleMaximize();
export const winClose = () => getCurrentWindow().close();
export const winToggleFullscreen = async () => {
  const w = getCurrentWindow();
  try {
    await w.setFullscreen(!(await w.isFullscreen()));
  } catch {
    /* ignore */
  }
};

// ---- Events ----
export const onSessionEvent = (id: string, cb: (e: SessionEvent) => void): Promise<UnlistenFn> =>
  listen<SessionEvent>(`session://${id}`, (evt) => cb(evt.payload));
