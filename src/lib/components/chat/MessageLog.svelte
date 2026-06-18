<script lang="ts">
  import { sessions } from "$lib/stores/sessions.svelte";
  import Icon from "../ui/Icon.svelte";
  import { fmtTok } from "./chat-config";
  import { renderMarkdown } from "./markdown";
  import { fly } from "svelte/transition";

  let { sid }: { sid: string } = $props();
  const session = $derived(sessions.map[sid]);

  let scroller = $state<HTMLDivElement>();

  // Réflexion (thinking) repliable par message.
  let openReason = $state<Record<number, boolean>>({});
  function toggleReason(i: number) {
    openReason = { ...openReason, [i]: !openReason[i] };
  }

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

  // Autoscroll vers le bas quand le contenu change (cap scroll : conteneur borné).
  $effect(() => {
    const msgs = session?.messages;
    void msgs?.length;
    void msgs?.[msgs.length - 1]?.text;
    void msgs?.[msgs.length - 1]?.toolCalls?.length;
    if (scroller) scroller.scrollTop = scroller.scrollHeight;
  });
</script>

<div
  class="messages term"
  class:blur={session?.priv}
  style={`zoom:${session?.zoom ?? 1}`}
  bind:this={scroller}
>
  {#if !session || session.messages.length === 0}
    <div class="empty">
      <div class="empty-icon"><Icon name="terminal" size={26} stroke={1.6} /></div>
      <p>Nouvelle session Claude Code</p>
      <span>Écris une instruction pour démarrer.</span>
    </div>
  {/if}
  {#each session?.messages ?? [] as msg, i (i)}
    {#if msg.role === "user"}
      <div class="line user" in:fly={{ y: 6, duration: 140 }}>
        <span class="pfx">❯</span>
        <div class="ucontent">
          {#if msg.images?.length}
            <div class="msg-imgs">
              {#each msg.images as src}
                {#if src}<img class="msg-img" {src} alt="pièce jointe" />{/if}
              {/each}
            </div>
          {/if}
          {#if msg.text}<span class="utext">{msg.text}</span>{/if}
        </div>
      </div>
    {:else}
      <div class="block assistant" in:fly={{ y: 6, duration: 140 }}>
        {#if msg.thinking}
          <button
            type="button"
            class="reason-head"
            class:open={openReason[i]}
            onclick={() => toggleReason(i)}
          >
            <span class="rchev"><Icon name="chevron" size={12} /></span>
            <span>réflexion</span>
          </button>
          {#if openReason[i]}<div class="reason-body">{msg.thinking}</div>{/if}
        {/if}
        {#each msg.toolCalls as t}
          <div class="toolline" in:fly={{ y: 4, duration: 130 }}>
            <span class="tname">▸ {t.name}</span>
            {#if t.input}<span class="targ">{t.input}</span>{/if}
          </div>
        {/each}
        {#if msg.text}
          <!-- Markdown rendu (HTML échappé en amont par renderMarkdown). -->
          <div class="atext md">{@html renderMarkdown(msg.text)}</div>
        {/if}
      </div>
    {/if}
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

<style>
  .messages {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .messages.blur {
    filter: blur(7px);
    opacity: 0.55;
    user-select: none;
    pointer-events: none;
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
  /* Mode terminal : log monospace */
  .messages.term {
    font-family: var(--font-mono);
    font-size: 12.5px;
    line-height: 1.55;
    gap: 9px;
  }
  .line.user {
    display: flex;
    gap: 8px;
    align-items: flex-start;
  }
  .line.user .pfx {
    color: var(--accent);
    font-weight: 700;
    flex-shrink: 0;
  }
  .ucontent {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .utext {
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-word;
  }
  .block.assistant {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .atext {
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-word;
  }
  /* Rendu markdown */
  .md :global(p) {
    margin: 0 0 6px;
  }
  .md :global(p:last-child) {
    margin-bottom: 0;
  }
  .md :global(h1),
  .md :global(h2),
  .md :global(h3),
  .md :global(h4) {
    margin: 8px 0 4px;
    font-size: 13.5px;
    font-weight: 700;
  }
  .md :global(ul) {
    margin: 4px 0;
    padding-left: 18px;
  }
  .md :global(li) {
    margin: 2px 0;
  }
  .md :global(code) {
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0 4px;
    font-size: 11.5px;
  }
  .md :global(pre) {
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
    overflow-x: auto;
    margin: 6px 0;
  }
  .md :global(pre code) {
    background: none;
    border: none;
    padding: 0;
  }
  .md :global(blockquote) {
    margin: 4px 0;
    padding-left: 9px;
    border-left: 2px solid var(--border-strong);
    color: var(--text-muted);
  }
  .md :global(a) {
    color: var(--accent);
    text-decoration: underline;
  }
  .md :global(strong) {
    font-weight: 700;
  }
  .md :global(em) {
    font-style: italic;
  }
  /* Ligne d'outil : « ▸ Nom  commande » */
  .toolline {
    display: flex;
    gap: 8px;
    align-items: baseline;
    font-size: 11.5px;
    padding: 2px 8px;
    border-left: 2px solid var(--accent);
    background: var(--accent-weak);
    border-radius: 0 4px 4px 0;
    min-width: 0;
  }
  .tname {
    color: var(--accent);
    font-weight: 600;
    flex-shrink: 0;
  }
  .targ {
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  /* Réflexion repliable */
  .reason-head {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    width: fit-content;
    color: var(--text-faint);
    font-family: var(--font-mono);
    font-size: 11px;
    font-style: italic;
  }
  .reason-head:hover {
    color: var(--text-muted);
  }
  .rchev {
    display: flex;
    transition: transform var(--transition);
  }
  .reason-head.open .rchev {
    transform: rotate(180deg);
  }
  .reason-body {
    color: var(--text-faint);
    font-style: italic;
    font-size: 11.5px;
    white-space: pre-wrap;
    word-break: break-word;
    border-left: 2px solid var(--border);
    padding-left: 8px;
    margin-left: 3px;
    opacity: 0.9;
  }
  .msg-imgs {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    max-width: 86%;
  }
  .msg-img {
    max-width: 160px;
    max-height: 160px;
    border-radius: 8px;
    border: 1px solid var(--border);
    object-fit: cover;
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
</style>
