<script lang="ts">
  // Un message assistant : réflexion repliable + lignes d'outils + texte markdown + footer.
  import Icon from "../../ui/Icon.svelte";
  import { renderMarkdown } from "../markdown";
  import type { Msg } from "$lib/stores/sessions.svelte";
  import { fly } from "svelte/transition";

  let { msg, open = false, ontoggle }: { msg: Msg; open?: boolean; ontoggle: () => void } = $props();
</script>

<div class="block assistant" in:fly={{ y: 6, duration: 140 }}>
  {#if msg.thinking}
    <button type="button" class="reason-head" class:open onclick={ontoggle}>
      <span class="rchev"><Icon name="chevron" size={12} /></span>
      <span>réflexion</span>
    </button>
    {#if open}<div class="reason-body">{msg.thinking}</div>{/if}
  {/if}
  {#each msg.toolCalls as t}
    <div class="toolline" in:fly={{ y: 4, duration: 130 }}>
      <span class="tname">▸ {t.name}</span>
      {#if t.input}<span class="targ">{t.input}</span>{/if}
    </div>
  {/each}
  {#if msg.text}
    <!-- Markdown rendu (HTML échappé en amont par renderMarkdown ; styles .md globaux). -->
    <div class="atext md">{@html renderMarkdown(msg.text)}</div>
  {/if}
  <div class="afoot">
    {#if msg.pass}<span class="pass-badge">Passe {msg.pass.n}/{msg.pass.total}</span>{/if}
    {#if msg.model}<span class="amodel">— {msg.model}{#if msg.effort} · {msg.effort}{/if}</span>{/if}
  </div>
</div>

<style>
  .block.assistant { display: flex; flex-direction: column; gap: 5px; }
  .atext { color: var(--text); white-space: pre-wrap; word-break: break-word; }
  .afoot { display: flex; align-items: center; gap: 8px; margin-top: 4px; flex-wrap: wrap; }
  .pass-badge { font-family: var(--font-mono); font-size: 10px; color: var(--accent); background: var(--accent-weak); border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent); border-radius: 4px; padding: 1px 5px; font-weight: 600; }
  .amodel { font-family: var(--font-mono); font-size: 10px; color: var(--text-faint); }
  .toolline { display: flex; gap: 8px; align-items: baseline; font-size: 11.5px; padding: 2px 8px; border-left: 2px solid var(--accent); background: var(--accent-weak); border-radius: 0 4px 4px 0; min-width: 0; }
  .tname { color: var(--accent); font-weight: 600; flex-shrink: 0; }
  .targ { color: var(--text-muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .reason-head { display: inline-flex; align-items: center; gap: 5px; width: fit-content; color: var(--text-faint); font-family: var(--font-mono); font-size: 11px; font-style: italic; transition: color var(--transition); }
  .reason-head:hover { color: var(--text-muted); }
  .rchev { display: flex; transition: transform var(--transition); }
  .reason-head.open .rchev { transform: rotate(180deg); }
  .reason-body { color: var(--text-faint); font-style: italic; font-size: 11.5px; white-space: pre-wrap; word-break: break-word; border-left: 2px solid var(--border); padding-left: 8px; margin-left: 3px; opacity: 0.9; }
</style>
