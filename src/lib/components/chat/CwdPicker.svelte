<script lang="ts">
  import { onMount } from "svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import * as ipc from "$lib/ipc";
  import Icon from "../ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { fly, fade } from "svelte/transition";

  let {
    initial,
    onpick,
    onclose,
  }: { initial?: string; onpick: (path: string) => void; onclose: () => void } = $props();

  let list = $state<ipc.DirList | null>(null);
  let loading = $state(true);

  async function go(path?: string | null) {
    loading = true;
    try {
      list = await ipc.listDirs(path);
    } catch {
      /* ignore */
    } finally {
      loading = false;
    }
  }
  onMount(() => go(initial || null));

  const base = (p: string) => p.replace(/[\\/]+$/, "").split(/[\\/]/).pop() || p;
</script>

<div
  class="ov"
  role="presentation"
  transition:fade={{ duration: 100 }}
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <div class="pop" role="dialog" aria-label="Dossier de travail" transition:fly={{ y: -8, duration: 150 }} onclick={(e) => e.stopPropagation()}>
    {#if sessions.cwdRecents.length}
      <div class="sec">Récents</div>
      <div class="recents">
        {#each sessions.cwdRecents as r (r)}
          <button class="rec" use:tooltip={r} onclick={() => { onpick(r); onclose(); }}>{base(r)}</button>
        {/each}
      </div>
    {/if}

    <div class="sec">Parcourir</div>
    <div class="path">{list?.path ?? "…"}</div>
    <div class="browser">
      {#if list?.parent}
        <button class="row up" onclick={() => go(list?.parent)}><Icon name="chevron" size={13} /> ..</button>
      {/if}
      {#if loading}
        <div class="empty">…</div>
      {:else}
        {#each list?.dirs ?? [] as d (d.path)}
          <button class="row" ondblclick={() => go(d.path)} onclick={() => go(d.path)}>
            <span class="fold">▸</span>{d.name}
          </button>
        {/each}
        {#if (list?.dirs.length ?? 0) === 0}<div class="empty">(aucun sous-dossier)</div>{/if}
      {/if}
    </div>

    <button class="choose" disabled={!list} onclick={() => { if (list) { onpick(list.path); onclose(); } }}>
      Choisir ce dossier
    </button>
  </div>
</div>

<style>
  .ov { position: fixed; inset: 0; z-index: 220; }
  .pop {
    position: absolute;
    top: 36px;
    left: 50%;
    transform: translateX(-50%);
    width: 380px;
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
    padding: 8px;
  }
  .sec {
    padding: 6px 4px 4px;
    font-size: 10px;
    color: var(--text-faint);
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .recents { display: flex; flex-wrap: wrap; gap: 4px; }
  .rec {
    max-width: 150px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 3px 8px;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-size: 11px;
    font-family: var(--font-mono);
  }
  .rec:hover { color: var(--accent); border-color: var(--accent); }
  .path {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    padding: 2px 4px 6px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
  }
  .browser {
    flex: 1;
    overflow-y: auto;
    overscroll-behavior: contain;
    min-height: 120px;
    max-height: 280px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 3px;
  }
  .row {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 7px;
    border-radius: var(--radius-sm);
    text-align: left;
    font-size: 12px;
    color: var(--text);
    font-family: var(--font-mono);
  }
  .row:hover { background: var(--surface-2); }
  .fold { color: var(--text-faint); }
  .up { color: var(--text-muted); }
  .empty { padding: 10px; text-align: center; font-size: 11.5px; color: var(--text-faint); }
  .choose {
    margin-top: 8px;
    padding: 7px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    font-size: 12.5px;
  }
  .choose:disabled { opacity: 0.4; }
</style>
