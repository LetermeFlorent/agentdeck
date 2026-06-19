<script lang="ts">
  // Écran de démarrage : vrai logo Claude (burst/étincelle) animé + lignes de boot, à plat.
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

  // Logo Claude : rayons radiaux de longueurs irrégulières (aspect "étincelle" organique).
  const CX = 50;
  const CY = 50;
  const INNER = 7;
  const RAY_LENS = [40, 31, 38, 30, 39, 32, 40, 30, 37, 33, 39, 31];
  const rays = RAY_LENS.map((len, i) => {
    const a = (i * 360) / RAY_LENS.length;
    const rad = (a * Math.PI) / 180;
    const cos = Math.cos(rad);
    const sin = Math.sin(rad);
    return {
      x1: CX + INNER * cos,
      y1: CY + INNER * sin,
      x2: CX + (INNER + len) * cos,
      y2: CY + (INNER + len) * sin,
    };
  });

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
    <!-- Vrai logo Claude : étincelle radiale, couleur Claude orange. -->
    <svg class="logo" viewBox="0 0 100 100" width="92" height="92" aria-hidden="true">
      <g class="burst">
        {#each rays as r}
          <line
            x1={r.x1}
            y1={r.y1}
            x2={r.x2}
            y2={r.y2}
            stroke="#d97757"
            stroke-width="7"
            stroke-linecap="round"
          />
        {/each}
      </g>
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
  .logo {
    animation: pulse 2.4s ease-in-out infinite;
    filter: drop-shadow(0 8px 22px rgba(217, 119, 87, 0.35));
  }
  /* Rotation lente + battement doux de l'étincelle. */
  .burst {
    transform-origin: 50px 50px;
    animation: spin 16s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  @keyframes pulse {
    0%,
    100% {
      transform: scale(0.94);
      opacity: 0.9;
    }
    50% {
      transform: scale(1);
      opacity: 1;
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
