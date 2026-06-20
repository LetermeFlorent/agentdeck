// Config partagée des chats : modèles, efforts, tarifs, formatage.

export const MODELS = [
  { v: "claude-opus-4-8", l: "Opus 4.8" },
  { v: "claude-opus-4-7", l: "Opus 4.7" },
  { v: "claude-opus-4-6", l: "Opus 4.6" },
  { v: "claude-sonnet-4-6", l: "Sonnet 4.6" },
  { v: "claude-haiku-4-5", l: "Haiku 4.5" },
  { v: "claude-fable-5", l: "Fable 5" },
];

/** Famille (tier) d'un modèle depuis son alias OU son ID exact (claude-opus-4-6 → "opus"). */
export function tierOf(model: string | null | undefined): "opus" | "sonnet" | "haiku" | "fable" | null {
  const m = model ?? "";
  if (m === "opus" || m.startsWith("claude-opus")) return "opus";
  if (m === "sonnet" || m.startsWith("claude-sonnet")) return "sonnet";
  if (m === "haiku" || m.startsWith("claude-haiku")) return "haiku";
  if (m === "fable" || m.startsWith("claude-fable")) return "fable";
  return null;
}

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

/** Niveaux d'effort valides selon le modèle.
 *  xhigh n'existe qu'à partir d'Opus 4.7 (donc PAS sur Opus 4.6) ; Sonnet n'a pas de xhigh ;
 *  Haiku n'a aucun effort. L'alias "opus" = dernier Opus → supporte xhigh + ultracode. */
export function effortsFor(model: string | null | undefined): { v: string; l: string }[] {
  switch (tierOf(model)) {
    case "opus":
      // Opus 4.6 : pas de xhigh ni ultracode (xhigh ajouté en 4.7).
      if (model === "claude-opus-4-6") return [LOW, MEDIUM, HIGH, MAX];
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
  return PRICES[tierOf(model) ?? "opus"] ?? null;
}

/** Construit l'instruction envoyée au modèle-choisisseur (mode Auto). Inclut prix + récence
 *  pour optimiser le coût tout en visant le meilleur résultat. Réutilisée pour l'aperçu (popup). */
export function autoPickPrompt(
  userPrompt: string,
  models: { v: string; l: string }[],
  efforts: { v: string; l: string }[],
): string {
  const parts: string[] = [
    "Tu choisis la configuration optimale (modèle et/ou effort) pour traiter la demande d'un " +
      "utilisateur à un agent de code. Objectif : le MEILLEUR résultat au MOINDRE coût.",
  ];
  if (models.length) {
    const rows = models
      .map((m) => {
        const p = priceOf(m.v);
        const price = p ? `entrée $${p[0]}/M tok, sortie $${p[1]}/M tok` : "tarif inconnu";
        return `  - ${m.l} (id: ${m.v} · ${price})`;
      })
      .join("\n");
    parts.push(
      "Modèles candidats (numéro de version plus élevé = sorti plus récemment et en général " +
        "plus performant, mais regarde le prix) :\n" + rows,
    );
  }
  if (efforts.length) {
    parts.push("Efforts possibles : " + efforts.map((e) => e.v).join(", ") + ".");
  }
  parts.push(
    "Règle : demande simple / question courte → modèle le moins cher qui suffit" +
      (efforts.length ? " + effort bas" : "") +
      " ; tâche complexe (code, archi, debug, raisonnement long) → modèle plus puissant/récent" +
      (efforts.length ? " + effort élevé" : "") +
      ". Ne prends un modèle cher que si la tâche le justifie vraiment.",
  );
  const shape =
    models.length && efforts.length
      ? '{"model":"<un id ou vide>","effort":"<un effort ou vide>"}'
      : models.length
        ? '{"model":"<un id ou vide>","effort":""}'
        : '{"model":"","effort":"<un effort ou vide>"}';
  parts.push("Réponds STRICTEMENT en JSON sur une ligne, sans texte autour : " + shape + ".");
  parts.push("Demande : " + userPrompt);
  return parts.join("\n");
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
