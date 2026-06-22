// Store d'usage : snapshot des barres 5h / semaine, rafraîchi périodiquement.

import * as ipc from "$lib/ipc";
import type { UsageSnapshot } from "$lib/ipc";

class UsageStore {
  snapshot = $state<UsageSnapshot | null>(null);
  gemini = $state<UsageSnapshot | null>(null);
  opencode = $state<UsageSnapshot | null>(null);
  private timer: number | null = null;

  async refresh() {
    try { this.snapshot = await ipc.usageGet(); } catch { /* ignore */ }
    try { this.gemini = await ipc.usageGetProvider("gemini"); } catch { /* ignore */ }
    try { this.opencode = await ipc.usageGetProvider("opencode"); } catch { /* ignore */ }
  }

  start() {
    if (this.timer !== null) return;
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
