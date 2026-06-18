<script lang="ts">
  import { sessions } from "$lib/stores/sessions.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import Icon from "./Icon.svelte";
  import Dropdown from "./Dropdown.svelte";
  import { tooltip } from "$lib/actions/tooltip";
  import { fly } from "svelte/transition";

  let {
    sid,
    nodeId,
    canMinimize = false,
    onsplit,
    onclose,
    onmove,
  }: {
    sid: string;
    nodeId: string;
    canMinimize?: boolean;
    onsplit: (dir: "row" | "column") => void;
    onclose: () => void;
    onmove: (fromNodeId: string) => void;
  } = $props();

  let draft = $state("");
  let dragOver = $state(false);
  let editing = $state(false);
  let titleDraft = $state("");
  let scroller = $state<HTMLDivElement>();

  function startEdit() {
    titleDraft = session?.title ?? "Claude";
    editing = true;
  }
  function saveTitle() {
    const t = titleDraft.trim();
    if (t) sessions.setTitle(sid, t);
    editing = false;
  }
  function autofocus(el: HTMLInputElement) {
    el.focus();
    el.select();
  }

  const session = $derived(sessions.map[sid]);
  const collapsed = $derived(session?.collapsed ?? false);

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
  // Ultracode : exclusif Opus (mappé sur --effort xhigh côté CLI).
  const ULTRACODE = { v: "ultracode", l: "Ultracode" };

  const models = $derived(MODELS.filter((m) => !settings.unavailableModels.includes(m.v)));
  const efforts = $derived(session?.model === "opus" ? [...EFFORTS, ULTRACODE] : EFFORTS);

  // Tarifs par million de tokens (entrée / sortie) du modèle choisi.
  const PRICES: Record<string, [number, number]> = {
    opus: [5, 25],
    sonnet: [3, 15],
    haiku: [1, 5],
    fable: [10, 50],
  };
  const price = $derived(PRICES[session?.model ?? "opus"] ?? null);

  // Indicateur de réflexion (façon Claude Code) : spinner + secondes + tokens.
  const FRAMES = ["✶", "✸", "✹", "✺", "✹", "✷"];
  const VERBS = ["Réflexion", "Cogitation", "Mijotage", "Élucubration", "Tergiversation"];
  let frame = $state(0);
  let seconds = $state(0);
  $effect(() => {
    if (!session?.streaming) {
      frame = 0;
      seconds = 0;
      return;
    }
    const start = session.turnStart ?? Date.now();
    const iv = setInterval(() => {
      frame = (frame + 1) % FRAMES.length;
      seconds = Math.max(0, Math.floor((Date.now() - start) / 1000));
    }, 130);
    return () => clearInterval(iv);
  });
  const verb = $derived(VERBS[Math.floor(seconds / 4) % VERBS.length]);
  function fmtTok(n: number): string {
    return n >= 1000 ? (n / 1000).toFixed(1) + "k" : String(n);
  }

  // Popup de commandes slash ("/") — liste récupérée dynamiquement de Claude Code.
  let cmdSel = $state(0);
  let cmdDismissed = $state(false);
  const cmdQuery = $derived(
    draft.startsWith("/") && !draft.includes(" ") ? draft.slice(1).toLowerCase() : null,
  );
  const cmdMatches = $derived(
    cmdQuery === null
      ? []
      : sessions.slashCommands.filter((c) => c.toLowerCase().startsWith(cmdQuery)).slice(0, 8),
  );
  const showCmds = $derived(cmdMatches.length > 0 && !cmdDismissed);
  $effect(() => {
    if (cmdSel >= cmdMatches.length) cmdSel = 0;
  });
  function pickCmd(c: string) {
    draft = "/" + c + " ";
    cmdDismissed = true;
  }

  // Autoscroll vers le bas quand le contenu change (cap scroll : conteneur borné).
  $effect(() => {
    const msgs = session?.messages;
    void msgs?.length;
    void msgs?.[msgs.length - 1]?.text;
    if (scroller) scroller.scrollTop = scroller.scrollHeight;
  });

  function submit(e: Event) {
    e.preventDefault();
    const text = draft.trim();
    if (!text || !session) return;
    draft = "";
    // Envoi possible même si Claude travaille : ça part en file d'attente.
    sessions.send(sid, text);
  }

  function onKey(e: KeyboardEvent) {
    if (showCmds) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        cmdSel = (cmdSel + 1) % cmdMatches.length;
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        cmdSel = (cmdSel - 1 + cmdMatches.length) % cmdMatches.length;
        return;
      }
      if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        pickCmd(cmdMatches[cmdSel]);
        return;
      }
      if (e.key === "Escape") {
        cmdDismissed = true;
        return;
      }
    }
    if (e.key === "Enter" && !e.shiftKey) submit(e);
  }

  // --- Drag & drop pour réarranger les chats ---
  function dragStart(e: DragEvent) {
    e.dataTransfer?.setData("text/plain", nodeId);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }
  function dragOverH(e: DragEvent) {
    e.preventDefault();
    dragOver = true;
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
  }
  function dropH(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    const from = e.dataTransfer?.getData("text/plain");
    if (from && from !== nodeId) onmove(from);
  }
