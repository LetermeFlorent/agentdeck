// Store de thème : choix utilisateur (system|light|dark) résolu en data-theme sur <html>.

import { STORAGE_KEYS } from "./keys";

export type ThemeChoice = "system" | "light" | "dark";

const KEY = STORAGE_KEYS.theme;

function systemDark(): boolean {
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? true;
}

function resolve(choice: ThemeChoice): "light" | "dark" {
  if (choice === "system") return systemDark() ? "dark" : "light";
  return choice;
}

function apply(choice: ThemeChoice) {
  document.documentElement.setAttribute("data-theme", resolve(choice));
}

class ThemeStore {
  choice = $state<ThemeChoice>("light");
  #mediaBound = false;

  init() {
    const saved = localStorage.getItem(KEY) as ThemeChoice | null;
    this.choice = saved ?? "light";
    apply(this.choice);
    // Suivre le système quand le choix est "system". Lié une seule fois (le listener vit
    // toute la durée de l'app) : évite d'empiler des doublons si init() est rappelé (HMR).
    if (!this.#mediaBound) {
      this.#mediaBound = true;
      window.matchMedia?.("(prefers-color-scheme: dark)").addEventListener("change", () => {
        if (this.choice === "system") apply("system");
      });
    }
  }

  set(choice: ThemeChoice) {
    this.choice = choice;
    localStorage.setItem(KEY, choice);
    apply(choice);
  }

  get resolved(): "light" | "dark" {
    return resolve(this.choice);
  }
}

export const theme = new ThemeStore();
