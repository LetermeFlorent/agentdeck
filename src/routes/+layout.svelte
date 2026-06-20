<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  let { children } = $props();

  // Fenêtre cachée au démarrage (tauri.conf visible:false) → on l'affiche une fois l'app montée
  // et peinte, pour ne jamais montrer un cadre blanc pendant l'init WebView2 + boot JS.
  onMount(() => {
    document.getElementById("preboot")?.remove();
    requestAnimationFrame(async () => {
      try {
        await getCurrentWindow().show();
      } catch {
        /* ignore (non-Tauri / déjà visible) */
      }
    });
  });
</script>

{@render children()}
