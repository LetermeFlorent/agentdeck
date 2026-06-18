<script lang="ts">
  import { onMount } from "svelte";
  import { auth } from "$lib/stores/auth.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { usage } from "$lib/stores/usage.svelte";
  import { layout } from "$lib/stores/layout.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import * as persist from "$lib/stores/persist";
  import { settings } from "$lib/stores/settings.svelte";
  import Onboarding from "$lib/components/app/Onboarding.svelte";
  import BootLoader from "$lib/components/app/BootLoader.svelte";
  import SplitContainer from "$lib/components/app/SplitContainer.svelte";
  import WindowControls from "$lib/components/ui/WindowControls.svelte";
  import UsageBars from "$lib/components/ui/UsageBars.svelte";
  import SettingsModal from "$lib/components/app/SettingsModal.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import * as ipc from "$lib/ipc";

  let initialized = $state(false);
  let booted = $state(false);
  let plan = $state<{ label: string; level: number }>({ label: "", level: 0 });
  let claudeReady = $state<boolean | null>(null);
  let installing = $state(false);
  let installErr = $state<string | null>(null);
  let showSettings = $state(false);
  let username = $state("");

  onMount(async () => {
    theme.init();
    settings.load();
    try {
      username = await ipc.osUsername();
    } catch {
      /* ignore */
    }
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
    // Clic sur l'icône de la barre des tâches → fenêtre au 1er plan → on stoppe le clignotement.
    window.addEventListener("focus", () => ipc.clearAttention());
    // F11 : bascule plein écran (la déco OS est désactivée, donc on le gère nous-mêmes).
    window.addEventListener("keydown", (e) => {
      if (e.key === "F11") {
        e.preventDefault();
        ipc.winToggleFullscreen();
      }
    });
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
    const t0 = Date.now();
    await sessions.loadDefaults();
    sessions.loadSlashCommands(); // pré-charge la liste des commandes "/" (cache + fetch)
    const saved = settings.restoreOnLaunch ? persist.load() : null;
    if (saved && saved.sessions.length > 0) {
      await sessions.hydrate(saved.sessions);
      layout.restore(saved.root);
    } else {
      await layout.init();
    }
    // Loader visible au moins 3 s (minimum, pas maximum).
    const wait = 3000 - (Date.now() - t0);
    if (wait > 0) await new Promise((r) => setTimeout(r, wait));
    booted = true;
    sessions.startPrivacyWatch(); // veille : passage auto en mode privé après inactivité
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
    sessions.stopPrivacyWatch();
    persist.clear();
    await auth.logout();
    layout.root = null;
    initialized = false;
    booted = false;
  }
</script>

{#if !(auth.connected && booted)}
  <WindowControls />
{/if}
{#if claudeReady === false}
  <div class="wrap" data-tauri-drag-region>
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
  <div class="boot" data-tauri-drag-region>
    <div class="boot-dot"></div>
    <span>agentdeck</span>
  </div>
{:else if !auth.connected}
  <Onboarding />
{:else if !booted}
  <BootLoader {username} />
{:else}
  <div class="app">
    <!-- Titlebar custom : barre app + contrôles système fusionnés (déco OS désactivée). -->
    <header class="topbar" data-tauri-drag-region>
      <div class="brand" data-tauri-drag-region>
        <div class="dot"></div>
        <span class="logo">agentdeck</span>
        {#if plan.label}
          <span class="plan plan-{plan.level}" use:tooltip={`Abonnement Claude : ${plan.label}`}>
            {plan.label}
          </span>
        {/if}
      </div>
      <div class="spacer" data-tauri-drag-region></div>
      <UsageBars />
      <div class="divider"></div>
      <button class="icon-btn" use:tooltip={"Paramètres"} onclick={() => (showSettings = true)}>
        <Icon name="settings" size={17} />
      </button>
      <button class="icon-btn" use:tooltip={"Se déconnecter"} onclick={logout}>
        <Icon name="logout" size={16} />
      </button>
      <div class="wsep"></div>
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
    gap: 10px;
    padding: 2px 12px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .topbar :global(.icon-btn) {
    width: 22px;
    height: 22px;
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
    font-size: 9px;
    padding: 2px 5px;
    color: var(--text-muted);
    background: var(--surface-2);
    border-color: var(--border);
  }
  .plan-2 {
    font-size: 9px;
    padding: 2px 5px;
    color: var(--accent);
    background: var(--accent-weak);
    border-color: color-mix(in srgb, var(--accent) 30%, transparent);
  }
  .plan-3 {
    font-size: 9.5px;
    padding: 2px 6px;
    color: #fff;
    background: linear-gradient(120deg, var(--accent), var(--accent-hover));
    box-shadow: 0 2px 10px color-mix(in srgb, var(--accent) 40%, transparent);
  }
  .plan-4 {
    font-size: 10.5px;
    padding: 2px 7px;
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
  /* Contrôles de fenêtre (réduire / agrandir / fermer) */
  .wsep {
    width: 1px;
    height: 22px;
    background: var(--border);
    margin-left: 4px;
  }
  .wctl {
    display: flex;
    align-items: stretch;
    gap: 0;
    /* Colle les contrôles au coin haut-droit (annule le padding droit de la barre). */
    margin: -2px -12px -2px 2px;
    align-self: stretch;
  }
  .wbtn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 38px;
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
  .deck {
    flex: 1;
    padding: 0;
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
