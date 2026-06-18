<script lang="ts">
  import Icon from "./Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let {
    value,
    options,
    label,
    onchange,
    btnClass = "",
  }: {
    value: string;
    options: { v: string; l: string }[];
    label: string;
    onchange: (v: string) => void;
    btnClass?: string;
  } = $props();

  let open = $state(false);
  let root: HTMLDivElement;
  let btn = $state<HTMLButtonElement>();
  // Menu en position fixe (échappe au `overflow:hidden` du pane) — calculé sur le bouton.
  let menuStyle = $state("");

  const current = $derived(options.find((o) => o.v === value)?.l ?? label);

  function toggle() {
    open = !open;
    if (open && btn) {
      const r = btn.getBoundingClientRect();
      const left = Math.max(6, Math.min(r.left, window.innerWidth - 142));
      menuStyle = `left:${left}px; bottom:${window.innerHeight - r.top + 6}px;`;
    }
  }

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
    class="dd-btn {btnClass}"
    class:open
    class:set={value !== ""}
    bind:this={btn}
    use:tooltip={label}
    onclick={toggle}
  >
    <span class="dd-cur">{current}</span>
    <span class="dd-chev" class:up={open}><Icon name="chevron" size={13} /></span>
  </button>

  {#if open}
    <ul class="dd-list" style={menuStyle} transition:fly={{ y: 6, duration: 150, easing: cubicOut }}>
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
    height: 24px;
    padding: 0 4px 0 7px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 10px;
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

  /* Design distinct du bouton effort selon la puissance (même taille) */
  .dd-btn.eff-low {
    color: var(--text-faint);
    background: var(--bg);
    border-color: var(--border);
  }
  .dd-btn.eff-medium {
    color: var(--text-muted);
    background: var(--bg);
    border-color: var(--border-strong);
  }
  .dd-btn.eff-high {
    color: var(--accent);
    background: var(--accent-weak);
    border-color: color-mix(in srgb, var(--accent) 45%, transparent);
  }
  .dd-btn.eff-xhigh {
    color: var(--accent);
    background: var(--accent-weak);
    border-color: var(--accent);
    box-shadow: 0 0 8px color-mix(in srgb, var(--accent) 28%, transparent);
  }
  .dd-btn.eff-max,
  .dd-btn.eff-ultracode {
    color: var(--accent);
    font-weight: 700;
    border: 1px solid transparent;
    background:
      linear-gradient(var(--surface-2), var(--surface-2)) padding-box,
      linear-gradient(90deg, var(--accent), #e6a988, var(--accent), #cf7ea6, var(--accent)) border-box;
    background-size: 100% 100%, 300% 100%;
    animation: ddFlow 4s linear infinite;
  }
  .dd-btn.eff-ultracode {
    animation: ddFlow 1.9s linear infinite;
    box-shadow: 0 0 10px color-mix(in srgb, var(--accent) 40%, transparent);
  }
  @keyframes ddFlow {
    to {
      background-position: 0 0, 300% 0;
    }
  }
  .dd-chev {
    display: flex;
    transition: transform var(--transition);
  }
  .dd-chev.up {
    transform: rotate(180deg);
  }
  .dd-list {
    position: fixed;
    min-width: 130px;
    max-height: calc(100vh - 24px);
    overflow-y: auto;
    list-style: none;
    margin: 0;
    padding: 5px;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.22);
    z-index: 1000;
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
