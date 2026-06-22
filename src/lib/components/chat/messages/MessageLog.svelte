<script lang="ts">
  // Log de conversation : assemble les messages (user/assistant), le spinner de réflexion
  // et l'erreur. Autoscroll borné au conteneur.
  import { sessions } from "$lib/stores/sessions.svelte";
  import Icon from "../../ui/Icon.svelte";
  import UserMessage from "./UserMessage.svelte";
  import AssistantMessage from "./AssistantMessage.svelte";
  import ThinkingSpinner from "./ThinkingSpinner.svelte";
  import { fly } from "svelte/transition";

  let { sid }: { sid: string } = $props();
  const session = $derived(sessions.map[sid]);

  let scroller = $state<HTMLDivElement>();

  // Réflexion (thinking) repliable par message (index → ouvert).
  let openReason = $state<Record<number, boolean>>({});
  const toggleReason = (i: number) => (openReason = { ...openReason, [i]: !openReason[i] });

  // Autoscroll vers le bas quand le contenu change (conteneur borné).
  $effect(() => {
    const msgs = session?.messages;
    void msgs?.length;
    void msgs?.[msgs.length - 1]?.text;
    void msgs?.[msgs.length - 1]?.toolCalls?.length;
    if (scroller) scroller.scrollTop = scroller.scrollHeight;
  });
</script>

<div class="messages term" class:blur={session?.priv} style={`zoom:${session?.zoom ?? 1}`} bind:this={scroller}>
  {#if !session || session.messages.length === 0}
    <div class="empty">
      <div class="empty-icon"><Icon name="terminal" size={26} stroke={1.6} /></div>
      <p>Nouvelle session Claude Code</p>
      <span>Écris une instruction pour démarrer.</span>
    </div>
  {/if}
  {#each session?.messages ?? [] as msg, i (i)}
    {#if msg.role === "user"}
      <UserMessage {msg} />
    {:else}
      <AssistantMessage {msg} open={openReason[i]} ontoggle={() => toggleReason(i)} />
    {/if}
  {/each}
  {#if session?.streaming}
    <ThinkingSpinner {session} />
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
    /* Réserve la hauteur du composer (overlay ancré en bas) pour ne pas masquer les
       derniers messages au repos ; sa croissance déborde par-dessus. */
    padding-bottom: 46px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .messages.blur { filter: blur(7px); opacity: 0.55; user-select: none; pointer-events: none; }
  .messages.term { font-family: var(--font-mono); font-size: 12.5px; line-height: 1.55; gap: 9px; }
  .empty { margin: auto; text-align: center; color: var(--text-faint); display: flex; flex-direction: column; align-items: center; gap: 4px; }
  .empty-icon { color: var(--text-faint); margin-bottom: 6px; animation: floaty 3.5s ease-in-out infinite; }
  @keyframes floaty { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-4px); } }
  .empty p { margin: 0; color: var(--text-muted); font-family: var(--font-mono); font-size: 13px; }
  .empty span { font-size: 12.5px; }
  .msg-error { font-size: 12px; color: var(--danger); background: color-mix(in srgb, var(--danger) 12%, transparent); border: 1px solid color-mix(in srgb, var(--danger) 35%, transparent); border-radius: var(--radius-sm); padding: 8px 10px; white-space: pre-wrap; }
</style>
