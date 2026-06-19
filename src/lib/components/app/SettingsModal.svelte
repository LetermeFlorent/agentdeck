<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import ThemeToggle from "../ui/ThemeToggle.svelte";
  import Dropdown from "../ui/Dropdown.svelte";
  import Icon from "../ui/Icon.svelte";
  import SkillsView from "./SkillsView.svelte";
  import McpView from "./McpView.svelte";

  async function pickDefaultCwd() {
    const p = await ipc.pickFolder(settings.defaultCwd);
    if (p) settings.setDefaultCwd(p);
  }
  import { tooltip } from "$lib/actions/tooltip";
  import { MODELS, effortsFor, PERM_MODES } from "../chat/chat-config";
  import * as ipc from "$lib/ipc";
  import { onMount } from "svelte";
  import { fly, fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  // Niveaux d'effort détectés dynamiquement (pour les cases « auto effort »).
  let effLevels = $state<string[]>(["low", "medium", "high", "xhigh", "max"]);
  onMount(async () => {
    try {
      const l = await ipc.effortLevels();
      if (l.length) effLevels = l;
    } catch {
      /* garde la liste par défaut */
    }
  });

  let { onclose }: { onclose: () => void } = $props();

  // Vue active du modal : réglages | skills | serveurs MCP.
  let view = $state<"settings" | "skills" | "mcp">("settings");
  const titles = { settings: "Paramètres", skills: "Skills", mcp: "Serveurs MCP" };
  function toggle(v: "skills" | "mcp") {
    view = view === v ? "settings" : v;
  }

  const models = $derived(MODELS.filter((m) => !settings.unavailableModels.includes(m.v)));
  // Même logique que la listbox du chat : l'effort dépend du modèle (xhigh Opus/Fable, rien sur Haiku).
  const selModel = $derived(settings.defaultModel ?? sessions.effModel);
  const efforts = $derived(effortsFor(selModel));
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
      <span class="m-title">{titles[view]}</span>
      <div class="m-actions">
        <button
          class="icon-btn"
          class:on={view === "skills"}
          use:tooltip={"Skills"}
          onclick={() => toggle("skills")}
        >
          <Icon name="book" size={17} />
        </button>
        <button
          class="icon-btn"
          class:on={view === "mcp"}
          use:tooltip={"Serveurs MCP"}
          onclick={() => toggle("mcp")}
        >
          <Icon name="plug" size={17} />
        </button>
        <span class="m-sep"></span>
        <button class="icon-btn" use:tooltip={"Fermer"} onclick={onclose}>
          <Icon name="close" />
        </button>
      </div>
    </header>

    {#if view === "skills"}
      <SkillsView />
    {:else if view === "mcp"}
      <McpView />
    {:else}
    <div class="s-scroll">
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
        <span class="sub">
          {efforts.length ? efforts.map((e) => e.v).join(" · ") : "Non réglable sur ce modèle"}
        </span>
      </div>
      {#if efforts.length}
        <Dropdown
          label="Effort"
          options={efforts}
          value={settings.defaultEffort ?? sessions.effEffort ?? ""}
          onchange={(v) => settings.setDefaultEffort(v)}
        />
      {:else}
        <span class="na">—</span>
      {/if}
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
        <span>Permissions par défaut</span>
        <span class="sub">Mode appliqué aux nouveaux chats (réglable par chat ensuite)</span>
      </div>
      <Dropdown
        label="Mode"
        options={PERM_MODES}
        value={settings.defaultPermMode}
        onchange={(v) => settings.setDefaultPermMode(v)}
      />
    </div>

    <button
      class="row check"
      use:tooltip={"Choisit automatiquement l'effort adapté à chaque demande (mini-analyse Haiku, coût négligeable)"}
      onclick={() => settings.setAutoEffort(!settings.autoEffort)}
    >
      <div class="lbl">
        <span>Effort automatique</span>
        <span class="sub">Analyse ta demande et règle l'effort tout seul</span>
      </div>
      <span class="switch" class:on={settings.autoEffort}><span class="knob"></span></span>
    </button>

    {#if settings.autoEffort}
      <div class="sublist">
        <span class="sub">Efforts que l'auto peut choisir :</span>
        <div class="opts">
          {#each effLevels as e (e)}
            <button
              type="button"
              class="opt"
              class:on={settings.autoEfforts.includes(e)}
              onclick={() => settings.toggleAutoEffortChoice(e)}
            >{e}</button>
          {/each}
        </div>
      </div>

      <button
        class="row check"
        use:tooltip={"Choisit aussi le modèle (léger pour le simple, puissant pour le complexe)"}
        onclick={() => settings.setAutoModel(!settings.autoModel)}
      >
        <div class="lbl">
          <span>Modèle automatique</span>
          <span class="sub">Route vers le modèle adapté selon la demande</span>
        </div>
        <span class="switch" class:on={settings.autoModel}><span class="knob"></span></span>
      </button>

      {#if settings.autoModel}
        <div class="sublist">
          <span class="sub">Modèles que l'auto peut choisir :</span>
          <div class="opts">
            {#each MODELS as m (m.v)}
              <button
                type="button"
                class="opt"
                class:on={settings.autoModels.includes(m.v)}
                onclick={() => settings.toggleAutoModelChoice(m.v)}
              >{m.l}</button>
            {/each}
          </div>
        </div>
      {/if}
    {/if}

    <button
      class="row check"
      use:tooltip={"L'agent consulte ses skills, et capitalise ses erreurs en nouveaux skills (global ou projet)"}
      onclick={() => settings.setHermesMode(!settings.hermesMode)}
    >
      <div class="lbl">
        <span>Apprendre de ses erreurs (mode Hermes)</span>
        <span class="sub">Consulte ses skills avant d'agir · transforme ses échecs en skills réutilisables</span>
      </div>
      <span class="switch" class:on={settings.hermesMode}><span class="knob"></span></span>
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

    <button class="row check" onclick={pickDefaultCwd}>
      <div class="lbl">
        <span>Dossier de travail par défaut</span>
        <span class="sub">{settings.defaultCwd || "Dossier personnel"}</span>
      </div>
      <span class="na">Changer</span>
    </button>

    <div class="row">
      <div class="lbl">
        <span>Historique</span>
        <span class="sub">Nombre de conversations récentes affichées</span>
      </div>
      <div class="priv-ctl">
        {#each [15, 30, 50, 100] as n}
          <button
            type="button"
            class="chip"
            class:on={settings.historyLimit === n}
            onclick={() => settings.setHistoryLimit(n)}
          >{n}</button>
        {/each}
        <input
          class="num"
          type="number"
          min="1"
          max="200"
          step="1"
          aria-label="Nombre de conversations"
          value={settings.historyLimit}
          oninput={(e) => settings.setHistoryLimit(+e.currentTarget.value)}
        />
      </div>
    </div>
    </div>
    {/if}
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
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.35);
    padding: 8px 18px 14px;
  }
  /* Corps défilant : la popup ne déborde jamais, quel que soit le nombre de réglages. */
  .s-scroll {
    overflow-y: auto;
    overscroll-behavior: contain;
    margin: 0 -18px;
    padding: 0 18px;
    min-height: 0;
  }
  /* Sous-liste compacte (cases auto effort / modèle). */
  .sublist {
    padding: 8px 0 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .opts {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
  .opt {
    padding: 3px 9px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 11px;
    transition: background var(--transition), border-color var(--transition), color var(--transition);
  }
  .opt.on {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-weak);
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
  .m-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .m-sep {
    width: 1px;
    height: 18px;
    background: var(--border);
    margin: 0 4px;
  }
  .m-actions :global(.icon-btn.on) {
    color: var(--accent);
    background: var(--accent-weak);
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
  .na {
    color: var(--text-faint);
    font-family: var(--font-mono);
    font-size: 12px;
  }
</style>
