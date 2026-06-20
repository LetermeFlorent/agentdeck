// Wrappers typés autour des commandes Tauri et du canal d'events par session.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow, UserAttentionType } from "@tauri-apps/api/window";
import { openUrl as openExternalUrl } from "@tauri-apps/plugin-opener";

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
  name?: string;
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
  | { kind: "init"; slash_commands: string[]; tools: string[] }
  | { kind: "assistant_start" }
  | { kind: "assistant_delta"; text: string }
  | { kind: "thinking"; text: string }
  | { kind: "tool_use"; name: string; input: string; id: string }
  | { kind: "tool_done"; id: string }
  | { kind: "task_started"; task_id: string; description: string; subagent_type: string; prompt: string }
  | { kind: "task_progress"; task_id: string; action: string; last_tool: string; tokens: number; duration_ms: number }
  | { kind: "task_ended"; task_id: string; status: string }
  | { kind: "tool_result"; ok: boolean }
  | { kind: "progress"; output_tokens: number }
  | { kind: "turn_done"; input_tokens: number; output_tokens: number; total_tokens: number; cost_usd: number; context_tokens: number; context_window: number; is_error: boolean }
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
  hermes?: boolean,
  perm?: { mode?: string | null; allowed?: string | null; disallowed?: string | null },
) =>
  invoke<void>("session_send", {
    id,
    text,
    model: model ?? null,
    effort: effort ?? null,
    images: images ?? [],
    hermes: hermes ?? false,
    permMode: perm?.mode ?? null,
    allowed: perm?.allowed ?? null,
    disallowed: perm?.disallowed ?? null,
  });

// ---- Dossier de travail (cwd) : home + navigateur de dossiers ----
export interface DirList {
  path: string;
  parent: string | null;
  dirs: { name: string; path: string }[];
}
export const homeDir = () => invoke<string>("home_dir");
export const pickFolder = (start?: string | null) =>
  invoke<string | null>("pick_folder", { start: start ?? null });
export const listDirs = (path?: string | null) => invoke<DirList>("list_dirs", { path: path ?? null });
export const sessionSetCwd = (id: string, cwd: string | null) =>
  invoke<void>("session_set_cwd", { id, cwd });

// ---- Historique des conversations ----
export interface SessionHist {
  id: string;
  title: string;
  cwd: string;
  ts: number;
  snippet?: string;
}
export const recentSessions = (limit: number, offset = 0) =>
  invoke<SessionHist[]>("recent_sessions", { limit, offset });
export const searchSessions = (query: string, limit: number) =>
  invoke<SessionHist[]>("search_sessions", { query, limit });
export const loadMessages = (id: string) =>
  invoke<{ role: "user" | "assistant"; text: string }[]>("load_messages", { id });

// ---- Mode Auto : niveaux d'effort dynamiques + choix modèle/effort par Haiku ----
export const effortLevels = () => invoke<string[]>("effort_levels");
export const autoPick = (prompt: string, models: string[], efforts: string[], picker?: string) =>
  invoke<{ model: string; effort: string }>("auto_pick", { prompt, models, efforts, picker: picker ?? null });

// ---- Mode Hermes : réflexion auto sur échec → écrit un skill ----
export const reflectAndLearn = (
  cwd: string | null,
  request: string,
  summary: string,
  error: string,
) => invoke<string>("reflect_and_learn", { cwd: cwd ?? null, request, summary, error });
export const sessionStop = (id: string) => invoke<void>("session_stop", { id });
export const sessionClose = (id: string) => invoke<void>("session_close", { id });

// ---- Usage ----
export const usageGet = () => invoke<UsageSnapshot>("usage_get");

// ---- Modèles disponibles (API Models, via token coffre) ----
export const claudeModels = () => invoke<{ v: string; l: string }[]>("claude_models");

// ---- Défauts modèle/effort (Claude Code courant) ----
export const claudeDefaults = () =>
  invoke<{ model: string; effort: string }>("claude_defaults");

// ---- Plan d'abonnement ----
export const subscriptionPlan = () =>
  invoke<{ label: string; level: number; account: string }>("subscription_plan");

// ---- Connectivité réseau (gate au lancement) ----
export const netCheck = () => invoke<boolean>("net_check");

// ---- Dépendance Claude Code ----
export const checkClaude = () => invoke<boolean>("check_claude");
export const installClaude = () => invoke<void>("install_claude");

// ---- Ouvrir une URL dans le navigateur par défaut ----
export const openUrl = (url: string) => openExternalUrl(url);

// ---- Commandes slash (liste dynamique) ----
export interface SlashCmd {
  name: string;
  description: string;
  /** Indice d'arguments (ex. "[message]") — vide si aucun. */
  args: string;
}
export const slashCommandsFetch = () => invoke<SlashCmd[]>("slash_commands");

// ---- Bibliothèque : skills + MCP ----
export interface SkillItem {
  name: string;
  description: string;
}
export interface McpItem {
  name: string;
  target: string;
  status: string;
  scope: string;
  removable: boolean;
}
export const skillsInstalled = () => invoke<SkillItem[]>("skills_installed");
export const skillWrite = (name: string, content: string) =>
  invoke<void>("skill_write", { name, content });
export const skillDelete = (name: string) => invoke<void>("skill_delete", { name });
export const mcpInstalled = () => invoke<McpItem[]>("mcp_installed");
export const mcpAdd = (name: string, target: string, transport?: string) =>
  invoke<void>("mcp_add", { name, target, transport: transport ?? null });
export const mcpAddJson = (name: string, json: string) =>
  invoke<void>("mcp_add_json", { name, json });
export const mcpRemove = (name: string) => invoke<void>("mcp_remove", { name });

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
