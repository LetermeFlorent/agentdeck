# agentdeck

Application desktop (Tauri + Rust + Svelte 5) pour piloter **plusieurs IA en parallèle**.
Premier provider : **Claude Code** (sessions CLI orchestrées). L'architecture est multi-IA :
d'autres providers se branchent en implémentant le trait `Provider` côté Rust, sans toucher au frontend.

## Fonctionnalités

- **Panneaux splittables** (tiling récursif, axe X ou Y) — chaque pane = une session Claude Code indépendante.
- **Connexion à la première ouverture**, deux voies :
  1. *Se connecter à Anthropic* — lance `claude setup-token` (OAuth navigateur).
  2. *Importer le token* — lit `claude-token.txt` du dossier Téléchargements (ou collage manuel).
  - Token stocké **chiffré** dans le gestionnaire d'identifiants Windows (`keyring`), jamais en clair.
- **Usage 5h / 7j** : barres minimalistes + pourcentage.
  - ⚠️ Aucune API publique n'expose les vrais % d'abonnement (`/usage` est interactif). On affiche donc un
    **comptage local** des tokens consommés via l'app, sur fenêtres glissantes (libellé « estimé »).
- **Thème clair / sombre** façon Claude Code, qui suit le système (toggle système/clair/sombre).
- **Sessions persistées** : à la réouverture de l'app, on retrouve ses Claudes (reprise via `claude --resume`).

## Prérequis

- [Claude Code CLI](https://code.claude.com) installé et dans le `PATH` (`claude --version`).
- Node.js + Rust (toolchain stable).

## Développement

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Architecture

```
src-tauri/src/
  auth.rs                 token OAuth <-> keyring (Windows Credential Manager)
  provider/mod.rs         trait Provider (multi-IA) + TurnConfig
  provider/claude_code.rs adapter Claude Code : spawn `claude`, parse NDJSON -> events
  session.rs              SessionManager (UUID = --session-id / --resume)
  usage.rs                comptage local 5h / 7j
  events.rs               events normalisés (session://{id})
src/
  lib/stores/             theme, auth, usage, sessions, layout (tiling), persist
  lib/components/         Onboarding, ChatPane, SplitContainer, UsageBars, ThemeToggle
```

Chaque tour de conversation lance `claude -p <prompt> --output-format stream-json --verbose
--include-partial-messages` (1er tour : `--session-id <uuid>` ; suivants : `--resume <uuid>`),
avec `CLAUDE_CODE_OAUTH_TOKEN` injecté en environnement.
