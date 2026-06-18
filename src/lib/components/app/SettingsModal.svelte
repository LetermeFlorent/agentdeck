<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import ThemeToggle from "../ui/ThemeToggle.svelte";
  import Dropdown from "../ui/Dropdown.svelte";
  import Icon from "../ui/Icon.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { MODELS, EFFORTS, ULTRACODE } from "../chat/chat-config";
  import { fly, fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let { onclose }: { onclose: () => void } = $props();

  const models = $derived(MODELS.filter((m) => !settings.unavailableModels.includes(m.v)));
  // Même logique que la listbox du chat : l'effort dépend du modèle (ultracode exclusif Opus).
  const selModel = $derived(settings.defaultModel ?? sessions.effModel);
  const efforts = $derived(selModel === "opus" ? [...EFFORTS, ULTRACODE] : EFFORTS);
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
        <span class="sub">{efforts.map((e) => e.v).join(" · ")}</span>
      </div>
      <Dropdown
        label="Effort"
        options={efforts}
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

    <div class="row">
      <div class="lbl">
        <span>Mode privé auto</span>
        <span class="sub">Floute un chat après X min sans activité · 0 = jamais</span>
      </div>
      <div class="priv-ctl">
        {#each [0, 5, 15, 30] as m}
          <button
            type="button"
            class="chip"
            class:on={(settings.privateAfterMin ?? 0) === m}
            use:tooltip={m === 0 ? "Désactivé" : `Après ${m} min d'inactivité`}
            onclick={() => settings.setPrivateAfterMin(m)}
          >{m === 0 ? "Off" : `${m}m`}</button>
        {/each}
        <input
          class="num"
          type="number"
          min="0"
          max="240"
          step="1"
          aria-label="Délai personnalisé en minutes"
          use:tooltip={"Délai personnalisé (minutes)"}
          value={settings.privateAfterMin ?? 0}
          oninput={(e) => settings.setPrivateAfterMin(+e.currentTarget.value)}
        />
        <span class="unit">min</span>
      </div>
    </div>

    <div class="row">
      <div class="lbl">
        <span>Zoom par défaut</span>
        <span class="sub">Taille du texte des nouveaux chats</span>
      </div>
      <div class="priv-ctl">
        {#each [0.8, 0.9, 1, 1.1, 1.25] as z}
          <button
            type="button"
            class="chip"
            class:on={settings.defaultZoom === z}
            onclick={() => settings.setDefaultZoom(z)}
          >{Math.round(z * 100)}%</button>
        {/each}
      </div>
    </div>
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

  /* Réglage mode privé auto : presets + temps custom */
  .priv-ctl {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
  }
  .chip {
    min-width: 30px;
    height: 24px;
    padding: 0 7px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 11px;
    transition: border-color var(--transition), color var(--transition), background var(--transition);
  }
  .chip:hover {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .chip.on {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-weak);
  }
  .num {
    width: 50px;
    height: 24px;
    margin-left: 4px;
    padding: 0 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 11px;
    text-align: right;
    outline: none;
  }
  .num:focus {
    border-color: var(--accent);
  }
  .unit {
    font-size: 11px;
    color: var(--text-faint);
    font-family: var(--font-mono);
  }
</style>
