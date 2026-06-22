<script lang="ts">
  import { usage } from "$lib/stores/data/usage.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { fly } from "svelte/transition";
  import Icon from "./Icon.svelte";
  import type { UsageSnapshot } from "$lib/ipc";

  interface Props {
    snapshot?: UsageSnapshot | null;
    label?: string;
    singleBar?: boolean;
    /** Si true : une seule barre visible + dropdown pour choisir l'autre. */
    collapse?: boolean;
  }
  let { snapshot = null, label = "Claude", singleBar = false, collapse = false }: Props = $props();

  const snap = $derived(snapshot ?? usage.snapshot);

  let expanded = $state(false);
  let active = $state<"5h" | "7j">("5h");
  let triggerEl = $state<HTMLButtonElement>();
  let popupEl = $state<HTMLDivElement>();
  let popupX = $state(0);
  let popupY = $state(0);

  function fmt(n: number): string {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + "M";
    if (n >= 1_000) return (n / 1_000).toFixed(0) + "k";
    return String(n);
  }
  function tone(pct: number): string {
    if (pct >= 90) return "var(--danger)";
    if (pct >= 70) return "var(--warn)";
    return "var(--accent)";
  }
  function resetIn(epoch: number | null): string {
    if (!epoch) return "";
    const s = epoch - Math.floor(Date.now() / 1000);
    if (s <= 0) return "réinit imminente";
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    return h > 0 ? `réinit dans ${h}h${String(m).padStart(2, "0")}` : `réinit dans ${m}min`;
  }
  function tip(lbl: string, b: { pct: number; tokens: number; cap: number; resets_at: number | null }, cost: number, real: boolean): string {
    if (real) return `${lbl} : ${b.pct}% utilisé · ${resetIn(b.resets_at)} · $${cost.toFixed(2)} via l'app`;
    if (b.cap === 0) return `${lbl} : ${fmt(b.tokens)} tokens aujourd'hui · $${cost.toFixed(2)}`;
    return `${lbl} : ${fmt(b.tokens)}/${fmt(b.cap)} tokens (estimé) · $${cost.toFixed(2)}`;
  }

  function openExpand() {
    if (!triggerEl) return;
    const r = triggerEl.getBoundingClientRect();
    popupX = r.left;
    popupY = r.bottom + 4;
    expanded = !expanded;
  }

  function pickBar(bar: "5h" | "7j") {
    active = bar;
    expanded = false;
  }

  $effect(() => {
    if (!expanded) return;
    function onOutside(e: MouseEvent) {
      const t = e.target as Node;
      const insideTrigger = triggerEl?.contains(t);
      const insidePopup = popupEl?.contains(t);
      if (!insideTrigger && !insidePopup) expanded = false;
    }
    document.addEventListener("mousedown", onOutside);
    return () => document.removeEventListener("mousedown", onOutside);
  });
</script>

