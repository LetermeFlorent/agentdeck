<script lang="ts">
  // Jeu d'icônes line épurées (style Material Symbols / SF Symbols), stroke = couleur courante.
  let {
    name,
    size = 18,
    stroke = 1.9,
  }: { name: string; size?: number; stroke?: number } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === "close"}
    <path d="M18 6 6 18M6 6l12 12" />
  {:else if name === "stop"}
    <rect x="6" y="6" width="12" height="12" rx="2.5" fill="currentColor" stroke="none" />
  {:else if name === "send"}
    <path d="M12 20V5M5.5 11.5 12 5l6.5 6.5" />
  {:else if name === "plus"}
    <path d="M12 5v14M5 12h14" />
  {:else if name === "split-h"}
    <!-- deux colonnes côte à côte -->
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <path d="M12 4v16" />
  {:else if name === "split-v"}
    <!-- deux lignes empilées -->
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <path d="M3 12h18" />
  {:else if name === "sun"}
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
  {:else if name === "moon"}
    <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
  {:else if name === "monitor"}
    <rect x="2" y="3.5" width="20" height="13" rx="2" />
    <path d="M8 20.5h8M12 16.5v4" />
  {:else if name === "logout"}
    <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9" />
  {:else if name === "lock"}
    <rect x="4.5" y="11" width="15" height="9" rx="2" />
    <path d="M8 11V7a4 4 0 0 1 8 0v4" />
  {:else if name === "download"}
    <path d="M12 4v10M8 10l4 4 4-4M5 19h14" />
  {:else if name === "key"}
    <circle cx="8" cy="15" r="4" />
    <path d="M10.8 12.2 20 3M16 7l2 2M14 9l2 2" />
  {:else if name === "terminal"}
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <path d="M7 9l3 3-3 3M13 15h4" />
  {:else if name === "grip"}
    <g fill="currentColor" stroke="none">
      <circle cx="9" cy="6" r="1.4" />
      <circle cx="15" cy="6" r="1.4" />
      <circle cx="9" cy="12" r="1.4" />
      <circle cx="15" cy="12" r="1.4" />
      <circle cx="9" cy="18" r="1.4" />
      <circle cx="15" cy="18" r="1.4" />
    </g>
  {/if}
</svg>

<style>
  svg {
    display: block;
    flex-shrink: 0;
  }
</style>
