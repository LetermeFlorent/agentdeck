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
</script>

<div class="usage">
  {#if usage.snapshot}
    {@const s = usage.snapshot}
    <div class="row" title={`${fmt(s.five_h.tokens)} / ${fmt(s.five_h.cap)} tokens · $${s.five_h_cost.toFixed(2)} sur 5h`}>
      <span class="lbl">5h</span>
      <div class="track">
        <div class="fill" style={`width:${s.five_h.pct}%;background:${tone(s.five_h.pct)}`}></div>
      </div>
      <span class="pct">{s.five_h.pct}%</span>
    </div>
    <div class="row" title={`${fmt(s.week.tokens)} / ${fmt(s.week.cap)} tokens · $${s.week_cost.toFixed(2)} sur 7j`}>
      <span class="lbl">7j</span>
      <div class="track">
        <div class="fill" style={`width:${s.week.pct}%;background:${tone(s.week.pct)}`}></div>
      </div>
      <span class="pct">{s.week.pct}%</span>
    </div>
    {#if s.source === "estimated"}
      <span class="src" title="Aucune API publique ne donne les vrais % d'abonnement — valeur estimée localement (tokens consommés via l'app).">estimé</span>
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
</style>
