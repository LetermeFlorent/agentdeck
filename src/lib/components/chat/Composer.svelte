<script lang="ts">
  import { sessions } from "$lib/stores/sessions.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import Icon from "../ui/Icon.svelte";
  import Dropdown from "../ui/Dropdown.svelte";
  import SlashPopup from "./SlashPopup.svelte";
  import PermissionsPopup from "./PermissionsPopup.svelte";
  import ContextGauge from "./ContextGauge.svelte";
  import { MODELS, effortsFor } from "./chat-config";
  import { tooltip } from "$lib/actions/tooltip";
  import { fly } from "svelte/transition";

  let { sid }: { sid: string } = $props();
  const session = $derived(sessions.map[sid]);

  let draft = $state("");
  let ta = $state<HTMLTextAreaElement>();

  const models = $derived(MODELS.filter((m) => !settings.unavailableModels.includes(m.v)));
  const efforts = $derived(effortsFor(session?.model));

  // --- Images jointes (avant envoi) ---
  type Attached = { dataUrl: string; media_type: string; data: string; name: string };
  let images = $state<Attached[]>([]);
  let fileInput = $state<HTMLInputElement>();

  function fileToImage(file: File): Promise<Attached> {
    return new Promise((resolve, reject) => {
      const r = new FileReader();
      r.onload = () => {
        const url = String(r.result);
        const m = /^data:([^;]+);base64,(.*)$/.exec(url);
        if (!m) return reject(new Error("format"));
        resolve({ dataUrl: url, media_type: m[1], data: m[2], name: file.name });
      };
      r.onerror = () => reject(r.error);
      r.readAsDataURL(file);
    });
  }
  async function addFiles(files: FileList | File[] | null | undefined) {
    if (!files) return;
    for (const f of Array.from(files)) {
      if (!f.type.startsWith("image/")) continue;
      try {
        images.push(await fileToImage(f));
      } catch {
        /* ignore */
      }
    }
  }
  function pickFiles() {
    fileInput?.click();
  }
  function removeImage(i: number) {
    images.splice(i, 1);
  }
  function onPaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (!items) return;
    const files: File[] = [];
    for (const it of Array.from(items)) {
      if (it.kind === "file" && it.type.startsWith("image/")) {
        const f = it.getAsFile();
        if (f) files.push(f);
      }
    }
    if (files.length) {
      e.preventDefault();
      addFiles(files);
    }
  }

  // Popup de commandes slash ("/").
  let cmdSel = $state(0);
  let cmdDismissed = $state(false);
  const cmdQuery = $derived(
    draft.startsWith("/") && !draft.includes(" ") ? draft.slice(1).toLowerCase() : null,
  );
  const cmdMatches = $derived(
    cmdQuery === null
      ? []
      : sessions.slashCommands.filter((c) => c.name.toLowerCase().startsWith(cmdQuery)).slice(0, 50),
  );
  const showCmds = $derived(cmdMatches.length > 0 && !cmdDismissed);
  $effect(() => {
    if (cmdSel >= cmdMatches.length) cmdSel = 0;
  });
  function pickCmd(c: { name: string }) {
    draft = "/" + c.name + " ";
    cmdDismissed = true;
  }
  function toggleCmds() {
    if (showCmds) {
      cmdDismissed = true;
      return;
    }
    sessions.loadSlashCommands();
    if (!draft.startsWith("/")) draft = "/";
    cmdDismissed = false;
    ta?.focus();
  }

  // Ferme la popup au clic hors du composer.
  let composerEl = $state<HTMLDivElement>();
  $effect(() => {
    if (!showCmds) return;
    const onDown = (e: PointerEvent) => {
      if (composerEl && !composerEl.contains(e.target as Node)) cmdDismissed = true;
    };
    window.addEventListener("pointerdown", onDown, true);
    return () => window.removeEventListener("pointerdown", onDown, true);
  });

  // Panneau Permissions (par chat).
  let showPerms = $state(false);
  const permActive = $derived(
    !!session &&
      ((session.permMode ?? "bypassPermissions") !== "bypassPermissions" ||
        (session.disabledTools?.length ?? 0) > 0 ||
        !!session.allowRules ||
        !!session.denyRules),
  );
  $effect(() => {
    if (!showPerms) return;
    const onDown = (e: PointerEvent) => {
      if (composerEl && !composerEl.contains(e.target as Node)) showPerms = false;
    };
    window.addEventListener("pointerdown", onDown, true);
    return () => window.removeEventListener("pointerdown", onDown, true);
  });

  // Auto-resize du champ : grandit jusqu'à 5 lignes (~78px) puis devient scrollable.
  // Réactif sur `draft` → gère la saisie ET le reset après envoi (draft = "").
  $effect(() => {
    void draft;
    if (!ta) return;
    ta.style.height = "auto";
    ta.style.height = Math.min(ta.scrollHeight, 78) + "px";
  });

  function submit(e: Event) {
    e.preventDefault();
    const text = draft.trim();
    if ((!text && images.length === 0) || !session) return;
    const imgs = images;
    draft = "";
    images = [];
    sessions.send(sid, text, imgs);
  }

  function onKey(e: KeyboardEvent) {
    if (showCmds) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        cmdSel = (cmdSel + 1) % cmdMatches.length;
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        cmdSel = (cmdSel - 1 + cmdMatches.length) % cmdMatches.length;
        return;
      }
      if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        pickCmd(cmdMatches[cmdSel]);
        return;
      }
      if (e.key === "Escape") {
        cmdDismissed = true;
        return;
      }
    }
    if (e.key === "Enter" && !e.shiftKey) submit(e);
  }
