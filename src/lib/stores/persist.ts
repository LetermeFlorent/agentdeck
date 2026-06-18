// Persistance locale du deck (sessions + disposition) pour retrouver ses Claudes au redémarrage.
// Le webview Tauri conserve localStorage entre les lancements ; les sessions Claude elles-mêmes
// sont reprises côté CLI via --resume (l'UUID est conservé).

import { sessions, type PersistedSession } from "./sessions.svelte";
import { layout, type Node } from "./layout.svelte";

const KEY = "agentdeck.deck.v1";

interface DeckState {
  root: Node | null;
  sessions: PersistedSession[];
}

export function save() {
  const state: DeckState = { root: layout.root, sessions: sessions.serialize() };
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
    return JSON.parse(raw) as DeckState;
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
