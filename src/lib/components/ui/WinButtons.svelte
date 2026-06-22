<script lang="ts">
  // Boutons système réduire / agrandir / fermer (partagés : overlay hors-app + topbar).
  // Le conteneur (.titlebar ou .wctl) et sa hauteur sont fournis par le parent.
  import * as ipc from "$lib/ipc";
  import Icon from "./Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
</script>

<button class="wbtn" use:tooltip={"Réduire"} onclick={() => ipc.winMinimize()}>
  <Icon name="win-min" size={14} />
</button>
<button class="wbtn" use:tooltip={"Agrandir / restaurer"} onclick={() => ipc.winToggleMaximize()}>
  <Icon name="win-max" size={12} />
</button>
<button class="wbtn close" use:tooltip={"Fermer"} onclick={() => ipc.winClose()}>
  <Icon name="win-close" size={14} />
</button>

<style>
  .wbtn {
    width: 38px;
    height: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    transition: background var(--transition), color var(--transition);
  }
  .wbtn:hover {
    background: var(--surface-2);
    color: var(--text);
  }
  .wbtn.close:hover {
    background: var(--danger);
    color: #fff;
  }
</style>
