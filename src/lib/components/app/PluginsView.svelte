<script lang="ts">
  import { onMount } from "svelte";
  import * as ipc from "$lib/ipc";
  import type { PluginItem } from "$lib/ipc";

  let plugins = $state<PluginItem[]>([]);
  let error = $state("");
  let confirmDel = $state("");
  let busy = $state("");

  onMount(refresh);

  async function refresh() {
    try {
      plugins = await ipc.pluginsInstalled();
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function uninstall(p: PluginItem) {
    busy = p.id;
    confirmDel = "";
    try {
      await ipc.pluginUninstall(p.id, p.scope);
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }
</script>

<div class="pv">
  <div class="pv-head">
    <span class="pv-label">Plugins installés · {plugins.length}</span>
    <span class="pv-hint">claude plugin</span>
  </div>

  {#if error}<div class="err">{error}</div>{/if}

  <div class="scroll">
    {#if plugins.length === 0}
      <div class="empty">Aucun plugin installé.</div>
    {/if}
    {#each plugins as p (p.id)}
      <div class="row">
        <div class="info">
          <span class="name">
            {p.name}
            {#if p.marketplace}<span class="mkt">@{p.marketplace}</span>{/if}
            {#if p.version}<span class="ver">v{p.version}</span>{/if}
          </span>
          <span class="meta">{p.skills} skill{p.skills > 1 ? "s" : ""}{#if p.description} · {p.description}{/if}</span>
        </div>
        <div class="acts">
          {#if confirmDel === p.id}
            <button class="act danger" disabled={busy === p.id} onclick={() => uninstall(p)}>✓ Désinstaller</button>
            <button class="act" onclick={() => (confirmDel = "")}>✕</button>
          {:else}
            <button class="act hov-danger" disabled={!!busy} onclick={() => (confirmDel = p.id)}>🗑</button>
          {/if}
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .pv {
    display: flex;
    flex-direction: column;
    min-height: 0;
    gap: 6px;
    padding-top: 10px;
  }
  .pv-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .pv-label {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-muted);
  }
  .pv-hint {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-faint);
  }
  .scroll {
    overflow-y: auto;
    max-height: 420px;
    overscroll-behavior: contain;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child { border-bottom: none; }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text);
  }
  .mkt { color: var(--text-faint); font-size: 10.5px; }
  .ver { color: var(--text-muted); font-size: 10px; margin-left: 4px; }
  .meta {
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .acts {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
  .act {
    height: 24px;
    padding: 0 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    font-size: 11px;
    transition: background var(--transition), border-color var(--transition), color var(--transition);
  }
  .act.danger { color: #fff; background: var(--danger); border-color: var(--danger); }
  .act.hov-danger:hover { color: var(--danger); border-color: var(--danger); }
  .act:disabled { opacity: 0.4; }
  .err {
    font-size: 11px;
    color: var(--danger);
    font-family: var(--font-mono);
  }
  .empty {
    font-size: 11.5px;
    color: var(--text-faint);
    padding: 10px 0;
  }
</style>
