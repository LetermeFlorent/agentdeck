<script lang="ts">
  import { sessions } from "$lib/stores/sessions.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import Icon from "../ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { priceOf, fmtTok } from "./chat-config";

  let {
    sid,
    nodeId,
    canMinimize = false,
    canMove = false,
    onsplit,
    onclose,
  }: {
    sid: string;
    nodeId: string;
    canMinimize?: boolean;
    canMove?: boolean;
    onsplit: (dir: "row" | "column") => void;
    onclose: () => void;
  } = $props();

  const session = $derived(sessions.map[sid]);
  const price = $derived(priceOf(session?.model));

  let editing = $state(false);
  let titleDraft = $state("");
  function startEdit() {
    titleDraft = session?.title ?? "Claude";
    editing = true;
  }
  function saveTitle() {
    const t = titleDraft.trim();
    if (t) sessions.setTitle(sid, t);
    editing = false;
  }
  function autofocus(el: HTMLInputElement) {
    el.focus();
    el.select();
  }
  function dragStart(e: DragEvent) {
    e.dataTransfer?.setData("text/plain", nodeId);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }
</script>

<header class="pane-head">
  <div
    class="title"
    class:movable={canMove}
    role="button"
    tabindex="0"
    aria-label={canMove ? "Glisser pour déplacer ce chat" : "Double-clic pour renommer"}
    draggable={canMove && !editing}
    ondragstart={dragStart}
    use:tooltip={canMove ? "Glisser pour déplacer ce chat" : ""}
  >
    {#if canMove}<span class="grip"><Icon name="grip" size={14} /></span>{/if}
    <span class="status" class:live={session?.streaming}></span>
    {#if editing}
      <input
        class="name-edit"
        bind:value={titleDraft}
        use:autofocus
        onblur={saveTitle}
        onkeydown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            saveTitle();
          } else if (e.key === "Escape") {
            editing = false;
          }
        }}
      />
    {:else}
      <span
        class="name"
        role="button"
        tabindex="0"
        use:tooltip={"Double-clic pour renommer"}
        ondblclick={startEdit}
      >{session?.title ?? "Claude"}</span>
    {/if}
  </div>
  {#if price}
    <span class="hdr-price" use:tooltip={"Tarif du modèle, par million de tokens (entrée / sortie)"}>
      ↑${price[0]} ↓${price[1]}/M
    </span>
  {/if}
  {#if (session?.totalTokens ?? 0) > 0}
    <span class="hdr-usage" use:tooltip={"Coût et tokens générés par ce chat"}>
      ${(session?.costUsd ?? 0).toFixed(3)} · {fmtTok(session?.totalTokens ?? 0)}
    </span>
  {/if}
  <div class="actions">
    {#if settings.autoModel}
      <button
        class="icon-btn"
        class:on={!session?.autoModelOff}
        class:off={session?.autoModelOff}
        use:tooltip={session?.autoModelOff ? "Auto-modèle désactivé pour ce chat — cliquer pour réactiver" : "Auto-modèle actif — cliquer pour désactiver sur ce chat"}
        onclick={() => sessions.toggleAutoModelOff(sid)}
      >
        <Icon name="cpu" size={14} />
      </button>
    {/if}
    {#if settings.autoEffort}
      <button
        class="icon-btn"
        class:on={!session?.autoEffortOff}
        class:off={session?.autoEffortOff}
        use:tooltip={session?.autoEffortOff ? "Auto-effort désactivé pour ce chat — cliquer pour réactiver" : "Auto-effort actif — cliquer pour désactiver sur ce chat"}
        onclick={() => sessions.toggleAutoEffortOff(sid)}
      >
        <Icon name="gauge" size={14} />
      </button>
    {/if}
    {#if session?.streaming}
      <button class="icon-btn" use:tooltip={"Arrêter"} onclick={() => sessions.stop(sid)}>
        <Icon name="stop" size={15} />
      </button>
    {/if}
    <button class="icon-btn" use:tooltip={"Dézoomer le chat"} onclick={() => sessions.setZoom(sid, -0.1)}>
      <Icon name="minus" size={15} />
    </button>
    <button class="icon-btn" use:tooltip={"Zoomer le chat"} onclick={() => sessions.setZoom(sid, 0.1)}>
      <Icon name="plus" size={15} />
    </button>
    <button
      class="icon-btn"
      class:on={session?.priv}
      use:tooltip={session?.priv ? "Désactiver le mode privé" : "Mode privé (flouter le contenu)"}
      onclick={() => sessions.setPrivate(sid, !session?.priv)}
    >
      <Icon name={session?.priv ? "eye-off" : "eye"} size={15} />
    </button>
    {#if canMinimize}
      <button class="icon-btn" use:tooltip={"Minimiser sur le côté"} onclick={() => sessions.setCollapsed(sid, true)}>
        <span class="chev close"><Icon name="chevron" size={15} /></span>
      </button>
    {/if}
    <button class="icon-btn" use:tooltip={"Diviser horizontalement (haut / bas)"} onclick={() => onsplit("column")}>
      <Icon name="split-v" />
    </button>
    <button class="icon-btn" use:tooltip={"Diviser verticalement (côte à côte)"} onclick={() => onsplit("row")}>
      <Icon name="split-h" />
    </button>
    <button class="icon-btn close" use:tooltip={"Fermer le pane"} onclick={onclose}>
      <Icon name="close" />
    </button>
  </div>
</header>

<style>
  .pane-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1px 5px 1px 4px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    flex-shrink: 0;
  }
  .actions {
    display: flex;
    gap: 2px;
  }
  .actions :global(.icon-btn) {
    width: 19px;
    height: 19px;
  }
  .actions :global(.icon-btn svg) {
    width: 14px;
    height: 14px;
  }
  .icon-btn.on {
    color: var(--accent);
    background: var(--accent-weak);
  }
  /* Auto désactivé pour ce chat : grisé + barré en diagonale. */
  .icon-btn.off {
    color: var(--text-faint);
    position: relative;
  }
  .icon-btn.off::after {
    content: "";
    position: absolute;
    left: 3px;
    right: 3px;
    top: 50%;
    height: 1.5px;
    background: var(--danger);
    transform: rotate(-45deg);
  }
  .close:hover {
    color: var(--danger);
  }
  .chev {
    display: flex;
    transition: transform var(--transition);
  }
  .chev.close {
    transform: rotate(90deg);
  }
  .title {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
    cursor: default;
    padding: 2px 4px;
    border-radius: 5px;
    transition: background var(--transition);
  }
  .title.movable {
    cursor: grab;
  }
  /* Surbrillance de la barre seulement quand elle sert de poignée (multi-panes). */
  .title.movable:hover {
    background: var(--elevated);
  }
  .title.movable:active {
    cursor: grabbing;
  }
  /* Hors déplacement, seul le nom réagit (cible du double-clic pour renommer). */
  .name:hover {
    background: var(--elevated);
    border-radius: 4px;
  }
  .grip {
    color: var(--text-faint);
    display: flex;
  }
  .status {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-faint);
    transition: background var(--transition), box-shadow var(--transition);
  }
  .status.live {
    background: var(--good);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--good) 25%, transparent);
    animation: pulseDot 1.6s ease-in-out infinite;
  }
  @keyframes pulseDot {
    0%, 100% { box-shadow: 0 0 0 2px color-mix(in srgb, var(--good) 22%, transparent); }
    50% { box-shadow: 0 0 0 4px color-mix(in srgb, var(--good) 10%, transparent); }
  }
  .name {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.01em;
    cursor: text;
    padding: 1px 4px;
    transition: background var(--transition);
  }
  .name-edit {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text);
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: 4px;
    padding: 1px 5px;
    outline: none;
    width: 130px;
    max-width: 100%;
  }
  .hdr-usage,
  .hdr-price {
    flex-shrink: 0;
    margin-right: 6px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-faint);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }
  .hdr-price {
    color: var(--text-muted);
  }
</style>
