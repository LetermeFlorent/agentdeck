<script lang="ts">
  import { auth } from "$lib/stores/auth.svelte";
  import Icon from "../ui/Icon.svelte";
  import { fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let busy = $state(false);
  let browsing = $state(false);
  let error = $state<string | null>(null);
  let token = $state("");

  async function connect(e: Event) {
    e.preventDefault();
    if (!token.trim()) return;
    busy = true;
    error = null;
    try {
      await auth.setToken(token.trim());
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  // Connexion navigateur : ouvre l'autorisation automatiquement et capte le token au retour.
  async function openBrowser() {
    browsing = true;
    error = null;
    try {
      await auth.login();
    } catch (err) {
      error = String(err);
    } finally {
      browsing = false;
    }
  }
</script>

<div class="wrap" data-tauri-drag-region>
  <div class="card" in:fly={{ y: 12, duration: 220, easing: cubicOut }}>
    <div class="brand">
      <div class="dot"></div>
      <h1>agentdeck</h1>
    </div>
    <p class="sub">Pilote plusieurs IA en parallèle. Connecte-toi pour commencer.</p>

    <button class="btn btn-accent browser" disabled={busy || browsing} onclick={openBrowser}>
      <Icon name="key" size={16} />
      {browsing ? "Autorise dans le navigateur…" : "Se connecter via le navigateur"}
    </button>

    <div class="sep"><span>ou colle ton token</span></div>

    <form class="paste" onsubmit={connect}>
      <label class="lab" for="tok">Token Claude</label>
      <input
        id="tok"
        type="password"
        placeholder="sk-ant-oat01-…"
        bind:value={token}
        disabled={busy}
        autocomplete="off"
      />
      <button class="btn btn-accent connect" type="submit" disabled={busy || !token.trim()}>
        {busy ? "Connexion…" : "Connecter"}
      </button>
    </form>

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
  .browser {
    width: 100%;
    justify-content: center;
    padding: 11px;
    font-size: 14px;
    font-weight: 500;
  }
  .sep {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 16px 0;
    color: var(--text-faint);
    font-size: 11.5px;
  }
  .sep::before,
  .sep::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--border);
  }
  .paste {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .lab {
    font-size: 12px;
    color: var(--text-muted);
    font-weight: 600;
  }
  .paste input {
    width: 100%;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    font-size: 13px;
    font-family: var(--font-mono);
    outline: none;
    transition: border-color var(--transition);
  }
  .paste input:focus {
    border-color: var(--accent);
  }
  .connect {
    justify-content: center;
    padding: 10px;
    font-size: 14px;
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
