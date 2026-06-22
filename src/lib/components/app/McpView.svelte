<script lang="ts">
  import { onMount } from "svelte";
  import * as ipc from "$lib/ipc";
  import type { McpItem } from "$lib/ipc";

  let raw = $state("{}");
  let saved = $state("{}");
  let error = $state("");
  let saving = $state(false);
  let confirmDel = $state("");

  // Serveurs MCP réellement actifs (claude mcp list + plugins), lecture seule.
  let installed = $state<McpItem[]>([]);
  let confirmRemove = $state("");

  onMount(async () => {
    try {
      raw = await ipc.mcpReadRaw();
      saved = raw;
    } catch (e) {
      error = String(e);
    }
    refreshInstalled();
  });

  async function refreshInstalled() {
    try {
      installed = await ipc.mcpInstalled();
    } catch {
      installed = [];
    }
  }

  async function removeInstalled(name: string) {
    confirmRemove = "";
    try {
      await ipc.mcpRemove(name);
      await refreshInstalled();
    } catch (e) {
      error = String(e);
    }
  }

  const dirty = $derived(raw !== saved);

  const parsed = $derived.by<Record<string, unknown> | null>(() => {
    try { return JSON.parse(raw); } catch { return null; }
  });

  const serverNames = $derived(parsed ? Object.keys(parsed) : []);

  function validate(): boolean {
    try { JSON.parse(raw); error = ""; return true; }
    catch (e) { error = String(e); return false; }
  }

  async function save() {
    if (!validate()) return;
    saving = true;
    try {
      await ipc.mcpWriteRaw(raw);
      saved = raw;
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function reset() { raw = saved; error = ""; }

  async function deleteServer(name: string) {
    if (!parsed) return;
    const next = { ...parsed };
    delete next[name];
    raw = JSON.stringify(next, null, 2);
    confirmDel = "";
    await save();
  }
</script>

<div class="mcp-editor">
  <div class="mcp-head">
    <span class="mcp-label">Serveurs actifs · {installed.length}</span>
    <span class="mcp-hint">claude mcp list + plugins</span>
  </div>

  {#if installed.length > 0}
    <div class="srv-list">
      {#each installed as m (m.name)}
        <div class="srv-row">
          <span class="srv-name">{m.name}</span>
          {#if m.target}<span class="srv-target">{m.target}</span>{/if}
          {#if m.status}<span class="srv-status">{m.status}</span>{/if}
          <span class="srv-scope">{m.scope}</span>
          <div class="srv-acts">
            {#if m.removable}
              {#if confirmRemove === m.name}
                <button class="act danger" onclick={() => removeInstalled(m.name)}>✓</button>
                <button class="act" onclick={() => (confirmRemove = "")}>✕</button>
              {:else}
                <button class="act hov-danger" onclick={() => (confirmRemove = m.name)}>🗑</button>
              {/if}
            {:else}
              <button class="act" disabled title="Géré par un plugin / claude.ai">🔒</button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else if !error}
    <div class="empty">Aucun serveur MCP actif.</div>
  {/if}

  <div class="mcp-head" style="margin-top: 6px;">
    <span class="mcp-label">mcpServers</span>
    <span class="mcp-hint">~/.claude/settings.json</span>
  </div>

  {#if serverNames.length > 0}
    <div class="srv-list">
      {#each serverNames as name (name)}
        <div class="srv-row">
          <span class="srv-name">{name}</span>
          <div class="srv-acts">
            {#if confirmDel === name}
              <button class="act danger" onclick={() => deleteServer(name)}>✓</button>
              <button class="act" onclick={() => (confirmDel = "")}>✕</button>
            {:else}
              <button class="act hov-danger" onclick={() => (confirmDel = name)}>🗑</button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else if !error}
    <div class="empty">Aucun serveur MCP configuré.</div>
  {/if}

  <div class="cfg-label">Config JSON brute</div>
  <textarea
    class="cfg-area"
    class:invalid={!!error}
    spellcheck={false}
    bind:value={raw}
    oninput={validate}
  ></textarea>

  {#if error}
    <div class="err">{error}</div>
  {/if}

  <div class="mcp-actions">
    <button class="act-reset" disabled={!dirty || saving} onclick={reset}>Annuler</button>
    <button class="act-save" disabled={!dirty || !!error || saving} onclick={save}>
      {saving ? "Enregistrement…" : "Enregistrer"}
    </button>
  </div>
</div>

<style>
  .mcp-editor {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 0 4px;
    min-height: 0;
  }
  .mcp-head {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }
  .mcp-label {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
  }
  .mcp-hint {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-faint);
  }
  .srv-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 4px 0 2px;
  }
  .srv-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
  }
  .srv-name {
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--text);
  }
  .srv-target {
    flex: 1;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .srv-status {
    font-size: 10px;
    color: var(--text-faint);
  }
  .srv-scope {
    font-size: 9.5px;
    color: var(--text-faint);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 1px 5px;
    font-family: var(--font-mono);
  }
  .srv-acts {
    display: flex;
    gap: 4px;
    margin-left: auto;
  }
  .act {
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-muted);
    font-size: 11px;
    transition: color var(--transition), border-color var(--transition);
  }
  .act.danger { color: #fff; background: var(--danger); border-color: var(--danger); }
  .act.hov-danger:hover { color: var(--danger); border-color: var(--danger); }
  .cfg-label {
    font-size: 10.5px;
    color: var(--text-faint);
    font-family: var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .cfg-area {
    resize: vertical;
    min-height: 160px;
    max-height: 320px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 1.5;
    outline: none;
    white-space: pre;
    overflow: auto;
    transition: border-color var(--transition);
  }
  .cfg-area:focus { border-color: var(--accent); }
  .cfg-area.invalid { border-color: var(--danger); }
  .err {
    font-size: 11px;
    color: var(--danger);
    font-family: var(--font-mono);
  }
  .mcp-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }
  .act-reset {
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-size: 12px;
    transition: background var(--transition), color var(--transition);
  }
  .act-reset:hover:not(:disabled) { background: var(--surface-2); color: var(--text); }
  .act-reset:disabled { opacity: 0.35; }
  .act-save {
    padding: 5px 14px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    font-size: 12px;
    transition: opacity var(--transition);
  }
  .act-save:disabled { opacity: 0.35; }
  .empty {
    font-size: 11.5px;
    color: var(--text-faint);
    padding: 2px 0;
  }
</style>
