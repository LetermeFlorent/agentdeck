<script lang="ts">
  import { onMount } from "svelte";
  import * as ipc from "$lib/ipc";
  import type { SkillItem } from "$lib/ipc";
  import { slide } from "svelte/transition";

  let skills = $state<SkillItem[]>([]);
  let error = $state("");
  let confirmDel = $state("");

  // new skill form
  let adding = $state(false);
  let newName = $state("");
  let newDesc = $state("");
  let newBody = $state("");

  // inline editor
  let expanded = $state<string | null>(null);
  let editContent = $state("");
  let editSaved = $state("");
  let editError = $state("");
  let editSaving = $state(false);

  onMount(refresh);

  async function refresh() {
    try {
      skills = await ipc.skillsInstalled();
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function expand(name: string) {
    if (expanded === name) { expanded = null; return; }
    editContent = "";
    editSaved = "";
    editError = "";
    expanded = name;
    try {
      const c = await ipc.skillRead(name);
      editContent = c;
      editSaved = c;
    } catch (e) {
      editError = String(e);
    }
  }

  async function saveEdit(name: string) {
    editSaving = true;
    try {
      await ipc.skillWrite(name, editContent);
      editSaved = editContent;
      editError = "";
      await refresh();
    } catch (e) {
      editError = String(e);
    } finally {
      editSaving = false;
    }
  }

  async function deleteSkill(name: string) {
    try {
      await ipc.skillDelete(name);
      confirmDel = "";
      if (expanded === name) expanded = null;
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function createSkill() {
    const n = newName.trim();
    if (!n) return;
    const content = `---\nname: ${n}\ndescription: ${newDesc.trim()}\n---\n\n${newBody}`;
    try {
      await ipc.skillWrite(n, content);
      newName = ""; newDesc = ""; newBody = ""; adding = false;
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="sv">
  <div class="sv-head">
    <span class="sv-label">Skills installés · {skills.length}</span>
    <button class="add-btn" onclick={() => (adding = !adding)}>+ Nouveau</button>
  </div>

  {#if adding}
    <div class="add-form" transition:slide={{ duration: 150 }}>
      <input class="f-in" placeholder="nom-du-skill" bind:value={newName} />
      <input class="f-in" placeholder="Description (quand utiliser ce skill)" bind:value={newDesc} />
      <textarea class="f-area" placeholder="Contenu du skill (Markdown)…" rows="5" bind:value={newBody}></textarea>
      <button class="f-ok" disabled={!newName.trim()} onclick={createSkill}>Créer</button>
    </div>
  {/if}

  {#if error}<div class="err">{error}</div>{/if}

  <div class="scroll">
    {#if skills.length === 0}
      <div class="empty">Aucun skill installé.</div>
    {/if}
    {#each skills as s (s.name)}
      <div class="row">
        <button class="row-name" onclick={() => expand(s.name)}>
          <span class="name">{s.name}</span>
          {#if s.description}<span class="desc">{s.description}</span>{/if}
        </button>
        <div class="row-acts">
          {#if s.removable}
            {#if confirmDel === s.name}
              <button class="act danger" onclick={() => deleteSkill(s.name)}>✓</button>
              <button class="act" onclick={() => (confirmDel = "")}>✕</button>
            {:else}
              <button class="act hov-danger" onclick={() => (confirmDel = s.name)}>🗑</button>
            {/if}
          {:else}
            <button class="act" disabled title="Fourni par un plugin">🔒</button>
          {/if}
        </div>
      </div>

      {#if expanded === s.name}
        <div class="editor" transition:slide={{ duration: 120 }}>
          <textarea
            class="ed-area"
            spellcheck={false}
            bind:value={editContent}
          ></textarea>
          {#if editError}<div class="err">{editError}</div>{/if}
          <div class="ed-actions">
            <button class="act-reset" disabled={editContent === editSaved || editSaving} onclick={() => (editContent = editSaved)}>Annuler</button>
            <button class="act-save" disabled={editContent === editSaved || editSaving} onclick={() => saveEdit(s.name)}>
              {editSaving ? "…" : "Enregistrer"}
            </button>
          </div>
        </div>
      {/if}
    {/each}
  </div>
</div>

<style>
  .sv {
    display: flex;
    flex-direction: column;
    min-height: 0;
    gap: 6px;
    padding-top: 10px;
  }
  .sv-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .sv-label {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-muted);
  }
  .add-btn {
    padding: 4px 9px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--accent);
    color: var(--accent);
    background: var(--accent-weak);
    font-size: 11.5px;
    transition: background var(--transition);
  }
  .add-btn:hover { background: color-mix(in srgb, var(--accent) 22%, transparent); }
  .add-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-bottom: 8px;
  }
  .f-in {
    height: 28px;
    padding: 0 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 12px;
    outline: none;
  }
  .f-area, .ed-area {
    resize: vertical;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 1.45;
    outline: none;
  }
  .f-area { min-height: 80px; }
  .ed-area { min-height: 180px; width: 100%; box-sizing: border-box; }
  .f-in:focus, .f-area:focus, .ed-area:focus { border-color: var(--accent); }
  .f-ok {
    align-self: flex-end;
    padding: 5px 14px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    font-size: 12px;
  }
  .f-ok:disabled { opacity: 0.4; }
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
  .row-name {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    min-width: 0;
  }
  .name {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text);
  }
  .desc {
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row-acts {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
  .act {
    width: 24px;
    height: 24px;
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
  .editor {
    padding: 8px 0 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ed-actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }
  .act-reset {
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-size: 11.5px;
  }
  .act-reset:disabled { opacity: 0.35; }
  .act-save {
    padding: 4px 12px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    font-size: 11.5px;
  }
  .act-save:disabled { opacity: 0.35; }
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
