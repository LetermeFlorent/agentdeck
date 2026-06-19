// Config partagée des chats : modèles, efforts, tarifs, formatage.

export const MODELS = [
  { v: "opus", l: "Opus" },
  { v: "sonnet", l: "Sonnet" },
  { v: "haiku", l: "Haiku" },
  { v: "fable", l: "Fable" },
];

/** Modes de permission du CLI Claude Code (--permission-mode). */
export const PERM_MODES = [
  { v: "bypassPermissions", l: "Tout autoriser" },
  { v: "acceptEdits", l: "Auto-éditer" },
  { v: "default", l: "Par défaut" },
  { v: "plan", l: "Plan (lecture seule)" },
  { v: "auto", l: "Auto" },
  { v: "dontAsk", l: "Ne pas demander" },
];

const LOW = { v: "low", l: "Low" };
const MEDIUM = { v: "medium", l: "Medium" };
const HIGH = { v: "high", l: "High" };
const XHIGH = { v: "xhigh", l: "Xhigh" };
const MAX = { v: "max", l: "Max" };
// Ultracode : exclusif Opus (mappé sur --effort xhigh côté CLI).
export const ULTRACODE = { v: "ultracode", l: "Ultracode" };

/** Niveaux d'effort valides selon le modèle (xhigh = Opus/Fable seulement ; Haiku = aucun). */
export function effortsFor(model: string | null | undefined): { v: string; l: string }[] {
  switch (model) {
    case "opus":
      return [LOW, MEDIUM, HIGH, XHIGH, MAX, ULTRACODE];
    case "fable":
      return [LOW, MEDIUM, HIGH, XHIGH, MAX];
    case "sonnet":
      return [LOW, MEDIUM, HIGH, MAX];
    default:
      return []; // haiku & inconnu : effort non supporté par le modèle
  }
}

// Tarifs par million de tokens (entrée / sortie) du modèle choisi.
export const PRICES: Record<string, [number, number]> = {
  opus: [5, 25],
  sonnet: [3, 15],
  haiku: [1, 5],
  fable: [10, 50],
};

export function priceOf(model: string | null | undefined): [number, number] | null {
  return PRICES[model ?? "opus"] ?? null;
}

// Commandes slash INTÉGRÉES interactives (TUI) : elles n'ont aucun effet utile en mode
// headless `claude -p` (le mode qu'agentdeck utilise) → on les masque du popup pour ne pas
// laisser croire qu'elles marchent. Les commandes custom / skills (deep-research, code-review,
// statusbar…) restent affichées car elles s'exécutent comme des prompts.
export const NON_HEADLESS_SLASH = new Set<string>([
  "clear", "compact", "config", "context", "init", "usage", "usage-credits", "extra-usage",
  "insights", "heapdump", "reload-skills", "goal", "team-onboarding", "model", "agents", "mcp",
  "login", "logout", "status", "doctor", "cost", "resume", "vim", "exit", "quit", "bug", "ide",
  "hooks", "permissions", "output-style", "memory", "add-dir", "release-notes", "privacy-settings",
  "terminal-setup", "pr-comments", "export", "feedback", "upgrade", "statusline", "todos",
]);

/** Tokens compacts : 1234 → "1.2k". */
export function fmtTok(n: number): string {
  return n >= 1000 ? (n / 1000).toFixed(1) + "k" : String(n);
}

/** Nombre complet avec séparateur de milliers (fr). */
export function fmtFull(n: number): string {
  return n.toLocaleString("fr-FR");
}
