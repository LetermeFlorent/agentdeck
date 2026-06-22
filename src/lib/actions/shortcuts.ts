// Raccourcis clavier globaux du deck. `installShortcuts()` pose l'écouteur et renvoie
// la fonction de nettoyage. Onglets : Ctrl+T/W/1-9. Chat focus : Ctrl+Tab/M/E. F11 : plein écran.

import { tabs } from "$lib/stores/tabs.svelte";
import { sessions } from "$lib/stores/sessions.svelte";
import { layout, type Node } from "$lib/stores/layout.svelte";
import * as ipc from "$lib/ipc";

/** Premier sid (leaf le plus à gauche) d'un arbre de panes. */
function firstSid(n: Node | null): string {
  if (!n) return "";
  return n.kind === "leaf" ? n.sid : firstSid(n.a);
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "F11") {
    e.preventDefault();
    ipc.winToggleFullscreen();
  }
  // Raccourcis onglets (ne nécessitent pas de chat focus).
  if (e.ctrlKey && !e.altKey) {
    if (e.key.toLowerCase() === "t") {
      e.preventDefault();
      tabs.create();
      return;
    }
    if (e.key.toLowerCase() === "w") {
      e.preventDefault();
      if (tabs.activeId) tabs.close(tabs.activeId);
      return;
    }
    if (/^[1-9]$/.test(e.key)) {
      const t = tabs.list[Number(e.key) - 1];
      if (t) {
        e.preventDefault();
        tabs.select(t.id);
      }
      return;
    }
  }
  // Raccourcis de cycle sur le chat focus. Fallback : 1er chat du layout courant.
  let sid = sessions.focusedSid;
  if (!sessions.map[sid]) sid = firstSid(layout.root);
  if (!e.ctrlKey || e.altKey || !sessions.map[sid]) return;
  const k = e.key.toLowerCase();
  if (e.key === "Tab") {
    e.preventDefault();
    sessions.cyclePermMode(sid); // Ctrl+Tab → mode de permission
  } else if (k === "m") {
    e.preventDefault();
    sessions.cycleModel(sid); // Ctrl+M → modèle
  } else if (k === "e") {
    e.preventDefault();
    sessions.cycleEffort(sid); // Ctrl+E → effort
  }
}

export function installShortcuts(): () => void {
  window.addEventListener("keydown", onKeydown, true);
  return () => window.removeEventListener("keydown", onKeydown, true);
}
