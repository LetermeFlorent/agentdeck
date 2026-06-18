<script lang="ts">
  import type { Node } from "$lib/stores/layout.svelte";
  import { layout } from "$lib/stores/layout.svelte";
  import ChatPane from "./ChatPane.svelte";
  import Self from "./SplitContainer.svelte";
  import { scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let { node }: { node: Node } = $props();
</script>

{#if node.kind === "leaf"}
  <div class="leaf" in:scale={{ duration: 200, start: 0.97, opacity: 0, easing: cubicOut }}>
    <ChatPane
      sid={node.sid}
      nodeId={node.nodeId}
      onsplit={(dir) => layout.split(node.nodeId, dir)}
      onclose={() => layout.close(node.nodeId, node.sid)}
      onmove={(fromNodeId) => layout.swap(fromNodeId, node.nodeId)}
    />
  </div>
{:else}
  <div class="split" style={`flex-direction:${node.dir}`}>
    <div class="cell"><Self node={node.a} /></div>
    <div class="cell"><Self node={node.b} /></div>
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
    gap: var(--pane-gap);
  }
  .cell {
    flex: 1 1 0;
    display: flex;
  }
  .cell > :global(*) {
    flex: 1;
    min-width: 0;
    min-height: 0;
  }
</style>
