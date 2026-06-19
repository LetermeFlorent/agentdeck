<script lang="ts">
  import { sessions } from "$lib/stores/sessions.svelte";
  import { PERM_MODES as MODES } from "./chat-config";
  import { fly } from "svelte/transition";

  let { sid }: { sid: string } = $props();
  const s = $derived(sessions.map[sid]);
  let advanced = $state(false);
</script>

<div class="perm-pop" transition:fly={{ y: 6, duration: 130 }}>
  <div class="sec-lbl">Mode de permission</div>
  <div class="modes">
    {#each MODES as m}
      <button
        type="button"
        class="mode"
        class:on={(s?.permMode ?? "bypassPermissions") === m.v}
        onclick={() => sessions.setPermMode(sid, m.v)}
      >{m.l}</button>
    {/each}
  </div>

  <div class="sec-lbl">Outils autorisés</div>
  <div class="tools">
    {#each sessions.tools as t (t)}
      <label class="tool">
        <input
          type="checkbox"
          checked={!(s?.disabledTools ?? []).includes(t)}
          onchange={() => sessions.toggleTool(sid, t)}
        />
        <span>{t}</span>
      </label>
    {/each}
    {#if !sessions.tools.length}
      <span class="hint">Liste disponible après le 1ᵉʳ message du chat.</span>
    {/if}
  </div>

  <button type="button" class="adv-toggle" onclick={() => (advanced = !advanced)}>
    {advanced ? "▾" : "▸"} Règles avancées (motifs)
  </button>
  {#if advanced}
    <input
      class="adv-in"
      placeholder="Autorisés : Bash(git *),Edit"
      value={s?.allowRules ?? ""}
      oninput={(e) => sessions.setAllowRules(sid, e.currentTarget.value)}
    />
    <input
      class="adv-in"
      placeholder="Refusés : Bash(rm *)"
      value={s?.denyRules ?? ""}
      oninput={(e) => sessions.setDenyRules(sid, e.currentTarget.value)}
    />
  {/if}
</div>

<style>
  .perm-pop {
    position: absolute;
    left: 7px;
    bottom: calc(100% + 4px);
    width: 280px;
    max-height: 320px;
    overflow-y: auto;
    overscroll-behavior: contain;
    background: var(--elevated);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 -8px 26px rgba(0, 0, 0, 0.22);
    padding: 8px;
    z-index: 30;
  }
  .sec-lbl {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-faint);
    font-family: var(--font-mono);
    margin: 4px 2px 5px;
  }
  .modes {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 4px;
    margin-bottom: 6px;
  }
  .mode {
    padding: 5px 6px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-size: 11px;
    text-align: left;
    transition: background var(--transition), border-color var(--transition), color var(--transition);
  }
  .mode:hover {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .mode.on {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-weak);
  }
  .tools {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 2px 6px;
    margin-bottom: 4px;
  }
  .tool {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--text);
    cursor: pointer;
    overflow: hidden;
  }
  .tool span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
  }
  .tool input {
    accent-color: var(--accent);
    flex-shrink: 0;
  }
  .hint {
    grid-column: 1 / -1;
    font-size: 10.5px;
    color: var(--text-faint);
    padding: 2px;
  }
  .adv-toggle {
    width: 100%;
    text-align: left;
    padding: 5px 2px 3px;
    font-size: 11px;
    color: var(--text-muted);
  }
  .adv-in {
    width: 100%;
    height: 26px;
    margin-top: 4px;
    padding: 0 7px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 11px;
    outline: none;
  }
  .adv-in:focus {
    border-color: var(--accent);
  }
</style>