<div class="usage" data-tauri-drag-region>
  {#if snap}
    {@const s = snap}
    {@const real = s.source === "real"}
    {@const iconName = label === "Claude" ? "logo-claude" : label === "Gemini" ? "logo-gemini" : label === "opencode" ? "logo-opencode" : null}
    {#if iconName}
      <span class="provider-icon" data-tauri-drag-region><Icon name={iconName} size={13} stroke={1.8} /></span>
    {/if}

    {#if singleBar}
      <!-- Providers tiers : barre 24h simple -->
      <div class="row" data-tauri-drag-region use:tooltip={tip("24h", s.five_h, s.five_h_cost, real)}>
        <span class="lbl" data-tauri-drag-region>24h</span>
        <div class="track" data-tauri-drag-region>
          <div class="fill" data-tauri-drag-region style={`width:${s.five_h.pct}%;background:${tone(s.five_h.pct)}`}></div>
        </div>
        <span class="pct" data-tauri-drag-region>
          {#if s.five_h.cap === 0}{fmt(s.five_h.tokens)}{:else}{s.five_h.pct}%{/if}
        </span>
      </div>

    {:else if collapse}
      <!-- Claude en mode compact : 1 barre + dropdown -->
      {@const b = active === "5h" ? s.five_h : s.week}
      {@const cost = active === "5h" ? s.five_h_cost : s.week_cost}
      <button
        type="button"
        class="row row-btn"
        bind:this={triggerEl}
        onclick={openExpand}
        use:tooltip={tip(active, b, cost, real)}
      >
        <span class="lbl">{active}</span>
        <div class="track">
          <div class="fill" style={`width:${b.pct}%;background:${tone(b.pct)}`}></div>
        </div>
        <span class="pct">{b.pct}%</span>
        <span class="chev" class:open={expanded}>▾</span>
      </button>

    {:else}
      <!-- Claude normal : 2 barres côte à côte -->
      <div class="row" data-tauri-drag-region use:tooltip={tip("5h", s.five_h, s.five_h_cost, real)}>
        <span class="lbl" data-tauri-drag-region>5h</span>
        <div class="track" data-tauri-drag-region>
          <div class="fill" data-tauri-drag-region style={`width:${s.five_h.pct}%;background:${tone(s.five_h.pct)}`}></div>
        </div>
        <span class="pct" data-tauri-drag-region>{s.five_h.pct}%</span>
      </div>
      <div class="row" data-tauri-drag-region use:tooltip={tip("7j", s.week, s.week_cost, real)}>
        <span class="lbl" data-tauri-drag-region>7j</span>
        <div class="track" data-tauri-drag-region>
          <div class="fill" data-tauri-drag-region style={`width:${s.week.pct}%;background:${tone(s.week.pct)}`}></div>
        </div>
        <span class="pct" data-tauri-drag-region>{s.week.pct}%</span>
      </div>
      {#if !real}
        <span class="src" data-tauri-drag-region use:tooltip={"Estimation locale — pas de donnée réelle dispo."}>estimé</span>
      {/if}
    {/if}
  {/if}
</div>

<!-- Dropdown fixed — échappe overflow parents -->
{#if expanded && snap}
  {@const s = snap}
  {@const real = s.source === "real"}
  <div
    class="popup"
    style={`left:${popupX}px;top:${popupY}px`}
    bind:this={popupEl}
    role="menu"
    in:fly={{ y: -4, duration: 130 }}
    out:fly={{ y: -4, duration: 100 }}
  >
    <button
      type="button"
      class="pop-row"
      class:pop-active={active === "5h"}
      onclick={() => pickBar("5h")}
      use:tooltip={tip("5h", s.five_h, s.five_h_cost, real)}
    >
      <span class="lbl">5h</span>
      <div class="track">
        <div class="fill" style={`width:${s.five_h.pct}%;background:${tone(s.five_h.pct)}`}></div>
      </div>
      <span class="pct">{s.five_h.pct}%</span>
    </button>
    <button
      type="button"
      class="pop-row"
      class:pop-active={active === "7j"}
      onclick={() => pickBar("7j")}
      use:tooltip={tip("7j", s.week, s.week_cost, real)}
    >
      <span class="lbl">7j</span>
      <div class="track">
        <div class="fill" style={`width:${s.week.pct}%;background:${tone(s.week.pct)}`}></div>
      </div>
      <span class="pct">{s.week.pct}%</span>
    </button>
    {#if !real}
      <span class="src" use:tooltip={"Estimation locale — pas de donnée réelle dispo."}>estimé</span>
    {/if}
  </div>
{/if}

<style>
  .usage {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .provider-icon {
    display: flex;
    align-items: center;
    color: var(--text-muted);
    opacity: 0.75;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .row-btn {
    background: none;
    border: none;
    padding: 2px 4px;
    cursor: pointer;
    border-radius: 4px;
    transition: background var(--transition);
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .row-btn:hover { background: var(--surface-2); }
  .lbl {
    font-size: 11px;
    color: var(--text-muted);
    width: 16px;
    font-variant-numeric: tabular-nums;
  }
  .track {
    width: 84px;
    height: 6px;
    background: var(--track);
    border-radius: 4px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 4px;
    transition: width var(--transition);
  }
  .pct {
    font-size: 11px;
    color: var(--text-muted);
    width: 30px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .chev {
    font-size: 10px;
    color: var(--text-faint);
    transition: transform 130ms ease;
  }
  .chev.open { transform: rotate(180deg); }
  .src {
    font-size: 10px;
    color: var(--text-faint);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 5px;
    cursor: help;
    width: fit-content;
  }
  /* Dropdown fixed */
  .popup {
    position: fixed;
    z-index: 9999;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 14px rgba(0,0,0,0.25);
    padding: 4px 5px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 155px;
  }
  .pop-row {
    display: flex;
    align-items: center;
    gap: 7px;
    background: none;
    border: none;
    padding: 4px 6px;
    border-radius: 4px;
    cursor: pointer;
    transition: background var(--transition);
    width: 100%;
  }
  .pop-row:hover { background: var(--surface-2); }
  .pop-active { background: var(--accent-weak); }
</style>
