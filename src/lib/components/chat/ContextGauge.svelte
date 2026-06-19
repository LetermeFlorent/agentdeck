<script lang="ts">
  import { sessions } from "$lib/stores/sessions.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { priceOf, fmtFull } from "./chat-config";
  import { fly } from "svelte/transition";

  let { sid }: { sid: string } = $props();

  const session = $derived(sessions.map[sid]);
  const price = $derived(priceOf(session?.model));

  // Jauge en carré aux bords arrondis (rounded square) qui se remplit.
  const SIDE = 17; // côté du carré dans le viewBox 24
  const OFF = (24 - SIDE) / 2;
  const RAD = 5; // rayon des coins
  const PERIM = 4 * (SIDE - 2 * RAD) + 2 * Math.PI * RAD; // périmètre du carré arrondi
  const ctxWindow = $derived(session?.contextWindow ?? 0);
  const ctxKnown = $derived(ctxWindow > 0);
  const ctxUsed = $derived(session?.contextTokens ?? 0);
  const ctxPct = $derived(ctxKnown ? Math.min(100, Math.round((ctxUsed / ctxWindow) * 100)) : 0);
  const ctxFree = $derived(Math.max(0, ctxWindow - ctxUsed));
  const ctxDash = $derived((PERIM * ctxPct) / 100);
  const ctxTone = $derived(
    ctxPct >= 90 ? "var(--danger)" : ctxPct >= 70 ? "var(--warn)" : "var(--good)",
  );

  let ctxOpen = $state(false);
  let ctxRoot = $state<HTMLDivElement>();
  // Ferme le panneau au clic extérieur / Échap.
  $effect(() => {
    if (!ctxOpen) return;
    const onDown = (e: PointerEvent) => {
      if (ctxRoot && !ctxRoot.contains(e.target as Node)) ctxOpen = false;
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") ctxOpen = false;
    };
    window.addEventListener("pointerdown", onDown, true);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onDown, true);
      window.removeEventListener("keydown", onKey);
    };
  });
</script>

<div class="ctx" bind:this={ctxRoot}>
  <button
    type="button"
    class="ctx-gauge"
    class:open={ctxOpen}
    aria-label="Contexte utilisé"
    use:tooltip={ctxKnown
      ? `Contexte : ${ctxPct}% utilisé — clic pour le détail`
      : "Contexte — clic pour le détail (mesuré au 1er tour)"}
    onclick={() => (ctxOpen = !ctxOpen)}
  >
    <svg viewBox="0 0 24 24" width="24" height="24">
      <rect
        class="ctx-track"
        x={OFF}
        y={OFF}
        width={SIDE}
        height={SIDE}
        rx={RAD}
        fill="none"
        stroke-width="3"
      />
      <rect
        x={OFF}
        y={OFF}
        width={SIDE}
        height={SIDE}
        rx={RAD}
        fill="none"
        stroke={ctxTone}
        stroke-width="3"
        stroke-linecap="round"
        stroke-dasharray={`${PERIM} ${PERIM}`}
        stroke-dashoffset={PERIM - ctxDash}
      />
    </svg>
  </button>

  {#if ctxOpen}
    <div class="ctx-pop" transition:fly={{ y: 6, duration: 140 }}>
      <div class="ctx-head">
        <span class="ctx-title">Fenêtre de contexte</span>
        <span class="ctx-big" style={`color:${ctxTone}`}>{ctxKnown ? `${ctxPct}%` : "—"}</span>
      </div>
      <div class="ctx-bar">
        <div class="ctx-bar-fill" style={`width:${ctxPct}%;background:${ctxTone}`}></div>
      </div>
      <ul class="ctx-rows">
        <li><span>Utilisé</span><b>{fmtFull(ctxUsed)}</b></li>
        <li><span>Libre</span><b>{fmtFull(ctxFree)}</b></li>
        <li class="muted"><span>Fenêtre</span><b>{ctxKnown ? fmtFull(ctxWindow) : "—"}</b></li>
      </ul>
      <div class="ctx-sep"></div>
      <ul class="ctx-rows">
        <li><span>Coût du chat</span><b>${(session?.costUsd ?? 0).toFixed(3)}</b></li>
        <li><span>Tokens générés</span><b>{fmtFull(session?.totalTokens ?? 0)}</b></li>
        {#if session?.model}<li class="muted"><span>Modèle</span><b>{session.model}</b></li>{/if}
        {#if price}<li class="muted"><span>Tarif /M</span><b>↑${price[0]} ↓${price[1]}</b></li>{/if}
      </ul>
      {#if !ctxKnown}
        <p class="ctx-note">Envoie un message — la fenêtre réelle du modèle est mesurée au 1er tour.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .ctx {
    position: relative;
    flex-shrink: 0;
    display: flex;
  }
  .ctx-gauge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    aspect-ratio: 1;
    padding: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    outline: none;
    transition: background var(--transition);
  }
  .ctx-gauge:hover,
  .ctx-gauge.open {
    background: var(--accent-weak);
  }
  /* Piste de fond bien visible (anneau net) sinon le cercle « disparaît » à 0%. */
  .ctx-track {
    stroke: color-mix(in srgb, var(--accent) 32%, var(--border-strong));
  }
  .ctx-gauge svg rect {
    transition: stroke-dashoffset var(--transition), stroke var(--transition);
  }
  .ctx-pop {
    position: absolute;
    right: 0;
    bottom: calc(100% + 6px);
    width: 218px;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.24);
    padding: 11px 12px;
    z-index: 40;
  }
  .ctx-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .ctx-title {
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text);
  }
  .ctx-big {
    font-family: var(--font-mono);
    font-size: 16px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .ctx-bar {
    height: 6px;
    background: var(--track);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 10px;
  }
  .ctx-bar-fill {
    height: 100%;
    border-radius: 4px;
    transition: width var(--transition), background var(--transition);
  }
  .ctx-rows {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .ctx-rows li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .ctx-rows li span {
    color: var(--text-muted);
  }
  .ctx-rows li b {
    color: var(--text);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .ctx-rows li.muted b {
    color: var(--text-faint);
    font-weight: 500;
  }
  .ctx-sep {
    height: 1px;
    background: var(--border);
    margin: 9px 0;
  }
  .ctx-note {
    margin: 9px 0 0;
    font-size: 10.5px;
    color: var(--text-faint);
    line-height: 1.4;
  }
</style>
