<script lang="ts">
  import type { Node } from "$lib/stores/layout.svelte";
  import { layout } from "$lib/stores/layout.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import ChatPane from "./ChatPane.svelte";
  import Self from "./SplitContainer.svelte";
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let { node, parentDir }: { node: Node; parentDir?: "row" | "column" } = $props();

  // Un panneau replié (collapsed) rétrécit au max : sa cellule ne prend que l'entête.
  const aMin = $derived(
    node.kind === "split" && node.a.kind === "leaf" && !!sessions.map[node.a.sid]?.collapsed,
  );
  const bMin = $derived(
    node.kind === "split" && node.b.kind === "leaf" && !!sessions.map[node.b.sid]?.collapsed,
  );

  let el = $state<HTMLDivElement>();

  // Redimensionnement : glisser le diviseur ajuste la part du panneau A.
  function startResize(e: PointerEvent) {
    if (node.kind !== "split" || !el) return;
    e.preventDefault();
    const split = node;
    const rect = el.getBoundingClientRect();
    const move = (ev: PointerEvent) => {
      const r =
        split.dir === "row"
          ? (ev.clientX - rect.left) / rect.width
          : (ev.clientY - rect.top) / rect.height;
      layout.setRatio(split.nodeId, r);
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      document.body.style.userSelect = "";
    };
    document.body.style.userSelect = "none";
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }
</script>

{#if node.kind === "leaf"}
  <div class="leaf" in:scale={{ duration: 200, start: 0.97, opacity: 0, easing: cubicOut }}>
    <ChatPane
      sid={node.sid}
      nodeId={node.nodeId}
      canMinimize={parentDir === "row"}
      onsplit={(dir) => layout.split(node.nodeId, dir)}
      onclose={() => layout.close(node.nodeId, node.sid)}
      onmove={(fromNodeId) => layout.swap(fromNodeId, node.nodeId)}
    />
  </div>
{:else}
  <div class="split" style={`flex-direction:${node.dir}`} bind:this={el}>
    <div class="cell" class:min={aMin} style={aMin ? "" : `flex-grow:${node.ratio}`}>
      <Self node={node.a} parentDir={node.dir} />
    </div>
    <div
      class="gutter {node.dir}"
      role="separator"
      aria-orientation={node.dir === "row" ? "vertical" : "horizontal"}
      onpointerdown={startResize}
    ></div>
    <div class="cell" class:min={bMin} style={bMin ? "" : `flex-grow:${1 - node.ratio}`}>
      <Self node={node.b} parentDir={node.dir} />
    </div>
  </div>
{/if}

<style>
  .leaf,
  .split,
  .cell {
    min-width: 0;
    min-height: 0;
  }
  .leaf {
    height: 100%;
  }
  .split {
    display: flex;
    height: 100%;
    width: 100%;
  }
  .cell {
    flex: 1 1 0;
    display: flex;
    flex-direction: column;
  }
  .cell > :global(*) {
    flex: 1 1 0;
    min-width: 0;
    min-height: 0;
  }
  /* cellule d'un panneau minimisé : bande latérale étroite */
  .cell.min {
    flex: 0 0 42px;
  }
  .gutter {
    flex: 0 0 auto;
    border: none;
    background: var(--bg);
    position: relative;
    z-index: 5;
    transition: background var(--transition);
  }
  .gutter.row {
    width: var(--pane-gap);
    cursor: col-resize;
  }
  .gutter.column {
    height: var(--pane-gap);
    cursor: row-resize;
  }
  /* zone de capture élargie autour de la ligne fine */
  .gutter::after {
    content: "";
    position: absolute;
    inset: 0;
  }
  .gutter.row::after {
    left: -3px;
    right: -3px;
  }
  .gutter.column::after {
    top: -3px;
    bottom: -3px;
  }
  .gutter:hover {
    background: var(--accent);
  }
</style>
