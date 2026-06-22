<script lang="ts">
  // Coquille de l'app connectée : topbar + deck (chats ou vue skills) + modales.
  import { layout } from "$lib/stores/layout.svelte";
  import { tabs } from "$lib/stores/tabs.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import { tour } from "$lib/stores/data/tour.svelte";
  import SplitContainer from "$lib/components/app/SplitContainer.svelte";
  import SettingsModal from "$lib/components/app/SettingsModal.svelte";
  import HistoryPopup from "$lib/components/app/HistoryPopup.svelte";
  import Tour from "$lib/components/app/Tour.svelte";
  import ObsidianView from "$lib/components/obsidian/ObsidianView.svelte";
  import Topbar from "./Topbar.svelte";

  let {
    plan,
    onlogout,
    onconnections,
  }: {
    plan: { label: string; level: number; account: string };
    onlogout: () => void;
    onconnections: () => void;
  } = $props();

  let showSettings = $state(false);
  let showHistory = $state(false);
  let showSkills = $state(false); // vue « Obsidian » (graphe skills) à la place du deck
  let historyNow = $state(0);

  // Depuis la vue skills : revient aux chats et saute sur le chat cliqué.
  function focusChat(sid: string) {
    showSkills = false;
    tabs.focusSession(sid);
    sessions.setFocused(sid);
  }
</script>

<div class="app">
  <Topbar
    {plan}
    bind:showSkills
    onhistory={() => { historyNow = Date.now(); showHistory = true; }}
    onsettings={() => (showSettings = true)}
    {onlogout}
  />

  <main class="deck">
    {#if showSkills}
      <ObsidianView onexit={() => (showSkills = false)} onopenchat={focusChat} />
    {:else if layout.root}
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
  <SettingsModal onclose={() => (showSettings = false)} onconnections={() => { showSettings = false; onconnections(); }} />
{/if}
{#if showHistory}
  <HistoryPopup now={historyNow} onclose={() => (showHistory = false)} />
{/if}
{#if tour.active}
  <Tour />
{/if}

<style>
  .app { display: flex; flex-direction: column; height: 100vh; }
  .deck { flex: 1; padding: 0; min-height: 0; overflow: hidden; }
  .deck-empty { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 14px; color: var(--text-muted); }
  .deck-empty p { margin: 0; }
</style>
