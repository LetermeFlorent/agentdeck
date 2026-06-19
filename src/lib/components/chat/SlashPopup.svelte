<script lang="ts">
  import type { SlashCmd } from "$lib/ipc";
  import { fly } from "svelte/transition";

  import Icon from "../ui/Icon.svelte";

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
      <span class="cmd-name"><Icon name="image" size={13} /> Joindre un fichier</span>
      <span class="cmd-desc">tous types · ou colle / glisse</span>
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
      <span class="cmd-name"
        ><span class="slash">/</span>{c.name}{#if c.args}<span class="cmd-args"> {c.args}</span>{/if}</span
      >
      {#if c.description}<span class="cmd-desc">{c.description}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .cmd-pop {
    position: absolute;
    left: 7px;
    bottom: calc(100% + 4px);
    /* S'ajuste à la largeur du contenu (nom + description), borné au champ. */
    width: max-content;
    min-width: 160px;
    max-width: calc(100% - 14px);
    max-height: 240px;
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
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 9px;
    border-radius: var(--radius-sm);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12.5px;
    text-align: left;
  }
  .cmd-name {
    flex-shrink: 0;
    white-space: nowrap;
  }
  .cmd-item .slash {
    color: var(--accent);
    font-weight: 700;
  }
  .cmd-desc {
    flex: 1;
    min-width: 0;
    text-align: right;
    color: var(--text-faint);
    font-size: 10.5px;
    line-height: 1.35;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .cmd-item:hover,
  .cmd-item.sel {
    background: var(--accent-weak);
  }
  .cmd-item.attach .cmd-name {
    color: var(--accent);
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .sep {
    height: 1px;
    background: var(--border);
    margin: 4px 2px;
  }
</style>
