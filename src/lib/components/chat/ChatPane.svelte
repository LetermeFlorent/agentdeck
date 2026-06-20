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
    collapseSide,
    collapseDir,
    siblingCollapsed = false,
    canMove = false,
    onsplit,
    onclose,
    onmove,
  }: {
    sid: string;
    nodeId: string;
    canMinimize?: boolean;
    collapseSide?: "a" | "b";
    collapseDir?: "row" | "column";
    siblingCollapsed?: boolean;
    canMove?: boolean;
    onsplit: (dir: "row" | "column") => void;
    onclose: () => void;
    onmove: (fromNodeId: string) => void;
  } = $props();

  const session = $derived(sessions.map[sid]);
  const collapsed = $derived(session?.collapsed ?? false);
  // Flèche de dépli : pointe vers où le pane va s'ouvrir (opposé du bord de repli).
  // chevron de base = bas. column: a(haut)→bas(0) b(bas)→haut(180) ; row: a(gauche)→droite(-90) b(droite)→gauche(90).
  const expandRot = $derived(
    collapseDir === "column"
      ? collapseSide === "a" ? 0 : 180
      : collapseSide === "a" ? -90 : 90,
  );

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
    <div
      class="strip"
      class:horizontal={collapseDir === "column"}
      role="button"
      tabindex="0"
      use:tooltip={`${session?.title ?? "Claude"} — déplier`}
      onclick={() => { sessions.setCollapsed(sid, false); sessions.wake(sid); }}
      onkeydown={(e) => e.key === "Enter" && (sessions.setCollapsed(sid, false), sessions.wake(sid))}
    >
      <span class="chev" style={`transform: rotate(${expandRot}deg)`}><Icon name="chevron" size={14} /></span>
      <span class="status" class:live={session?.streaming}></span>
      <span class="strip-title">{session?.title ?? "Claude"}</span>
      <span class="strip-state" class:work={session?.streaming}></span>
    </div>
  {:else}
    <PaneHeader {sid} {nodeId} {canMinimize} {collapseSide} {collapseDir} {siblingCollapsed} {canMove} {onsplit} {onclose} />
    <MessageLog {sid} />
    <Composer {sid} />
    {#if session?.asleep}
      <button class="sleep-ov" onclick={() => sessions.wake(sid)} use:tooltip={"Clique pour réveiller ce chat"}>
        <span class="sl-ic"><Icon name="sleep" size={24} /></span>
        <span class="sl-t">Chat en veille</span>
        <span class="sl-s">Mémoire libérée · clique pour reprendre</span>
      </button>
    {/if}
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
    position: relative; /* ancre le composer en overlay (absolute) */
    transition: border-color var(--transition), box-shadow var(--transition);
  }
  .pane.drag-over {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
  .chev {
    display: flex;
    transition: transform var(--transition), color var(--transition);
  }
  /* Overlay « en veille » : couvre le chat suspendu, clic = réveil. */
  .sleep-ov {
    position: absolute;
    inset: 0;
    z-index: 5;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background: color-mix(in srgb, var(--surface) 78%, transparent);
    backdrop-filter: blur(3px);
    cursor: pointer;
    transition: background var(--transition);
  }
  .sleep-ov:hover {
    background: color-mix(in srgb, var(--surface) 64%, transparent);
  }
  .sl-ic {
    display: flex;
    color: var(--text-muted);
    opacity: 0.8;
    animation: sleepPulse 2.4s ease-in-out infinite;
  }
  .sl-t {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text);
  }
  .sl-s {
    font-size: 11px;
    color: var(--text-muted);
  }
  @keyframes sleepPulse {
    0%, 100% { opacity: 0.45; }
    50% { opacity: 0.9; }
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
    cursor: pointer;
    transition: background var(--transition);
  }
  .strip:hover {
    background: var(--elevated);
  }
  .strip:hover .chev {
    color: var(--accent);
  }
  /* Bande horizontale (chat minimisé en haut/bas d'un split vertical). */
  .strip.horizontal {
    flex-direction: row;
    padding: 0 6px;
  }
  .strip.horizontal .strip-title {
    writing-mode: horizontal-tb;
    transform: none;
    max-height: none;
    max-width: 60%;
  }
  .strip.horizontal .strip-state {
    margin-top: 0;
    margin-left: auto;
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
