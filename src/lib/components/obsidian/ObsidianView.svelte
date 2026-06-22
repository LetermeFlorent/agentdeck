<script lang="ts">
  import * as ipc from "$lib/ipc";
  import type { SkillItem, McpItem, PluginItem } from "$lib/ipc";
  import { sessions } from "$lib/stores/sessions.svelte";
  import { tabs } from "$lib/stores/tabs.svelte";
  import { buildGraph, type OpenChat } from "./graph-model";
  import type { GNode } from "./graph-types";
  import GraphCanvas from "./GraphCanvas.svelte";
  import SkillInspector from "./SkillInspector.svelte";

  let { onexit, onopenchat }: { onexit: () => void; onopenchat: (sid: string) => void } = $props();

  let chats = $state<OpenChat[]>([]);
  let globalSkills = $state<SkillItem[]>([]);
  let projectSkills = $state<Map<string, SkillItem[]>>(new Map());
  let mcps = $state<McpItem[]>([]);
  let plugins = $state<PluginItem[]>([]);
  let loading = $state(true);
  let error = $state("");

  let selected = $state<GNode | null>(null);
  let hoverId = $state<string | null>(null);

  const graph = $derived(buildGraph(chats, globalSkills, projectSkills, mcps, plugins));
  const focusId = $derived(hoverId ?? selected?.id ?? null);

  $effect(() => void load());

  // Garde-fou : un invoke qui ne répond jamais ne doit pas bloquer le graphe → fallback après 4s.
  function withTimeout<T>(p: Promise<T>, fb: T): Promise<T> {
    return Promise.race([
      p.catch((e) => { console.error("[obsidian] ipc échec", e); return fb; }),
      new Promise<T>((res) => setTimeout(() => res(fb), 4000)),
    ]);
  }

  async function load() {
    loading = true;
    try {
      const ids = tabs.allSessionIds();
      chats = ids
        .map((sid) => sessions.map[sid])
        .filter(Boolean)
        .map((s) => ({ sid: s.id, title: s.title, provider: s.provider, cwd: s.cwd || "" }));
      globalSkills = await withTimeout(ipc.skillsInstalled(), []);
      mcps = await withTimeout(ipc.mcpInstalled(), []);
      plugins = await withTimeout(ipc.pluginsInstalled(), []);
      const cwds = [...new Set(chats.map((c) => c.cwd).filter(Boolean))];
      const proj = new Map<string, SkillItem[]>();
      await Promise.all(cwds.map(async (cwd) => proj.set(cwd, await withTimeout(ipc.projectSkills(cwd), []))));
      projectSkills = proj;
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function pick(n: GNode) {
    if (n.kind === "chat" && n.sid) onopenchat(n.sid);
    else if (n.kind === "skill") selected = n;
    else selected = null; // claude / mcp / plugin : pas d'inspecteur, juste le survol
  }
</script>

<section class="obs">
  <div class="stage">
    {#if loading}
      <div class="hint">Chargement du graphe…</div>
    {:else if error}
      <div class="hint err">{error}</div>
    {:else if graph.nodes.length === 0}
      <div class="hint">Aucun chat ouvert ni skill installé. Ouvre un chat ou crée un skill.</div>
    {:else}
      <GraphCanvas {graph} {focusId} onpick={pick} onhover={(id) => (hoverId = id)} />
    {/if}
    {#if selected}
      <SkillInspector node={selected} onclose={() => (selected = null)} onchanged={load} />
    {/if}
  </div>
</section>

<style>
  .obs { display: flex; flex-direction: column; height: 100%; background: radial-gradient(circle at 50% 40%, var(--surface) 0%, var(--bg) 70%); }
  .stage { position: relative; flex: 1; min-height: 0; }
  .hint { position: absolute; inset: 0; display: grid; place-items: center; color: var(--text-muted); font-size: 13px; }
  .hint.err { color: var(--danger); font-family: var(--font-mono); }
</style>
