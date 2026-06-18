// Store d'auth : état connecté + actions (import fichier, token collé, login OAuth, logout).

import * as ipc from "$lib/ipc";

class AuthStore {
  connected = $state(false);
  checking = $state(true);

  async check() {
    this.checking = true;
    try {
      this.connected = await ipc.authStatus();
    } finally {
      this.checking = false;
    }
  }

  async importFromDownloads() {
    await ipc.authImportFromFile();
    this.connected = true;
  }

  async setToken(token: string) {
    await ipc.authSetToken(token.trim());
    this.connected = true;
  }

  async login() {
    await ipc.authLogin();
    this.connected = true;
  }

  async logout() {
    await ipc.authClear();
    this.connected = false;
  }
}

export const auth = new AuthStore();
