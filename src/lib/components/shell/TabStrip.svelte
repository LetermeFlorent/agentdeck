<script lang="ts">
  // Bande d'onglets (workspaces) : sélection, renommage inline, réordonnancement par glisser.
  import { tabs } from "$lib/stores/tabs.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";

  // Renommage d'onglet inline.
  let renamingTab = $state("");
  let renameDraft = $state("");
  function startRename(t: { id: string; name: string }) {
    renamingTab = t.id;
    renameDraft = t.name;
  }
  function commitRename() {
    if (renamingTab) tabs.rename(renamingTab, renameDraft);
    renamingTab = "";
  }
  function tabAutofocus(el: HTMLInputElement) {
    el.focus();
    el.select();
  }

  // Réordonnancement par glisser (pointer-based : pas de DnD natif, donc aucun contour parasite).
  let dragTab = $state("");
  let dropTab = $state("");
  let dragging = $state(false);
  let dragStartX = 0;
  let justDragged = false;

  function onDown(e: PointerEvent, id: string) {
    if (e.button !== 0 || renamingTab === id) return;
    dragTab = id;
    dragStartX = e.clientX;
    dragging = false;
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }
  function onMove(e: PointerEvent) {
    if (!dragTab) return;
    if (!dragging) {
      if (Math.abs(e.clientX - dragStartX) < 5) return;
      dragging = true;
    }
    const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
    const id = el?.closest<HTMLElement>(".tab")?.dataset.tabId ?? "";
    dropTab = id && id !== dragTab ? id : "";
  }
  function onUp() {
    if (dragging && dropTab && dropTab !== dragTab) {
      tabs.move(dragTab, dropTab);
      justDragged = true;
    }
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    dragTab = "";
    dropTab = "";
    dragging = false;
  }
</script>

<div class="tabs" data-tauri-drag-region>
  {#each tabs.list as t, i (t.id)}
    {#if i > 0}<span class="tab-sep" data-tauri-drag-region></span>{/if}
    <div
      class="tab"
      role="tab"
      tabindex="-1"
      aria-selected={t.id === tabs.activeId}
      data-tab-id={t.id}
      class:active={t.id === tabs.activeId}
      class:dragging={dragging && dragTab === t.id}
      class:dropzone={dropTab === t.id}
      onpointerdown={(e) => onDown(e, t.id)}
    >
      {#if tabs.isTabBusy(t.id)}
        <span class="tab-live" use:tooltip={"Un Claude travaille dans cet onglet"}></span>
      {/if}
      {#if renamingTab === t.id}
        <input
          class="tab-edit"
          bind:value={renameDraft}
          use:tabAutofocus
          onblur={commitRename}
          onkeydown={(e) => {
            if (e.key === "Enter") { e.preventDefault(); commitRename(); }
            else if (e.key === "Escape") renamingTab = "";
          }}
        />
      {:else}
        <button
          class="tab-btn"
          use:tooltip={"Clic : ouvrir · glisser : réordonner · double-clic : renommer"}
          onclick={() => { if (justDragged) { justDragged = false; return; } tabs.select(t.id); }}
          ondblclick={() => startRename(t)}
        >{t.name}</button>
      {/if}
      {#if tabs.list.length > 1}
        <button class="tab-x" use:tooltip={"Fermer l'onglet"} onclick={() => tabs.close(t.id)}>
          <Icon name="close" size={11} />
        </button>
      {/if}
    </div>
  {/each}
  <button class="tab-add" data-tour="tabs" use:tooltip={"Nouvel onglet"} onclick={() => tabs.create()}>
    <Icon name="plus" size={14} />
  </button>
</div>

<style>
  .tabs { display: flex; align-items: center; gap: 3px; flex: 0 1 auto; min-width: 0; max-width: 46vw; overflow-x: auto; scrollbar-width: none; }
  .tabs::-webkit-scrollbar { height: 0; }
  .tab-sep { flex-shrink: 0; width: 1px; height: 16px; background: var(--border-strong); }
  .tab { display: flex; align-items: center; flex-shrink: 0; border-radius: var(--radius-sm); border: 1px solid transparent; transition: background var(--transition), border-color var(--transition); }
  .tab.active { background: var(--surface-2); border-color: var(--border); }
  .tab.dragging { opacity: 0.4; cursor: grabbing; }
  .tab.dropzone { border-color: var(--accent); box-shadow: inset 2px 0 0 var(--accent); }
  .tab-live { flex-shrink: 0; width: 6px; height: 6px; margin-left: 7px; border-radius: 50%; background: var(--good); animation: tabPulse 1.6s ease-in-out infinite; }
  @keyframes tabPulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.35; } }
  .tab-btn { max-width: 130px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; padding: 3px 8px; font-family: var(--font-mono); font-size: 11.5px; color: var(--text-muted); transition: color var(--transition); }
  .tab.active .tab-btn, .tab-btn:hover { color: var(--text); }
  .tab-x { display: inline-flex; align-items: center; justify-content: center; width: 16px; height: 16px; border-radius: 4px; color: var(--text-faint); transition: background var(--transition), color var(--transition); margin-right: 2px; }
  .tab-x:hover { color: var(--danger); background: var(--elevated); }
  .tab-edit { width: 110px; padding: 2px 6px; font-family: var(--font-mono); font-size: 11.5px; color: var(--text); background: var(--bg); border: 1px solid var(--accent); border-radius: 4px; outline: none; }
  .tab-add { display: inline-flex; align-items: center; justify-content: center; width: 22px; height: 22px; flex-shrink: 0; border-radius: var(--radius-sm); color: var(--text-muted); transition: background var(--transition), color var(--transition); }
  .tab-add:hover { background: var(--surface-2); color: var(--text); }
</style>
