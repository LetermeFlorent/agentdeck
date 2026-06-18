<script lang="ts">
  import { auth } from "$lib/stores/auth.svelte";
  import Icon from "./Icon.svelte";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let busy = $state(false);
  let error = $state<string | null>(null);
  let token = $state("");
  let showPaste = $state(false);

  async function run(fn: () => Promise<void>) {
    busy = true;
    error = null;
    try {
      await fn();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="wrap">
  <div class="card" in:fly={{ y: 12, duration: 320, easing: cubicOut }}>
    <div class="brand">
      <div class="dot"></div>
      <h1>agentdeck</h1>
    </div>
    <p class="sub">Pilote plusieurs IA en parallèle. Connecte-toi pour commencer.</p>

    <div class="options">
      <!-- Voie A : connexion Anthropic de base -->
      <button class="opt" disabled={busy} onclick={() => run(() => auth.login())}>
        <div class="opt-ic"><Icon name="key" size={19} /></div>
        <div class="opt-body">
          <div class="opt-head">
            <span class="opt-title">Se connecter à Anthropic</span>
            <span class="badge">OAuth</span>
          </div>
          <span class="opt-desc">Ouvre le navigateur (claude setup-token) et stocke le token de façon sécurisée.</span>
        </div>
      </button>

      <!-- Voie B : import du token -->
      <button class="opt" disabled={busy} onclick={() => run(() => auth.importFromDownloads())}>
        <div class="opt-ic"><Icon name="download" size={19} /></div>
        <div class="opt-body">
          <div class="opt-head">
            <span class="opt-title">Importer le token</span>
            <span class="badge">Téléchargements</span>
          </div>
          <span class="opt-desc">Lit claude-token.txt dans le dossier Téléchargements et l'enregistre dans le coffre Windows.</span>
        </div>
      </button>
    </div>

    <button class="link" onclick={() => (showPaste = !showPaste)}>
      {showPaste ? "Masquer" : "Coller un token manuellement"}
    </button>

    {#if showPaste}
      <form
        class="paste"
        onsubmit={(e) => {
          e.preventDefault();
          run(() => auth.setToken(token));
        }}
      >
        <input
          type="password"
          placeholder="sk-ant-oat01-…"
          bind:value={token}
          disabled={busy}
        />
        <button class="btn btn-accent" type="submit" disabled={busy || !token.trim()}>
          Connecter
        </button>
      </form>
    {/if}

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <p class="note">
      <Icon name="lock" size={13} />
      Le token n'est jamais écrit en clair : il est chiffré dans le gestionnaire d'identifiants Windows.
    </p>
  </div>
</div>

<style>
  .wrap {
    height: 100vh;
    display: grid;
    place-items: center;
    background: var(--bg);
    padding: 24px;
  }
  .card {
    width: 100%;
    max-width: 430px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 30px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .dot {
    width: 11px;
    height: 11px;
    border-radius: 3px;
    background: var(--accent);
  }
  h1 {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 19px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  .sub {
    color: var(--text-muted);
    font-size: 14px;
    margin: 10px 0 22px;
  }
  .options {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .opt {
    text-align: left;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 13px 14px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface-2);
    transition: border-color var(--transition), background var(--transition), transform var(--transition);
  }
  .opt:hover:not(:disabled) {
    border-color: var(--accent);
    transform: translateY(-1px);
  }
  .opt-ic {
    color: var(--accent);
    margin-top: 1px;
  }
  .opt-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }
  .opt:disabled {
    opacity: 0.6;
    cursor: default;
  }
  .opt-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .opt-title {
    font-size: 14px;
    font-weight: 600;
  }
  .opt-desc {
    font-size: 12.5px;
    color: var(--text-muted);
    line-height: 1.45;
  }
  .badge {
    font-size: 10px;
    color: var(--accent);
    background: var(--accent-weak);
    border-radius: 4px;
    padding: 2px 7px;
  }
  .link {
    margin: 16px 0 0;
    color: var(--text-muted);
    font-size: 12.5px;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .link:hover {
    color: var(--text);
  }
  .paste {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }
  .paste input {
    flex: 1;
    padding: 9px 11px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    font-size: 13px;
    outline: none;
  }
  .paste input:focus {
    border-color: var(--accent);
  }
  .error {
    margin-top: 14px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    border: 1px solid color-mix(in srgb, var(--danger) 40%, transparent);
    color: var(--danger);
    font-size: 12.5px;
  }
  .note {
    display: flex;
    align-items: center;
    gap: 7px;
    margin: 18px 0 0;
    font-size: 11.5px;
    color: var(--text-faint);
    line-height: 1.5;
  }
  .note :global(svg) {
    flex-shrink: 0;
  }
</style>
