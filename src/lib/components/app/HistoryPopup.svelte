<script lang="ts">
  import { onMount } from "svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import { tabs } from "$lib/stores/tabs.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import * as ipc from "$lib/ipc";
  import { fly, fade } from "svelte/transition";

  let { onclose, now }: { onclose: () => void; now: number } = $props();

  let items = $state<ipc.SessionHist[]>([]);
  let results = $state<ipc.SessionHist[]>([]);
  let query = $state("");
  let loading = $state(true);
  let searching = $state(false);
  let busy = $state("");
  const shown = $derived(query.trim() ? results : items);

  onMount(async () => {
    try {
      items = await ipc.recentSessions(settings.historyLimit);
    } catch {
      /* ignore */
    } finally {
      loading = false;
    }
  });

  // Recherche globale debouncée (250 ms).
  let timer: number | undefined;
  $effect(() => {
    const q = query.trim();
    clearTimeout(timer);
    if (!q) {
      results = [];
      searching = false;
      return;
    }
    searching = true;
    timer = window.setTimeout(async () => {
      try {
        results = await ipc.searchSessions(q, settings.historyLimit);
      } catch {
        results = [];
      } finally {
        searching = false;
      }
    }, 250);
  });

  /** Date relative compacte (fr). `now` est passé par le parent (epoch ms). */
  function ago(ts: number): string {
    const s = Math.max(0, Math.floor(now / 1000) - ts);
    if (s < 60) return "à l'instant";
    const m = Math.floor(s / 60);
    if (m < 60) return `il y a ${m} min`;
    const h = Math.floor(m / 60);
    if (h < 24) return `il y a ${h} h`;
    const d = Math.floor(h / 24);
    return d < 7 ? `il y a ${d} j` : `il y a ${Math.floor(d / 7)} sem`;
  }
  /** Dernier segment du cwd (nom du dossier projet). */
  function proj(cwd: string): string {
    if (!cwd) return "";
    return cwd.replace(/[\\/]+$/, "").split(/[\\/]/).pop() ?? "";
  }

  async function open(h: ipc.SessionHist) {
    if (busy) return;
    busy = h.id;
    try {
      const msgs = await ipc.loadMessages(h.id);
      await sessions.openExisting(h.id, h.title, h.cwd, msgs);
      tabs.openSession(h.id, h.title.slice(0, 24));
      onclose();
    } catch {
      busy = "";
    }
  }
</script>

<div class="ov" role="presentation" transition:fade={{ duration: 100 }} onclick={onclose}>
  <div
    class="pop"
    role="dialog"
    aria-label="Historique"
    transition:fly={{ y: -8, duration: 150 }}
    onclick={(e) => e.stopPropagation()}
  >
    <div class="head">
      <input
        class="search"
        placeholder="Rechercher dans toutes les conversations…"
        bind:value={query}
        autofocus
      />
    </div>
    <div class="list">
      {#if loading}
        <div class="empty">Chargement…</div>
      {:else if searching}
        <div class="empty">Recherche…</div>
      {:else if shown.length === 0}
        <div class="empty">{query.trim() ? "Aucun résultat." : "Aucune conversation."}</div>
      {:else}
        {#each shown as h (h.id)}
          <button class="item" disabled={busy === h.id} onclick={() => open(h)}>
            <span class="t">{h.title}</span>
            {#if h.snippet}<span class="snip">{h.snippet}</span>{/if}
            <span class="meta">
              {ago(h.ts)}{#if proj(h.cwd)} · {proj(h.cwd)}{/if}
            </span>
          </button>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .ov {
    position: fixed;
    inset: 0;
    z-index: 200;
  }
  .pop {
    position: absolute;
    top: 36px;
    right: 12px;
    width: 340px;
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
    overflow: hidden;
  }
  .head {
    padding: 8px;
    border-bottom: 1px solid var(--border);
  }
  .search {
    width: 100%;
    height: 28px;
    padding: 0 9px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 12px;
    outline: none;
  }
  .search:focus {
    border-color: var(--accent);
  }
  .snip {
    font-size: 10.5px;
    color: var(--text-faint);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .list {
    overflow-y: auto;
    overscroll-behavior: contain;
    padding: 4px;
  }
  .item {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 7px 9px;
    border-radius: var(--radius-sm);
    text-align: left;
  }
  .item:hover {
    background: var(--surface-2);
  }
  .item:disabled {
    opacity: 0.5;
  }
  .t {
    font-size: 12.5px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    font-size: 10.5px;
    color: var(--text-faint);
    font-family: var(--font-mono);
  }
  .empty {
    padding: 16px;
    text-align: center;
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
