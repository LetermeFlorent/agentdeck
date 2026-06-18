// Config partagée des chats : modèles, efforts, tarifs, formatage.

export const MODELS = [
  { v: "opus", l: "Opus" },
  { v: "sonnet", l: "Sonnet" },
  { v: "haiku", l: "Haiku" },
  { v: "fable", l: "Fable" },
];

export const EFFORTS = [
  { v: "low", l: "Low" },
  { v: "medium", l: "Medium" },
  { v: "high", l: "High" },
  { v: "xhigh", l: "Xhigh" },
  { v: "max", l: "Max" },
];

// Ultracode : exclusif Opus (mappé sur --effort xhigh côté CLI).
export const ULTRACODE = { v: "ultracode", l: "Ultracode" };

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

/** Tokens compacts : 1234 → "1.2k". */
export function fmtTok(n: number): string {
  return n >= 1000 ? (n / 1000).toFixed(1) + "k" : String(n);
}

/** Nombre complet avec séparateur de milliers (fr). */
export function fmtFull(n: number): string {
  return n.toLocaleString("fr-FR");
}
