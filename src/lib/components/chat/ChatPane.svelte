<script lang="ts">
  import { sessions } from "$lib/stores/sessions.svelte";
  import Icon from "../ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import PaneHeader from "./PaneHeader.svelte";
  import MessageLog from "./MessageLog.svelte";
  import Composer from "./Composer.svelte";

  let {
    sid,
    nodeId,
    canMinimize = false,
    canMove = false,
    onsplit,
    onclose,
    onmove,
  }: {
    sid: string;
    nodeId: string;
    canMinimize?: boolean;
    canMove?: boolean;
    onsplit: (dir: "row" | "column") => void;
    onclose: () => void;
    onmove: (fromNodeId: string) => void;
  } = $props();

  const session = $derived(sessions.map[sid]);
  const collapsed = $derived(session?.collapsed ?? false);

  let dragOver = $state(false);
  function dragOverH(e: DragEvent) {
    e.preventDefault();
    dragOver = true;
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
  }
  function dropH(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    const from = e.dataTransfer?.getData("text/plain");
    if (from && from !== nodeId) onmove(from);
  }
</script>

<div
  class="pane"
  class:drag-over={dragOver}
  class:collapsed
  role="group"
  ondragover={dragOverH}
  ondragleave={() => (dragOver = false)}
  ondrop={dropH}
>
  {#if collapsed}
    <div class="strip">
      <button class="icon-btn" use:tooltip={"Déplier le chat"} onclick={() => sessions.setCollapsed(sid, false)}>
        <span class="chev open"><Icon name="chevron" size={15} /></span>
      </button>
      <span class="status" class:live={session?.streaming}></span>
      <span class="strip-title">{session?.title ?? "Claude"}</span>
      <span class="strip-state" class:work={session?.streaming} use:tooltip={session?.streaming ? "Claude travaille" : "Inactif"}></span>
    </div>
  {:else}
    <PaneHeader {sid} {nodeId} {canMinimize} {canMove} {onsplit} {onclose} />
    <MessageLog {sid} />
    <Composer {sid} />
  {/if}
</div>

<style>
  .pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
    border: 1px solid var(--border);
    overflow: hidden;
    min-width: 0;
    min-height: 0;
    transition: border-color var(--transition), box-shadow var(--transition);
  }
  .pane.drag-over {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
  .chev {
    display: flex;
    transition: transform var(--transition);
  }
  .chev.open {
    transform: rotate(-90deg);
  }
  /* Bande latérale quand le chat est minimisé sur le côté */
  .strip {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    background: var(--surface-2);
  }
  .strip-title {
    writing-mode: vertical-rl;
    text-orientation: mixed;
    transform: rotate(180deg);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-height: 60%;
    margin: 2px 0;
  }
  .status {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-faint);
  }
  .status.live {
    background: var(--good);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--good) 25%, transparent);
    animation: pulseDot 1.6s ease-in-out infinite;
  }
  .strip-state {
    margin-top: auto;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-faint);
  }
  .strip-state.work {
    background: var(--good);
    animation: pulseDot 1.6s ease-in-out infinite;
  }
  @keyframes pulseDot {
    0%, 100% { box-shadow: 0 0 0 2px color-mix(in srgb, var(--good) 22%, transparent); }
    50% { box-shadow: 0 0 0 4px color-mix(in srgb, var(--good) 10%, transparent); }
  }
</style>
