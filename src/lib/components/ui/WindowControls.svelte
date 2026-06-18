<script lang="ts">
  // Barre système minimale (drag + réduire/agrandir/fermer) pour les écrans hors « app »
  // (connexion, boot…) où la topbar fusionnée n'est pas affichée.
  import * as ipc from "$lib/ipc";
  import Icon from "./Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="wctl">
    <button class="wbtn" use:tooltip={"Réduire"} onclick={() => ipc.winMinimize()}>
      <Icon name="win-min" size={14} />
    </button>
    <button class="wbtn" use:tooltip={"Agrandir / restaurer"} onclick={() => ipc.winToggleMaximize()}>
      <Icon name="win-max" size={12} />
    </button>
    <button class="wbtn close" use:tooltip={"Fermer"} onclick={() => ipc.winClose()}>
      <Icon name="win-close" size={14} />
    </button>
  </div>
</div>

<style>
  .titlebar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 32px;
    display: flex;
    justify-content: flex-end;
    z-index: 300;
  }
  .wctl {
    display: flex;
  }
  .wbtn {
    width: 38px;
    height: 32px;
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
