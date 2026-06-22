// Bibliothèque : skills + serveurs MCP. Listes « installées » via le backend (disque / CLI),
// catalogues « disponibles » récupérés dynamiquement sur le web (fetch, CSP désactivée).

import * as ipc from "$lib/ipc";

export interface CatalogSkill {
  name: string;
  description: string;
}
export interface CatalogMcp {
  name: string; // identifiant court (slug) pour `claude mcp add`
  title: string;
  description: string;
  url: string;
  transport: string;
}

const SKILLS_REPO = "anthropics/skills";
const SKILLS_RAW = `https://raw.githubusercontent.com/${SKILLS_REPO}/main/skills`;
const MCP_REGISTRY = "https://registry.modelcontextprotocol.io/v0/servers?limit=100";

/** Extrait `description:` du frontmatter YAML d'un SKILL.md. */
function parseDesc(md: string): string {
  const lines = md.split(/\r?\n/);
  if (lines[0]?.trim() !== "---") return "";
  for (let i = 1; i < lines.length; i++) {
    const t = lines[i].trim();
    if (t === "---") break;
    const m = /^description:\s*(.+)$/.exec(t);
    if (m) return m[1].trim().replace(/^["']|["']$/g, "").trim();
  }
  return "";
}

function slug(name: string): string {
  return (name.split("/").pop() || name).replace(/[^a-zA-Z0-9_-]/g, "-").toLowerCase();
}

class LibraryStore {
  installedSkills = $state<ipc.SkillItem[]>([]);
  installedPlugins = $state<ipc.PluginItem[]>([]);
  installedMcp = $state<ipc.McpItem[]>([]);
  catalogSkills = $state<CatalogSkill[]>([]);
  catalogMcp = $state<CatalogMcp[]>([]);
  loadingCat = $state(false);
  busy = $state(""); // nom de l'item en cours d'action (install/delete)
  error = $state("");
  /** Skills dont la description a déjà été demandée (évite les fetch répétés). */
  private skillDescReq = new Set<string>();

  async refreshSkills() {
    try {
      this.installedSkills = await ipc.skillsInstalled();
    } catch (e) {
      this.error = String(e);
    }
  }
  async refreshPlugins() {
    try {
      this.installedPlugins = await ipc.pluginsInstalled();
    } catch (e) {
      this.error = String(e);
    }
  }
  async uninstallPlugin(id: string, scope?: string) {
    this.busy = id;
    this.error = "";
    try {
      await ipc.pluginUninstall(id, scope);
      await this.refreshPlugins();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.busy = "";
    }
  }
  async refreshMcp() {
    try {
      this.installedMcp = await ipc.mcpInstalled();
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Catalogue de skills : liste rapide des dossiers du repo (descriptions chargées à la demande). */
  async loadSkillCatalog() {
    if (this.catalogSkills.length) return;
    this.loadingCat = true;
    this.error = "";
    try {
      const res = await fetch(`https://api.github.com/repos/${SKILLS_REPO}/contents/skills`, {
        headers: { Accept: "application/vnd.github+json" },
      });
      const dirs: { name: string; type: string }[] = await res.json();
      // On affiche les noms immédiatement ; chaque description est chargée quand sa carte apparaît.
      this.catalogSkills = dirs
        .filter((d) => d.type === "dir")
        .map((d) => ({ name: d.name, description: "" }));
    } catch (e) {
      this.error = `Catalogue skills indisponible : ${e}`;
    } finally {
      this.loadingCat = false;
    }
  }

  /** Charge à la demande la description d'un skill du catalogue (carte devenue visible). */
  async loadSkillDesc(name: string) {
    if (this.skillDescReq.has(name)) return;
    this.skillDescReq.add(name);
    try {
      const md = await (await fetch(`${SKILLS_RAW}/${name}/SKILL.md`)).text();
      const desc = parseDesc(md);
      const it = this.catalogSkills.find((c) => c.name === name);
      if (it) it.description = desc;
    } catch {
      this.skillDescReq.delete(name); // échec → autorise une nouvelle tentative
    }
  }

  /** Catalogue MCP : registre officiel, on garde les serveurs avec un endpoint distant (URL). */
  async loadMcpCatalog() {
    if (this.catalogMcp.length) return;
    this.loadingCat = true;
    this.error = "";
    try {
      const res = await fetch(MCP_REGISTRY);
      const data: { servers: { server: any }[] } = await res.json();
      const seen = new Set<string>();
      const items: CatalogMcp[] = [];
      for (const { server } of data.servers ?? []) {
        const name = slug(server.name);
        if (seen.has(name)) continue;
        const base = { name, title: server.title || server.name, description: server.description || "" };
        const remote = server.remotes?.[0];
        const pkg = server.packages?.[0];
        if (remote?.url) {
          // Serveur distant (URL) → transport http/sse.
          items.push({ ...base, url: remote.url, transport: remote.type === "sse" ? "sse" : "http" });
        } else if (pkg?.identifier) {
          // Serveur local (paquet) → commande stdio (npx / uvx).
          const runner =
            pkg.registryType === "pypi" ? `uvx ${pkg.identifier}` : `npx -y ${pkg.identifier}`;
          items.push({ ...base, url: runner, transport: "stdio" });
        } else {
          continue;
        }
        seen.add(name);
      }
      this.catalogMcp = items;
    } catch (e) {
      this.error = `Catalogue MCP indisponible : ${e}`;
    } finally {
      this.loadingCat = false;
    }
  }

  async installSkill(name: string) {
    this.busy = name;
    this.error = "";
    try {
      const md = await (await fetch(`${SKILLS_RAW}/${name}/SKILL.md`)).text();
      await ipc.skillWrite(name, md);
      await this.refreshSkills();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.busy = "";
    }
  }

  async addOwnSkill(name: string, description: string, body: string) {
    const md = `---\nname: ${name}\ndescription: ${description}\n---\n\n${body.trim()}\n`;
    this.error = "";
    try {
      await ipc.skillWrite(name, md);
      await this.refreshSkills();
    } catch (e) {
      this.error = String(e);
      throw e;
    }
  }

  async deleteSkill(name: string) {
    this.busy = name;
    try {
      await ipc.skillDelete(name);
      await this.refreshSkills();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.busy = "";
    }
  }

  async addMcp(name: string, target: string, transport?: string) {
    this.busy = name;
    this.error = "";
    try {
      await ipc.mcpAdd(name, target, transport);
      await this.refreshMcp();
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.busy = "";
    }
  }

  async addMcpJson(name: string, json: string) {
    this.busy = name;
    this.error = "";
    try {
      await ipc.mcpAddJson(name, json);
      await this.refreshMcp();
    } catch (e) {
      this.error = String(e);
      throw e;
    } finally {
      this.busy = "";
    }
  }

  async removeMcp(name: string) {
    this.busy = name;
    try {
      await ipc.mcpRemove(name);
      await this.refreshMcp();
    } catch (e) {
      this.error = String(e);
    } finally {
      this.busy = "";
    }
  }

  isSkillInstalled(name: string): boolean {
    return this.installedSkills.some((s) => s.name === name);
  }
  isMcpInstalled(name: string): boolean {
    return this.installedMcp.some((m) => m.name === name || slug(m.name) === name);
  }
}

export const library = new LibraryStore();
