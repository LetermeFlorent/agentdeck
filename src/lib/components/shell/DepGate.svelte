<script lang="ts">
  // Écrans bloquants plein écran : Claude Code manquant, ou pas de connexion internet.
  import Icon from "$lib/components/ui/Icon.svelte";
  import * as ipc from "$lib/ipc";

  let {
    kind,
    installing = false,
    installErr = null,
    oninstall,
    onretry,
  }: {
    kind: "claude" | "offline";
    installing?: boolean;
    installErr?: string | null;
    oninstall?: () => void;
    onretry?: () => void;
  } = $props();
</script>

<div class="wrap" data-tauri-drag-region>
  <div class="dep">
    {#if kind === "claude"}
      <div class="dep-ic"><Icon name="terminal" size={26} stroke={1.6} /></div>
      <h1>Claude Code requis</h1>
      <p>agentdeck pilote le CLI <code>claude</code>. Il n'est pas installé sur ce PC — installe-le en un clic.</p>
      <button class="btn btn-accent" disabled={installing} onclick={oninstall}>
        {installing ? "Installation…" : "Installer Claude Code"}
      </button>
      {#if installErr}
        <div class="dep-err">{installErr}</div>
        <div class="dep-fallback">
          <button class="btn-link" disabled={installing} onclick={oninstall}>Réessayer</button>
          <span class="dep-sep">·</span>
          <button class="btn-link" onclick={() => ipc.openUrl("https://docs.claude.com/en/docs/claude-code/setup")}>
            Voir la doc d'installation
          </button>
        </div>
      {/if}
    {:else}
      <div class="dep-ic"><Icon name="wifi-off" size={26} stroke={1.6} /></div>
      <h1>Pas de connexion</h1>
      <p>agentdeck a besoin d'internet (connexion à Claude, login). Vérifie ton réseau puis réessaie.</p>
      <button class="btn btn-accent" onclick={onretry}>Réessayer</button>
    {/if}
  </div>
</div>

<style>
  .wrap { height: 100vh; display: grid; place-items: center; background: var(--bg); padding: 24px; }
  .dep { max-width: 380px; text-align: center; background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius); padding: 30px; }
  .dep-ic { color: var(--accent); margin-bottom: 10px; }
  .dep h1 { margin: 0 0 8px; font-family: var(--font-mono); font-size: 18px; }
  .dep p { color: var(--text-muted); font-size: 13px; line-height: 1.5; margin: 0 0 18px; }
  .dep code { font-family: var(--font-mono); background: var(--surface-2); padding: 1px 5px; border-radius: 4px; }
  .dep-err { margin-top: 14px; font-size: 12px; color: var(--danger); }
  .dep-fallback { margin-top: 10px; display: flex; align-items: center; justify-content: center; gap: 8px; font-size: 12px; }
  .btn-link { color: var(--accent); font-size: 12px; text-decoration: underline; text-underline-offset: 2px; }
  .btn-link:hover { color: var(--accent-hover); }
  .btn-link:disabled { opacity: 0.5; text-decoration: none; }
  .dep-sep { color: var(--text-faint); }
</style>
