# Contribuer à agentdeck

Merci de l'intérêt porté au projet. agentdeck est sous licence
**PolyForm Noncommercial 1.0.0** : les contributions sont les bienvenues pour
un usage personnel et non-commercial.

## Prérequis dev

- **Node.js** 20+
- **Rust** toolchain stable (`rustup`)
- **Windows 10/11** (cible principale ; macOS/Linux à venir)
- [Claude Code CLI](https://code.claude.com) dans le `PATH` pour tester l'app

## Installation

```bash
npm install
npm run tauri dev
```

## Avant d'ouvrir une PR

Vérifie que tout passe :

```bash
npm run check                       # svelte-check + typecheck
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

## Style & commits

- Commits courts et explicites (format conventionnel apprécié : `fix:`, `feat:`, `docs:`…).
- Une PR = un sujet. Décris le *pourquoi*, pas seulement le *quoi*.
- Pas de secret/token/clé dans les commits. Le coffre token reste géré par le `keyring` OS.

## Signaler un bug

Ouvre une [issue](../../issues/new/choose) avec : étapes de repro, comportement
attendu vs observé, version de l'app, version Windows, et le fichier de log si pertinent
(dossier de logs de l'app).

## Architecture

Voir la section *Architecture* du [README](README.md). Pour ajouter un provider IA,
implémente le trait `Provider` côté Rust (`src-tauri/src/provider/`) — le frontend
n'a pas à changer.
