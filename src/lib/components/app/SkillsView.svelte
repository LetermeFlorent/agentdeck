<script lang="ts">
  import { onMount } from "svelte";
  import { library } from "$lib/stores/library.svelte";
  import Icon from "../ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { slide } from "svelte/transition";

  let adding = $state(false);
  let newName = $state("");
  let newDesc = $state("");
  let newBody = $state("");
  let confirmDel = $state(""); // skill dont la suppression est en attente de confirmation

  onMount(() => {
    library.refreshSkills();
    library.loadSkillCatalog();
  });

  async function submitOwn() {
    const n = newName.trim();
    if (!n) return;
    try {
      await library.addOwnSkill(n, newDesc.trim(), newBody);
      newName = "";
      newDesc = "";
      newBody = "";
      adding = false;
    } catch {
      /* erreur affichée via library.error */
    }
  }
  const available = $derived(library.catalogSkills.filter((c) => !library.isSkillInstalled(c.name)));
</script>

<div class="lib">
  <div class="lib-top">
    <span class="sec">Mes skills · {library.installedSkills.length}</span>
    <button class="add-btn" use:tooltip={"Créer mon propre skill"} onclick={() => (adding = !adding)}>
      <Icon name="plus" size={13} /> Ajouter le mien
    </button>
  </div>

  {#if adding}
    <div class="add-form" transition:slide={{ duration: 150 }}>
      <input class="f-in" placeholder="nom-du-skill" bind:value={newName} />
      <input class="f-in" placeholder="Description : quand utiliser ce skill" bind:value={newDesc} />
      <textarea
        class="f-area"
        placeholder="Contenu du skill (instructions en Markdown)…"
        rows="5"
        bind:value={newBody}
      ></textarea>
      <button class="f-ok" disabled={!newName.trim()} onclick={submitOwn}>Créer le skill</button>
    </div>
  {/if}

  {#if library.error}<div class="err">{library.error}</div>{/if}

  <div class="scroll">
    <div class="grid">
      {#each library.installedSkills as s (s.name)}
        <div class="card">
          <div class="c-top">
            <span class="c-name">{s.name}</span>
            {#if confirmDel === s.name}
              <button class="c-act danger" use:tooltip={"Confirmer la suppression"} onclick={() => { library.deleteSkill(s.name); confirmDel = ""; }}>
                <Icon name="check" size={13} />
              </button>
              <button class="c-act" use:tooltip={"Annuler"} onclick={() => (confirmDel = "")}>
                <Icon name="close" size={13} />
              </button>
            {:else}
              <button class="c-act hov-danger" use:tooltip={"Supprimer ce skill"} onclick={() => (confirmDel = s.name)}>
                <Icon name="trash" size={13} />
              </button>
            {/if}
          </div>
          {#if s.description}<span class="c-desc">{s.description}</span>{/if}
        </div>
      {/each}
    </div>

    <div class="sep">Disponibles {library.loadingCat ? "…" : ""}</div>

    <div class="grid">
      {#each available as c (c.name)}
        <div class="card">
          <div class="c-top">
            <span class="c-name">{c.name}</span>
            <button
              class="c-act accent"
              disabled={library.busy === c.name}
              use:tooltip={"Installer ce skill"}
              onclick={() => library.installSkill(c.name)}
            >
              {#if library.busy === c.name}…{:else}<Icon name="download" size={13} />{/if}
            </button>
          </div>
          {#if c.description}<span class="c-desc">{c.description}</span>{/if}
        </div>
      {/each}
    </div>
    {#if !library.loadingCat && available.length === 0}
      <div class="empty">Tous les skills du catalogue sont installés.</div>
    {/if}
  </div>
</div>

<style>
  .lib {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .lib-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 0 8px;
  }
  .sec {
    font-size: 12px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }
  .add-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 9px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--accent);
    color: var(--accent);
    background: var(--accent-weak);
    font-size: 11.5px;
  }
  .add-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 4px 0 10px;
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
  .f-area {
    resize: vertical;
    min-height: 70px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 1.4;
    outline: none;
  }
  .f-in:focus,
  .f-area:focus {
    border-color: var(--accent);
  }
  .f-ok {
    align-self: flex-end;
    padding: 5px 14px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    font-size: 12px;
  }
  .f-ok:disabled {
    opacity: 0.4;
  }
  .err {
    font-size: 11px;
    color: var(--danger);
    padding: 2px 0 6px;
  }
  .scroll {
    overflow-y: auto;
    max-height: 380px;
    overscroll-behavior: contain;
    margin: 0 -4px;
    padding: 0 4px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-height: 74px;
    padding: 9px 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--bg);
  }
  .c-top {
    display: flex;
    align-items: flex-start;
    gap: 5px;
  }
  .c-name {
    flex: 1;
    min-width: 0;
    font-size: 12px;
    color: var(--text);
    font-family: var(--font-mono);
    word-break: break-word;
  }
  .c-desc {
    font-size: 10.5px;
    color: var(--text-muted);
    line-height: 1.35;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
  }
  .c-act {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    border: 1px solid var(--border);
    background: var(--surface);
  }
  .c-act.accent {
    color: var(--accent);
    border-color: var(--accent);
  }
  .c-act.danger {
    color: #fff;
    background: var(--danger);
    border-color: var(--danger);
  }
  .c-act.hov-danger:hover {
    color: var(--danger);
    border-color: var(--danger);
  }
  .c-act:disabled {
    opacity: 0.5;
  }
  .sep {
    margin-top: 12px;
    padding: 6px 0;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .empty {
    padding: 10px 0;
    font-size: 11.5px;
    color: var(--text-faint);
  }
</style>
