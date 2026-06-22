<script lang="ts">
  import { providerColor, neighborsOf } from "./graph-model";
  import { screenToWorld, zoomAt, type Viewport } from "./graph-interaction";
  import type { Graph, GNode } from "./graph-types";

  let { graph, focusId = null, onpick, onhover }: {
    graph: Graph;
    focusId?: string | null;
    onpick: (n: GNode) => void;
    onhover: (id: string | null) => void;
  } = $props();

  let wrap = $state<HTMLDivElement>();
  let w = $state(800);
  let h = $state(600);
  let frame = $state(0); // bumpé chaque step → redraw
  const vp = $state<Viewport>({ tx: 0, ty: 0, scale: 1 });

  // Positions physiques (hors état réactif : mutées en place, le compteur `frame` déclenche le redraw).
  type P = { x: number; y: number; vx: number; vy: number; fx: number | null; fy: number | null };
  const pos = new Map<string, P>();
  let raf = 0;
  let alpha = 1;
  let drag: { id: string | null; sx: number; sy: number; moved: boolean } | null = null;

  // Constantes force (façon Obsidian : répulsion + ressorts + gravité, amortissement).
  const REPULSION = 5200;
  const SPRING = 0.028;
  const REST = 110;
  const DAMP = 0.82;
  const GRAVITY = 0.02;
  const CHAT_CENTER = 0.12; // les chats tendent vers le centre (hub)
  const MIN_ALPHA = 0.02;

  // Index déterministe pour le seed (pas de Math.random → reproductible).
  function seed(id: string, i: number, total: number) {
    const cx = (w || 800) / 2;
    const cy = (h || 600) / 2;
    const a = (i / Math.max(1, total)) * Math.PI * 2;
    // Naissance proche du centre → animation de déploiement vers l'extérieur.
    const r = 24 + (i % 7) * 6;
    pos.set(id, { x: cx + Math.cos(a) * r, y: cy + Math.sin(a) * r, vx: 0, vy: 0, fx: null, fy: null });
  }

  // Synchronise la map de positions avec les nœuds courants (ajoute les nouveaux, retire les disparus).
  function sync() {
    const ids = new Set(graph.nodes.map((n) => n.id));
    for (const id of [...pos.keys()]) if (!ids.has(id)) pos.delete(id);
    graph.nodes.forEach((n, i) => { if (!pos.has(n.id)) seed(n.id, i, graph.nodes.length); });
  }

  function step() {
    const ns = graph.nodes;
    const cx = (w || 800) / 2;
    const cy = (h || 600) / 2;

    // Répulsion O(n²).
    for (let i = 0; i < ns.length; i++) {
      const a = pos.get(ns[i].id)!;
      for (let j = i + 1; j < ns.length; j++) {
        const b = pos.get(ns[j].id)!;
        let dx = a.x - b.x, dy = a.y - b.y;
        let d2 = dx * dx + dy * dy || 0.01;
        const d = Math.sqrt(d2);
        const f = (REPULSION / d2) * alpha;
        const ux = dx / d, uy = dy / d;
        a.vx += ux * f; a.vy += uy * f;
        b.vx -= ux * f; b.vy -= uy * f;
      }
    }
    // Ressorts le long des liens.
    for (const l of graph.links) {
      const a = pos.get(l.source), b = pos.get(l.target);
      if (!a || !b) continue;
      const dx = b.x - a.x, dy = b.y - a.y;
      const d = Math.hypot(dx, dy) || 0.01;
      const f = (d - REST) * SPRING * alpha;
      const ux = dx / d, uy = dy / d;
      a.vx += ux * f; a.vy += uy * f;
      b.vx -= ux * f; b.vy -= uy * f;
    }
    // Gravité + intégration.
    for (const n of ns) {
      const p = pos.get(n.id)!;
      const g = n.kind === "claude" ? CHAT_CENTER : GRAVITY;
      p.vx += (cx - p.x) * g * alpha;
      p.vy += (cy - p.y) * g * alpha;
      if (p.fx != null) { p.x = p.fx; p.vx = 0; } else { p.vx *= DAMP; p.x += p.vx; }
      if (p.fy != null) { p.y = p.fy; p.vy = 0; } else { p.vy *= DAMP; p.y += p.vy; }
    }
    alpha = Math.max(MIN_ALPHA, alpha * 0.985);
    frame++;
  }

  // Boucle d'animation : (re)seed au changement de graphe/taille, puis tourne en continu.
  $effect(() => {
    void graph.nodes.length;
    void w; void h;
    sync();
    alpha = 1; // reheat → animation d'apparition / réorganisation
    cancelAnimationFrame(raf);
    const loop = () => { step(); raf = requestAnimationFrame(loop); };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  });

  $effect(() => {
    if (!wrap) return;
    const ro = new ResizeObserver(() => {
      const nw = wrap!.clientWidth, nh = wrap!.clientHeight;
      if (!nw || !nh) return;
      w = nw; h = nh;
    });
    ro.observe(wrap);
    return () => ro.disconnect();
  });

  const active = $derived(focusId ? neighborsOf(graph, focusId).add(focusId) : null);
  const view = $derived(`translate(${vp.tx},${vp.ty}) scale(${vp.scale})`);
  const G = (id: string) => pos.get(id) ?? { x: 0, y: 0 };

  const nodes = $derived.by(() => (void frame, graph.nodes.map((n) => ({ n, ...G(n.id) }))));
  const links = $derived.by(() =>
    (void frame, graph.links.flatMap((l) => {
      const a = pos.get(l.source), b = pos.get(l.target);
      if (!a || !b) return [];
      const mx = (a.x + b.x) / 2, my = (a.y + b.y) / 2;
      const dx = b.x - a.x, dy = b.y - a.y;
      const len = Math.hypot(dx, dy) || 1;
      const bow = Math.min(50, len * 0.16);
      const d = `M${a.x},${a.y} Q${mx + (-dy / len) * bow},${my + (dx / len) * bow} ${b.x},${b.y}`;
      return [{ k: l.source + l.target, d, kind: l.kind, aId: l.source, bId: l.target }];
    })),
  );

  const nodeColor = (n: GNode) => {
    switch (n.kind) {
      case "claude": return "var(--accent)";
      case "chat": return providerColor(n.provider);
      case "mcp": return "var(--prov-mcp, #b58cf0)";
      case "plugin": return "var(--prov-plugin, #e0a45e)";
      default: return n.scope === "project" ? "var(--good, #5bbf7a)" : "var(--accent)";
    }
  };
  const dim = (id: string) => active != null && !active.has(id);

  function onDown(e: PointerEvent, n: GNode | null) {
    (e.target as Element).setPointerCapture?.(e.pointerId);
    drag = { id: n?.id ?? null, sx: e.clientX, sy: e.clientY, moved: false };
    if (n) {
      const p = pos.get(n.id);
      if (p) {
        const w0 = screenToWorld(vp, e.clientX, e.clientY, wrap!.getBoundingClientRect());
        p.fx = w0.x; p.fy = w0.y;
      }
    }
  }
  function onMove(e: PointerEvent) {
    if (!drag) return;
    if (Math.abs(e.clientX - drag.sx) + Math.abs(e.clientY - drag.sy) > 3) drag.moved = true;
    if (drag.id) {
      const p = pos.get(drag.id);
      if (p) {
        const wpt = screenToWorld(vp, e.clientX, e.clientY, wrap!.getBoundingClientRect());
        p.fx = wpt.x; p.fy = wpt.y;
        alpha = Math.max(alpha, 0.6); // réchauffe → les voisins suivent (ressorts)
      }
    } else {
      vp.tx += e.clientX - drag.sx; vp.ty += e.clientY - drag.sy;
      drag.sx = e.clientX; drag.sy = e.clientY;
    }
  }
  function onUp() {
    if (drag?.id) {
      const p = pos.get(drag.id);
      if (p) { p.fx = null; p.fy = null; } // relâche → la physique reprend
      if (!drag.moved) {
        const n = graph.nodes.find((x) => x.id === drag!.id);
        if (n) onpick(n);
      }
    }
    drag = null;
  }
  function onWheel(e: WheelEvent) {
    e.preventDefault();
    zoomAt(vp, e.clientX, e.clientY, wrap!.getBoundingClientRect(), e.deltaY);
  }
