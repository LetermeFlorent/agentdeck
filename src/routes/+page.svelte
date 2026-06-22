<script lang="ts">
  import { onMount } from "svelte";
  import { auth } from "$lib/stores/auth.svelte";
  import { theme } from "$lib/stores/data/theme.svelte";
  import { usage } from "$lib/stores/data/usage.svelte";
  import { layout } from "$lib/stores/layout.svelte";
  import { tabs } from "$lib/stores/tabs.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import { tour } from "$lib/stores/data/tour.svelte";
  import { modelStore } from "$lib/stores/data/models.svelte";
  import * as persist from "$lib/stores/persist";
  import { settings } from "$lib/stores/settings.svelte";
  import Onboarding from "$lib/components/app/Onboarding.svelte";
  import BootLoader from "$lib/components/app/BootLoader.svelte";
  import WindowControls from "$lib/components/ui/WindowControls.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import DepGate from "$lib/components/shell/DepGate.svelte";
  import AppShell from "$lib/components/shell/AppShell.svelte";
  import { installShortcuts } from "$lib/actions/shortcuts";
  import { checkForUpdates } from "$lib/updater";
  import * as ipc from "$lib/ipc";

  let initialized = $state(false);
  let booted = $state(false);
  // Écran connexions forcé (logout / « Connexions IA ») ; remis à false au prochain démarrage.
  let forceOnboarding = $state(false);
  let plan = $state<{ label: string; level: number; account: string }>({ label: "", level: 0, account: "" });
  let claudeReady = $state<boolean | null>(null);
  let online = $state<boolean | null>(null);
  let installing = $state(false);
  let installErr = $state<string | null>(null);
  let username = $state("");

  onMount(async () => {
    theme.init();
    settings.load();
    checkForUpdates(); // auto-update en arrière-plan (silencieux si rien à faire)
    // Vérifs indépendantes en parallèle → durée ≈ la plus lente.
    await Promise.all([
      ipc.osUsername().then((u) => (username = u)).catch(() => {}),
      ipc.checkClaude().then((r) => (claudeReady = r)).catch(() => (claudeReady = false)),
      checkNet(),
      auth.check(),
    ]);
    ipc.subscriptionPlan().then((p) => (plan = p)).catch(() => {});
    // Reconnexion OS → re-vérifie ; focus fenêtre → stoppe le clignotement d'icône.
    window.addEventListener("online", checkNet);
    window.addEventListener("offline", () => (online = false));
    window.addEventListener("focus", () => ipc.clearAttention());
    installShortcuts(); // raccourcis clavier (onglets, chat focus, plein écran)
  });

  // Connexion internet : navigator.onLine (instantané) + ping backend (réel).
  async function checkNet() {
    online = null;
    if (!navigator.onLine) {
      online = false;
      return;
    }
    try {
      online = await ipc.netCheck();
    } catch {
      online = false;
    }
  }

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

  // Démarre le deck dès qu'une IA est connectée (restaure ou crée une session).
  $effect(() => {
    if (auth.anyConnected && !initialized && !forceOnboarding) {
      initialized = true;
      usage.start();
      modelStore.load(); // liste réelle des modèles (API), fallback si échec
      bootDeck();
    }
  });

  async function bootDeck() {
    const t0 = Date.now();
    await sessions.loadDefaults();
    sessions.loadSlashCommands("claude_code"); // pré-charge Claude (opencode = instantané)
    const saved = settings.restoreOnLaunch ? persist.load() : null;
    if (saved && saved.tabs.length > 0) {
      await sessions.hydrate(saved.sessions);
      tabs.hydrate(saved.tabs, saved.activeId);
    } else {
      await layout.init();
      tabs.initFromLayout();
    }
    const wait = 3000 - (Date.now() - t0); // loader visible ≥ 3 s même si déjà prêt
    if (wait > 0) await new Promise((r) => setTimeout(r, wait));
    booted = true;
    sessions.startPrivacyWatch(); // veille : passage auto en mode privé après inactivité
    setTimeout(() => tour.maybeStart(), 500); // tour guidé au 1er lancement (DOM peint)
  }

  // Sauvegarde auto à chaque changement (après boot, pour ne pas écraser l'état restauré).
  $effect(() => {
    void sessions.persistRev;
    void layout.root;
    void tabs.rev;
    if (booted) persist.save();
  });

  async function logout() {
    usage.stop();
    sessions.stopPrivacyWatch();
    persist.clear(); // efface l'état persisté du deck (localStorage)
    await sessions.closeAll(); // tue les process CLI + fichiers session backend
    await auth.logoutAll(); // déconnecte toutes les IA → retour onboarding
    tabs.reset();
    layout.root = null;
    initialized = false;
    booted = false;
    forceOnboarding = true; // bloque l'auto-entrée : on reste sur l'écran connexions
  }
</script>

{#if !(auth.anyConnected && booted && !forceOnboarding)}
  <WindowControls />
{/if}
{#if claudeReady === false}
  <DepGate kind="claude" {installing} {installErr} oninstall={installClaude} />
{:else if online === false}
  <DepGate kind="offline" onretry={checkNet} />
{:else if auth.checking || claudeReady === null || (auth.anyConnected && !booted && !forceOnboarding)}
  <BootLoader {username} />
{:else if !auth.anyConnected || forceOnboarding}
  <Onboarding />
  {#if auth.anyConnected}
    <button class="btn btn-accent onboarding-skip" onclick={() => (forceOnboarding = false)}>
      Continuer
      <Icon name="chevron" size={14} />
    </button>
  {/if}
{:else}
  <AppShell {plan} onlogout={logout} onconnections={() => (forceOnboarding = true)} />
{/if}

<style>
  /* Bouton « Continuer » : centré en bas, style btn-accent standard. */
  .onboarding-skip { position: fixed; bottom: 28px; left: 50%; transform: translateX(-50%); z-index: 110; display: inline-flex; align-items: center; gap: 8px; }
</style>