</script>

<div
  class="pane"
  class:drag-over={dragOver}
  class:collapsed
  role="group"
  ondragover={dragOverH}
  ondragleave={() => (dragOver = false)}
  ondrop={dropH}
>
  {#if collapsed}
    <div class="strip">
      <button class="icon-btn" use:tooltip={"Déplier le chat"} onclick={() => sessions.setCollapsed(sid, false)}>
        <span class="chev open"><Icon name="chevron" size={15} /></span>
      </button>
      <span class="status" class:live={session?.streaming}></span>
      <span class="strip-title">{session?.title ?? "Claude"}</span>
      <span class="strip-state" class:work={session?.streaming} use:tooltip={session?.streaming ? "Claude travaille" : "Inactif"}></span>
    </div>
  {:else}
  <header class="pane-head">
    <div
      class="title"
      role="button"
      tabindex="0"
      aria-label="Glisser pour déplacer ce chat"
      draggable={!editing}
      ondragstart={dragStart}
      use:tooltip={"Glisser pour déplacer · double-clic pour renommer"}
    >
      <span class="grip"><Icon name="grip" size={14} /></span>
      <span class="status" class:live={session?.streaming}></span>
      {#if editing}
        <input
          class="name-edit"
          bind:value={titleDraft}
          use:autofocus
          onblur={saveTitle}
          onkeydown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              saveTitle();
            } else if (e.key === "Escape") {
              editing = false;
            }
          }}
        />
      {:else}
        <span
          class="name"
          role="button"
          tabindex="0"
          use:tooltip={"Double-clic pour renommer"}
          ondblclick={startEdit}
        >{session?.title ?? "Claude"}</span>
      {/if}
    </div>
    {#if price}
      <span class="hdr-price" use:tooltip={"Tarif du modèle, par million de tokens (entrée / sortie)"}>
        ↑${price[0]} ↓${price[1]}/M
      </span>
    {/if}
    {#if (session?.totalTokens ?? 0) > 0}
      <span class="hdr-usage" use:tooltip={"Coût et tokens générés par ce chat"}>
        ${(session?.costUsd ?? 0).toFixed(3)} · {fmtTok(session?.totalTokens ?? 0)}
      </span>
    {/if}
    <div class="actions">
      {#if session?.streaming}
        <button class="icon-btn" use:tooltip={"Arrêter"} onclick={() => sessions.stop(sid)}>
          <Icon name="stop" size={15} />
        </button>
      {/if}
      <button
        class="icon-btn"
        class:on={session?.priv}
        use:tooltip={session?.priv ? "Désactiver le mode privé" : "Mode privé (flouter le contenu)"}
        onclick={() => sessions.setPrivate(sid, !session?.priv)}
      >
        <Icon name={session?.priv ? "eye-off" : "eye"} size={15} />
      </button>
      {#if canMinimize}
        <button class="icon-btn" use:tooltip={"Minimiser sur le côté"} onclick={() => sessions.setCollapsed(sid, true)}>
          <span class="chev close"><Icon name="chevron" size={15} /></span>
        </button>
      {/if}
      <button class="icon-btn" use:tooltip={"Diviser horizontalement (haut / bas)"} onclick={() => onsplit("column")}>
        <Icon name="split-v" />
      </button>
      <button class="icon-btn" use:tooltip={"Diviser verticalement (côte à côte)"} onclick={() => onsplit("row")}>
        <Icon name="split-h" />
      </button>
      <button class="icon-btn close" use:tooltip={"Fermer le pane"} onclick={onclose}>
        <Icon name="close" />
      </button>
    </div>
  </header>

  <div class="messages" class:blur={session?.priv} bind:this={scroller}>
    {#if !session || session.messages.length === 0}
      <div class="empty">
        <div class="empty-icon"><Icon name="terminal" size={26} stroke={1.6} /></div>
        <p>Nouvelle session Claude Code</p>
        <span>Écris une instruction pour démarrer.</span>
      </div>
    {/if}
    {#each session?.messages ?? [] as msg, i (i)}
      <div class="msg {msg.role}" in:fly={{ y: 6, duration: 160 }}>
        {#if msg.role === "assistant" && msg.tools.length}
          <div class="tools">
            {#each msg.tools as t}<span class="tool" in:fly={{ y: 4, duration: 140 }}>⚙ {t}</span>{/each}
          </div>
        {/if}
        {#if msg.text}
          <div class="bubble">{msg.text}</div>
        {/if}
      </div>
    {/each}
    {#if session?.streaming}
      <div class="thinking" in:fly={{ y: 4, duration: 140 }}>
        <span class="spin">{FRAMES[frame]}</span>
        <span class="verb">{verb}…</span>
        <span class="tmeta">{seconds}s · ↑ {fmtTok(session.turnTokens)} tokens</span>
      </div>
    {/if}
    {#if session?.error}
      <div class="msg-error" in:fly={{ y: 6, duration: 160 }}>{session.error}</div>
    {/if}
  </div>

  <div class="composer">
    {#if showCmds}
      <div class="cmd-pop" in:fly={{ y: 6, duration: 130 }}>
        {#each cmdMatches as c, i}
          <button
            type="button"
            class="cmd-item"
            class:sel={i === cmdSel}
            onmousedown={(e) => {
              e.preventDefault();
              pickCmd(c);
            }}
          >
            <span class="slash">/</span>{c}
          </button>
        {/each}
      </div>
    {/if}
    <div class="meta">
      <Dropdown
        label="Modèle"
        options={models}
        value={session?.model ?? ""}
        onchange={(v) => sessions.setModel(sid, v)}
      />
      <Dropdown
        label="Effort"
        options={efforts}
        value={session?.effort ?? ""}
        btnClass={`eff-${session?.effort ?? "medium"}`}
        onchange={(v) => sessions.setEffort(sid, v)}
      />
    </div>
    <form class="field eff-{session?.effort ?? 'medium'}" onsubmit={submit}>
      <textarea
        placeholder="Message à Claude…  (/ pour les commandes)"
        bind:value={draft}
        onkeydown={onKey}
        oninput={() => (cmdDismissed = false)}
        rows="1"
      ></textarea>
      {#if (session?.queue?.length ?? 0) > 0}
        <span class="qchip" use:tooltip={"Messages en file (envoyés l'un après l'autre)"}>{session.queue.length}</span>
      {/if}
      <button
        class="send"
        type="submit"
        disabled={!draft.trim()}
        use:tooltip={session?.streaming ? "Envoyer (pris en cours de route)" : "Envoyer (Entrée)"}
      ><Icon name="send" size={13} /></button>
    </form>
  </div>
  {/if}
</div>

<style>
  .pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
    border: 1px solid var(--border);
    overflow: hidden;
    min-width: 0;
    min-height: 0;
    transition: border-color var(--transition), box-shadow var(--transition);
  }
  .pane.drag-over {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
  .chev {
    display: flex;
    transition: transform var(--transition);
  }
  .chev.close {
    transform: rotate(90deg);
  }
  .chev.open {
    transform: rotate(-90deg);
  }
  .icon-btn.on {
    color: var(--accent);
    background: var(--accent-weak);
  }
  /* Mode privé : floute le contenu (le statut reste visible dans l'entête) */
  .messages.blur {
    filter: blur(7px);
    opacity: 0.55;
    user-select: none;
    pointer-events: none;
  }

  /* Bande latérale quand le chat est minimisé sur le côté */
  .strip {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    background: var(--surface-2);
  }
  .strip-title {
    writing-mode: vertical-rl;
    text-orientation: mixed;
    transform: rotate(180deg);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-height: 60%;
    margin: 2px 0;
  }
  .strip-state {
    margin-top: auto;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-faint);
  }
  .strip-state.work {
    background: var(--good);
    animation: pulseDot 1.6s ease-in-out infinite;
  }
  .pane-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 7px 4px 6px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    flex-shrink: 0;
  }
  /* header de chat compact (–20%) : boutons d'action plus petits */
  .actions :global(.icon-btn) {
    width: 23px;
    height: 23px;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 7px;
    flex: 1;
    min-width: 0;
    cursor: grab;
    padding: 3px 4px;
    border-radius: 5px;
    transition: background var(--transition);
  }
  .title:hover {
    background: var(--elevated);
  }
  .title:active {
    cursor: grabbing;
  }
  .grip {
    color: var(--text-faint);
    display: flex;
  }
  .status {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-faint);
    transition: background var(--transition), box-shadow var(--transition);
  }
  .status.live {
    background: var(--good);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--good) 25%, transparent);
    animation: pulseDot 1.6s ease-in-out infinite;
  }
  @keyframes pulseDot {
    0%, 100% { box-shadow: 0 0 0 2px color-mix(in srgb, var(--good) 22%, transparent); }
    50% { box-shadow: 0 0 0 4px color-mix(in srgb, var(--good) 10%, transparent); }
  }
  .name {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.01em;
    cursor: text;
  }
  .name-edit {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--text);
    background: var(--bg);
    border: 1px solid var(--accent);
    border-radius: 4px;
    padding: 1px 5px;
    outline: none;
    width: 130px;
    max-width: 100%;
  }
  .actions {
    display: flex;
    gap: 2px;
  }
  .close:hover {
    color: var(--danger);
  }
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .empty {
    margin: auto;
    text-align: center;
    color: var(--text-faint);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .empty-icon {
    color: var(--text-faint);
    margin-bottom: 6px;
    animation: floaty 3.5s ease-in-out infinite;
  }
  @keyframes floaty {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-4px); }
  }
  .empty p {
    margin: 0;
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-size: 13px;
  }
  .empty span {
    font-size: 12.5px;
  }
  .msg {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .msg.user {
    align-items: flex-end;
  }
  .bubble {
    max-width: 86%;
    padding: 9px 12px;
    border-radius: 11px;
    font-size: 13.5px;
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .msg.user .bubble {
    background: var(--user-bubble);
    border: 1px solid var(--border);
    border-bottom-right-radius: 3px;
  }
  .msg.assistant .bubble {
    background: transparent;
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12.5px;
    padding-left: 0;
  }
  .tools {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
  .tool {
    font-size: 10.5px;
    color: var(--accent);
    background: var(--accent-weak);
    border-radius: 4px;
    padding: 2px 7px;
    font-family: var(--font-mono);
  }
  .typing {
    display: inline-flex;
    gap: 4px;
  }
  .typing span {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-faint);
    animation: blink 1.2s infinite both;
  }
  .typing span:nth-child(2) { animation-delay: 0.2s; }
  .typing span:nth-child(3) { animation-delay: 0.4s; }
  @keyframes blink {
    0%, 80%, 100% { opacity: 0.25; }
    40% { opacity: 1; }
  }
  .msg-error {
    font-size: 12px;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--danger) 35%, transparent);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
    white-space: pre-wrap;
  }

  /* Composer minimaliste : barre unique collée au bord. Modèle/effort (dropdowns custom)
     + saisie + envoi sur une seule ligne. */
  .composer {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 5px;
    border-top: 1px solid var(--border);
    background: var(--surface-2);
    flex-shrink: 0;
  }
  .composer {
    position: relative;
  }
  /* Popup des commandes slash ("/") au-dessus du champ */
  .cmd-pop {
    position: absolute;
    left: 7px;
    right: 7px;
    bottom: calc(100% + 4px);
    max-height: 220px;
    overflow-y: auto;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 -8px 26px rgba(0, 0, 0, 0.22);
    padding: 5px;
    z-index: 30;
  }
  .cmd-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 9px;
    border-radius: var(--radius-sm);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12.5px;
    text-align: left;
  }
  .cmd-item .slash {
    color: var(--accent);
    font-weight: 700;
  }
  .cmd-item:hover,
  .cmd-item.sel {
    background: var(--accent-weak);
  }
  .hdr-usage,
  .hdr-price {
    flex-shrink: 0;
    margin-right: 6px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--text-faint);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }
  .hdr-price {
    color: var(--text-muted);
  }

  /* Indicateur de réflexion façon Claude Code */
  .thinking {
    display: flex;
    align-items: center;
    gap: 8px;
    font-family: var(--font-mono);
    font-size: 12px;
    padding: 2px 0;
  }
  .spin {
    color: var(--accent);
    font-size: 13px;
  }
  .verb {
    color: var(--text);
  }
  .tmeta {
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
  }
  .field {
    flex: 1;
    display: flex;
    align-items: flex-end;
    gap: 5px;
    min-width: 0;
    padding: 2px 2px 2px 3px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    transition: border-color var(--transition);
  }
  .field:not(.eff-xhigh):not(.eff-max):focus-within {
    border-color: var(--accent);
  }
  /* Contour animé selon l'effort demandé (low → max de plus en plus marqué). */
  .field.eff-low {
    border-color: var(--border);
  }
  .field.eff-medium {
    border-color: color-mix(in srgb, var(--accent) 25%, var(--border));
  }
  .field.eff-high {
    border-color: color-mix(in srgb, var(--accent) 55%, var(--border));
    animation: effPulse 2.4s ease-in-out infinite;
  }
  @keyframes effPulse {
    0%, 100% { box-shadow: 0 0 0 0 transparent; }
    50% { box-shadow: 0 0 10px color-mix(in srgb, var(--accent) 32%, transparent); }
  }
  .field.eff-xhigh,
  .field.eff-max {
    border: 1px solid transparent;
    background:
      linear-gradient(var(--bg), var(--bg)) padding-box,
      linear-gradient(90deg, var(--accent), #e6a988, var(--accent), #cf7ea6, var(--accent)) border-box;
    background-size: 100% 100%, 300% 100%;
    animation: effFlow 5s linear infinite;
  }
  .field.eff-max {
    animation: effFlow 2.3s linear infinite, effGlow 2s ease-in-out infinite;
  }
  @keyframes effFlow {
    to { background-position: 0 0, 300% 0; }
  }
  @keyframes effGlow {
    0%, 100% { box-shadow: 0 0 6px color-mix(in srgb, var(--accent) 30%, transparent); }
    50% { box-shadow: 0 0 18px color-mix(in srgb, var(--accent) 58%, transparent); }
  }
  textarea {
    flex: 1;
    resize: none;
    max-height: 84px;
    padding: 3px 6px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 11px;
    line-height: 1.3;
    outline: none;
    min-width: 0;
  }
  textarea::placeholder {
    color: var(--text-faint);
  }
  .qchip {
    flex-shrink: 0;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 9px;
    background: var(--accent);
    color: #fff;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
  }
  .send {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 23px;
    height: 22px;
    flex-shrink: 0;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    transition: background var(--transition), transform var(--transition), opacity var(--transition);
  }
  .send:not(:disabled):hover {
    background: var(--accent-hover);
  }
  .send:not(:disabled):active {
    transform: scale(0.9);
  }
  .send:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
