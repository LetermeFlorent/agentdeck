<script lang="ts">
  // Marque agentdeck : le spark Claude (12 branches effilées) généré, statique. Réutilisable.
  let { size = 16 }: { size?: number } = $props();

  const N = 28;
  const c = N / 2;
  const SPOKES = 12;
  const R = 13;
  const CORE = 2.4;
  const W0 = 0.42;
  const BASE = "#d97757";
  const COREC = "#ecaf95";
  const cells: { x: number; y: number; fill: string }[] = [];
  for (let y = 0; y < N; y++) {
    for (let x = 0; x < N; x++) {
      const dx = x + 0.5 - c;
      const dy = y + 0.5 - c;
      const r = Math.hypot(dx, dy);
      if (r > R) continue;
      const th = Math.atan2(dy, dx);
      const a = (2 * Math.PI) / SPOKES;
      const d = Math.abs(th - Math.round(th / a) * a);
      if (r <= CORE || d <= W0 * (1 - r / R)) {
        cells.push({ x, y, fill: r < CORE * 1.25 ? COREC : BASE });
      }
    }
  }
</script>

<svg
  viewBox="0 0 {N} {N}"
  width={size}
  height={size}
  shape-rendering="crispEdges"
  aria-hidden="true"
  style="display:block"
>
  {#each cells as p}
    <rect x={p.x} y={p.y} width="1" height="1" fill={p.fill} />
  {/each}
</svg>
