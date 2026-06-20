// Tour guidé du premier lancement (coachmarks). Affiché une seule fois, rejouable
// depuis les Paramètres. L'état d'« onboarding fait » vit dans localStorage.

import { STORAGE_KEYS } from "./keys";

const KEY = STORAGE_KEYS.onboarded;

class TourStore {
  active = $state(false);

  /** Déjà vu le tour ? */
  get seen(): boolean {
    try {
      return localStorage.getItem(KEY) === "1";
    } catch {
      return true; // en cas de doute, ne pas harceler
    }
  }

  /** Lance le tour (1er lancement ou relance manuelle). */
  start() {
    this.active = true;
  }

  /** Lance le tour seulement s'il n'a jamais été vu. */
  maybeStart() {
    if (!this.seen) this.active = true;
  }

  /** Termine le tour et mémorise qu'il a été vu. */
  done() {
    this.active = false;
    try {
      localStorage.setItem(KEY, "1");
    } catch {
      /* ignore */
    }
  }
}

export const tour = new TourStore();
