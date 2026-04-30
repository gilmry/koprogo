# Local env-dev — GitOps cron poller (Mode A)

Topologie docker-compose tournant **sur la machine du superviseur**, suivie automatiquement par le cron poller `gitops-deploy.sh`. Itère sur la branche `dev` (la branche officielle GitFlow, pas `feature/dev`).

## Différence avec `make dev`

| | `make dev` (root Makefile) | `env-dev` cron poller |
|---|---|---|
| **Branche source** | code local non commit | branche `dev` officielle (post-merge feature/dev → dev) |
| **Build** | local `cargo-watch` + Astro hot reload | image GHCR pré-buildée par `docker-build-push.yml` |
| **Ports** | 80 / 5432 / 9000 (host) | 8082 / 5433 / 9002 (host) — évite collision |
| **Volumes** | `koprogo_postgres_data` | `koprogo_envdev_postgres_data` (isolé) |
| **Cas d'usage** | dev itératif main, debug, hot-reload | valider que le merge vers `dev` produit bien un déploiement fonctionnel avant promotion vers integration |

Tu peux faire tourner les **deux en parallèle** (ports différents).

## Setup

### 1. Copier les variables

```bash
cp infrastructure/monosite/local/env-dev/.env.example \
   infrastructure/monosite/local/env-dev/.env
# Éditer .env si nécessaire (changer JWT_SECRET pour de l'aléatoire)
```

> Le fichier `.env` est gitignoré (CRITICAL.md règle 1 — pas de secrets en clair commit).

### 2. Configurer `/etc/hosts` (Linux/Mac) ou `C:\Windows\System32\drivers\etc\hosts` (Windows)

```
127.0.0.1 envdev.koprogo.local api-envdev.koprogo.local
```

### 3. Installer le poller

**Linux/Mac (systemd user mode)** :
```bash
./infrastructure/_shared/scripts/install-systemd-poller.sh
systemctl --user enable --now koprogo-gitops-env-dev
journalctl --user -u koprogo-gitops-env-dev -f
```

**Windows (Task Scheduler)** :
```powershell
# Run as user (not admin)
.\infrastructure\_shared\scripts\windows-task-poller.ps1 -Action Install
.\infrastructure\_shared\scripts\windows-task-poller.ps1 -Action Status
```

### 4. Vérifier

```bash
curl http://envdev.koprogo.local:8082/health        # frontend
curl http://api-envdev.koprogo.local:8082/api/v1/health   # backend
```

## Test du flux GitOps multi-topologie

1. Le poller env-dev tourne (systemd ou Task Scheduler).
2. `git push origin dev` (depuis n'importe quel clone).
3. Sous 2-3 min : poller détecte le commit, `compose pull` + `compose up -d` automatique.
4. Refresh la page → nouvelle version visible.

Si **PR-B** (ArgoCD bootstrap) est aussi installée sur Docker Desktop K8s, tu peux observer **les deux** se redéployer en parallèle :
- `journalctl --user -u koprogo-gitops-env-dev -f` (compose env-dev)
- `kubectl get app koprogo-app-dev -n argocd -w` (K8s dev namespace)

## Désinstallation

```bash
# Linux/Mac
systemctl --user disable --now koprogo-gitops-env-dev
rm ~/.config/systemd/user/koprogo-gitops-env-dev.service
docker compose -f infrastructure/_shared/docker-compose/docker-compose.base.yml \
               -f infrastructure/monosite/local/env-dev/docker-compose.override.yml \
               --env-file infrastructure/monosite/local/env-dev/.env \
               down -v

# Windows
.\infrastructure\_shared\scripts\windows-task-poller.ps1 -Action Uninstall
```

## Troubleshooting

| Symptôme | Cause probable | Fix |
|---|---|---|
| `manifest unknown` lors de pull | Tag `dev-{sha7}` pas encore publié (CI en cours) | poller retry 10× automatiquement, puis fallback `dev-latest` |
| Port 8082 already in use | Déjà un service sur 8082 | éditer `docker-compose.override.yml` pour autre port |
| Backend `JWT_SECRET must be at least 32 characters` | `.env` non chargé | vérifier que `.env` existe dans `monosite/local/env-dev/` |
| `git pull` fail avec auth | clone HTTPS sans token | utiliser ssh remote ou personal access token |
