<script lang="ts">
  // Titlebar custom fusionnée : marque + onglets + jauges d'usage + actions + contrôles fenêtre.
  import { auth } from "$lib/stores/auth.svelte";
  import { usage } from "$lib/stores/data/usage.svelte";
  import UsageBars from "$lib/components/ui/UsageBars.svelte";
  import WinButtons from "$lib/components/ui/WinButtons.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import Brand from "./Brand.svelte";
  import TabStrip from "./TabStrip.svelte";

  let {
    plan,
    showSkills = $bindable(false),
    onhistory,
    onsettings,
    onlogout,
  }: {
    plan: { label: string; level: number; account: string };
    showSkills?: boolean;
    onhistory: () => void;
    onsettings: () => void;
    onlogout: () => void;
  } = $props();

  const geminiActive = $derived(auth.byProvider["gemini"] && usage.gemini && usage.gemini.five_h.tokens > 0);
  const opencodeActive = $derived(auth.byProvider["opencode"] && usage.opencode && usage.opencode.five_h.tokens > 0);
</script>

<header class="topbar" data-tauri-drag-region>
  <Brand {plan} bind:showSkills />

  <div class="divider" data-tauri-drag-region></div>
  <TabStrip />
  <div class="divider" data-tauri-drag-region></div>

  <div class="spacer" data-tauri-drag-region></div>
  {#if auth.connected}
    <span class="usage-wrap" data-tour="usage"><UsageBars collapse={!!(geminiActive || opencodeActive)} /></span>
  {/if}
  {#if geminiActive}
    <div class="divider" data-tauri-drag-region></div>
    <span class="usage-wrap"><UsageBars snapshot={usage.gemini} label="Gemini" singleBar={true} /></span>
  {/if}
  {#if opencodeActive}
    <div class="divider" data-tauri-drag-region></div>
    <span class="usage-wrap"><UsageBars snapshot={usage.opencode} label="opencode" singleBar={true} /></span>
  {/if}
  <div class="divider" data-tauri-drag-region></div>
  <button class="icon-btn" data-tour="history" use:tooltip={"Historique des conversations"} onclick={onhistory}>
    <Icon name="history" size={17} />
  </button>
  <button class="icon-btn" data-tour="settings" use:tooltip={"Paramètres"} onclick={onsettings}>
    <Icon name="settings" size={17} />
  </button>
  <button class="icon-btn" use:tooltip={"Se déconnecter"} onclick={onlogout}>
    <Icon name="logout" size={16} />
  </button>
  <div class="wsep" data-tauri-drag-region></div>
  <div class="wctl"><WinButtons /></div>
</header>

<style>
  .topbar { display: flex; align-items: center; gap: 10px; padding: 2px 12px; background: var(--surface); border-bottom: 1px solid var(--border); flex-shrink: 0; }
  .topbar :global(.icon-btn) { width: 22px; height: 22px; }
  .spacer { flex: 1; }
  .usage-wrap { display: inline-flex; align-items: center; }
  .divider { width: 1px; height: 22px; background: var(--border); }
  .wsep { width: 1px; height: 22px; background: var(--border); margin-left: 4px; }
  .wctl { display: flex; align-items: stretch; gap: 0; margin: -2px -12px -2px 2px; align-self: stretch; }
</style>
