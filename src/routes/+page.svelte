<script lang="ts">
  import { onMount } from "svelte";
  import { auth } from "$lib/stores/auth.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { usage } from "$lib/stores/usage.svelte";
  import { layout } from "$lib/stores/layout.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import * as persist from "$lib/stores/persist";
  import Onboarding from "$lib/components/Onboarding.svelte";
  import SplitContainer from "$lib/components/SplitContainer.svelte";
  import UsageBars from "$lib/components/UsageBars.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let initialized = $state(false);
  let booted = $state(false);

  onMount(async () => {
    theme.init();
    await auth.check();
  });

  // Démarre le deck une fois connecté : restaure les sessions persistées, sinon en crée une.
  $effect(() => {
    if (auth.connected && !initialized) {
      initialized = true;
      usage.start();
      bootDeck();
    }
  });

  async function bootDeck() {
    const saved = persist.load();
    if (saved && saved.sessions.length > 0) {
      await sessions.hydrate(saved.sessions);
      layout.restore(saved.root);
    } else {
      await layout.init();
    }
    booted = true;
  }

  // Sauvegarde automatique à chaque changement (après le boot, pour ne pas écraser l'état restauré).
  $effect(() => {
    // dépendances suivies
    void sessions.persistRev;
    void layout.root;
    if (booted) persist.save();
  });

  async function logout() {
    usage.stop();
    persist.clear();
    await auth.logout();
    layout.root = null;
    initialized = false;
    booted = false;
  }
</script>

{#if auth.checking}
  <div class="boot">
    <div class="boot-dot"></div>
    <span>agentdeck</span>
  </div>
{:else if !auth.connected}
  <Onboarding />
{:else}
  <div class="app">
    <header class="topbar">
      <div class="brand">
        <div class="dot"></div>
        <span class="logo">agentdeck</span>
      </div>
      <div class="spacer"></div>
      <UsageBars />
      <div class="divider"></div>
      <ThemeToggle />
      <button class="icon-btn" title="Se déconnecter" onclick={logout}>
        <Icon name="logout" size={16} />
      </button>
    </header>

    <main class="deck">
      {#if layout.root}
        <SplitContainer node={layout.root} />
      {:else}
        <div class="deck-empty">
          <p>Aucun pane ouvert.</p>
          <button class="btn btn-accent" onclick={() => layout.addRoot()}>Nouveau pane</button>
        </div>
      {/if}
    </main>
  </div>
{/if}

<style>
  .boot {
    height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    letter-spacing: 0.04em;
  }
  .boot-dot {
    width: 16px;
    height: 16px;
    border-radius: 5px;
    background: var(--accent);
    animation: pulse 1.1s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.4; transform: scale(0.9); }
    50% { opacity: 1; transform: scale(1.05); }
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .topbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 14px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 3px;
    background: var(--accent);
  }
  .logo {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 13.5px;
    letter-spacing: -0.01em;
  }
  .spacer {
    flex: 1;
  }
  .divider {
    width: 1px;
    height: 22px;
    background: var(--border);
  }
  .deck {
    flex: 1;
    padding: var(--pane-gap);
    min-height: 0;
    overflow: hidden;
  }
  .deck-empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    color: var(--text-muted);
  }
  .deck-empty p {
    margin: 0;
  }
</style>
