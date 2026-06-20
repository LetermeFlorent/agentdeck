<script lang="ts">
  // Tour guidé du 1er lancement : spotlight sur un élément + bulle explicative, séquentiel.
  // Cible les éléments via leur attribut data-tour. Une étape sans cible est sautée.
  // La bulle et le spotlight restent toujours dans la fenêtre (clamp).
  import { tour } from "$lib/stores/tour.svelte";
  import { fade } from "svelte/transition";

  interface Step {
    sel: string;
    title: string;
    text: string;
  }
  const ALL: Step[] = [
    { sel: '[data-tour="brand"]', title: "Bienvenue sur agentdeck", text: "Pilote plusieurs sessions Claude Code en parallèle. Petit tour des fonctions clés." },
    { sel: '[data-tour="usage"]', title: "Ta consommation", text: "Tes limites Claude sur 5 h et 7 jours, en un coup d'œil." },
    { sel: '[data-tour="tabs"]', title: "Espaces de travail", text: "Crée des onglets pour séparer tes contextes (un projet = un onglet)." },
    { sel: '[data-tour="history"]', title: "Historique", text: "Retrouve et rouvre n'importe quelle conversation passée." },
    { sel: '[data-tour="settings"]', title: "Réglages", text: "Modèle par défaut, skills, serveurs MCP, veille des chats, thème…" },
    { sel: '[data-tour="cwd"]', title: "Dossier de travail", text: "Le dossier où ce chat agit. Clique pour le changer (ton projet)." },
    { sel: '[data-tour="split"]', title: "Plusieurs Claude à la fois", text: "Divise un pane en deux : fais bosser plusieurs sessions côte à côte." },
    { sel: '[data-tour="model"]', title: "Modèle", text: "Choisis le modèle de ce chat (Opus, Sonnet, Haiku…)." },
    { sel: '[data-tour="effort"]', title: "Effort", text: "Règle l'effort de réflexion. Plus haut = plus fouillé (et plus coûteux)." },
    { sel: '[data-tour="perms"]', title: "Permissions", text: "Mode de permission + outils autorisés/refusés de l'agent." },
    { sel: '[data-tour="slash"]', title: "Commandes /", text: "Accède aux commandes slash de Claude Code." },
    { sel: '[data-tour="priv"]', title: "Mode privé", text: "Floute le contenu d'un chat (par-dessus l'épaule). Auto possible dans les réglages." },
    { sel: '[data-tour="sleep"]', title: "Veille du chat", text: "Met le chat en veille : libère la RAM. Un clic sur le chat le réveille." },
    { sel: '[data-tour="composer"]', title: "Écris à Claude", text: "Tape ici. Entrée pour envoyer, Maj+Entrée pour un retour à la ligne." },
  ];

  let steps = $state<Step[]>([]);
  let idx = $state(0);
  let rect = $state<{ x: number; y: number; w: number; h: number } | null>(null);
  let pos = $state({ left: 0, top: 0 });
  let placed = $state(false); // évite le flash de la bulle au coin (0,0) avant calcul
  let bubEl = $state<HTMLDivElement | null>(null);

  const M = 16; // marge écran
  const step = $derived(steps[idx]);

  // Spotlight clampé dans la fenêtre (jamais hors écran).
  const spot = $derived.by(() => {
    if (!rect) return null;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const x = Math.max(2, rect.x - 6);
    const y = Math.max(2, rect.y - 6);
    return {
      x,
      y,
      w: Math.min(rect.w + 12, vw - x - 2),
      h: Math.min(rect.h + 12, vh - y - 2),
    };
  });

  function clamp(v: number, min: number, max: number) {
    return Math.max(min, Math.min(v, max));
  }

  function measure() {
    const s = steps[idx];
    const el = s ? (document.querySelector(s.sel) as HTMLElement | null) : null;
    if (!el) {
      rect = null;
    } else {
      const r = el.getBoundingClientRect();
      rect = {
        x: clamp(r.left, 0, window.innerWidth),
        y: clamp(r.top, 0, window.innerHeight),
        w: Math.min(r.width, window.innerWidth),
        h: Math.min(r.height, window.innerHeight),
      };
    }
    // Place la bulle après rendu (taille réelle connue), clampée dans la fenêtre.
    requestAnimationFrame(place);
  }

  function place() {
    const bw = bubEl?.offsetWidth ?? 300;
    const bh = bubEl?.offsetHeight ?? 140;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    // Bulle centrée horizontalement, en bande haute ou basse — toujours dans la fenêtre,
    // et placée à l'opposé de l'élément surligné pour ne pas le cacher.
    const left = clamp((vw - bw) / 2, M, vw - bw - M);
    let top: number;
    if (!rect) {
      top = (vh - bh) / 2; // étape sans cible : centré
    } else {
      const elemTopHalf = rect.y + rect.h / 2 < vh / 2;
      top = elemTopHalf ? vh - bh - 24 : 24; // élément en haut → bulle en bas, et inversement
    }
    pos = { left, top: clamp(top, M, vh - bh - M) };
    placed = true;
  }

  $effect(() => {
    void idx;
    requestAnimationFrame(measure);
  });

  $effect(() => {
    steps = ALL.filter((s) => document.querySelector(s.sel));
    idx = 0;
    requestAnimationFrame(measure);
    const onResize = () => measure();
    window.addEventListener("resize", onResize);
    // Replace la bulle dès que sa taille réelle est connue / change (contenu variable).
    const ro = bubEl ? new ResizeObserver(() => place()) : null;
    if (bubEl && ro) ro.observe(bubEl);
    return () => {
      window.removeEventListener("resize", onResize);
      ro?.disconnect();
    };
  });

  function next() {
    if (idx < steps.length - 1) idx += 1;
    else tour.done();
  }
  function prev() {
    if (idx > 0) idx -= 1;
  }
</script>

{#if step}
  <div class="tour" transition:fade={{ duration: 150 }}>
    {#if spot}
      <div
        class="spot"
        style={`left:${spot.x}px;top:${spot.y}px;width:${spot.w}px;height:${spot.h}px`}
      ></div>
    {:else}
      <div class="dim"></div>
    {/if}

    <div class="bub" bind:this={bubEl} style={`left:${pos.left}px;top:${pos.top}px;visibility:${placed ? "visible" : "hidden"}`}>
      <div class="b-head">
        <span class="b-title">{step.title}</span>
        <span class="b-count">{idx + 1}/{steps.length}</span>
      </div>
      <p class="b-text">{step.text}</p>
      <div class="b-actions">
        <button class="b-skip" onclick={() => tour.done()}>Passer</button>
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
    width: min(300px, calc(100vw - 32px));
    max-height: calc(100vh - 32px);
    overflow-y: auto;
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
