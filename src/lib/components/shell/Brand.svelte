<script lang="ts">
  // Marque agentdeck (icône + nom) cliquable : bascule la vue skills (graphe). Affiche
  // aussi le badge d'abonnement Claude et le compte connecté.
  import SparkMark from "$lib/components/ui/SparkMark.svelte";
  import { tooltip } from "$lib/actions/tooltip";

  let {
    plan,
    showSkills = $bindable(false),
  }: {
    plan: { label: string; level: number; account: string };
    showSkills?: boolean;
  } = $props();
</script>

<div class="brand" data-tour="brand" data-tauri-drag-region>
  <button
    class="brand-btn"
    class:on={showSkills}
    use:tooltip={showSkills ? "Retour aux chats" : "Vue skills (graphe)"}
    onclick={() => (showSkills = !showSkills)}
  >
    <span class="mark"><SparkMark size={16} /></span>
    <span class="logo">agentdeck</span>
  </button>
  {#if plan.label}
    <span class="plan plan-{plan.level}" data-tauri-drag-region use:tooltip={`Abonnement Claude : ${plan.label}`}>
      {plan.label}
    </span>
  {/if}
  {#if plan.account}
    <span class="account" data-tauri-drag-region use:tooltip={`Compte connecté : ${plan.account}`}>{plan.account}</span>
  {/if}
</div>

<style>
  .brand { display: flex; align-items: center; gap: 9px; }
  .brand-btn { display: flex; align-items: center; gap: 9px; padding: 2px 5px; margin: -2px -3px; border-radius: var(--radius-sm); cursor: pointer; transition: background var(--transition); }
  .brand-btn:hover { background: var(--surface-2); }
  .brand-btn.on { background: var(--accent-weak); }
  .brand-btn.on .logo { color: var(--accent); }
  .mark { display: flex; align-items: center; }
  .logo { font-family: var(--font-mono); font-weight: 600; font-size: 13.5px; letter-spacing: -0.01em; }
  .plan { margin-left: 2px; font-family: var(--font-mono); font-weight: 700; line-height: 1; border-radius: 5px; white-space: nowrap; border: 1px solid transparent; }
  .plan-1 { font-size: 9px; padding: 2px 5px; color: var(--text-muted); background: var(--surface-2); border-color: var(--border); }
  .plan-2 { font-size: 9px; padding: 2px 5px; color: var(--accent); background: var(--accent-weak); border-color: color-mix(in srgb, var(--accent) 30%, transparent); }
  .plan-3 { font-size: 9.5px; padding: 2px 6px; color: #fff; background: linear-gradient(120deg, var(--accent), var(--accent-hover)); box-shadow: 0 2px 10px color-mix(in srgb, var(--accent) 40%, transparent); }
  .plan-4 { font-size: 10.5px; padding: 2px 7px; color: #fff; background: linear-gradient(120deg, #e0825f, var(--accent) 45%, #b94f9e); background-size: 200% 100%; box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 35%, transparent), 0 3px 16px color-mix(in srgb, var(--accent) 55%, transparent); animation: planShimmer 4s linear infinite, planGlow 2.6s ease-in-out infinite; text-shadow: 0 0 8px rgba(255, 255, 255, 0.35); }
  @keyframes planShimmer { 0% { background-position: 0% 0; } 100% { background-position: 200% 0; } }
  @keyframes planGlow { 0%, 100% { box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 35%, transparent), 0 3px 14px color-mix(in srgb, var(--accent) 45%, transparent); } 50% { box-shadow: 0 0 0 1px color-mix(in srgb, var(--accent) 50%, transparent), 0 4px 22px color-mix(in srgb, var(--accent) 70%, transparent); } }
  .account { font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
