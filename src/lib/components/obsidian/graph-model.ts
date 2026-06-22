// Construit le graphe « Obsidian ».
// Modèle hiérarchique :
//   Claude (racine) ── skills globaux / serveurs MCP / plugins (hubs partagés)
//   Claude (racine) ── chaque dossier/tchat (nommé d'après le chat)
//   chaque dossier/tchat ── ses skills projet (<cwd>/.claude/skills, sauvés par Hermes)

import type { SkillItem, McpItem, PluginItem } from "$lib/ipc";
import type { Graph, GNode, GNodeKind } from "./graph-types";

export interface OpenChat {
  sid: string;
  title: string;
  provider: string;
  cwd: string;
}

export const ROOT_ID = "claude:root";

/** Couleur d'un nœud chat selon son provider (variables CSS du thème). */
export function providerColor(provider: string | undefined): string {
  switch (provider) {
    case "opencode":
      return "var(--prov-opencode, #6ea8fe)";
    case "gemini":
      return "var(--prov-gemini, #d98a5b)";
    default:
      return "var(--accent)";
  }
}

const RADIUS: Record<GNodeKind, number> = {
  claude: 20,
  chat: 13,
  skill: 9,
  mcp: 9,
  plugin: 10,
};

function node(kind: GNodeKind, id: string, label: string, extra: Partial<GNode>): GNode {
  return {
    id,
    label,
    kind,
    x: 0,
    y: 0,
    vx: 0,
    vy: 0,
    fx: null,
    fy: null,
    r: RADIUS[kind],
    ...extra,
  };
}

export function buildGraph(
  chats: OpenChat[],
  globalSkills: SkillItem[],
  projectSkills: Map<string, SkillItem[]>,
  mcps: McpItem[] = [],
  plugins: PluginItem[] = [],
): Graph {
  const nodes: GNode[] = [];
  const links: Graph["links"] = [];

  // Racine Claude.
  nodes.push(node("claude", ROOT_ID, "Claude", {}));

  // Hubs directement reliés à Claude : skills globaux, MCP, plugins.
  for (const s of globalSkills) {
    const id = `skill:global:${s.name}`;
    nodes.push(node("skill", id, s.name, { scope: "global", skill: s.name, removable: s.removable, source: s.source }));
    links.push({ source: ROOT_ID, target: id, kind: "global" });
  }
  for (const m of mcps) {
    const id = `mcp:${m.name}`;
    nodes.push(node("mcp", id, m.name, { mcp: m.name }));
    links.push({ source: ROOT_ID, target: id, kind: "mcp" });
  }
  for (const p of plugins) {
    const id = `plugin:${p.id}`;
    nodes.push(node("plugin", id, p.name, { plugin: p.id }));
    links.push({ source: ROOT_ID, target: id, kind: "plugin" });
  }

  // Dossiers / tchats : reliés à Claude, puis à leurs skills projet.
  const seen = new Set<string>();
  for (const c of chats) {
    const cid = `chat:${c.sid}`;
    nodes.push(node("chat", cid, c.title || "Chat", { provider: c.provider, cwd: c.cwd, sid: c.sid }));
    links.push({ source: ROOT_ID, target: cid, kind: "chat" });

    const skills = c.cwd ? projectSkills.get(c.cwd) ?? [] : [];
    for (const s of skills) {
      const id = `skill:proj:${c.cwd}:${s.name}`;
      if (!seen.has(id)) {
        seen.add(id);
        nodes.push(node("skill", id, s.name, { scope: "project", skill: s.name, cwd: c.cwd, removable: s.removable, source: s.source }));
      }
      links.push({ source: cid, target: id, kind: "project" });
    }
  }

  return { nodes, links };
}

/** Voisins immédiats d'un nœud (ids), pour la mise en évidence au survol/sélection. */
export function neighborsOf(graph: Graph, id: string): Set<string> {
  const out = new Set<string>();
  for (const l of graph.links) {
    if (l.source === id) out.add(l.target);
    else if (l.target === id) out.add(l.source);
  }
  return out;
}
