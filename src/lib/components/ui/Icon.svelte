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
  {:else if name === "minus"}
    <path d="M5 12h14" />
  {:else if name === "win-min"}
    <path d="M5 12h14" />
  {:else if name === "win-max"}
    <rect x="5" y="5" width="14" height="14" rx="2" />
  {:else if name === "win-close"}
    <path d="M6 6l12 12M18 6 6 18" />
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
  {:else if name === "settings"}
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
  {:else if name === "eye"}
    <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z" />
    <circle cx="12" cy="12" r="3" />
  {:else if name === "eye-off"}
    <path d="M9.9 4.24A9.1 9.1 0 0 1 12 4c6.5 0 10 7 10 7a18.5 18.5 0 0 1-2.16 3.19M6.6 6.6A18.3 18.3 0 0 0 2 11s3.5 7 10 7a9 9 0 0 0 5.4-1.8" />
    <path d="M9.9 9.9a3 3 0 0 0 4.2 4.2" />
    <path d="M2 2l20 20" />
  {:else if name === "wifi-off"}
    <path d="M2 2l20 20" />
    <path d="M8.5 16.4a5 5 0 0 1 7 0" />
    <path d="M5 12.9a10 10 0 0 1 3.1-2.1M19 12.9a10 10 0 0 0-7.5-2.85" />
    <path d="M1.4 9.2A15 15 0 0 1 6 6.2M22.6 9.2a15 15 0 0 0-9.5-3.06" />
    <path d="M12 20h.01" />
  {:else if name === "coin-up"}
    <circle cx="12" cy="12" r="8" />
    <path d="M12 15.5v-7M9.5 11 12 8.5 14.5 11" />
  {:else if name === "coin-down"}
    <circle cx="12" cy="12" r="8" />
    <path d="M12 8.5v7M9.5 13l2.5 2.5 2.5-2.5" />
  {:else if name === "sleep"}
    <path d="M4 8h6l-6 8h6" />
    <path d="M14 4h5l-5 6h5" />
  {:else if name === "chevron"}
    <path d="M6 9l6 6 6-6" />
  {:else if name === "check"}
    <path d="M5 12l4.5 4.5L19 7" />
  {:else if name === "image"}
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <circle cx="8.5" cy="9" r="1.6" />
    <path d="M21 16l-5-5L5 20" />
  {:else if name === "folder"}
    <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z" />
  {:else if name === "history"}
    <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
    <path d="M3 4v4h4" />
    <path d="M12 8v4l3 2" />
  {:else if name === "cpu"}
    <rect x="6" y="6" width="12" height="12" rx="2" />
    <rect x="9.5" y="9.5" width="5" height="5" rx="1" />
    <path d="M9 2.5v3.5M15 2.5v3.5M9 18v3.5M15 18v3.5M2.5 9h3.5M2.5 15h3.5M18 9h3.5M18 15h3.5" />
  {:else if name === "gauge"}
    <path d="M4.5 18a8 8 0 1 1 15 0" />
    <path d="M12 13.5l4.5-3.5" />
    <circle cx="12" cy="13.8" r="1.3" fill="currentColor" stroke="none" />
  {:else if name === "trash"}
    <path d="M4 7h16M9 7V5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2M6 7l1 13a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-13M10 11v6M14 11v6" />
  {:else if name === "book"}
    <path d="M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2V5Z" />
    <path d="M6 19h13" />
  {:else if name === "plug"}
    <path d="M9 2v6M15 2v6M7 8h10v3a5 5 0 0 1-10 0V8ZM12 16v6" />
  {:else if name === "package"}
    <path d="M21 8 12 3 3 8v8l9 5 9-5V8ZM3 8l9 5 9-5M12 13v8" />
  {:else if name === "logo-claude"}
    <!-- Anthropic "A" mark -->
    <path d="M12 3.5 L4 20.5 h3.8 l1.7-4.2h5l1.7 4.2h3.8z M10.3 13.1l1.7-4.2 1.7 4.2z" fill="currentColor" stroke="none" />
  {:else if name === "logo-opencode"}
    <!-- brackets >_ terminal -->
    <path d="M7 7.5 L3.5 12 L7 16.5" stroke-width={stroke} />
    <path d="M17 7.5 L20.5 12 L17 16.5" stroke-width={stroke} />
    <path d="M10.5 12 h7" stroke-width={stroke * 0.8} />
  {:else if name === "logo-gemini"}
    <!-- Gemini 4-pointed star -->
    <path d="M12 2C10.5 7.5 7.5 10.5 2 12C7.5 13.5 10.5 16.5 12 22C13.5 16.5 16.5 13.5 22 12C16.5 10.5 13.5 7.5 12 2Z" fill="currentColor" stroke="none" />
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
