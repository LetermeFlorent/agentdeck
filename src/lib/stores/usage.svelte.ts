// Store d'usage : snapshot des barres 5h / semaine, rafraîchi périodiquement.

import * as ipc from "$lib/ipc";
import type { UsageSnapshot } from "$lib/ipc";

class UsageStore {
  snapshot = $state<UsageSnapshot | null>(null);
  private timer: number | null = null;

  async refresh() {
    try {
      this.snapshot = await ipc.usageGet();
    } catch {
      /* ignore */
    }
  }

  start() {
    if (this.timer !== null) return; // déjà démarré : pas de second intervalle
    this.refresh();
    this.timer = window.setInterval(() => this.refresh(), 30_000);
  }

  stop() {
    if (this.timer !== null) {
      window.clearInterval(this.timer);
      this.timer = null;
    }
  }
}

export const usage = new UsageStore();
