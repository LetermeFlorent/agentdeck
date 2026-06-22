<script lang="ts">
  // Indicateur de réflexion (façon Claude Code) : spinner animé + secondes + tokens du tour.
  import { fmtTok } from "../chat-config";
  import type { SessionState } from "$lib/stores/sessions.svelte";
  import { fly } from "svelte/transition";

  let { session }: { session: SessionState } = $props();

  const FRAMES = ["✶", "✸", "✹", "✺", "✹", "✷"];
  const VERBS = ["Réflexion", "Cogitation", "Mijotage", "Élucubration", "Tergiversation"];
  let frame = $state(0);
  let seconds = $state(0);

  $effect(() => {
    if (!session.streaming) {
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
</script>

<div class="thinking" in:fly={{ y: 4, duration: 140 }}>
  <span class="spin">{FRAMES[frame]}</span>
  <span class="verb">{verb}…</span>
  {#if session.reflectPass > 0 && session.reflectTotal > 0}
    <span class="pass-meta">Passe {session.reflectPass}/{session.reflectTotal}</span>
  {/if}
  <span class="tmeta">{seconds}s · ↑ {fmtTok(session.turnTokens)} tokens</span>
</div>

<style>
  .thinking { display: flex; align-items: center; gap: 8px; font-family: var(--font-mono); font-size: 12px; padding: 2px 0; }
  .spin { color: var(--accent); font-size: 13px; }
  .verb { color: var(--text); }
  .pass-meta { color: var(--accent); font-size: 11px; font-weight: 600; }
  .tmeta { color: var(--text-faint); font-variant-numeric: tabular-nums; }
</style>
