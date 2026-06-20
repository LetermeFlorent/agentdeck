<script lang="ts">
  // Tour guidé du 1er lancement : spotlight sur un élément + bulle explicative, séquentiel.
  // Cible les éléments via leur attribut data-tour. Une étape sans cible est sautée.
  import { tour } from "$lib/stores/tour.svelte";
  import { fade } from "svelte/transition";

  interface Step {
    sel: string;
    title: string;
    text: string;
  }
  const ALL: Step[] = [
    { sel: '[data-tour="usage"]', title: "Ta consommation", text: "Tes limites Claude sur 5 h et 7 jours, en un coup d'œil." },
    { sel: '[data-tour="tabs"]', title: "Espaces de travail", text: "Crée des onglets pour séparer tes contextes (un projet = un onglet)." },
    { sel: '[data-tour="split"]', title: "Plusieurs Claude à la fois", text: "Divise un pane en deux : fais bosser plusieurs sessions côte à côte." },
    { sel: '[data-tour="history"]', title: "Historique", text: "Retrouve et rouvre n'importe quelle conversation passée." },
    { sel: '[data-tour="settings"]', title: "Réglages", text: "Modèle par défaut, skills, serveurs MCP, veille des chats, thème…" },
  ];

  let steps = $state<Step[]>([]);
  let idx = $state(0);
  let rect = $state<{ x: number; y: number; w: number; h: number } | null>(null);

  // Place la bulle sous la cible si elle est en haut de l'écran, sinon au-dessus.
  const below = $derived(rect ? rect.y + rect.h < window.innerHeight / 2 : true);

  function measure() {
    const s = steps[idx];
    if (!s) return;
    const el = document.querySelector(s.sel) as HTMLElement | null;
    if (!el) {
      rect = null;
      return;
    }
    const r = el.getBoundingClientRect();
    rect = { x: r.left, y: r.top, w: r.width, h: r.height };
  }

  $effect(() => {
    void idx; // recalcule à chaque changement d'étape
    requestAnimationFrame(measure);
  });

  $effect(() => {
    // Ne garde que les étapes dont la cible existe au lancement du tour.
    steps = ALL.filter((s) => document.querySelector(s.sel));
    idx = 0;
    requestAnimationFrame(measure);
    const onResize = () => measure();
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  });

  function next() {
    if (idx < steps.length - 1) idx += 1;
    else tour.done();
  }
  function prev() {
    if (idx > 0) idx -= 1;
  }
  function skip() {
    tour.done();
  }

  const step = $derived(steps[idx]);
  // Position de la bulle (clampée à l'écran).
  const bubble = $derived.by(() => {
    if (!rect) return { left: window.innerWidth / 2 - 150, top: window.innerHeight / 2 - 60 };
    const W = 300;
    let left = rect.x + rect.w / 2 - W / 2;
    left = Math.max(12, Math.min(left, window.innerWidth - W - 12));
    const top = below ? rect.y + rect.h + 12 : rect.y - 12;
    return { left, top };
  });
</script>

{#if step}
  <div class="tour" transition:fade={{ duration: 150 }}>
    {#if rect}
      <div
        class="spot"
        style={`left:${rect.x - 6}px;top:${rect.y - 6}px;width:${rect.w + 12}px;height:${rect.h + 12}px`}
      ></div>
    {:else}
      <div class="dim"></div>
    {/if}

    <div
      class="bub"
      class:above={!below}
      style={`left:${bubble.left}px;top:${bubble.top}px;${below ? "" : "transform:translateY(-100%);"}`}
    >
      <div class="b-head">
        <span class="b-title">{step.title}</span>
        <span class="b-count">{idx + 1}/{steps.length}</span>
      </div>
      <p class="b-text">{step.text}</p>
      <div class="b-actions">
        <button class="b-skip" onclick={skip}>Passer</button>
        <div class="b-nav">
          {#if idx > 0}<button class="b-btn ghost" onclick={prev}>Précédent</button>{/if}
          <button class="b-btn" onclick={next}>{idx < steps.length - 1 ? "Suivant" : "Terminer"}</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .tour {
    position: fixed;
    inset: 0;
    z-index: 300;
  }
  /* Spotlight : trou clair sur la cible, le reste assombri via l'ombre portée géante. */
  .spot {
    position: absolute;
    border-radius: 8px;
    box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.62);
    outline: 2px solid var(--accent);
    outline-offset: 1px;
    transition: left 0.18s ease, top 0.18s ease, width 0.18s ease, height 0.18s ease;
    pointer-events: none;
  }
  .dim {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.62);
  }
  .bub {
    position: absolute;
    width: 300px;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.45);
    padding: 12px 14px;
  }
  .b-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 5px;
  }
  .b-title {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 13px;
    color: var(--text);
  }
  .b-count {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-faint);
  }
  .b-text {
    margin: 0 0 12px;
    font-size: 12.5px;
    line-height: 1.45;
    color: var(--text-muted);
  }
  .b-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .b-nav {
    display: flex;
    gap: 6px;
  }
  .b-skip {
    font-size: 11.5px;
    color: var(--text-faint);
  }
  .b-skip:hover {
    color: var(--text-muted);
  }
  .b-btn {
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    font-size: 12px;
  }
  .b-btn.ghost {
    background: var(--surface-2);
    color: var(--text-muted);
    border: 1px solid var(--border);
  }
  .b-btn:hover {
    background: var(--accent-hover);
  }
  .b-btn.ghost:hover {
    color: var(--text);
  }
</style>
