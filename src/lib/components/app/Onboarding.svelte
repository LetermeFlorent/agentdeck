<script lang="ts">
  import { auth } from "$lib/stores/auth.svelte";
  import { PROVIDERS } from "../chat/chat-config";
  import Icon from "../ui/Icon.svelte";
  import * as ipc from "$lib/ipc";
  import { onMount } from "svelte";

  let claudeToken = $state("");
  let geminiKey = $state("");
  let busy = $state("");
  let error = $state<string | null>(null);
  let installed = $state<Record<string, boolean | null>>({});
  let installCmd = $state<Record<string, string>>({});

  async function refreshInstalled() {
    for (const p of PROVIDERS) {
      ipc.providerInstalled(p.id).then((v) => (installed = { ...installed, [p.id]: v }));
      ipc.providerInstallCmd(p.id).then((c) => (installCmd = { ...installCmd, [p.id]: c }));
    }
    auth.refresh();
  }
  onMount(refreshInstalled);

  const HOWTO: Record<string, string> = {
    claude_code:
      "Si le CLI Claude est déjà connecté, agentdeck le détecte automatiquement. Sinon connecte-toi via le navigateur ou colle un token sk-ant-oat.",
    opencode:
      "opencode gère ses clés par fournisseur. Connecte-le dans son terminal (opencode auth login), puis reviens vérifier.",
    gemini:
      "Colle une clé API Gemini (aistudio.google.com) — écrite dans ~/.gemini/.env. Sinon connecte Google/Vertex dans le terminal du CLI.",
  };

  async function browserLogin(id: string) {
    busy = id + ":login";
    error = null;
    try {
      await auth.login(id);
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }
  async function cliLogin(id: string) {
    busy = id + ":cli";
    error = null;
    try {
      await auth.cliLogin(id);
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }
  async function pasteToken(id: string, token: string) {
    if (!token.trim()) return;
    busy = id + ":token";
    error = null;
    try {
      await auth.setToken(token.trim(), id);
      if (id === "claude_code") claudeToken = "";
      if (id === "gemini") geminiKey = "";
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }
  async function disconnect(id: string) {
    busy = id + ":out";
    try {
      await auth.logout(id);
    } finally {
      busy = "";
    }
  }
</script>

<div class="wrap" data-tauri-drag-region>
  <div class="card">
    <div class="providers">
      {#each PROVIDERS as p, i (p.id)}
        {@const on = auth.isConnected(p.id)}
        {@const inst = installed[p.id]}
        <div class="prov" class:connected={on && inst} style={`--i:${i}`}>
          <div class="head">
            <span class="dot" class:on={on && inst} class:warn={inst === false}></span>
            <span class="name">{p.label}</span>
            <span class="status">
              {#if inst === null}Vérification…
              {:else if inst === false}Pas installé
              {:else if on}Connecté
              {:else}Installé · non connecté{/if}
            </span>
            {#if inst && on}
              <button class="btn ghost" disabled={busy.startsWith(p.id)} onclick={() => disconnect(p.id)}>
                <Icon name="logout" size={14} /> Déconnecter
              </button>
            {/if}
          </div>

          {#if inst === false}
            <div class="actions">
              <div class="install">
                <span class="install-lbl">Installe le CLI puis vérifie :</span>
                <code>{installCmd[p.id] ?? "…"}</code>
              </div>
              <button class="btn" onclick={refreshInstalled}>
                <Icon name="check" size={14} /> J'ai installé — vérifier
              </button>
            </div>
          {:else if inst && !on}
            <div class="actions">
              <span class="howto">{HOWTO[p.id]}</span>
              {#if p.id === "claude_code"}
                <button class="btn accent" disabled={busy === p.id + ":login"} onclick={() => browserLogin(p.id)}>
                  <Icon name="key" size={15} />
                  {busy === p.id + ":login" ? "Autorise dans le navigateur…" : "Se connecter via le navigateur"}
                </button>
                <div class="paste">
                  <input type="password" placeholder="Token Claude (sk-ant-oat01-…)" bind:value={claudeToken} autocomplete="off" />
                  <button class="btn" disabled={!claudeToken.trim() || busy === p.id + ":token"} onclick={() => pasteToken(p.id, claudeToken)}>Coller</button>
                </div>
                <button class="btn ghost" disabled={busy === p.id + ":cli"} onclick={() => cliLogin(p.id)}>
                  <Icon name="terminal" size={14} /> Connecter aussi le CLI claude (terminal)
                </button>
              {:else if p.id === "gemini"}
                <div class="paste">
                  <input type="password" placeholder="Clé API Gemini (AIza…)" bind:value={geminiKey} autocomplete="off" />
                  <button class="btn accent" disabled={!geminiKey.trim() || busy === p.id + ":token"} onclick={() => pasteToken(p.id, geminiKey)}>Connecter</button>
                </div>
                <button class="btn ghost" disabled={busy === p.id + ":cli"} onclick={() => cliLogin(p.id)}>
                  <Icon name="terminal" size={14} /> Connexion Google / Vertex (terminal)
                </button>
              {:else}
                <button class="btn accent" disabled={busy === p.id + ":cli"} onclick={() => cliLogin(p.id)}>
                  <Icon name="terminal" size={15} /> Connecter (terminal : opencode auth login)
                </button>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <p class="note">
      <Icon name="lock" size={13} />
      Tokens/clés chiffrés dans le gestionnaire d'identifiants Windows. Les tokens ne sont jamais écrits en clair.
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
    overflow-y: auto;
  }
  .card {
    width: 100%;
    max-width: 500px;
    animation: cardIn 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
  }
  @keyframes cardIn {
    from { opacity: 0; transform: translateY(10px) scale(0.99); }
    to { opacity: 1; transform: none; }
  }
  .providers {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  /* Entrée en cascade des cartes IA (décalage par --i). */
  .prov {
    animation: provIn 0.45s cubic-bezier(0.16, 1, 0.3, 1) both;
    animation-delay: calc(var(--i, 0) * 70ms + 60ms);
  }
  @keyframes provIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: none; }
  }
  .prov {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 14px;
    background: var(--surface);
    transition: border-color var(--transition), transform var(--transition), box-shadow var(--transition);
  }
  .prov:hover {
    border-color: var(--border-strong);
    transform: translateY(-1px);
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.12);
  }
  .prov.connected {
    border-color: color-mix(in srgb, #3fb950 40%, var(--border));
  }
  .dot.on {
    box-shadow: 0 0 0 3px color-mix(in srgb, #3fb950 22%, transparent);
    animation: dotPulse 2s ease-in-out infinite;
  }
  @keyframes dotPulse {
    0%, 100% { box-shadow: 0 0 0 2px color-mix(in srgb, #3fb950 22%, transparent); }
    50% { box-shadow: 0 0 0 4px color-mix(in srgb, #3fb950 8%, transparent); }
  }
  .head {
    display: flex;
    align-items: center;
    gap: 9px;
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 999px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    flex-shrink: 0;
  }
  .dot.on {
    background: #3fb950;
    border-color: #3fb950;
  }
  .dot.warn {
    background: #d29922;
    border-color: #d29922;
  }
  .name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }
  .status {
    font-size: 11.5px;
    color: var(--text-muted);
    margin-right: auto;
  }
  .actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 12px;
  }
  .install {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .install-lbl {
    font-size: 12px;
    color: var(--text-muted);
  }
  .install code {
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 6px 9px;
    color: var(--text);
    user-select: all;
  }
  .howto {
    font-size: 12px;
    color: var(--text-muted);
    line-height: 1.5;
  }
  .paste {
    display: flex;
    gap: 8px;
  }
  .paste input {
    flex: 1;
    min-width: 0;
    padding: 9px 11px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12.5px;
    outline: none;
  }
  .paste input:focus {
    border-color: var(--accent);
  }
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    padding: 9px 13px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text);
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
    transition: border-color var(--transition), background var(--transition);
  }
  .btn:hover {
    border-color: var(--border-strong);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: default;
  }
  .btn.accent {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .btn.ghost {
    background: none;
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
    align-items: flex-start;
    gap: 7px;
    margin: 18px 0 0;
    font-size: 11.5px;
    color: var(--text-faint);
    line-height: 1.5;
  }
  .note :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
  }
</style>
