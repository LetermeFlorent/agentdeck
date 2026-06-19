// Activité en cours par chat : sous-agents (Task) + shells (Bash/PowerShell) qui tournent.
// Alimenté par les events du flux (task_*, tool_use/tool_done). Runtime seulement (non persisté).

import type { SessionEvent } from "$lib/ipc";

export interface SubAgent {
  taskId: string;
  description: string;
  type: string;
  prompt: string;
  action: string;
  lastTool: string;
  tokens: number;
  durationMs: number;
  startTs: number;
  status: string; // "" = en cours
}
export interface Shell {
  id: string;
  command: string;
  startTs: number;
}

const SHELL_TOOLS = new Set(["Bash", "PowerShell"]);

class ActivityStore {
  subagents = $state<Record<string, Record<string, SubAgent>>>({});
  shells = $state<Record<string, Record<string, Shell>>>({});
  private now = () => Date.now();

  private subs(sid: string) {
    return (this.subagents[sid] ??= {});
  }
  private sh(sid: string) {
    return (this.shells[sid] ??= {});
  }

  /** Traite un event du flux pour la session `sid`. */
  handle(sid: string, e: SessionEvent) {
    switch (e.kind) {
      case "task_started":
        this.subs(sid)[e.task_id] = {
          taskId: e.task_id,
          description: e.description,
          type: e.subagent_type,
          prompt: e.prompt,
          action: "",
          lastTool: "",
          tokens: 0,
          durationMs: 0,
          startTs: this.now(),
          status: "",
        };
        break;
      case "task_progress": {
        const s = this.subs(sid)[e.task_id];
        if (s) {
          s.action = e.action;
          s.lastTool = e.last_tool;
          s.tokens = e.tokens;
          s.durationMs = e.duration_ms;
        }
        break;
      }
      case "task_ended":
        delete this.subs(sid)[e.task_id];
        break;
      case "tool_use":
        if (SHELL_TOOLS.has(e.name) && e.id) {
          this.sh(sid)[e.id] = { id: e.id, command: e.input, startTs: this.now() };
        }
        break;
      case "tool_done":
        if (this.shells[sid]) delete this.shells[sid][e.id];
        break;
    }
  }

  /** Vide l'activité d'un chat (fin de tour / fermeture). */
  clear(sid: string) {
    this.subagents[sid] = {};
    this.shells[sid] = {};
  }

  subList(sid: string): SubAgent[] {
    return Object.values(this.subagents[sid] ?? {});
  }
  shellList(sid: string): Shell[] {
    return Object.values(this.shells[sid] ?? {});
  }
  count(sid: string): number {
    return this.subList(sid).length + this.shellList(sid).length;
  }
}

export const activity = new ActivityStore();