</script>

<div class="composer" bind:this={composerEl} onfocusin={() => sessions.setFocused(sid)}>
  {#if showCmds}
    <SlashPopup matches={cmdMatches} sel={cmdSel} onpick={pickCmd} />
  {/if}
  {#if showPerms}
    <PermissionsPopup {sid} />
  {/if}
  {#if images.length}
    <div class="thumbs">
      {#each images as img, i (img.dataUrl + i)}
        <div class="thumb" in:fly={{ y: 4, duration: 130 }}>
          <img src={img.dataUrl} alt={img.name} />
          <button
            type="button"
            class="thumb-x"
            use:tooltip={"Retirer l'image"}
            onclick={() => removeImage(i)}
          ><Icon name="close" size={11} /></button>
        </div>
      {/each}
    </div>
  {/if}
  <div class="meta">
    <button
      type="button"
      class="cmd-btn"
      class:on={showCmds}
      use:tooltip={"Commandes de Claude (/)"}
      onclick={toggleCmds}
    >/</button>
    <button
      type="button"
      class="cmd-btn"
      use:tooltip={"Joindre une image (ou coller depuis le presse-papiers)"}
      onclick={pickFiles}
    ><Icon name="image" size={13} /></button>
    <span class="perm-wrap">
      <button
        type="button"
        class="cmd-btn"
        class:on={permActive}
        use:tooltip={"Permissions de l'agent (mode + outils) · Ctrl+Tab pour changer de mode"}
        onclick={() => (showPerms = !showPerms)}
      ><Icon name="lock" size={13} /></button>
      {#if session?.permFlash}
        <span class="perm-flash" transition:fly={{ y: 4, duration: 120 }}>{session.permFlash}</span>
      {/if}
    </span>
    <input
      bind:this={fileInput}
      type="file"
      accept="image/*"
      multiple
      hidden
      onchange={(e) => {
        addFiles(e.currentTarget.files);
        e.currentTarget.value = "";
      }}
    />
    <span class="perm-wrap">
      <Dropdown
        label="Modèle"
        options={models}
        value={session?.model ?? ""}
        onchange={(v) => sessions.setModel(sid, v)}
      />
      {#if session?.modelFlash}
        <span class="perm-flash" transition:fly={{ y: 4, duration: 120 }}>{session.modelFlash}</span>
      {/if}
    </span>
    {#if efforts.length}
      <span class="perm-wrap">
        <Dropdown
          label="Effort"
          options={efforts}
          value={session?.effort ?? ""}
          btnClass={`eff-${session?.effort ?? "medium"}`}
          onchange={(v) => sessions.setEffort(sid, v)}
        />
        {#if session?.effortFlash}
          <span class="perm-flash" transition:fly={{ y: 4, duration: 120 }}>{session.effortFlash}</span>
        {/if}
      </span>
    {/if}
  </div>
  <form class="field eff-{session?.effort ?? 'medium'}" onsubmit={submit}>
    <textarea
      bind:this={ta}
      placeholder="Message à Claude…  (/ commandes · coller une image)"
      bind:value={draft}
      onkeydown={onKey}
      oninput={() => {
        cmdDismissed = false;
        sessions.touchActivity(sid);
      }}
      onfocus={() => { sessions.touchActivity(sid); sessions.setFocused(sid); }}
      onpaste={onPaste}
      rows="1"
    ></textarea>
    {#if (session?.queue?.length ?? 0) > 0}
      <span class="qchip" use:tooltip={"Messages en file (envoyés l'un après l'autre)"}>{session.queue.length}</span>
    {/if}
    <button
      class="send"
      type="submit"
      disabled={!draft.trim() && images.length === 0}
      use:tooltip={session?.streaming ? "Envoyer (pris en cours de route)" : "Envoyer (Entrée)"}
    ><Icon name="send" size={13} /></button>
  </form>

  <ContextGauge {sid} />
</div>

<style>
  .composer {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 4px;
    padding: 4px 5px;
    border-top: 1px solid var(--border);
    background: var(--surface-2);
    flex-shrink: 0;
    position: relative;
  }
  .thumbs {
    order: -1;
    width: 100%;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 2px 1px 4px;
  }
  .thumb {
    position: relative;
    width: 46px;
    height: 46px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    border: 1px solid var(--border);
    flex-shrink: 0;
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .thumb-x {
    position: absolute;
    top: 1px;
    right: 1px;
    width: 16px;
    height: 16px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: color-mix(in srgb, var(--bg) 70%, transparent);
    color: var(--text);
    backdrop-filter: blur(2px);
  }
  .thumb-x:hover {
    background: var(--danger);
    color: #fff;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
  }
  .cmd-btn {
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text-muted);
    font-family: var(--font-mono);
    font-weight: 700;
    font-size: 13px;
    line-height: 1;
    transition: border-color var(--transition), color var(--transition), background var(--transition);
  }
  .cmd-btn:hover {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .cmd-btn.on {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-weak);
  }
  .perm-wrap {
    position: relative;
    display: inline-flex;
  }
  .perm-flash {
    position: absolute;
    left: 50%;
    bottom: calc(100% + 6px);
    transform: translateX(-50%);
    white-space: nowrap;
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    font-size: 10.5px;
    font-family: var(--font-mono);
    box-shadow: 0 2px 10px color-mix(in srgb, var(--accent) 45%, transparent);
    z-index: 40;
    pointer-events: none;
  }
  .field {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
    padding: 2px 2px 2px 3px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    transition: border-color var(--transition);
  }
  .field:not(.eff-xhigh):not(.eff-max):focus-within {
    border-color: var(--accent);
  }
  .field.eff-low {
    border-color: var(--border);
  }
  .field.eff-medium {
    border-color: color-mix(in srgb, var(--accent) 25%, var(--border));
  }
  .field.eff-high {
    border-color: color-mix(in srgb, var(--accent) 55%, var(--border));
    animation: effPulse 2.4s ease-in-out infinite;
  }
  @keyframes effPulse {
    0%, 100% { box-shadow: 0 0 0 0 transparent; }
    50% { box-shadow: 0 0 10px color-mix(in srgb, var(--accent) 32%, transparent); }
  }
  .field.eff-xhigh,
  .field.eff-max {
    border: 1px solid transparent;
    background:
      linear-gradient(var(--bg), var(--bg)) padding-box,
      linear-gradient(90deg, var(--accent), #e6a988, var(--accent), #cf7ea6, var(--accent)) border-box;
    background-size: 100% 100%, 300% 100%;
    animation: effFlow 5s linear infinite;
  }
  .field.eff-max {
    animation: effFlow 2.3s linear infinite, effGlow 2s ease-in-out infinite;
  }
  @keyframes effFlow {
    to { background-position: 0 0, 300% 0; }
  }
  @keyframes effGlow {
    0%, 100% { box-shadow: 0 0 6px color-mix(in srgb, var(--accent) 30%, transparent); }
    50% { box-shadow: 0 0 18px color-mix(in srgb, var(--accent) 58%, transparent); }
  }
  textarea {
    flex: 1;
    resize: none;
    max-height: 78px; /* ~5 lignes */
    overflow-y: auto;
    overscroll-behavior: contain;
    padding: 3px 6px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 11px;
    line-height: 1.3;
    outline: none;
    min-width: 0;
  }
  textarea::placeholder {
    color: var(--text-faint);
  }
  .qchip {
    flex-shrink: 0;
    min-width: 18px;
    height: 18px;
    padding: 0 5px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 9px;
    background: var(--accent);
    color: #fff;
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
  }
  .send {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #fff;
    transition: background var(--transition), transform var(--transition), opacity var(--transition);
  }
  .send:not(:disabled):hover {
    background: var(--accent-hover);
  }
  .send:not(:disabled):active {
    transform: scale(0.9);
  }
  .send:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
