<script lang="ts">
  // Loader réutilisable : le spark de la mascotte qui tourne + pulse, avec libellé optionnel.
  // Remplace les « Chargement… » / « … » disséminés dans l'app.
  import SparkMark from "./SparkMark.svelte";

  let {
    size = 18,
    label = "",
    inline = false,
  }: { size?: number; label?: string; inline?: boolean } = $props();
</script>

<div class="loader" class:inline aria-live="polite" aria-busy="true">
  <span class="spin" style={`width:${size}px;height:${size}px`}><SparkMark {size} /></span>
  {#if label}<span class="lbl">{label}</span>{/if}
</div>

<style>
  .loader {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 14px;
    color: var(--text-muted);
  }
  .loader.inline {
    flex-direction: row;
    gap: 7px;
    padding: 0;
  }
  .spin {
    display: inline-flex;
    animation: ldrSpin 1.5s linear infinite, ldrPulse 1.1s ease-in-out infinite;
    transform-origin: center;
  }
  .lbl {
    font-family: var(--font-mono);
    font-size: 11.5px;
    letter-spacing: 0.03em;
  }
  @keyframes ldrSpin {
    to { transform: rotate(360deg); }
  }
  @keyframes ldrPulse {
    0%, 100% { opacity: 0.55; }
    50% { opacity: 1; }
  }
  @media (prefers-reduced-motion: reduce) {
    .spin { animation: ldrPulse 1.4s ease-in-out infinite; }
  }
</style>
