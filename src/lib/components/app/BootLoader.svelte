<script lang="ts">
  // Écran de démarrage : spark Claude propre (12 branches effilées, généré) + lignes de boot.
  import { fly } from "svelte/transition";

  let { username = "" }: { username?: string } = $props();

  const FRAMES = ["✶", "✸", "✹", "✺", "✹", "✷"];
  const STEPS = [
    "connexion à Claude Code",
    "restauration des sessions",
    "préparation du deck",
  ];
  let frame = $state(0);
  let step = $state(0);

  // Spark généré : 12 branches effilées + cœur. Symétrique et net (pas une pixelisation floue).
  const N = 28;
  const c = N / 2;
  const SPOKES = 12;
  const R = 13;
  const CORE = 2.4;
  const W0 = 0.42; // demi-largeur angulaire à la base
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
      const halfw = W0 * (1 - r / R);
      if (r <= CORE || d <= halfw) {
        cells.push({ x, y, fill: r < CORE * 1.25 ? COREC : BASE });
      }
    }
  }

  $effect(() => {
    const iv = setInterval(() => (frame = (frame + 1) % FRAMES.length), 120);
    const sv = setInterval(() => (step = Math.min(STEPS.length - 1, step + 1)), 650);
    return () => {
      clearInterval(iv);
      clearInterval(sv);
    };
  });
</script>

<div class="boot-screen" data-tauri-drag-region>
  <div class="stage" in:fly={{ y: 10, duration: 240 }}>
    <svg
      class="mascot"
      viewBox="0 0 {N} {N}"
      width="92"
      height="92"
      shape-rendering="crispEdges"
      aria-hidden="true"
    >
      <g class="halo">
        <g class="halo-in">
          {#each cells as p}
            <rect x={p.x} y={p.y} width="1" height="1" fill={p.fill} />
          {/each}
        </g>
      </g>
      <!-- pulsation lumineuse au centre -->
      <circle class="pulse" cx={c} cy={c} r="3.4" fill="#fff" />
    </svg>

    <p class="hello">
      <span class="prompt">$</span> bonjour {username || "à toi"}
      <span class="caret"></span>
    </p>

    <div class="steps">
      {#each STEPS as s, i}
        {#if i <= step}
          <p class="line" in:fly={{ x: -6, duration: 160 }}>
            {#if i < step}
              <span class="ok">✓</span>
            {:else}
              <span class="spin">{FRAMES[frame]}</span>
            {/if}
            <span class="txt">{s}{i < step ? "" : "…"}</span>
          </p>
        {/if}
      {/each}
    </div>
  </div>
</div>

<style>
  .boot-screen {
    height: 100vh;
    display: grid;
    place-items: center;
    background: var(--bg);
    padding: 24px;
  }
  .stage {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    font-family: var(--font-mono);
  }
  /* Spark statique. Seuls le glow et le cœur s'animent. */
  .mascot {
    image-rendering: pixelated;
    animation: glow 2.6s ease-in-out infinite;
  }
  @keyframes glow {
    0%,
    100% {
      filter: drop-shadow(0 6px 10px rgba(156, 74, 48, 0.3));
    }
    50% {
      filter: drop-shadow(0 6px 18px rgba(217, 119, 87, 0.65));
    }
  }
  /* Pulsation du cœur. */
  .pulse {
    transform-box: fill-box;
    transform-origin: center;
    animation: corepulse 2.6s ease-in-out infinite;
  }
  @keyframes corepulse {
    0%,
    100% {
      transform: scale(0.5);
      opacity: 0.15;
    }
    50% {
      transform: scale(1.15);
      opacity: 0.6;
    }
  }
  .hello {
    margin: 0;
    color: var(--text);
    font-size: 14.5px;
    font-weight: 600;
  }
  .prompt {
    color: var(--accent);
    margin-right: 4px;
  }
  .caret {
    display: inline-block;
    width: 8px;
    height: 15px;
    margin-left: 2px;
    background: var(--accent);
    vertical-align: text-bottom;
    animation: blink 1s steps(1) infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0;
    }
  }
  .steps {
    display: flex;
    flex-direction: column;
    gap: 2px;
    align-items: flex-start;
  }
  .line {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-muted);
    font-size: 12.5px;
    line-height: 1.9;
  }
  .spin {
    color: var(--accent);
  }
  .ok {
    color: var(--good);
  }
</style>
