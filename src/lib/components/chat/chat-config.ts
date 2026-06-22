// Config partagée des chats : providers (multi-IA), modèles, efforts, tarifs, formatage.

export const MODELS = [
  { v: "claude-opus-4-8", l: "Opus 4.8" },
  { v: "claude-opus-4-7", l: "Opus 4.7" },
  { v: "claude-opus-4-6", l: "Opus 4.6" },
  { v: "claude-sonnet-4-6", l: "Sonnet 4.6" },
  { v: "claude-haiku-4-5", l: "Haiku 4.5" },
  { v: "claude-fable-5", l: "Fable 5" },
];

// ---- Providers (multi-IA) ------------------------------------------------

export type ProviderId = "claude_code" | "opencode" | "gemini";

export interface ProviderInfo {
  id: ProviderId;
  label: string;
  /** Effort réglable (Claude : --effort ; opencode : --variant ; Gemini : aucun). */
  hasEffort: boolean;
  /** Modes de permission (Claude uniquement). */
  hasPerm: boolean;
  /** Commandes slash dynamiques (Claude uniquement). */
  hasSlash: boolean;
  /** Liste de secours de modèles (utilisée tant que la liste live n'est pas chargée). */
  fallbackModels: { v: string; l: string }[];
}

/** Modèles Gemini de secours (le CLI n'expose pas de liste machine fiable). */
const GEMINI_FALLBACK = [
  { v: "gemini-2.5-pro", l: "2.5 Pro" },
  { v: "gemini-2.5-flash", l: "2.5 Flash" },
  { v: "gemini-2.5-flash-lite", l: "2.5 Flash-Lite" },
  { v: "gemini-2.0-flash", l: "2.0 Flash" },
];

export const PROVIDERS: ProviderInfo[] = [
  { id: "claude_code", label: "Claude", hasEffort: true, hasPerm: true, hasSlash: true, fallbackModels: MODELS },
  { id: "opencode", label: "opencode", hasEffort: true, hasPerm: false, hasSlash: true, fallbackModels: [] },
  { id: "gemini", label: "Gemini", hasEffort: false, hasPerm: false, hasSlash: false, fallbackModels: GEMINI_FALLBACK },
];

export function providerInfo(id: string | null | undefined): ProviderInfo {
  return PROVIDERS.find((p) => p.id === id) ?? PROVIDERS[0];
}

/** Devine l'IA d'un id de modèle : gemini-* → gemini ; "provider/model" → opencode ; sinon Claude. */
export function providerOfModel(id: string | null | undefined): ProviderId {
  const m = id ?? "";
  if (m.startsWith("gemini")) return "gemini";
  if (m.includes("/")) return "opencode";
  return "claude_code";
}

/** Efforts valides selon (provider, modèle). opencode = --variant ; Gemini = aucun. */
export function effortsForProvider(
  provider: string | null | undefined,
  model: string | null | undefined,
): { v: string; l: string }[] {
  switch (providerInfo(provider).id) {
    case "claude_code":
      return effortsFor(model);
    case "opencode":
      return [
        { v: "minimal", l: "Minimal" },
        { v: "low", l: "Low" },
        { v: "medium", l: "Medium" },
        { v: "high", l: "High" },
        { v: "max", l: "Max" },
      ];
    default:
      return []; // gemini : pas d'effort
  }
}

/** Liste affichée pour un provider : Claude collapse par tier ; les autres passent tels quels. */
export function visibleForProvider(
  provider: string | null | undefined,
  models: { v: string; l: string }[],
): { v: string; l: string }[] {
  return providerInfo(provider).id === "claude_code" ? latestPerTier(models) : models;
}

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

/** Tarifs Gemini (clé API, $/M tokens entrée/sortie) par famille. Free OAuth = 0 mais on garde
 *  les prix API (cas le plus courant maintenant). */
const GEMINI_PRICES: Record<string, [number, number]> = {
  "g-pro": [1.25, 10],
  "g-flash": [0.3, 2.5],
  "g-flash-lite": [0.1, 0.4],
};
function geminiTier(m: string): string | null {
  if (!m.startsWith("gemini")) return null;
  if (m.includes("flash-lite")) return "g-flash-lite";
  if (m.includes("flash")) return "g-flash";
  if (m.includes("pro")) return "g-pro";
  return "g-flash";
}

/** Tarif (entrée/sortie par M tok) d'un modèle, multi-IA. null si inconnu. */
export function priceOf(model: string | null | undefined): [number, number] | null {
  const m = model ?? "";
  const gt = geminiTier(m);
  if (gt) return GEMINI_PRICES[gt] ?? null;
  if (m.includes("/")) return /[-:]free$/.test(m) ? [0, 0] : null; // opencode : free détecté, sinon inconnu
  return PRICES[tierOf(m) ?? "opus"] ?? null;
}

/** Tarif compact à afficher à côté d'un modèle : "$5/$25", "free", ou "" si inconnu. */
export function priceHint(model: string | null | undefined): string {
  const m = model ?? "";
  const gt = geminiTier(m);
  if (gt) {
    const p = GEMINI_PRICES[gt];
    return p ? `$${p[0]}/$${p[1]}` : "";
  }
  if (m.includes("/")) return /[-:]free$/.test(m) ? "free" : ""; // opencode
  const t = tierOf(m);
  const p = t ? PRICES[t] : null;
  return p ? `$${p[0]}/$${p[1]}` : "";
}

/** Ajoute le prix (hint) à une liste de modèles pour les dropdowns. */
export function withPrices(models: { v: string; l: string }[]): { v: string; l: string; hint: string }[] {
  return models.map((m) => ({ ...m, hint: priceHint(m.v) }));
}

/** Groupes d'entiers d'un ID, pour comparer les versions d'un même tier (claude-opus-4-8 → [4,8]). */
function verOf(id: string): number[] {
  return (id.match(/\d+/g) ?? []).map(Number);
}

/** Compare deux versions élément par élément (le plus long/grand gagne). >0 si a plus récent que b. */
function cmpVer(a: number[], b: number[]): number {
  const n = Math.max(a.length, b.length);
  for (let i = 0; i < n; i++) {
    const d = (a[i] ?? 0) - (b[i] ?? 0);
    if (d !== 0) return d;
  }
  return 0;
}

/** Garde un seul modèle par tier : le plus récent. Tier inconnu → chaque modèle conservé tel quel.
 *  Préserve l'ordre de première apparition par tier. */
export function latestPerTier(models: { v: string; l: string }[]): { v: string; l: string }[] {
  const best = new Map<string, { v: string; l: string }>();
  for (const m of models) {
    const key = tierOf(m.v) ?? m.v;
    const cur = best.get(key);
    if (!cur || cmpVer(verOf(m.v), verOf(cur.v)) > 0) best.set(key, m);
  }
  return [...best.values()];
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
        return `  - ${m.v}  (${m.l} · ${price})`;
      })
      .join("\n");
    parts.push(
      "Modèles candidats — réponds avec l'id EXACT (1ʳᵉ colonne), pas le nom. Numéro de version " +
        "plus élevé = sorti plus récemment et en général plus performant, mais regarde le prix :\n" + rows,
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
