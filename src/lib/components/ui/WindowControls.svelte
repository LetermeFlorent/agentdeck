<script lang="ts">
  // Barre système minimale (drag + réduire/agrandir/fermer) pour les écrans hors « app »
  // (connexion, boot…) où la topbar fusionnée n'est pas affichée.
  import WinButtons from "./WinButtons.svelte";
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="wctl"><WinButtons /></div>
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
    height: 32px;
  }
</style>
