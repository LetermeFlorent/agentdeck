// Store d'auth multi-provider : état connecté par IA + actions (import fichier, clé/token, login, logout).

import * as ipc from "$lib/ipc";

class AuthStore {
  /** Connecté ? par provider ("claude_code" | "opencode" | "gemini"). */
  byProvider = $state<Record<string, boolean>>({});
  checking = $state(true);

  /** Compat Claude : état connecté du provider par défaut. */
  get connected(): boolean {
    return this.byProvider["claude_code"] ?? false;
  }

  /** true si n'importe quel provider est connecté. */
  get anyConnected(): boolean {
    return Object.values(this.byProvider).some(Boolean);
  }

  isConnected(provider: string): boolean {
    return this.byProvider[provider] ?? false;
  }

  async check() {
    this.checking = true;
    try {
      await this.refresh();
    } finally {
      this.checking = false;
    }
  }

  /** Rafraîchit l'état connecté de chaque IA SANS toucher `checking` (sinon le gate de boot
   *  de +page bascule sur le BootLoader et démonte l'app → flash/boucle quand on ouvre les
   *  réglages). À utiliser depuis l'UI ouverte. */
  async refresh() {
    const provs = ["claude_code", "opencode", "gemini"];
    const res = await Promise.all(provs.map((p) => ipc.authStatus(p).catch(() => false)));
    this.byProvider = Object.fromEntries(provs.map((p, i) => [p, res[i]]));
  }

  async checkProvider(provider: string) {
    try {
      this.byProvider = { ...this.byProvider, [provider]: await ipc.authStatus(provider) };
    } catch {
      /* ignore */
    }
  }

  async importFromDownloads() {
    await ipc.authImportFromFile();
    this.byProvider = { ...this.byProvider, claude_code: true };
  }

  /** Colle un token/clé pour un provider (Claude : token OAuth ; Gemini : clé API). */
  async setToken(token: string, provider = "claude_code") {
    await ipc.authSetToken(token.trim(), provider);
    this.byProvider = { ...this.byProvider, [provider]: true };
  }

  /** Lance le flux de connexion d'un provider (Claude : navigateur ; autres : terminal CLI). */
  async login(provider = "claude_code") {
    await ipc.authLogin(provider);
    if (provider === "claude_code") {
      this.byProvider = { ...this.byProvider, claude_code: true };
    } else {
      // Gemini/opencode : la connexion se fait dans leur CLI → on re-sonde l'état.
      await this.checkProvider(provider);
    }
  }

  async logout(provider = "claude_code") {
    await ipc.authClear(provider);
    this.byProvider = { ...this.byProvider, [provider]: false };
  }

  /** Déconnecte TOUTES les IA connectées (vrai logout → retour onboarding). */
  async logoutAll() {
    const provs = Object.keys(this.byProvider).filter((p) => this.byProvider[p]);
    await Promise.all(provs.map((p) => ipc.authClear(p).catch(() => {})));
    this.byProvider = {};
  }

  /** Ouvre un terminal sur le login NATIF du CLI (claude / opencode / gemini), puis re-sonde. */
  async cliLogin(provider: string) {
    await ipc.cliTerminalLogin(provider);
    await this.checkProvider(provider);
  }
}

export const auth = new AuthStore();
