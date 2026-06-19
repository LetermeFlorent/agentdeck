# Changelog

Format basé sur [Keep a Changelog](https://keepachangelog.com/), versions en [SemVer](https://semver.org/).

## [0.1.0] — 2026-06-19

Première version distribuée.

### Ajouté
- Pilotage de plusieurs sessions Claude Code en parallèle (panneaux splittables, tiling récursif).
- Onboarding : connexion OAuth (`claude setup-token`) ou import de token ; token chiffré dans le
  gestionnaire d'identifiants de l'OS (keyring).
- Barres d'usage 5h / 7j (comptage local estimé + vrai usage si disponible via l'endpoint OAuth).
- Thème clair / sombre suivant le système.
- Sessions persistées et reprises (`claude --resume`).
- Mode Hermes (auto-apprentissage : l'agent consulte/crée des skills).
- Auto-model / auto-effort, modes de permission, modes privés.

### Robustesse / perf
- `parking_lot::Mutex` (plus de mutex poisoning).
- Poller usage : token mis en cache + client HTTP réutilisé ; `snapshot()` en une seule passe.
- stderr du process `claude` capturé et remonté en cas de crash (diagnostics).
- Temp dir des pièces jointes mis en cache.
- Persistance localStorage debouncée ; listeners non dupliqués ; hydratation des sessions parallélisée.

### Sécurité
- CSP durcie dans la webview.

[0.1.0]: https://github.com/LetermeFlorent/agentdeck/releases/tag/v0.1.0
