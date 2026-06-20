// Action Svelte : appelle `cb` quand l'élément entre dans le viewport (conteneur scrollable
// le plus proche). Sert à 2 usages : révéler le lot suivant d'une liste (sentinelle en bas),
// et charger paresseusement le contenu d'un item (description skill) quand sa carte apparaît.

interface Options {
  /** Callback déclenché à l'entrée dans le viewport. */
  onenter?: () => void;
  /** Se déconnecte après le 1er déclenchement (défaut : true). */
  once?: boolean;
  /** Marge autour du root pour précharger un peu avant (défaut : "150px"). */
  rootMargin?: string;
}

export function inView(node: HTMLElement, opts: Options | (() => void) = {}) {
  let options: Options = typeof opts === "function" ? { onenter: opts } : opts;

  const make = () =>
    new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) {
            options.onenter?.();
            if (options.once !== false) observer.disconnect();
          }
        }
      },
      { rootMargin: options.rootMargin ?? "150px" },
    );

  let observer = make();
  observer.observe(node);

  return {
    update(next: Options | (() => void)) {
      options = typeof next === "function" ? { onenter: next } : next;
    },
    destroy() {
      observer.disconnect();
    },
  };
}
