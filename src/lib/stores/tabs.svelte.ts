// Onglets (workspaces) : chaque onglet a sa propre disposition de chats.
// L'onglet actif est piloté par `layout.root` ; changer d'onglet sauve la dispo courante
// dans l'onglet actif puis charge celle du nouvel onglet.

import { layout, type Node } from "./layout.svelte";
import { sessions } from "./sessions.svelte";

export interface Tab {
  id: string;
  name: string;
  root: Node | null;
}

function newId(): string {
  try {
    return crypto.randomUUID();
  } catch {
    return `t${Date.now()}${Math.floor(Math.random() * 1000)}`;
  }
}

/** Toutes les sessions (sids) référencées par une disposition. */
function collectSids(node: Node | null): string[] {
  if (!node) return [];
  if (node.kind === "leaf") return [node.sid];
  return [...collectSids(node.a), ...collectSids(node.b)];
}

class TabsStore {
  list = $state<Tab[]>([]);
  activeId = $state<string>("");
  /** Incrémenté à chaque changement à persister. */
  rev = $state(0);
  private seq = 1;

  private touch() {
    this.rev++;
  }

  get active(): Tab | undefined {
    return this.list.find((t) => t.id === this.activeId);
  }

  /** Vrai si au moins un Claude est en train de répondre dans cet onglet. */
  isTabBusy(id: string): boolean {
    const t = this.list.find((t) => t.id === id);
    if (!t) return false;
    const root = id === this.activeId ? layout.root : t.root;
    return collectSids(root).some((sid) => sessions.map[sid]?.streaming);
  }

  /** Sauve la disposition courante (layout.root) dans l'onglet actif. */
  commitActive() {
    const t = this.active;
    if (t) t.root = layout.root;
  }

  /** Démarrage sans onglets persistés : crée un 1er onglet depuis la dispo courante. */
  initFromLayout() {
    const id = newId();
    this.list = [{ id, name: "Onglet 1", root: layout.root }];
    this.activeId = id;
    this.seq = 1;
    this.touch();
  }

  /** Restaure les onglets persistés + charge l'onglet actif dans le layout. */
  hydrate(list: Tab[], activeId: string) {
    this.list = list;
    this.activeId = list.some((t) => t.id === activeId) ? activeId : (list[0]?.id ?? "");
    this.seq = list.length;
    layout.restore(this.active?.root ?? null);
    this.touch();
  }

  select(id: string) {
    if (id === this.activeId) return;
    this.commitActive();
    this.activeId = id;
    layout.restore(this.active?.root ?? null);
    this.touch();
  }

  async create() {
    this.commitActive();
    const id = newId();
    this.seq += 1;
    this.list = [...this.list, { id, name: `Onglet ${this.seq}`, root: null }];
    this.activeId = id;
    layout.restore(null);
    await layout.init(); // un pane par défaut dans le nouvel onglet
    this.touch();
  }

  /** Ouvre une session existante (historique) dans un nouvel onglet. */
  openSession(sid: string, name: string) {
    this.commitActive();
    const id = newId();
    this.seq += 1;
    this.list = [...this.list, { id, name: name || "Conversation", root: null }];
    this.activeId = id;
    layout.openSingle(sid);
    this.touch();
  }

  rename(id: string, name: string) {
    const n = name.trim();
    const t = this.list.find((t) => t.id === id);
    if (t && n) {
      t.name = n;
      this.touch();
    }
  }

  async close(id: string) {
    const idx = this.list.findIndex((t) => t.id === id);
    if (idx < 0) return;
    // Ferme les sessions de cet onglet (dispo courante si actif, sinon la dispo stockée).
    const root = id === this.activeId ? layout.root : this.list[idx].root;
    for (const sid of collectSids(root)) await sessions.close(sid);
    this.list = this.list.filter((t) => t.id !== id);
    if (this.activeId === id) {
      const next = this.list[idx] ?? this.list[idx - 1];
      if (next) {
        this.activeId = next.id;
        layout.restore(next.root);
      } else {
        // Plus aucun onglet → on en recrée un vide.
        this.activeId = "";
        await this.create();
        return;
      }
    }
    this.touch();
  }

  /** Réordonne : déplace l'onglet `fromId` à la position de `toId`. */
  move(fromId: string, toId: string) {
    if (fromId === toId) return;
    const from = this.list.findIndex((t) => t.id === fromId);
    const to = this.list.findIndex((t) => t.id === toId);
    if (from < 0 || to < 0) return;
    const next = [...this.list];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    this.list = next;
    this.touch();
  }

  serialize(): Tab[] {
    this.commitActive();
    return this.list.map((t) => ({ id: t.id, name: t.name, root: t.root }));
  }

  reset() {
    this.list = [];
    this.activeId = "";
    this.seq = 1;
  }
}

export const tabs = new TabsStore();