</script>

<div class="canvas" bind:this={wrap} role="application"
  onpointerdown={(e) => onDown(e, null)} onpointermove={onMove} onpointerup={onUp} onwheel={onWheel}>
  <svg width={w} height={h} aria-label="Graphe des skills">
    <defs>
      <radialGradient id="chatGlow" cx="50%" cy="50%" r="50%">
        <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.35" />
        <stop offset="100%" stop-color="var(--accent)" stop-opacity="0" />
      </radialGradient>
    </defs>
    <g transform={view}>
      {#each links as l (l.k)}
        <path d={l.d} fill="none" class="link {l.kind}" class:dim={dim(l.aId) && dim(l.bId)}
          class:hot={focusId && (l.aId === focusId || l.bId === focusId)} />
      {/each}
      {#each nodes as item (item.n.id)}
        <g class="node" class:dim={dim(item.n.id)} class:focus={item.n.id === focusId}
          transform="translate({item.x},{item.y})" role="button" tabindex="-1"
          onpointerdown={(e) => { e.stopPropagation(); onDown(e, item.n); }}
          onpointerenter={() => onhover(item.n.id)} onpointerleave={() => onhover(null)}>
          {#if item.n.kind === "claude"}
            <circle r={item.n.r * 3} fill="url(#chatGlow)" class="glow" />
          {/if}
          <circle r={item.n.r + 3} class="halo {item.n.kind}" />
          <circle r={item.n.r} fill={nodeColor(item.n)} class="dot {item.n.kind}" />
          <text y={item.n.r + 14} text-anchor="middle" class="lbl">{item.n.label}</text>
        </g>
      {/each}
    </g>
  </svg>
</div>

<style>
  .canvas { width: 100%; height: 100%; overflow: hidden; cursor: grab; touch-action: none; }
  .canvas:active { cursor: grabbing; }
  svg { display: block; }

  .link { stroke: var(--accent); stroke-width: 1.4; stroke-linecap: round; transition: stroke-opacity 0.18s; }
  .link.global { stroke-opacity: 0.13; }
  .link.chat { stroke-opacity: 0.32; stroke-width: 1.6; }
  .link.project { stroke: var(--good, #5bbf7a); stroke-opacity: 0.4; }
  .link.mcp { stroke: var(--prov-mcp, #b58cf0); stroke-opacity: 0.22; }
  .link.plugin { stroke: var(--prov-plugin, #e0a45e); stroke-opacity: 0.22; }
  .link.dim { stroke-opacity: 0.04; }
  .link.hot { stroke-opacity: 0.75; stroke-width: 1.9; }

  /* Animation d'apparition façon Obsidian : fondu + léger pop. */
  @keyframes node-in { from { opacity: 0; } to { opacity: 1; } }
  .node { cursor: pointer; transition: opacity 0.18s; animation: node-in 0.4s ease both; }
  .glow { pointer-events: none; }
  .halo { fill: none; stroke: var(--surface); stroke-width: 4; }
  .halo.claude { stroke: color-mix(in srgb, var(--accent) 30%, var(--surface)); }
  .dot { stroke: var(--bg); stroke-width: 1.5; transition: r 0.15s; }
  .dot.claude { stroke-width: 2.5; }
  .node.focus .dot { stroke: var(--text); }
  .lbl {
    fill: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 10px;
    pointer-events: none;
    paint-order: stroke;
    stroke: var(--bg);
    stroke-width: 3px;
    stroke-linejoin: round;
  }
  .node.focus .lbl { fill: var(--text); }
  .node.dim { opacity: 0.25; }
</style>
