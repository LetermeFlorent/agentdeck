<script lang="ts">
  import { onMount } from "svelte";
  import { auth } from "$lib/stores/auth.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { usage } from "$lib/stores/usage.svelte";
  import { layout } from "$lib/stores/layout.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import * as persist from "$lib/stores/persist";
  import { settings } from "$lib/stores/settings.svelte";
  import Onboarding from "$lib/components/Onboarding.svelte";
  import SplitContainer from "$lib/components/SplitContainer.svelte";
  import UsageBars from "$lib/components/UsageBars.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import * as ipc from "$lib/ipc";

  let initialized = $state(false);
  let booted = $state(false);
  let plan = $state<{ label: string; level: number }>({ label: "", level: 0 });
  let claudeReady = $state<boolean | null>(null);
  let installing = $state(false);
  let installErr = $state<string | null>(null);
  let showSettings = $state(false);

  onMount(async () => {
    theme.init();
    settings.load();
    try {
      claudeReady = await ipc.checkClaude();
    } catch {
      claudeReady = false;
    }
    await auth.check();
    try {
      plan = await ipc.subscriptionPlan();
    } catch {
      /* ignore */
    }
  });

  async function installClaude() {
    installing = true;
    installErr = null;
    try {
      await ipc.installClaude();
      claudeReady = await ipc.checkClaude();
      if (!claudeReady) installErr = "Installation faite, mais `claude` pas détecté — redémarre l'app.";
    } catch (e) {
      installErr = String(e);
    } finally {
      installing = false;
    }
  }

  // Démarre le deck une fois connecté : restaure les sessions persistées, sinon en crée une.
  $effect(() => {
    if (auth.connected && !initialized) {
      initialized = true;
      usage.start();
      bootDeck();
    }
  });

  async function bootDeck() {
    await sessions.loadDefaults();
    const saved = settings.restoreOnLaunch ? persist.load() : null;
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

{#if claudeReady === false}
  <div class="wrap">
    <div class="dep">
      <div class="dep-ic"><Icon name="terminal" size={26} stroke={1.6} /></div>
      <h1>Claude Code requis</h1>
      <p>agentdeck pilote le CLI <code>claude</code>. Il n'est pas installé sur ce PC — installe-le en un clic.</p>
      <button class="btn btn-accent" disabled={installing} onclick={installClaude}>
        {installing ? "Installation…" : "Installer Claude Code"}
      </button>
      {#if installErr}<div class="dep-err">{installErr}</div>{/if}
    </div>
  </div>
{:else if auth.checking || claudeReady === null}
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
        {#if plan.label}
          <span class="plan plan-{plan.level}" use:tooltip={`Abonnement Claude : ${plan.label}`}>
            {plan.label}
          </span>
        {/if}
      </div>
      <div class="spacer"></div>
      <UsageBars />
      <div class="divider"></div>
      <button class="icon-btn" use:tooltip={"Paramètres"} onclick={() => (showSettings = true)}>
        <Icon name="settings" size={17} />
      </button>
      <button class="icon-btn" use:tooltip={"Se déconnecter"} onclick={logout}>
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

  {#if showSettings}
    <SettingsModal onclose={() => (showSettings = false)} />
  {/if}
{/if}

<style>
  .wrap {
    height: 100vh;
    display: grid;
    place-items: center;
    background: var(--bg);
    padding: 24px;
  }
  .dep {
    max-width: 380px;
    text-align: center;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 30px;
  }
  .dep-ic {
    color: var(--accent);
    margin-bottom: 10px;
  }
  .dep h1 {
    margin: 0 0 8px;
    font-family: var(--font-mono);
    font-size: 18px;
  }
  .dep p {
    color: var(--text-muted);
    font-size: 13px;
    line-height: 1.5;
    margin: 0 0 18px;
  }
  .dep code {
    font-family: var(--font-mono);
    background: var(--surface-2);
    padding: 1px 5px;
    border-radius: 4px;
  }
  .dep-err {
    margin-top: 14px;
    font-size: 12px;
    color: var(--danger);
  }
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
    padding: 4px 14px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .topbar :global(.icon-btn) {
    width: 24px;
    height: 24px;
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
  /* Badge d'abonnement : plus le plan est élevé, plus l'effet est marqué. */
  .plan {
    margin-left: 2px;
    font-family: var(--font-mono);
    font-weight: 700;
    line-height: 1;
    border-radius: 5px;
    white-space: nowrap;
    border: 1px solid transparent;
  }
  .plan-1 {
    font-size: 10px;
    padding: 3px 6px;
    color: var(--text-muted);
    background: var(--surface-2);
    border-color: var(--border);
  }
  .plan-2 {
    font-size: 10.5px;
    padding: 3px 7px;
    color: var(--accent);
    background: var(--accent-weak);
    border-color: color-mix(in srgb, var(--accent) 30%, transparent);
  }
  .plan-3 {
    font-size: 11.5px;
    padding: 3px 8px;
    color: #fff;
    background: linear-gradient(120deg, var(--accent), var(--accent-hover));
    box-shadow: 0 2px 10px color-mix(in srgb, var(--accent) 40%, transparent);
  }
  .plan-4 {
    font-size: 13px;
    padding: 4px 10px;
    color: #fff;
    background: linear-gradient(120deg, #e0825f, var(--accent) 45%, #b94f9e);
    background-size: 200% 100%;
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 35%, transparent),
      0 3px 16px color-mix(in srgb, var(--accent) 55%, transparent);
    animation: planShimmer 4s linear infinite, planGlow 2.6s ease-in-out infinite;
    text-shadow: 0 0 8px rgba(255, 255, 255, 0.35);
  }
  @keyframes planShimmer {
    0% { background-position: 0% 0; }
    100% { background-position: 200% 0; }
  }
  @keyframes planGlow {
    0%, 100% { box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 35%, transparent), 0 3px 14px color-mix(in srgb, var(--accent) 45%, transparent); }
    50% { box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 50%, transparent), 0 4px 22px color-mix(in srgb, var(--accent) 70%, transparent); }
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
