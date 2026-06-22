<script lang="ts">
  import type { SlashCmd } from "$lib/ipc";
  import { fly } from "svelte/transition";

  import Icon from "../../ui/Icon.svelte";

  let {
    matches,
    sel,
    onpick,
    onattach,
  }: {
    matches: SlashCmd[];
    sel: number;
    onpick: (c: SlashCmd) => void;
    onattach?: () => void;
  } = $props();
</script>

<div class="cmd-pop" transition:fly={{ y: 6, duration: 130 }}>
  {#if onattach}
    <button type="button" class="cmd-item attach" onmousedown={(e) => { e.preventDefault(); onattach?.(); }}>
      <span class="cmd-cli"><Icon name="image" size={13} /></span>
      <span class="cmd-name">Joindre un fichier</span>
    </button>
    <div class="sep"></div>
  {/if}
  {#each matches as c, i}
    <button
      type="button"
      class="cmd-item"
      class:sel={i === sel}
      onmousedown={(e) => {
        e.preventDefault();
        onpick(c);
      }}
    >
      <span class="cmd-cli">{c.cli || "CLI"}</span>
      <span class="cmd-name"><span class="slash">/</span>{c.name}{#if c.args}<span class="cmd-args"> {c.args}</span>{/if}</span>
    </button>
  {/each}
</div>

<style>
  .cmd-pop {
    position: absolute;
    left: 7px;
    bottom: calc(100% + 4px);
    width: max-content;
    min-width: 200px;
    max-width: calc(100% - 14px);
    max-height: 300px;
    overflow-y: auto;
    overscroll-behavior: contain;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 -8px 26px rgba(0, 0, 0, 0.22);
    padding: 5px;
    z-index: 30;
  }
  .cmd-item {
    width: 100%;
    display: grid;
    grid-template-columns: 64px 1fr;
    align-items: baseline;
    gap: 8px;
    padding: 5px 9px;
    border-radius: var(--radius-sm);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12.5px;
    text-align: left;
  }
  .cmd-cli {
    color: var(--text-faint);
    font-size: 10.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .cmd-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .cmd-item .slash {
    color: var(--accent);
    font-weight: 700;
  }
  .cmd-args {
    color: var(--text-faint);
    font-size: 11px;
  }
  .cmd-item:hover,
  .cmd-item.sel {
    background: var(--accent-weak);
  }
  .cmd-item.attach .cmd-cli {
    color: var(--accent);
    display: inline-flex;
    align-items: center;
  }
  .cmd-item.attach .cmd-name {
    color: var(--text-faint);
    font-size: 11px;
  }
  .sep {
    height: 1px;
    background: var(--border);
    margin: 4px 2px;
  }
</style>
