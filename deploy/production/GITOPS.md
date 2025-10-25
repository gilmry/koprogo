# GitOps Deployment pour KoproGo

Guide pour déployer KoproGo avec un système GitOps simple et efficace sur VPS.

## 🎯 Architecture

```
┌─────────────────┐
│  GitHub Repo    │  ← Source of Truth
│  (main branch)  │
└────────┬────────┘
         │
         │ polls every 3min
         ▼
┌─────────────────────────────┐
│  GitOps Script (systemd)    │
│  - Detects changes          │
│  - Pulls new images         │
│  - Auto-deploys             │
└────────┬────────────────────┘
         │
         │ updates
         ▼
┌─────────────────────────────┐
│  Docker Compose             │
│  - Backend (ghcr.io)        │
│  - Frontend (ghcr.io)       │
│  - PostgreSQL               │
│  - Traefik                  │
└─────────────────────────────┘
```

## 🚀 Installation

### Étape 1 : Arrêter Argo CD (si installé)

```bash
cd ~/koprogo/argocd
docker compose -f docker-compose.argocd.yml down -v
```

### Étape 2 : Configurer l'environnement

```bash
cd ~/koprogo/deploy/production

# Copier et éditer .env
cp .env.example .env
nano .env
```

**Variables importantes à configurer** :

```bash
# GitHub
GITHUB_REPOSITORY=gillesmaury/koprogo
IMAGE_TAG=latest

# Domains
API_DOMAIN=api2.koprogo.com
FRONTEND_DOMAIN=app.koprogo.com

# Secrets (IMPORTANT!)
POSTGRES_PASSWORD=STRONG_PASSWORD_HERE
JWT_SECRET=SUPER_LONG_RANDOM_STRING_32_CHARS_MIN

# Traefik
ACME_EMAIL=ton-email@example.com
TRAEFIK_DOMAIN=traefik.koprogo.com
```

### Étape 3 : Rendre le script exécutable

```bash
chmod +x gitops-deploy.sh
```

### Étape 4 : Test manuel

```bash
# Déployer manuellement pour tester
./gitops-deploy.sh deploy
```

**Output attendu** :
```
[2025-10-25 10:00:00] 🚀 Starting deployment...
[2025-10-25 10:00:01] INFO: Pulling Docker images...
[2025-10-25 10:00:15] INFO: Deploying services...
[2025-10-25 10:00:25] ✅ Deployment complete!
```

### Étape 5 : Installer le service systemd

```bash
# Copier le fichier service
sudo cp koprogo-gitops.service /etc/systemd/system/

# Recharger systemd
sudo systemctl daemon-reload

# Activer le service au démarrage
sudo systemctl enable koprogo-gitops

# Démarrer le service
sudo systemctl start koprogo-gitops
```

### Étape 6 : Vérifier que ça tourne

```bash
# Voir le statut
sudo systemctl status koprogo-gitops

# Voir les logs en temps réel
sudo journalctl -u koprogo-gitops -f
```

**Output attendu** :
```
● koprogo-gitops.service - KoproGo GitOps Deployment Service
   Loaded: loaded (/etc/systemd/system/koprogo-gitops.service; enabled)
   Active: active (running) since...

Oct 25 10:00:00 koprogo-gitops[1234]: [2025-10-25 10:00:00] 👀 Starting GitOps watch mode...
Oct 25 10:00:00 koprogo-gitops[1234]: [2025-10-25 10:00:00] Branch: main
Oct 25 10:00:00 koprogo-gitops[1234]: [2025-10-25 10:00:00] Repository: /home/debian/koprogo
Oct 25 10:03:00 koprogo-gitops[1234]: [2025-10-25 10:03:00] INFO: Checking for updates...
Oct 25 10:03:01 koprogo-gitops[1234]: [2025-10-25 10:03:01] INFO: No changes detected
Oct 25 10:03:01 koprogo-gitops[1234]: [2025-10-25 10:03:01] INFO: Next check in 180s...
```

## 🔄 Workflow GitOps

### Déploiement automatique

