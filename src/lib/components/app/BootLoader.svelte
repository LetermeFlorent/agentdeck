<script lang="ts">
  // Écran de démarrage : accueil « Bonjour {user} » + console de boot façon Claude Code.
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
  <div class="term" in:fly={{ y: 10, duration: 240 }}>
    <div class="bar">
      <span class="d r"></span><span class="d y"></span><span class="d g"></span>
      <span class="bar-title">agentdeck</span>
    </div>
    <div class="body">
      <p class="hello">
        <span class="prompt">$</span> bonjour {username || "à toi"}
        <span class="caret"></span>
      </p>
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
  .term {
    width: 100%;
    max-width: 380px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 7px 10px;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
  }
  .d {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    display: inline-block;
  }
  .d.r {
    background: #e06c6c;
  }
  .d.y {
    background: #d8b24a;
  }
  .d.g {
    background: var(--good);
  }
  .bar-title {
    margin-left: 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-faint);
  }
  .body {
    padding: 16px 16px 18px;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.9;
  }
  .hello {
    margin: 0 0 6px;
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
  .line {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-muted);
    font-size: 12.5px;
  }
  .spin {
    color: var(--accent);
  }
  .ok {
    color: var(--good);
  }
</style>
