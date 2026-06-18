<script lang="ts">
  import Icon from "./Icon.svelte";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let {
    value,
    options,
    label,
    onchange,
  }: {
    value: string;
    options: { v: string; l: string }[];
    label: string;
    onchange: (v: string) => void;
  } = $props();

  let open = $state(false);
  let root: HTMLDivElement;

  const current = $derived(options.find((o) => o.v === value)?.l ?? label);

  function pick(v: string) {
    onchange(v);
    open = false;
  }

  // Ferme au clic extérieur / Échap.
  $effect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (root && !root.contains(e.target as Node)) open = false;
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") open = false;
    };
    window.addEventListener("pointerdown", onDown, true);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onDown, true);
      window.removeEventListener("keydown", onKey);
    };
  });
</script>

<div class="dd" bind:this={root}>
  <button
    type="button"
    class="dd-btn"
    class:open
    class:set={value !== ""}
    title={label}
    onclick={() => (open = !open)}
  >
    <span class="dd-cur">{current}</span>
    <span class="dd-chev" class:up={open}><Icon name="chevron" size={13} /></span>
  </button>

  {#if open}
    <ul class="dd-list" transition:fly={{ y: 6, duration: 150, easing: cubicOut }}>
      {#each options as o, i}
        <li in:fly={{ y: 5, duration: 130, delay: 20 + i * 22, easing: cubicOut }}>
          <button
            type="button"
            class="dd-item"
            class:sel={o.v === value}
            onclick={() => pick(o.v)}
          >
            <span>{o.l}</span>
            {#if o.v === value}<Icon name="check" size={13} />{/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .dd {
    position: relative;
    flex-shrink: 0;
  }
  .dd-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 28px;
    padding: 0 6px 0 9px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 11px;
    transition: border-color var(--transition), color var(--transition), background var(--transition);
  }
  .dd-btn:hover {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .dd-btn.set {
    color: var(--accent);
    border-color: var(--accent-weak);
    background: var(--accent-weak);
  }
  .dd-btn.open {
    border-color: var(--accent);
  }
  .dd-chev {
    display: flex;
    transition: transform var(--transition);
  }
  .dd-chev.up {
    transform: rotate(180deg);
  }
  .dd-list {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 0;
    min-width: 130px;
    list-style: none;
    margin: 0;
    padding: 5px;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.22);
    z-index: 40;
  }
  .dd-item {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 6px 9px;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 12px;
    text-align: left;
    transition: background var(--transition), color var(--transition);
  }
  .dd-item:hover {
    background: var(--surface-2);
    color: var(--text);
  }
  .dd-item.sel {
    color: var(--accent);
  }
</style>
