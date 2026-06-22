<script lang="ts">
  import * as ipc from "$lib/ipc";
  import type { GNode } from "./graph-types";
  import { slide } from "svelte/transition";

  let { node, onclose, onchanged }: { node: GNode; onclose: () => void; onchanged: () => void } = $props();

  // cwd à transmettre au backend : skill projet → dossier du dépôt ; global → null.
  const cwd = $derived(node.scope === "project" ? node.cwd ?? null : null);
  // Skill fourni par un plugin : lecture seule (édition/suppression désactivées).
  const readonly = $derived(node.removable === false);
  const pluginName = $derived(node.source?.startsWith("plugin:") ? node.source.slice(7) : "");

  let content = $state("");
  let saved = $state("");
  let error = $state("");
  let busy = $state(false);
  let confirmDel = $state(false);

  // Recharge le SKILL.md à chaque changement de nœud sélectionné.
  $effect(() => {
    const name = node.skill ?? "";
    content = "";
    saved = "";
    error = "";
    confirmDel = false;
    ipc
      .skillRead(name, cwd)
      .then((c) => { content = c; saved = c; })
      .catch((e) => (error = String(e)));
  });

  async function save() {
    busy = true;
    try {
      await ipc.skillWrite(node.skill ?? "", content, cwd);
      saved = content;
      error = "";
      onchanged();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove() {
    busy = true;
    try {
      await ipc.skillDelete(node.skill ?? "", cwd);
      onchanged();
      onclose();
    } catch (e) {
      error = String(e);
      busy = false;
    }
  }
</script>

<aside class="insp" transition:slide={{ axis: "x", duration: 160 }}>
  <header class="ih">
    <div class="title">
      <span class="badge {readonly ? 'plugin' : node.scope}">{readonly ? "plugin" : node.scope === "project" ? "projet" : "global"}</span>
      <span class="name">{node.skill}</span>
    </div>
    <button class="x" onclick={onclose} aria-label="Fermer">✕</button>
  </header>
  {#if readonly && pluginName}
    <div class="cwd" title={node.source}>Fourni par le plugin « {pluginName} » — lecture seule</div>
  {:else if node.scope === "project" && node.cwd}
    <div class="cwd" title={node.cwd}>{node.cwd}</div>
  {/if}

  <textarea class="ed" spellcheck={false} readonly={readonly} bind:value={content}></textarea>
  {#if error}<div class="err">{error}</div>{/if}

  {#if !readonly}
    <div class="acts">
      {#if confirmDel}
        <button class="del on" disabled={busy} onclick={remove}>Confirmer</button>
        <button class="ghost" disabled={busy} onclick={() => (confirmDel = false)}>Annuler</button>
      {:else}
        <button class="del" disabled={busy} onclick={() => (confirmDel = true)}>Supprimer</button>
      {/if}
      <span class="sp"></span>
      <button class="ghost" disabled={content === saved || busy} onclick={() => (content = saved)}>Réinitialiser</button>
      <button class="save" disabled={content === saved || busy} onclick={save}>{busy ? "…" : "Enregistrer"}</button>
    </div>
  {/if}
</aside>

<style>
  .insp {
    position: absolute;
    top: 0;
    right: 0;
    width: 380px;
    max-width: 80vw;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px;
    background: var(--surface);
    border-left: 1px solid var(--border);
    box-shadow: -12px 0 40px rgba(0, 0, 0, 0.25);
    z-index: 2;
  }
  .ih { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .title { display: flex; align-items: center; gap: 8px; min-width: 0; }
  .name { font-family: var(--font-mono); font-size: 13px; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .badge { font-size: 9.5px; font-weight: 700; padding: 2px 6px; border-radius: 5px; text-transform: uppercase; letter-spacing: 0.04em; }
  .badge.global { color: var(--accent); background: var(--accent-weak); }
  .badge.project { color: var(--good, #5bbf7a); background: color-mix(in srgb, var(--good, #5bbf7a) 18%, transparent); }
  .badge.plugin { color: var(--text-muted); background: var(--surface-2); }
  .cwd { font-family: var(--font-mono); font-size: 10.5px; color: var(--text-faint); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .x { width: 24px; height: 24px; border-radius: var(--radius-sm); color: var(--text-muted); }
  .x:hover { background: var(--surface-2); color: var(--text); }
  .ed { flex: 1; resize: none; padding: 8px; border-radius: var(--radius-sm); border: 1px solid var(--border); background: var(--bg); color: var(--text); font-family: var(--font-mono); font-size: 11.5px; line-height: 1.5; outline: none; }
  .ed:focus { border-color: var(--accent); }
  .err { font-size: 11px; color: var(--danger); font-family: var(--font-mono); }
  .acts { display: flex; align-items: center; gap: 6px; }
  .sp { flex: 1; }
  .acts button { padding: 5px 11px; border-radius: var(--radius-sm); font-size: 11.5px; border: 1px solid var(--border); background: var(--bg); color: var(--text-muted); }
  .ghost:disabled { opacity: 0.4; }
  .save { background: var(--accent); color: #fff; border-color: var(--accent); }
  .save:disabled { opacity: 0.4; }
  .del { color: var(--danger); border-color: transparent; }
  .del:hover { border-color: var(--danger); }
  .del.on { background: var(--danger); color: #fff; border-color: var(--danger); }
</style>
