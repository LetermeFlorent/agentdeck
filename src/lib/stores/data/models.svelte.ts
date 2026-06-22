// Liste des modèles sélectionnables, par provider (multi-IA). Par défaut = liste de secours
// (chat-config), remplacée à la demande par les modèles réels (provider_models côté backend).

import * as ipc from "$lib/ipc";
import { MODELS, tierOf, providerInfo, visibleForProvider } from "$lib/components/chat/chat-config";

class ModelStore {
  /** Cache des modèles par provider. claude_code pré-rempli avec la liste de secours. */
  byProvider = $state<Record<string, { v: string; l: string }[]>>({ claude_code: MODELS });

  /** Par provider : true pendant le chargement via IPC. */
  loading = $state<Record<string, boolean>>({});

  /** Liste brute d'un provider (live si chargée, sinon secours). */
  availableFor(provider: string): { v: string; l: string }[] {
    return this.byProvider[provider] ?? providerInfo(provider).fallbackModels;
  }

  /** Liste affichée d'un provider (Claude collapse par tier ; autres = tels quels). */
  visibleFor(provider: string): { v: string; l: string }[] {
    return visibleForProvider(provider, this.availableFor(provider));
  }

  /** true si le provider est en train de charger ses modèles. */
  loadingFor(provider: string): boolean {
    return this.loading[provider] ?? false;
  }

  /** Charge les modèles réels d'un provider (no-op si vide / échec → secours conservé). */
  async loadFor(provider: string) {
    this.loading[provider] = true;
    try {
      const list = await ipc.providerModels(provider);
      if (list.length) this.byProvider[provider] = list;
    } catch {
      /* offline / non connecté → fallback conservé */
    } finally {
      this.loading[provider] = false;
    }
  }

  // --- Compat Claude (anciens appels sans provider) ---
  get available(): { v: string; l: string }[] {
    return this.availableFor("claude_code");
  }
  get visible(): { v: string; l: string }[] {
    return this.visibleFor("claude_code");
  }

  /** Modèle choisisseur par défaut (mode Auto) = dernier Haiku, sinon alias "haiku". */
  get pickerDefault(): string {
    const haikus = this.available.filter((m) => tierOf(m.v) === "haiku");
    if (!haikus.length) return "haiku";
    return [...haikus].sort((a, b) => b.v.localeCompare(a.v))[0].v;
  }

  /** Choisisseur par défaut selon l'IA (modèle pas cher de cette IA). */
  pickerDefaultFor(provider: string): string {
    if (provider === "claude_code") return this.pickerDefault;
    const list = this.availableFor(provider);
    if (provider === "gemini") {
      return list.find((m) => m.v.includes("flash-lite"))?.v ?? list.find((m) => m.v.includes("flash"))?.v ?? list[0]?.v ?? "";
    }
    // opencode : un modèle gratuit "flash"/"mini" si possible, sinon le 1er gratuit, sinon le 1er.
    const free = list.filter((m) => /[-:]free$/.test(m.v));
    return (
      free.find((m) => /flash|mini|nano/.test(m.v))?.v ?? free[0]?.v ?? list[0]?.v ?? ""
    );
  }

  /** Compat : charge la liste Claude. */
  async load() {
    return this.loadFor("claude_code");
  }
}

export const modelStore = new ModelStore();