1. **Push code** vers `main`
2. **GitHub Actions** build les images → push vers `ghcr.io` avec tag `main-sha-<commit>`
3. **GitOps script** détecte le changement (< 3min)
4. **Calcul du tag** : Détermine automatiquement le tag d'image basé sur le commit SHA
5. **Attente intelligente** : Réessaie toutes les 90s pendant 15 minutes pour attendre que GitHub Actions finisse le build
6. **Auto-deploy** : Dès que l'image est disponible, tire et déploie
7. **Redémarrage** automatique des containers avec la version exacte du commit

**Avantages** :
- ✅ Déploie toujours l'image correspondant au commit exact (traçabilité)
- ✅ Attend automatiquement que le build soit terminé (pas de déploiement prématuré)
- ✅ Fallback sur `latest` si le build échoue après 15 minutes

### Déploiement standard (automatique)

Le déploiement est **entièrement automatique** :

```bash
# 1. Développer et commit
git add .
git commit -m "feat: nouvelle fonctionnalité"
git push

# 2. C'est tout ! Le reste est automatique :
# - GitHub Actions build l'image (5-10 min)
# - GitOps détecte le commit (< 3 min)
# - GitOps attend que l'image soit prête (jusqu'à 15 min)
# - GitOps déploie automatiquement
```

### Déploiement manuel (si nécessaire)

Si tu veux déployer immédiatement sans attendre le cycle automatique :

```bash
cd ~/koprogo/deploy/production

# Déployer la version actuelle
./gitops-deploy.sh deploy

# Ou déployer une version spécifique
export IMAGE_TAG=v1.2.3
./gitops-deploy.sh deploy
```

## 📊 Commandes disponibles

### Voir le statut

```bash
cd ~/koprogo/deploy/production
./gitops-deploy.sh status
```

**Output** :
```
=========================================
GitOps Deployment Status
=========================================
Current branch: main
Current commit: a2ba365
Latest commit message: feat(load-tests): Implement configurable rate limiting

Docker Compose Services:
NAME                  IMAGE                                    STATUS
koprogo-backend       ghcr.io/gillesmaury/koprogo/backend:latest   Up 2 hours
koprogo-frontend      ghcr.io/gillesmaury/koprogo/frontend:latest  Up 2 hours
koprogo-postgres      postgres:15-alpine                            Up 2 hours (healthy)
koprogo-traefik       traefik:v3.5.3                                Up 2 hours
```

### Voir les logs

```bash
# Logs du script GitOps
./gitops-deploy.sh logs

# Logs du service systemd
sudo journalctl -u koprogo-gitops -f

# Logs des containers
docker compose -f docker-compose.yml logs -f
```

### Déploiement manuel

```bash
./gitops-deploy.sh deploy
```

### Rollback

Le rollback permet de revenir à une version précédente en un seul clic :

```bash
./gitops-deploy.sh rollback
```

**Ce qui se passe** :
1. Affiche les 10 derniers commits avec leurs SHA
2. Demande le commit cible (ou utilise le précédent par défaut)
3. Demande confirmation
4. Checkout le commit cible
5. Déploie automatiquement l'image `main-sha-<commit>` correspondante
6. Donne des instructions pour rendre le rollback permanent

**Exemple d'utilisation** :

```bash
$ ./gitops-deploy.sh rollback

Recent deployments (last 10 commits):
========================================
abc1234 (HEAD -> main) fix: correction bug critique
def5678 feat: nouvelle fonctionnalité
ghi9012 chore: mise à jour dépendances

Enter commit SHA to rollback to (or press Enter for previous commit): def5678
Rolling back from abc1234 to def5678
Are you sure? (y/N) y

✅ Rollback complete to commit def5678
```

**Rendre le rollback permanent** :

```bash
# Option 1: Créer une branche de rollback
git checkout -b rollback-to-def5678
git push origin rollback-to-def5678

# Option 2: Forcer main à revenir en arrière (ATTENTION: destructif!)
git checkout main
git reset --hard def5678
git push --force
```

## 🛠️ Dépannage

### Le service ne démarre pas

