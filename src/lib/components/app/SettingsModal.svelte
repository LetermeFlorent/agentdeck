<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import { sessions } from "$lib/stores/sessions.svelte";
  import ThemeToggle from "../ui/ThemeToggle.svelte";
  import Dropdown from "../ui/Dropdown.svelte";
  import Icon from "../ui/Icon.svelte";
  import SkillsView from "./SkillsView.svelte";
  import McpView from "./McpView.svelte";
  import { tour } from "$lib/stores/tour.svelte";

  async function pickDefaultCwd() {
    const p = await ipc.pickFolder(settings.defaultCwd);
    if (p) settings.setDefaultCwd(p);
  }
  import { tooltip } from "$lib/actions/tooltip";
  import { effortsFor, effortsForProvider, PERM_MODES, PROVIDERS, tierOf, priceHint } from "../chat/chat-config";
  import { modelStore } from "$lib/stores/models.svelte";

  // Active l'auto-modèle ; à l'activation, coche par défaut tous les modèles dispo sauf Fable
  // (si aucun choix valide actuel).
  function toggleAutoModel() {
    const on = !settings.autoModel;
    settings.setAutoModel(on);
    if (on) {
      // Pré-coche pour chaque provider connecté s'il n'a aucun candidat valide.
      for (const p of PROVIDERS) {
        if (!auth.isConnected(p.id)) continue;
        const list = modelStore.visibleFor(p.id);
        const valid = settings.autoModelsFor(p.id).filter((v) => list.some((m) => m.v === v));
        if (!valid.length && list.length) {
          const free = list.filter((m) => /[-:]free$/.test(m.v));
          const def = free.length
            ? free.map((m) => m.v)
            : list.filter((m) => tierOf(m.v) !== "fable").map((m) => m.v);
          settings.setAutoModels(p.id, def);
        }
      }
    }
  }
  import * as ipc from "$lib/ipc";
  import { onMount } from "svelte";
  import { fly, fade, slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { auth } from "$lib/stores/auth.svelte";

  // Niveaux d'effort détectés dynamiquement (pour les cases « auto effort »).
  let effLevels = $state<string[]>(["low", "medium", "high", "xhigh", "max"]);
  onMount(async () => {
    try {
      const l = await ipc.effortLevels();
      if (l.length) effLevels = l;
    } catch {
      /* garde la liste par défaut */
    }
    await auth.refresh(); // rafraîchit l'état connecté de chaque IA (sans toucher `checking`)
    // Précharge les modèles de tous les providers connectés dès l'ouverture de la modal.
    for (const p of PROVIDERS) {
      if (auth.isConnected(p.id)) modelStore.loadFor(p.id);
    }
  });

  // onconnections : « Connexions IA » → ferme la modale et ouvre l'écran connexions plein écran (parent).
  let { onclose, onconnections }: { onclose: () => void; onconnections: () => void } = $props();

  // Vue active du modal : réglages | skills | serveurs MCP.
  let view = $state<"settings" | "skills" | "mcp">("settings");
  let settingsTab = $state<"ia" | "agent" | "app" | "general">("ia");
  const titles = { settings: "Paramètres", skills: "Skills", mcp: "Serveurs MCP" };
  function toggle(v: "skills" | "mcp") {
    view = view === v ? "settings" : v;
  }


  // --- Section Auto par IA : providers indépendants pour efforts vs modèles ---
  const autoProvOpts = $derived(
    PROVIDERS.filter((p) => auth.isConnected(p.id)).map((p) => ({ v: p.id, l: p.label })),
  );
  let autoProvEff = $state("claude_code");
  let autoProvMod = $state("claude_code");
  // Si une IA déconnectée, bascule sur la 1ʳᵉ connectée.
  $effect(() => {
    if (autoProvOpts.length && !autoProvOpts.some((o) => o.v === autoProvEff)) autoProvEff = autoProvOpts[0].v;
    if (autoProvOpts.length && !autoProvOpts.some((o) => o.v === autoProvMod)) autoProvMod = autoProvOpts[0].v;
  });
  const autoCandEfforts = $derived(
    autoProvEff === "claude_code" ? effLevels : effortsForProvider(autoProvEff, null).map((e) => e.v),
  );
  const autoCandModels = $derived(modelStore.visibleFor(autoProvMod));
  // Charge les modèles à la demande.
  $effect(() => { modelStore.loadFor(autoProvMod); });
  // Defaults efforts.
  $effect(() => {
    if (settings.autoEffort && settings.autoEffortsUnset(autoProvEff) && autoCandEfforts.length) {
      settings.setAutoEfforts(autoProvEff, [...autoCandEfforts]);
    }
  });
  // Defaults modèles.
  $effect(() => {
    if (settings.autoModel && settings.autoModelsUnset(autoProvMod) && autoCandModels.length) {
      const free = autoCandModels.filter((m) => /[-:]free$/.test(m.v));
      const def = free.length
        ? free.map((m) => m.v)
        : autoCandModels.filter((m) => tierOf(m.v) !== "fable").map((m) => m.v);
      settings.setAutoModels(autoProvMod, def);
    }
  });
  // Picker GLOBAL : tous les modèles des IA connectées, label préfixé par l'IA.
  const pickerOptions = $derived(
    PROVIDERS.filter((p) => auth.isConnected(p.id)).flatMap((p) =>
      modelStore.visibleFor(p.id).map((m) => ({ v: m.v, l: `${p.label}: ${m.l}`, hint: priceHint(m.v) })),
    ),
  );
  // Picker Claude uniquement (pour Hermes — reflect_and_learn utilise claude_bin exclusivement).
  const claudePickerOptions = $derived(
    modelStore.visibleFor("claude_code").map((m) => ({ v: m.v, l: m.l, hint: priceHint(m.v) })),
  );
  // Filtre de réduction des candidats (ex. taper "claude" / "opencode" / "gpt").
  let candQuery = $state("");
  const autoCandShown = $derived(
    candQuery.trim()
      ? autoCandModels.filter((m) => (m.l + " " + m.v).toLowerCase().includes(candQuery.trim().toLowerCase()))
      : autoCandModels,
  );
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
    <div class="s-tabs">
      <button class="s-tab" class:on={settingsTab === "ia"} onclick={() => (settingsTab = "ia")}>IA</button>
      <button class="s-tab" class:on={settingsTab === "agent"} onclick={() => (settingsTab = "agent")}>Agent</button>
      <button class="s-tab" class:on={settingsTab === "app"} onclick={() => (settingsTab = "app")}>App</button>
      <button class="s-tab" class:on={settingsTab === "general"} onclick={() => (settingsTab = "general")}>Général</button>
    </div>
    <div class="s-scroll">

      {#if settingsTab === "ia"}

      <!-- ── IA & Connexions ────────────────────────────── -->
      <div class="s-section">IA & Connexions</div>

      <button class="row nav-row" use:tooltip={"Connecte ou déconnecte Claude, opencode et Gemini"} onclick={onconnections}>
        <div class="lbl">
          <span>Connexions IA</span>
          <span class="sub">
            {#if autoProvOpts.length}
              Connecté : {autoProvOpts.map((p) => p.l).join(" · ")}
            {:else}
              Aucune IA connectée
            {/if}
          </span>
        </div>
        <span class="chev-right"><Icon name="chevron" size={16} /></span>
      </button>

      <div class="row" use:tooltip={"Contrôle ce que l'agent peut faire sans demander : bypassPermissions = tout autoriser, plan = lecture seule"}>
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

      <!-- ── Mode Auto ──────────────────────────────────── -->
      <div class="s-section">Mode Auto</div>

      <button
        class="row check"
        use:tooltip={"Choisit automatiquement l'effort adapté à chaque demande (mini-analyse Haiku, coût négligeable)"}
        onclick={() => settings.setAutoEffort(!settings.autoEffort)}
      >
        <div class="lbl">
          <span>Effort automatique <span class="cbadge save" use:tooltip={"Économise : route vers un effort moins coûteux quand c'est simple"}><Icon name="coin-down" size={11} /></span></span>
          <span class="sub">Analyse ta demande et règle l'effort tout seul</span>
        </div>
        <span class="switch" class:on={settings.autoEffort}><span class="knob"></span></span>
      </button>

      {#if settings.autoEffort}
        <div class="sublist" transition:slide={{ duration: 150 }}>
          <div class="sublist-head">
            <span class="sub">Efforts :</span>
            <div class="prov-tabs">
              {#each autoProvOpts as p}
                <button type="button" class="prov-tab" class:on={autoProvEff === p.v} onclick={() => (autoProvEff = p.v)}>{p.l}</button>
              {/each}
            </div>
          </div>
          <div class="opts">
            {#each autoCandEfforts as e (e)}
              <button
                type="button"
                class="opt"
                class:on={settings.autoEffortsFor(autoProvEff).includes(e)}
                onclick={() => settings.toggleAutoEffortChoice(autoProvEff, e)}
              >{e}</button>
            {:else}
              <span class="sub">Cette IA n'a pas d'effort réglable.</span>
            {/each}
          </div>
        </div>

        <button
          class="row check"
          use:tooltip={"Choisit aussi le modèle (léger pour le simple, puissant pour le complexe)"}
          onclick={toggleAutoModel}
        >
          <div class="lbl">
            <span>Modèle automatique <span class="cbadge save" use:tooltip={"Économise : route vers un modèle moins cher quand c'est simple"}><Icon name="coin-down" size={11} /></span></span>
            <span class="sub">Route vers le modèle adapté selon la demande</span>
          </div>
          <span class="switch" class:on={settings.autoModel}><span class="knob"></span></span>
        </button>

        {#if settings.autoModel}
          <div class="sublist" transition:slide={{ duration: 150 }}>
            <div class="sublist-head">
              <span class="sub">Modèles :</span>
              <div class="prov-tabs">
                {#each autoProvOpts as p}
                  <button type="button" class="prov-tab" class:on={autoProvMod === p.v} onclick={() => (autoProvMod = p.v)}>{p.l}</button>
                {/each}
              </div>
              {#if autoCandModels.length > 8}
                <input class="cand-search" type="text" placeholder="filtrer…" bind:value={candQuery} />
              {/if}
            </div>
            <div class="opts">
              {#each autoCandShown as m (m.v)}
                <button
                  type="button"
                  class="opt"
                  class:on={settings.autoModelsFor(autoProvMod).includes(m.v)}
                  onclick={() => settings.toggleAutoModelChoice(autoProvMod, m.v)}
                >{m.l}{#if priceHint(m.v)}<span class="opt-hint">{priceHint(m.v)}</span>{/if}</button>
              {:else}
                <span class="sub">Aucun modèle{candQuery ? " pour ce filtre" : " détecté pour cette IA"}.</span>
              {/each}
            </div>
          </div>
        {/if}

        <div class="row" use:tooltip={"Ce modèle lit ta demande et décide quel modèle candidat utiliser — prend un modèle rapide et peu coûteux"}>
          <div class="lbl">
            <span>Modèle qui choisit (global)</span>
            <span class="sub">UN décideur · choisit l'IA + le modèle parmi tous les candidats cochés</span>
          </div>
          <Dropdown
            label="Auto"
            options={pickerOptions}
            maxVisible={8}
            value={settings.autoPicker || modelStore.pickerDefaultFor("claude_code")}
            onchange={(v) => settings.setAutoPicker(v)}
          />
        </div>
      {/if}

      {:else if settingsTab === "agent"}

      <!-- ── Agent & Apprentissage ──────────────────────── -->
      <div class="s-section">Agent & Apprentissage</div>

      <button
        class="row check"
        use:tooltip={"L'agent consulte ses skills, et capitalise ses erreurs en nouveaux skills (global ou projet)"}
        onclick={() => settings.setHermesMode(!settings.hermesMode)}
      >
        <div class="lbl">
          <span>Apprendre des erreurs <span class="cbadge cost" use:tooltip={"Consomme plus : appel IA supplémentaire à chaque échec"}><Icon name="coin-up" size={11} /></span></span>
          <span class="sub">Transforme ses échecs en skills · plus le temps passe, moins de tokens sont consommés (connaît déjà)</span>
        </div>
        <span class="switch" class:on={settings.hermesMode}><span class="knob"></span></span>
      </button>

      <div class="row" use:tooltip={"Modèle appelé après chaque échec pour en tirer une leçon — préfère un modèle rapide et peu cher"}>
        <div class="lbl">
          <span>Modèle de réflexion</span>
          <span class="sub">IA utilisée pour analyser les échecs et créer un skill</span>
        </div>
        <Dropdown
          label="Modèle"
          options={claudePickerOptions}
          maxVisible={8}
          value={settings.hermesModel || modelStore.pickerDefaultFor("claude_code")}
          onchange={(v) => settings.setHermesModel(v)}
        />
      </div>

      <div class="row" use:tooltip={"Nombre de passes de réflexion injectées avant d'exécuter — le modèle analyse la demande sous plusieurs angles avant d'agir"}>
        <div class="lbl">
          <span>Passes de réflexion <span class="cbadge cost" use:tooltip={"Consomme plus de tokens à chaque passe"}><Icon name="coin-up" size={11} /></span></span>
          <span class="sub">Réflexions avant d'agir · 1 = désactivé</span>
        </div>
        <div class="priv-ctl">
          {#each [1, 2, 3, 5] as n}
            <button
              type="button"
              class="chip"
              class:on={settings.hermesReflectPasses === n}
              use:tooltip={n === 1 ? "Désactivé (pas de pré-réflexion)" : `${n} passes de réflexion avant d'agir`}
              onclick={() => settings.setHermesReflectPasses(n)}
            >{n === 1 ? "Off" : `×${n}`}</button>
          {/each}
          <input
            class="num"
            type="number"
            min="1"
            max="10"
            step="1"
            aria-label="Nombre de passes personnalisé"
            use:tooltip={"Nombre personnalisé (1–10)"}
            value={settings.hermesReflectPasses}
            oninput={(e) => settings.setHermesReflectPasses(+e.currentTarget.value)}
          />
        </div>
      </div>

      {:else if settingsTab === "app"}

      <!-- ── Apparence ──────────────────────────────────── -->
      <div class="s-section">Apparence</div>

      <div class="row" use:tooltip={"Bascule entre clair, sombre, ou suit automatiquement le thème du système"}>
        <div class="lbl">
          <span>Thème</span>
          <span class="sub">Clair, sombre ou selon le système</span>
        </div>
        <ThemeToggle />
      </div>

      <div class="row" use:tooltip={"Ajuste la taille du texte dans les chats (80% = plus compact, 125% = plus grand)"}>
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
          <input
            class="num"
            type="number"
            min="50"
            max="200"
            step="5"
            aria-label="Zoom personnalisé en %"
            use:tooltip={"Zoom personnalisé (50–200%)"}
            value={Math.round(settings.defaultZoom * 100)}
            oninput={(e) => settings.setDefaultZoom(Math.max(0.5, Math.min(2, +e.currentTarget.value / 100)))}
          />
          <span class="unit">%</span>
        </div>
      </div>

      <!-- ── Confidentialité ───────────────────────────── -->
      <div class="s-section">Confidentialité</div>

      <div class="row" use:tooltip={"Floute automatiquement le contenu d'un chat inactif pour protéger les données visibles à l'écran"}>
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

      {:else}

      <!-- ── Performances ──────────────────────────────── -->
      <div class="s-section">Performances</div>

      <button
        class="row check"
        use:tooltip={"Au prochain lancement, restaure exactement les onglets et conversations ouverts"}
        onclick={() => settings.setRestore(!settings.restoreOnLaunch)}
      >
        <div class="lbl">
          <span>Réouvrir mes onglets</span>
          <span class="sub">À la fermeture, retrouver les mêmes chats et discussions</span>
        </div>
        <span class="switch" class:on={settings.restoreOnLaunch}><span class="knob"></span></span>
      </button>

      <button
        class="row check"
        use:tooltip={"Suspend les chats inactifs : le process en fond est arrêté pour libérer la RAM. Un clic réveille le chat."}
        onclick={() => settings.setChatSleepEnabled(!settings.chatSleepEnabled)}
      >
        <div class="lbl">
          <span>Veille des chats <span class="cbadge save" use:tooltip={"Économise : libère la RAM des chats inactifs"}><Icon name="coin-down" size={11} /></span></span>
          <span class="sub">Met en veille un chat inactif (libère la RAM) · clic pour réveiller</span>
        </div>
        <span class="switch" class:on={settings.chatSleepEnabled}><span class="knob"></span></span>
      </button>

      {#if settings.chatSleepEnabled}
        <div class="row" transition:slide={{ duration: 150 }}>
          <div class="lbl">
            <span>Délai de veille</span>
            <span class="sub">Minutes d'inactivité avant mise en veille</span>
          </div>
          <div class="priv-ctl">
            {#each [5, 15, 30, 60] as m}
              <button
                type="button"
                class="chip"
                class:on={settings.chatSleepMin === m}
                use:tooltip={`Après ${m} min d'inactivité`}
                onclick={() => settings.setChatSleepMin(m)}
              >{m}m</button>
            {/each}
            <input
              class="num"
              type="number"
              min="1"
              max="240"
              step="1"
              aria-label="Délai de veille en minutes"
              value={settings.chatSleepMin}
              oninput={(e) => settings.setChatSleepMin(+e.currentTarget.value)}
            />
            <span class="unit">min</span>
          </div>
        </div>
      {/if}

      <!-- ── Général ────────────────────────────────────── -->
      <div class="s-section">Général</div>

      <button class="row check" use:tooltip={"Dossier ouvert par défaut dans les nouveaux chats — l'agent travaille depuis cet emplacement"} onclick={pickDefaultCwd}>
        <div class="lbl">
          <span>Dossier de travail par défaut</span>
          <span class="sub">{settings.defaultCwd || "Dossier personnel"}</span>
        </div>
        <span class="na">Changer</span>
      </button>

      <div class="row" use:tooltip={"Nombre de conversations affichées dans le panneau historique (icône horloge)"}>
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

      {/if}

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
  .s-tabs {
    display: flex;
    gap: 4px;
    padding: 8px 0 10px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 2px;
  }
  .s-tab {
    padding: 4px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-size: 12px;
    font-family: var(--font-mono);
    transition: background var(--transition), border-color var(--transition), color var(--transition);
  }
  .s-tab:hover {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .s-tab.on {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-weak);
  }
  /* En-tête de section : séparateur visuel avec label. */
  .s-section {
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: var(--text-faint);
    padding: 14px 0 6px;
    border-top: 1px solid var(--border);
    margin-top: 2px;
  }
  .s-section:first-child {
    border-top: none;
    padding-top: 6px;
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
  .sublist-head {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .sublist-head .sub {
    flex-shrink: 0;
  }
  .prov-tabs {
    display: flex;
    gap: 3px;
  }
  .prov-tab {
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-size: 11px;
    font-family: var(--font-mono);
    transition: background var(--transition), border-color var(--transition), color var(--transition);
  }
  .prov-tab.on {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-weak);
  }
  .cand-search {
    margin-left: auto;
  }
  .cand-search {
    width: 120px;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 11.5px;
    outline: none;
  }
  .cand-search:focus {
    border-color: var(--accent);
  }
  .opts {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    max-height: 220px;
    overflow-y: auto;
    scrollbar-width: thin;
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
  .opt-hint {
    margin-left: 6px;
    color: var(--text-faint);
    font-size: 9px;
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
  .nav-row {
    cursor: pointer;
    border-bottom: none;
    padding: 13px 14px;
    margin: 0 -14px; /* étire la carte de survol sans décaler le texte (aligné aux autres rows) */
    width: calc(100% + 28px);
    border-radius: var(--radius);
    border: 1px solid transparent;
    transition: background var(--transition), border-color var(--transition);
  }
  .nav-row:hover {
    background: var(--surface-2);
    border-color: var(--border);
  }
  .nav-row:hover .chev-right {
    color: var(--accent);
    transform: rotate(-90deg) translateY(3px);
  }
  .chev-right {
    display: inline-flex;
    color: var(--text-muted);
    transform: rotate(-90deg);
    transition: color var(--transition), transform var(--transition);
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
  /* Badge coût : indique qu'un réglage consomme plus (coin-up) ou économise (coin-down). */
  .cbadge {
    display: inline-flex;
    align-items: center;
    vertical-align: middle;
    margin-left: 4px;
    padding: 1px;
    border-radius: 50%;
  }
  .cbadge.cost {
    color: var(--warn);
    background: color-mix(in srgb, var(--warn) 14%, transparent);
  }
  .cbadge.save {
    color: var(--good);
    background: color-mix(in srgb, var(--good) 14%, transparent);
  }
</style>
