import { describe, it, expect } from "vitest";

// ── Logique passes de réflexion (copiée de sessions.svelte.ts ligne 668) ──────
function buildFinalText(text: string, passes: number): string {
  return passes > 1
    ? `[Avant d'agir, réfléchis ${passes} fois à cette demande : analyse le problème sous ${passes} angles différents, identifie les pièges potentiels, puis exécute seulement après ces ${passes} passes de réflexion.]\n\n${text}`
    : text;
}

// ── Logique clamp passes (copiée de settings.svelte.ts) ──────────────────────
function clampPasses(v: number): number {
  return Math.max(1, Math.min(10, v));
}

// ── Regex détection modèle free ───────────────────────────────────────────────
const FREE_RE = /[-:]free$/;

describe("Passes de réflexion", () => {
  it("passes=1 → texte inchangé", () => {
    const msg = "Crée un fichier hello.txt";
    expect(buildFinalText(msg, 1)).toBe(msg);
  });

  it("passes=2 → préfixe injecté avec ×2", () => {
    const result = buildFinalText("Fais X", 2);
    expect(result).toContain("réfléchis 2 fois");
    expect(result).toContain("2 angles différents");
    expect(result).toContain("2 passes de réflexion");
    expect(result).toContain("Fais X");
  });

  it("passes=5 → préfixe injecté avec ×5", () => {
    const result = buildFinalText("Fais Y", 5);
    expect(result).toContain("réfléchis 5 fois");
    expect(result).toContain("5 passes de réflexion");
  });

  it("passes=0 → traité comme ≤1 → texte inchangé", () => {
    expect(buildFinalText("msg", 0)).toBe("msg");
  });

  it("préfixe séparé du texte par double newline", () => {
    const result = buildFinalText("Texte réel", 3);
    expect(result).toMatch(/\]\n\nTexte réel$/);
  });
});

describe("Clamp passes (1–10)", () => {
  it("valeur normale → inchangée", () => {
    expect(clampPasses(3)).toBe(3);
    expect(clampPasses(1)).toBe(1);
    expect(clampPasses(10)).toBe(10);
  });

  it("en dessous de 1 → 1", () => {
    expect(clampPasses(0)).toBe(1);
    expect(clampPasses(-5)).toBe(1);
  });

  it("au-dessus de 10 → 10", () => {
    expect(clampPasses(11)).toBe(10);
    expect(clampPasses(99)).toBe(10);
  });
});

describe("Regex détection modèle free", () => {
  it("opencode -free → détecté", () => {
    expect(FREE_RE.test("opencode/flash-lite-free")).toBe(true);
    expect(FREE_RE.test("opencode/gemini-nano-free")).toBe(true);
  });

  it("openrouter :free → détecté", () => {
    expect(FREE_RE.test("openrouter/mistral-7b-instruct:free")).toBe(true);
    expect(FREE_RE.test("openrouter/meta-llama/llama-3:free")).toBe(true);
  });

  it("modèle payant → non détecté", () => {
    expect(FREE_RE.test("claude-3-5-sonnet-20241022")).toBe(false);
    expect(FREE_RE.test("opencode/gemini-pro")).toBe(false);
    expect(FREE_RE.test("openrouter/gpt-4o")).toBe(false);
  });

  it("free au milieu du nom → non détecté (doit être en fin)", () => {
    expect(FREE_RE.test("opencode/free-flash")).toBe(false);
    expect(FREE_RE.test("some-free-model-pro")).toBe(false);
  });
});
