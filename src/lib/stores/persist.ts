// Persistance locale du deck (onglets + dispositions + sessions) pour retrouver son espace
// au redémarrage. Le webview Tauri conserve localStorage ; les sessions Claude sont reprises
// côté CLI via --resume (l'UUID est conservé).

import { sessions, type PersistedSession } from "./sessions.svelte";
import { tabs, type Tab } from "./tabs.svelte";
import { type Node } from "./layout.svelte";

const KEY = "agentdeck.deck.v1";

interface DeckState {
  tabs: Tab[];
  activeId: string;
  sessions: PersistedSession[];
}

export function save() {
  const state: DeckState = {
    tabs: tabs.serialize(),
    activeId: tabs.activeId,
    sessions: sessions.serialize(),
  };
  try {
    localStorage.setItem(KEY, JSON.stringify(state));
  } catch {
    /* quota / indispo : on ignore */
  }
}

export function load(): DeckState | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<DeckState> & { root?: Node | null };
    // Migration : ancien format { root, sessions } → un seul onglet.
    if (!parsed.tabs && "root" in parsed) {
      return {
        tabs: [{ id: "t0", name: "Onglet 1", root: parsed.root ?? null }],
        activeId: "t0",
        sessions: parsed.sessions ?? [],
      };
    }
    return {
      tabs: parsed.tabs ?? [],
      activeId: parsed.activeId ?? "",
      sessions: parsed.sessions ?? [],
    };
  } catch {
    return null;
  }
}

export function clear() {
  try {
    localStorage.removeItem(KEY);
  } catch {
    /* ignore */
  }
}
