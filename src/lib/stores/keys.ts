// Clés localStorage centralisées (un seul endroit pour éviter les fautes de frappe / doublons).

export const STORAGE_KEYS = {
  deck: "agentdeck.deck.v1",
  theme: "agentdeck.theme",
  settings: "agentdeck.settings.v1",
  slash: "agentdeck.slash.v2",
  onboarded: "agentdeck.onboarded.v1",
} as const;
