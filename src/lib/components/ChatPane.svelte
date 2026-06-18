<script lang="ts">
  import { sessions } from "$lib/stores/sessions.svelte";
  import Icon from "./Icon.svelte";

  let {
    sid,
    onsplit,
    onclose,
  }: {
    sid: string;
    onsplit: (dir: "row" | "column") => void;
    onclose: () => void;
  } = $props();

  let draft = $state("");
  let scroller: HTMLDivElement;

  const session = $derived(sessions.map[sid]);

  // Autoscroll vers le bas quand le contenu change (cap scroll : conteneur borné).
  $effect(() => {
    // dépendances : nombre de messages + texte du dernier
    const msgs = session?.messages;
    void msgs?.length;
    void msgs?.[msgs.length - 1]?.text;
    if (scroller) scroller.scrollTop = scroller.scrollHeight;
  });

  function submit(e: Event) {
    e.preventDefault();
    const text = draft.trim();
    if (!text || !session || session.streaming) return;
    draft = "";
    sessions.send(sid, text);
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      submit(e);
    }
  }
</script>

<div class="pane">
  <header class="pane-head">
    <div class="title">
      <span class="status" class:live={session?.streaming}></span>
      <span class="name">{session?.title ?? "Claude"}</span>
      {#if session?.model}<span class="model">{session.model}</span>{/if}
    </div>
    <div class="actions">
      {#if session?.streaming}
        <button class="icon-btn" title="Arrêter" onclick={() => sessions.stop(sid)}>
          <Icon name="stop" size={15} />
        </button>
      {/if}
      <button class="icon-btn" title="Diviser horizontalement (haut / bas)" onclick={() => onsplit("column")}>
        <Icon name="split-v" />
      </button>
      <button class="icon-btn" title="Diviser verticalement (côte à côte)" onclick={() => onsplit("row")}>
        <Icon name="split-h" />
      </button>
      <button class="icon-btn close" title="Fermer le pane" onclick={onclose}>
        <Icon name="close" />
      </button>
    </div>
  </header>

  <div class="messages" bind:this={scroller}>
    {#if !session || session.messages.length === 0}
      <div class="empty">
        <div class="empty-icon"><Icon name="terminal" size={26} stroke={1.6} /></div>
        <p>Nouvelle session Claude Code</p>
        <span>Écris une instruction pour démarrer.</span>
      </div>
    {/if}
    {#each session?.messages ?? [] as msg}
      <div class="msg {msg.role}">
        {#if msg.role === "assistant" && msg.tools.length}
          <div class="tools">
            {#each msg.tools as t}<span class="tool">⚙ {t}</span>{/each}
          </div>
        {/if}
        {#if msg.text}
          <div class="bubble">{msg.text}</div>
        {:else if msg.role === "assistant" && session.streaming}
          <div class="bubble typing"><span></span><span></span><span></span></div>
        {/if}
      </div>
    {/each}
    {#if session?.error}
      <div class="msg-error">{session.error}</div>
    {/if}
  </div>

  <form class="composer" onsubmit={submit}>
    <textarea
      placeholder="Message à Claude…  (Entrée pour envoyer)"
      bind:value={draft}
      onkeydown={onKey}
      rows="1"
    ></textarea>
    <button
      class="btn btn-accent send"
      type="submit"
      disabled={!draft.trim() || session?.streaming}
      title="Envoyer"
    ><Icon name="send" size={17} /></button>
  </form>
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
  }
  .pane-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 9px 7px 12px;
    border-bottom: 1px solid var(--border);
    background: var(--surface-2);
    flex-shrink: 0;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .status {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-faint);
  }
  .status.live {
    background: var(--good);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--good) 25%, transparent);
  }
  .name {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  .model {
    font-size: 10.5px;
    color: var(--text-muted);
    background: var(--elevated);
    border-radius: 4px;
    padding: 1px 6px;
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
  .typing span:nth-child(2) {
    animation-delay: 0.2s;
  }
  .typing span:nth-child(3) {
    animation-delay: 0.4s;
  }
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
  .composer {
    display: flex;
    align-items: flex-end;
    gap: 8px;
    padding: 10px;
    border-top: 1px solid var(--border);
    background: var(--surface-2);
    flex-shrink: 0;
  }
  textarea {
    flex: 1;
    resize: none;
    max-height: 140px;
    padding: 9px 11px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    font-size: 13px;
    line-height: 1.4;
    outline: none;
  }
  textarea:focus {
    border-color: var(--accent);
  }
  .send {
    width: 36px;
    height: 36px;
    padding: 0;
    justify-content: center;
    font-size: 16px;
    flex-shrink: 0;
  }
  .send:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
