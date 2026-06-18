<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";
  import Dropdown from "./Dropdown.svelte";
  import Icon from "./Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { fly, fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let { onclose }: { onclose: () => void } = $props();

  const MODELS = [
    { v: "opus", l: "Opus" },
    { v: "sonnet", l: "Sonnet" },
    { v: "haiku", l: "Haiku" },
    { v: "fable", l: "Fable" },
  ];
  const EFFORTS = [
    { v: "low", l: "Low" },
    { v: "medium", l: "Medium" },
    { v: "high", l: "High" },
    { v: "xhigh", l: "Xhigh" },
    { v: "max", l: "Max" },
  ];
  const models = $derived(MODELS.filter((m) => !settings.unavailableModels.includes(m.v)));
</script>

<div
  class="overlay"
  role="presentation"
  transition:fade={{ duration: 120 }}
  onclick={onclose}
  onkeydown={(e) => e.key === "Escape" && onclose()}
>
  <div
    class="modal"
    role="dialog"
    aria-label="Paramètres"
    tabindex="-1"
    transition:fly={{ y: 12, duration: 200, easing: cubicOut }}
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <header class="m-head">
      <span class="m-title">Paramètres</span>
      <button class="icon-btn" use:tooltip={"Fermer"} onclick={onclose}>
        <Icon name="close" />
      </button>
    </header>

    <div class="row">
      <div class="lbl">
        <span>Thème</span>
        <span class="sub">Apparence de l'app</span>
      </div>
      <ThemeToggle />
    </div>

    <div class="row">
      <div class="lbl">
        <span>Modèle par défaut</span>
        <span class="sub">Pour les nouveaux chats</span>
      </div>
      <Dropdown
        label="Modèle"
        options={models}
        value={settings.defaultModel ?? sessions.effModel ?? ""}
        onchange={(v) => settings.setDefaultModel(v)}
      />
    </div>

    <div class="row">
      <div class="lbl">
        <span>Effort par défaut</span>
        <span class="sub">low · medium · high · xhigh · max</span>
      </div>
      <Dropdown
        label="Effort"
        options={EFFORTS}
        value={settings.defaultEffort ?? sessions.effEffort ?? ""}
        onchange={(v) => settings.setDefaultEffort(v)}
      />
    </div>

    <button
      class="row check"
      onclick={() => settings.setRestore(!settings.restoreOnLaunch)}
    >
      <div class="lbl">
        <span>Réouvrir mes onglets</span>
        <span class="sub">À la fermeture, retrouver les mêmes chats et discussions</span>
      </div>
      <span class="switch" class:on={settings.restoreOnLaunch}><span class="knob"></span></span>
    </button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.4);
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .modal {
    width: 100%;
    max-width: 460px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.35);
    padding: 8px 18px 18px;
  }
  .m-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0 12px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 6px;
  }
  .m-title {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 14px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 13px 0;
    border-bottom: 1px solid var(--border);
    width: 100%;
    background: none;
    border-left: none;
    border-right: none;
    border-top: none;
    text-align: left;
  }
  .row:last-child {
    border-bottom: none;
  }
  .check {
    cursor: pointer;
  }
  .lbl {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .lbl > span:first-child {
    font-size: 13px;
    color: var(--text);
    font-weight: 500;
  }
  .sub {
    font-size: 11.5px;
    color: var(--text-muted);
  }
  .switch {
    flex-shrink: 0;
    width: 40px;
    height: 23px;
    border-radius: 999px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    position: relative;
    transition: background var(--transition), border-color var(--transition);
  }
  .switch.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 17px;
    height: 17px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
    transition: transform var(--transition);
  }
  .switch.on .knob {
    transform: translateX(17px);
  }
</style>
