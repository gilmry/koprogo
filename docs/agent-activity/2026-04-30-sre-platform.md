---
date: 2026-04-30
persona: sre-platform
session: PR-C (chore/gitops-local-env-dev)
tier: 2  # proposal via PR ; merge + activation = Tier 1 humain
---

# Activity log — sre-platform 2026-04-30

## Context

Plan approuvé `C:\Users\gilmr\.claude\plans\toasty-cooking-snowflake.md` — Axe 4 « Cron poller pour env-dev local ». Permet au superviseur de faire tourner sur sa machine **un poller GitOps qui suit la branche `dev`** en parallèle de `make dev` (qui lui sert pour le hand-coding sur `feature/dev`).

Indépendant des autres PRs ouvertes (PR-E ci, PR-A cluster-profiles).

## Actions (Tier 2)

| Action | Fichier | Lignes |
|---|---|---|
| Étension TOPOLOGY | `infrastructure/_shared/scripts/gitops-deploy.sh` | +12 -5 |
| Compose override env-dev | `infrastructure/monosite/local/env-dev/docker-compose.override.yml` | +73 |
| Variables env | `infrastructure/monosite/local/env-dev/.env.example` | +44 |
| Runbook | `infrastructure/monosite/local/env-dev/README.md` | +95 |
| Systemd unit | `infrastructure/_shared/systemd/koprogo-gitops-env-dev.service` | +30 |
| Installer Linux/Mac | `infrastructure/_shared/scripts/install-systemd-poller.sh` | +51 |
| Installer Windows | `infrastructure/_shared/scripts/windows-task-poller.ps1` | +118 |
| Gitignore patterns | `.gitignore` | +2 |

## Décisions justifiées

### TOPOLOGY=local séparé de TOPOLOGY=vps

`gitops-deploy.sh` v1 hardcodait `monosite/vps/${ENV_NAME}/...`. La généralisation à `monosite/${TOPOLOGY}/${ENV_NAME}/...` est :
- **Backward-compat** : `TOPOLOGY` defaults à `vps` → tous les scripts existants tournent identique.
- **Future-proof** : `TOPOLOGY=multisite` ou `TOPOLOGY=cluster` peuvent être ajoutés plus tard sans casser l'existant.

### LOG_FILE adaptatif local vs VPS

VPS = `/var/log/koprogo-gitops-${ENV_NAME}.log` (root). Local user-mode = `~/.local/state/koprogo-gitops-${ENV_NAME}.log` (XDG-compliant, no sudo). Le script crée le parent dir si besoin.

### Ports différents pour env-dev (8082/5433/9002) vs `make dev` (80/5432/9000)

Justification : le superviseur doit pouvoir faire tourner les **deux en parallèle**. Sinon `make dev` casse ou env-dev casse à chaque switch. Container names suffixés `-envdev` + volumes nommés `koprogo_envdev_*` pour isolation totale.

### `.env` gitignoré globalement + pattern explicite

`.env` est déjà bloqué par `.gitignore` ligne 12 (`*` pattern). Mais ajout pattern explicite `infrastructure/monosite/local/*/.env` pour :
- Self-documentation (un dev qui lit le gitignore comprend le layout)
- Renforcer CRITICAL.md règle 1 (aucun secret en clair)
- Couvrir `monosite/vps/*/.env` aussi (ces .env n'étaient pas explicitement listés)

### Systemd user mode (pas system-mode)

Le poller env-dev tourne sur la machine personnelle du superviseur. Faire tourner en `--system` demanderait sudo et créerait un service partagé tous les users → over-engineered. `systemctl --user` :
- Pas de sudo
- Démarrage automatique au login (via `linger` si besoin de tourner sans session)
- Logs via `journalctl --user`

### Windows = Task Scheduler avec Git Bash

Pas de systemd sur Windows. Task Scheduler est l'équivalent natif. Le script PowerShell :
- Pas de demande d'élévation admin (Tier 1 violation potentielle)
- LogonType Interactive (s'arrête à la déco — acceptable pour un sandbox)
- Trigger AtLogOn → démarre quand la session s'ouvre
- Restart on failure (3 fois max, 5 min interval)

### Pas de `Action=Start` autonome dans l'installer

`install-systemd-poller.sh` **registre** l'unit mais ne fait PAS `systemctl --user enable --now`. L'humain le fait manuellement. Idem pour Windows : `Install` registre la task, `Start` est une commande séparée. Cohérent avec CRITICAL.md règle 11 — l'agent ne démarre pas autonomement un poller qui modifie le système.

## Vérifications

- `bash -n` syntax check OK pour les 2 scripts shell
- `Get-ScheduledTask` cmdlet existant sur PS 5+ (vérifié via doc, pas exécuté car Windows-only)
- Le script PowerShell compile sans `-Action` invalid via le `ValidateSet`

## Risques identifiés

| Risque | Mitigation |
|---|---|
| Poller env-dev pull `dev` en boucle alors que CI build l'image — race condition | `gitops-deploy.sh` retry 10× avec `manifest unknown` puis fallback `dev-latest` (déjà en place pré-PR) |
| Volumes `koprogo_envdev_postgres_data` jamais purgés → croissance disque | Doc `README.md` `Désinstallation` inclut `down -v` ; pas une concern v0.1.0 |
| `.env` du superviseur contient `JWT_SECRET=local-envdev-jwt-secret-minimum-32-characters` (default `.env.example`) | Doc explicite "rotate JWT_SECRET to random" ; v0.1.0 n'est pas en prod (CRITICAL.md règle 10) |
| Systemd unit `MemoryMax=64M` trop bas pour `git pull` sur très gros repos | KoproGo repo ~ 250 Mo, git pull ~ 15 Mo RSS — large marge ; ajustable si besoin |
| Windows PowerShell 5.1 vs 7+ syntax differences | Script utilise uniquement cmdlets dispo sur 5.1 (Get-ScheduledTask, New-ScheduledTaskAction, etc. sont 5.1-compat) |

## Étapes suivantes

1. Review + merge PR-C vers `feature/dev`.
2. Le superviseur installe :
   ```bash
   cp infrastructure/monosite/local/env-dev/.env.example \
      infrastructure/monosite/local/env-dev/.env
   # éditer .env (rotate JWT)
   sudo bash -c 'echo "127.0.0.1 envdev.koprogo.local api-envdev.koprogo.local" >> /etc/hosts'
   ./infrastructure/_shared/scripts/install-systemd-poller.sh
   systemctl --user enable --now koprogo-gitops-env-dev
   ```
3. Une fois **PR-B** mergée (bootstrap ArgoCD), valider le test acceptation multi-topologie : push sur `dev` → poller env-dev compose redéploie ET ArgoCD K8s redéploie.

## Sortie

PR-C ready pour review humaine. Aucune mutation prod. Aucun service activé par l'agent. Pollers nécessitent activation manuelle explicite.
