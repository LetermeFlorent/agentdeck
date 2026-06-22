<script lang="ts">
  // Un message utilisateur : ligne « ❯ … » (+ images), ou ligne compacte « passe auto ».
  import type { Msg } from "$lib/stores/sessions.svelte";
  import { fly } from "svelte/transition";

  let { msg }: { msg: Msg } = $props();
</script>

{#if msg.autoPass}
  <div class="auto-pass-line" in:fly={{ y: 4, duration: 120 }}>
    <span class="ap-badge">↻ {msg.pass ? `Passe ${msg.pass.n}/${msg.pass.total}` : "Vérification"}</span>
  </div>
{:else}
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
{/if}

<style>
  .line.user { display: flex; gap: 8px; align-items: flex-start; }
  .line.user .pfx { color: var(--accent); font-weight: 700; flex-shrink: 0; }
  .ucontent { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .utext { color: var(--text); white-space: pre-wrap; word-break: break-word; }
  .auto-pass-line { display: flex; align-items: center; gap: 6px; opacity: 0.55; }
  .ap-badge { font-family: var(--font-mono); font-size: 10.5px; color: var(--accent); background: var(--accent-weak); border: 1px solid color-mix(in srgb, var(--accent) 20%, transparent); border-radius: 4px; padding: 1px 6px; letter-spacing: 0.02em; }
  .msg-imgs { display: flex; flex-wrap: wrap; gap: 6px; max-width: 86%; }
  .msg-img { max-width: 160px; max-height: 160px; border-radius: 8px; border: 1px solid var(--border); object-fit: cover; }
</style>
