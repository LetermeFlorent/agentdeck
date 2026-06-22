// Maths de la vue (pan / zoom) du graphe. `Viewport` est un objet d'état Svelte
// ($state) muté en place : translation (tx/ty) et facteur d'échelle (scale).

export interface Viewport {
  tx: number;
  ty: number;
  scale: number;
}

const MIN_SCALE = 0.25;
const MAX_SCALE = 3;

/** Convertit un point écran (clientX/Y) en coordonnées « monde » du graphe. */
export function screenToWorld(vp: Viewport, clientX: number, clientY: number, rect: DOMRect) {
  return {
    x: (clientX - rect.left - vp.tx) / vp.scale,
    y: (clientY - rect.top - vp.ty) / vp.scale,
  };
}

/** Zoom centré sur le curseur (molette). Conserve le point sous le curseur fixe. */
export function zoomAt(vp: Viewport, clientX: number, clientY: number, rect: DOMRect, deltaY: number) {
  const k = deltaY < 0 ? 1.1 : 1 / 1.1;
  const ns = Math.min(MAX_SCALE, Math.max(MIN_SCALE, vp.scale * k));
  const mx = clientX - rect.left;
  const my = clientY - rect.top;
  vp.tx = mx - (mx - vp.tx) * (ns / vp.scale);
  vp.ty = my - (my - vp.ty) * (ns / vp.scale);
  vp.scale = ns;
}
