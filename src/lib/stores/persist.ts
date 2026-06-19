// Persistance locale du deck (onglets + dispositions + sessions) pour retrouver son espace
// au redémarrage. Le webview Tauri conserve localStorage ; les sessions Claude sont reprises
// côté CLI via --resume (l'UUID est conservé).

import { sessions, type PersistedSession } from "./sessions.svelte";
import { tabs, type Tab } from "./tabs.svelte";
import { type Node } from "./layout.svelte";
import { debounce } from "../util/debounce";
import { STORAGE_KEYS } from "./keys";

const KEY = STORAGE_KEYS.deck;

interface DeckState {
  tabs: Tab[];
  activeId: string;
  sessions: PersistedSession[];
}

function saveNow() {
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

// Sauvegarde debouncée : appelée à chaque frappe/changement réactif, mais n'écrit dans
// localStorage qu'après une courte inactivité (au lieu d'une sérialisation complète par frappe).
export const save = debounce(saveNow, 400);
/** Force l'écriture immédiate de l'état en attente (avant fermeture/déconnexion). */
export const flush = save.flush;

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
  save.cancel(); // évite qu'une sauvegarde en attente ne réécrive l'état juste après le clear
  try {
    localStorage.removeItem(KEY);
  } catch {
    /* ignore */
  }
}
