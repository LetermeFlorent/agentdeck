<script lang="ts">
  // Écran de démarrage : mascotte agentdeck en PIXEL ART (grille de pixels SVG) + lignes de boot.
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

  // Grille 12x12. Légende : A=antenne, B=corps, E=œil droit, L=œil gauche (clin), M=bouche,
  // D=carte du "deck". '.' = transparent.
  const GRID = [
    ".....AA.....",
    ".....AA.....",
    "..BBBBBBBB..",
    ".BBBBBBBBBB.",
    "BBBBBBBBBBBB",
    "BB.LL..EE.BB",
    "BB.LL..EE.BB",
    "BBBBBBBBBBBB",
    "BB.MMMMMM.BB",
    ".BBBBBBBBBB.",
    "..BBBBBBBB..",
    ".DD.DD.DD...",
  ];
  const COLOR: Record<string, string> = {
    A: "var(--accent)",
    B: "var(--accent)",
    E: "#fff",
    L: "#fff",
    M: "#fff",
    D: "var(--accent)",
  };
  const cells = GRID.flatMap((row, y) =>
    [...row].flatMap((ch, x) =>
      ch === "." ? [] : [{ x, y, fill: COLOR[ch], cls: ch === "L" ? "eye-l" : ch === "A" ? "bulb" : "", op: ch === "D" ? 0.5 : 1 }],
    ),
  );

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
    <!-- Mascotte agentdeck en pixel art. -->
    <svg
      class="mascot"
      viewBox="0 0 12 12"
      width="108"
      height="108"
      shape-rendering="crispEdges"
      aria-hidden="true"
    >
      {#each cells as c}
        <rect
          class={c.cls}
          x={c.x}
          y={c.y}
          width="1"
          height="1"
          fill={c.fill}
          opacity={c.op}
        />
      {/each}
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
  .mascot {
    image-rendering: pixelated;
    animation: float 3s steps(3) infinite;
    filter: drop-shadow(0 6px 0 rgba(0, 0, 0, 0.18));
  }
  /* Flottement par pas (rendu "jeu rétro"). */
  @keyframes float {
    50% {
      transform: translateY(-6px);
    }
  }
  /* Clin d'œil : les pixels de l'œil gauche se replient brièvement. */
  .eye-l {
    transform-box: fill-box;
    transform-origin: center;
    animation: wink 3.2s steps(1) infinite;
  }
  @keyframes wink {
    0%,
    88%,
    100% {
      transform: scaleY(1);
    }
    92%,
    96% {
      transform: scaleY(0.05);
    }
  }
  /* Antenne qui clignote. */
  .bulb {
    animation: twinkle 1.4s steps(2) infinite;
  }
  @keyframes twinkle {
    0%,
    100% {
      opacity: 0.45;
    }
    50% {
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