```bash
# Vérifier les logs
sudo journalctl -u koprogo-gitops -n 50

# Vérifier la config
sudo systemctl status koprogo-gitops

# Redémarrer
sudo systemctl restart koprogo-gitops
```

### Les changements ne sont pas détectés

```bash
# Vérifier que Git peut pull
cd ~/koprogo
git pull origin main

# Vérifier les permissions
ls -la deploy/production/gitops-deploy.sh

# Forcer un déploiement manuel
cd deploy/production
./gitops-deploy.sh deploy
```

### Les containers ne démarrent pas

```bash
# Vérifier le .env
cat deploy/production/.env

# Vérifier les logs Docker
docker compose -f deploy/production/docker-compose.yml logs

# Vérifier les images
docker images | grep ghcr.io
```

### Erreur "images not found"

Vérifier que les images sont publiques sur GitHub :

1. Aller sur https://github.com/gillesmaury/koprogo/packages
2. Cliquer sur chaque package (backend, frontend)
3. Package settings → Change visibility → Public

## 🔐 Sécurité

### Permissions du script

```bash
# Le script doit être exécutable mais pas world-writable
chmod 755 deploy/production/gitops-deploy.sh
```

### Protection du .env

```bash
# Le .env ne doit JAMAIS être commité
# Vérifier qu'il est dans .gitignore
grep -q "\.env$" .gitignore && echo "OK" || echo "AJOUTER .env AU .gitignore!"

# Permissions restrictives
chmod 600 deploy/production/.env
```

### Logs

```bash
# Créer le fichier de log avec les bonnes permissions
sudo touch /var/log/koprogo-gitops.log
sudo chown debian:debian /var/log/koprogo-gitops.log
sudo chmod 644 /var/log/koprogo-gitops.log
```

## 📈 Monitoring

### Health Checks

Le script vérifie automatiquement l'état des services après chaque déploiement :

```bash
docker compose ps
```

### Logs centralisés

Tous les déploiements sont loggés dans `/var/log/koprogo-gitops.log` :

```bash
tail -f /var/log/koprogo-gitops.log
```

### Alertes (optionnel)

Modifier le script `gitops-deploy.sh` pour ajouter des notifications :

```bash
# Dans la fonction deploy(), ajouter :
if [ $? -eq 0 ]; then
    curl -X POST "https://hooks.slack.com/YOUR_WEBHOOK" \
         -d '{"text": "✅ KoproGo deployed successfully"}'
else
    curl -X POST "https://hooks.slack.com/YOUR_WEBHOOK" \
         -d '{"text": "❌ KoproGo deployment failed!"}'
fi
```

## 🎓 Comparaison avec Argo CD

| Feature                  | Argo CD       | Ce script GitOps |
|--------------------------|---------------|------------------|
| **Plateforme**           | Kubernetes    | Docker Compose   |
| **Interface Web**        | ✅ Oui        | ❌ Non (logs CLI) |
| **Auto-sync**            | ✅ Oui (3min) | ✅ Oui (3min)    |
| **Rollback**             | ✅ Oui        | ✅ Oui           |
| **Resource usage**       | ~200MB RAM    | ~5MB RAM         |
| **Setup complexity**     | Élevé         | Faible           |
| **VPS compatibility**    | ❌ Non        | ✅ Oui           |

## ✅ Avantages de cette approche

1. **Simple** : Un seul script bash, pas de stack complexe
2. **Léger** : ~5MB de RAM vs ~200MB pour Argo CD
3. **Compatible VPS** : Fonctionne directement avec Docker Compose
4. **GitOps** : Même principe qu'Argo CD (Git = source of truth)
5. **Auto-sync** : Détection automatique des changements toutes les 3 minutes
6. **Rollback** : Retour en arrière facile vers commit précédent
7. **Logs** : Historique complet des déploiements

## 🚀 Prochaines étapes recommandées

- ✅ Configurer des notifications Slack/Discord
- ✅ Ajouter un environnement staging
- ✅ Mettre en place des health checks personnalisés
- ✅ Intégrer avec Prometheus/Grafana
- ✅ Ajouter des tests pré-déploiement

---

**Support** : Si problème, consulter les logs avec `./gitops-deploy.sh logs`
