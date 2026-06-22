<script lang="ts">
  import { onMount } from "svelte";
  import { activity } from "$lib/stores/data/activity.svelte";
  import Icon from "../ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { fly } from "svelte/transition";

  let { sid, onclose }: { sid: string; onclose: () => void } = $props();

  const subs = $derived(activity.subList(sid));
  const shells = $derived(activity.shellList(sid));
  let detail = $state(""); // taskId du sous-agent déplié

  // Minuteur live pour les durées écoulées.
  let now = $state(Date.now());
  onMount(() => {
    const t = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(t);
  });
  function elapsed(ts: number): string {
    const s = Math.max(0, Math.floor((now - ts) / 1000));
    if (s < 60) return `${s}s`;
    return `${Math.floor(s / 60)}m${String(s % 60).padStart(2, "0")}`;
  }
</script>

<div class="act-pop" transition:fly={{ y: 6, duration: 130 }}>
  <div class="head">
    <span>Activité · {subs.length + shells.length}</span>
    <button class="x" use:tooltip={"Fermer"} onclick={onclose}><Icon name="close" size={13} /></button>
  </div>

  {#if subs.length === 0 && shells.length === 0}
    <div class="empty">Rien ne tourne pour l'instant.</div>
  {/if}

  {#if subs.length}
    <div class="sec">Sous-agents</div>
    {#each subs as s (s.taskId)}
      <button class="item" onclick={() => (detail = detail === s.taskId ? "" : s.taskId)}>
        <div class="row1">
          <Icon name="cpu" size={13} />
          <span class="nm">{s.description || s.type}</span>
          <span class="t">{elapsed(s.startTs)}</span>
        </div>
        <div class="row2">
          {s.action || "…"}{#if s.lastTool} · {s.lastTool}{/if}{#if s.tokens} · {Math.round(s.tokens / 1000)}k tok{/if}
        </div>
        {#if detail === s.taskId && s.prompt}
          <div class="prompt">{s.prompt}</div>
        {/if}
      </button>
    {/each}
  {/if}

  {#if shells.length}
    <div class="sec">Shells</div>
    {#each shells as sh (sh.id)}
      <div class="item static">
        <div class="row1">
          <Icon name="terminal" size={13} />
          <span class="nm mono">{sh.command}</span>
          <span class="t">{elapsed(sh.startTs)}</span>
        </div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .act-pop {
    position: absolute;
    top: calc(100% + 4px);
    right: 4px;
    width: 300px;
    max-height: 360px;
    overflow-y: auto;
    overscroll-behavior: contain;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 8px 26px rgba(0, 0, 0, 0.3);
    padding: 6px;
    z-index: 40;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 6px 8px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .x {
    color: var(--text-muted);
    display: flex;
  }
  .sec {
    padding: 6px 6px 3px;
    font-size: 10px;
    color: var(--text-faint);
    font-family: var(--font-mono);
    text-transform: uppercase;
  }
  .item {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 6px 7px;
    border-radius: var(--radius-sm);
    text-align: left;
  }
  .item:not(.static):hover {
    background: var(--surface-2);
  }
  .row1 {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
  }
  .nm {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .nm.mono {
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .t {
    flex-shrink: 0;
    font-size: 10.5px;
    color: var(--accent);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .row2 {
    font-size: 10.5px;
    color: var(--text-muted);
    padding-left: 19px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .prompt {
    margin: 4px 0 2px 19px;
    padding: 6px 8px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: pre-wrap;
    max-height: 120px;
    overflow-y: auto;
  }
  .empty {
    padding: 14px;
    text-align: center;
    font-size: 11.5px;
    color: var(--text-faint);
  }
</style>
