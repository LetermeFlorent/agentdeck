// Types du graphe « Obsidian » : nœuds (chats + skills) et liens. La position (x/y/v)
// est mutée en place par la simulation de force — gardée hors de l'état réactif Svelte
// pour éviter une tempête de re-rendu (un seul compteur de frame déclenche le redraw).

export type GNodeKind = "claude" | "chat" | "skill" | "mcp" | "plugin";
export type SkillScope = "global" | "project";

export interface GNode {
  id: string;
  label: string;
  kind: GNodeKind;
  /** Skills uniquement : portée globale (~/.claude) ou projet (<cwd>/.claude). */
  scope?: SkillScope;
  /** Chats uniquement : provider piloté (claude_code / opencode / gemini). */
  provider?: string;
  /** Chat : dossier de travail ; skill projet : dossier propriétaire. */
  cwd?: string;
  /** Chat uniquement : id de session (pour y sauter au clic). */
  sid?: string;
  /** Skill uniquement : nom de dossier (pour lire/écrire le SKILL.md). */
  skill?: string;
  /** MCP uniquement : nom du serveur. */
  mcp?: string;
  /** Plugin uniquement : identifiant "nom@marketplace". */
  plugin?: string;
  /** Skill uniquement : supprimable (faux = fourni par un plugin, lecture seule). */
  removable?: boolean;
  /** Skill uniquement : origine ("user" / "project" / "plugin:<nom>"). */
  source?: string;
  /** Layout (muté par la simulation). */
  x: number;
  y: number;
  vx: number;
  vy: number;
  /** Épinglé pendant un glisser : position figée. */
  fx: number | null;
  fy: number | null;
  /** Rayon de rendu. */
  r: number;
}

export type GLinkKind = "global" | "project" | "chat" | "mcp" | "plugin";

export interface GLink {
  source: string;
  target: string;
  kind: GLinkKind;
}

export interface Graph {
  nodes: GNode[];
  links: GLink[];
}
