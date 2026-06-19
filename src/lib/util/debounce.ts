// Petit utilitaire de debounce : regroupe les appels rapprochés en un seul, après `wait` ms
// d'inactivité. `flush()` force l'exécution immédiate d'un appel en attente (utile avant
// fermeture/déconnexion pour ne pas perdre le dernier état). `cancel()` annule l'appel en attente.

export interface Debounced {
  (): void;
  flush: () => void;
  cancel: () => void;
}

export function debounce(fn: () => void, wait = 400): Debounced {
  let timer: ReturnType<typeof setTimeout> | null = null;
  const run: Debounced = () => {
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      fn();
    }, wait);
  };
  run.flush = () => {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
      fn();
    }
  };
  run.cancel = () => {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  };
  return run;
}
