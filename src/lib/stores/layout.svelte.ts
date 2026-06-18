// Store de disposition : arbre de tiling récursif. Chaque feuille = un pane lié à une session.
// Dir = valeur CSS flex-direction directe : "row" = côte à côte, "column" = empilé (haut/bas).

import { sessions } from "./sessions.svelte";

export type Dir = "row" | "column";

export interface LeafNode {
  kind: "leaf";
  nodeId: string;
  sid: string;
}
export interface SplitNode {
  kind: "split";
  nodeId: string;
  dir: Dir;
  a: Node;
  b: Node;
}
export type Node = LeafNode | SplitNode;

let counter = 0;
const nid = () => `n${counter++}`;

/** Remet le compteur au-delà du plus grand nodeId d'un arbre restauré. */
function syncCounter(node: Node) {
  const n = Number(node.nodeId.replace(/^n/, ""));
  if (!Number.isNaN(n) && n >= counter) counter = n + 1;
  if (node.kind === "split") {
    syncCounter(node.a);
    syncCounter(node.b);
  }
}

/** Migre les anciens arbres persistés (dir "col" → "column" CSS valide). */
function migrateDirs(node: Node) {
  if (node.kind === "split") {
    if ((node.dir as string) === "col") node.dir = "column";
    migrateDirs(node.a);
    migrateDirs(node.b);
  }
}

function mapTree(node: Node, fn: (n: Node) => Node): Node {
  const replaced = fn(node);
  if (replaced !== node) return replaced;
  if (node.kind === "split") {
    return { ...node, a: mapTree(node.a, fn), b: mapTree(node.b, fn) };
  }
  return node;
}

function removeLeaf(node: Node, targetNodeId: string): Node | null {
  if (node.kind === "leaf") {
    return node.nodeId === targetNodeId ? null : node;
  }
  const a = removeLeaf(node.a, targetNodeId);
  const b = removeLeaf(node.b, targetNodeId);
  if (a === null) return b; // collapse vers le frère
  if (b === null) return a;
  if (a === node.a && b === node.b) return node;
  return { ...node, a, b };
}

class LayoutStore {
  root = $state<Node | null>(null);

  async init() {
    if (this.root) return;
    const sid = await sessions.create({ title: "Claude" });
    this.root = { kind: "leaf", nodeId: nid(), sid };
  }

  /** Restaure un arbre persité (les sessions correspondantes doivent déjà être hydratées). */
  restore(root: Node | null) {
    if (root) {
      migrateDirs(root);
      syncCounter(root);
    }
    this.root = root;
  }

  /** Scinde une feuille : crée une nouvelle session dans le nouveau pane. */
  async split(targetNodeId: string, dir: Dir) {
    const newSid = await sessions.create({ title: "Claude" });
    if (!this.root) {
      this.root = { kind: "leaf", nodeId: nid(), sid: newSid };
      return;
    }
    this.root = mapTree(this.root, (n) => {
      if (n.kind === "leaf" && n.nodeId === targetNodeId) {
        return {
          kind: "split",
          nodeId: nid(),
          dir,
          a: n,
          b: { kind: "leaf", nodeId: nid(), sid: newSid },
        };
      }
      return n;
    });
  }

  /** Ferme un pane : stoppe la session et replie l'arbre. */
  async close(targetNodeId: string, sid: string) {
    if (this.root) this.root = removeLeaf(this.root, targetNodeId);
    await sessions.close(sid);
  }

  /** Échange la position de deux panes (drag & drop pour réarranger les chats). */
  swap(nodeIdA: string, nodeIdB: string) {
    if (!this.root || nodeIdA === nodeIdB) return;
    let sidA: string | null = null;
    let sidB: string | null = null;
    const find = (n: Node) => {
      if (n.kind === "leaf") {
        if (n.nodeId === nodeIdA) sidA = n.sid;
        if (n.nodeId === nodeIdB) sidB = n.sid;
      } else {
        find(n.a);
        find(n.b);
      }
    };
    find(this.root);
    if (sidA === null || sidB === null) return;
    this.root = mapTree(this.root, (n) => {
      if (n.kind === "leaf" && n.nodeId === nodeIdA) return { ...n, sid: sidB! };
      if (n.kind === "leaf" && n.nodeId === nodeIdB) return { ...n, sid: sidA! };
      return n;
    });
  }

  /** Ajoute un pane à la racine (split horizontal). */
  async addRoot() {
    if (!this.root) {
      await this.init();
      return;
    }
    // Split la première feuille trouvée pour rester simple.
    const firstLeaf = (n: Node): LeafNode => (n.kind === "leaf" ? n : firstLeaf(n.a));
    await this.split(firstLeaf(this.root).nodeId, "row");
  }
}

export const layout = new LayoutStore();
