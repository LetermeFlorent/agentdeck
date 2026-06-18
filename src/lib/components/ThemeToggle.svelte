<script lang="ts">
  import { theme, type ThemeChoice } from "$lib/stores/theme.svelte";

  const opts: { value: ThemeChoice; label: string; icon: string }[] = [
    { value: "system", label: "Système", icon: "🖥" },
    { value: "light", label: "Clair", icon: "☀" },
    { value: "dark", label: "Sombre", icon: "☾" },
  ];
</script>

<div class="seg" role="group" aria-label="Thème">
  {#each opts as o}
    <button
      class="seg-item"
      class:active={theme.choice === o.value}
      title={o.label}
      aria-pressed={theme.choice === o.value}
      onclick={() => theme.set(o.value)}
    >
      <span aria-hidden="true">{o.icon}</span>
    </button>
  {/each}
</div>

<style>
  .seg {
    display: inline-flex;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 2px;
    gap: 2px;
  }
  .seg-item {
    width: 30px;
    height: 26px;
    border-radius: 5px;
    color: var(--text-muted);
    font-size: 13px;
    transition: background var(--transition), color var(--transition);
  }
  .seg-item:hover {
    color: var(--text);
  }
  .seg-item.active {
    background: var(--elevated);
    color: var(--text);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.12);
  }
</style>
