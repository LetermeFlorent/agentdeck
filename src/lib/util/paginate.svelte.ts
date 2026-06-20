// Rendu incrémental : fenêtre croissante sur un nombre total d'items. On affiche `count`
// items et on en révèle `step` de plus à chaque appel `more()` (déclenché par une
// sentinelle `use:inView` en bas de liste). Simple — pas de virtualisation.

export class Reveal {
  count = $state(0);
  private step: number;

  constructor(step = 20) {
    this.step = step;
    this.count = step;
  }

  /** Révèle le lot suivant, borné à `total`. */
  more(total: number) {
    if (this.count < total) this.count = Math.min(this.count + this.step, total);
  }

  /** Réinitialise la fenêtre (ex. nouvelle recherche / nouveau jeu de données). */
  reset() {
    this.count = this.step;
  }

  /** Reste-t-il des items cachés ? */
  hasMore(total: number): boolean {
    return this.count < total;
  }
}
