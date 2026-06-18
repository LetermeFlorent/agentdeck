<script lang="ts">
  import { usage } from "$lib/stores/usage.svelte";

  function fmt(n: number): string {
    if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + "M";
    if (n >= 1_000) return (n / 1_000).toFixed(0) + "k";
    return String(n);
  }
  function tone(pct: number): string {
    if (pct >= 90) return "var(--danger)";
    if (pct >= 70) return "var(--warn)";
    return "var(--accent)";
  }
  // Compte à rebours avant réinitialisation de la fenêtre.
  function resetIn(epoch: number | null): string {
    if (!epoch) return "";
    const s = epoch - Math.floor(Date.now() / 1000);
    if (s <= 0) return "réinit imminente";
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    return h > 0 ? `réinit dans ${h}h${String(m).padStart(2, "0")}` : `réinit dans ${m}min`;
  }
  function tip(label: string, b: { pct: number; tokens: number; cap: number; resets_at: number | null }, cost: number, real: boolean): string {
    if (real) return `${label} : ${b.pct}% utilisé · ${resetIn(b.resets_at)} · $${cost.toFixed(2)} via l'app`;
    return `${label} : ${fmt(b.tokens)}/${fmt(b.cap)} tokens (estimé) · $${cost.toFixed(2)}`;
  }
</script>

<div class="usage">
  {#if usage.snapshot}
    {@const s = usage.snapshot}
    {@const real = s.source === "real"}
    <div class="row" title={tip("5h", s.five_h, s.five_h_cost, real)}>
      <span class="lbl">5h</span>
      <div class="track">
        <div class="fill" style={`width:${s.five_h.pct}%;background:${tone(s.five_h.pct)}`}></div>
      </div>
      <span class="pct">{s.five_h.pct}%</span>
    </div>
    <div class="row" title={tip("7j", s.week, s.week_cost, real)}>
      <span class="lbl">7j</span>
      <div class="track">
        <div class="fill" style={`width:${s.week.pct}%;background:${tone(s.week.pct)}`}></div>
      </div>
      <span class="pct">{s.week.pct}%</span>
    </div>
    {#if real}
      <span class="src real" title="Vraies limites d'abonnement (5h / 7j) lues depuis claude-statusbar.">réel</span>
    {:else}
      <span class="src" title="Pas de donnée réelle dispo (claude-statusbar absent) — estimation locale des tokens consommés via l'app.">estimé</span>
    {/if}
  {/if}
</div>

<style>
  .usage {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .lbl {
    font-size: 11px;
    color: var(--text-muted);
    width: 16px;
    font-variant-numeric: tabular-nums;
  }
  .track {
    width: 84px;
    height: 6px;
    background: var(--track);
    border-radius: 4px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: 4px;
    transition: width var(--transition);
  }
  .pct {
    font-size: 11px;
    color: var(--text-muted);
    width: 30px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .src {
    font-size: 10px;
    color: var(--text-faint);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 5px;
    cursor: help;
  }
  .src.real {
    color: var(--accent);
    border-color: var(--accent-weak);
    background: var(--accent-weak);
  }
</style>
