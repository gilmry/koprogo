# Argo CD GitOps pour KoproGo

Guide complet pour déployer KoproGo avec Argo CD en mode GitOps.

## 🎯 Architecture

```
┌─────────────────┐
│  GitHub Repo    │  ← Source of Truth
│  (main branch)  │
└────────┬────────┘
         │
         │ watches every 3min
         ▼
┌─────────────────────────────┐
│  Argo CD (on VPS)           │
│  - Detects changes          │
│  - Pulls new images         │
│  - Auto-syncs               │
└────────┬────────────────────┘
         │
         │ deploys to
         ▼
┌─────────────────────────────┐
│  Docker Compose             │
│  - Backend (ghcr.io)        │
│  - Frontend (ghcr.io)       │
│  - PostgreSQL               │
└─────────────────────────────┘
```

## 📋 Prérequis

- VPS avec Docker & Docker Compose installés
- Git repository sur GitHub
- Images Docker publiées sur `ghcr.io` (via GitHub Actions)

## 🚀 Installation rapide

### Étape 1 : Sur votre VPS

```bash
# Cloner le repo
git clone https://github.com/YOUR_USERNAME/koprogo.git
cd koprogo/argocd

# Installer Argo CD
./argocd-helper.sh install
```

**Output attendu** :
```
========================================
Installing Argo CD
========================================
✅ Argo CD is starting...
Waiting for Argo CD to be ready...

========================================
Argo CD Initial Password
========================================
Username: admin
Password: xKj9mP2nQ8rT...

Access Argo CD UI at: http://localhost:8080
✅ Installation complete!
```

### Étape 2 : Configurer l'environnement

```bash
cd ../deploy/production

# Copier et éditer .env
cp .env.example .env
nano .env
```

Remplir avec vos valeurs :
```bash
GITHUB_REPOSITORY=your-username/koprogo
IMAGE_TAG=latest
POSTGRES_PASSWORD=STRONG_PASSWORD_HERE
JWT_SECRET=SUPER_LONG_RANDOM_STRING_32_CHARS_MIN
PUBLIC_API_URL=https://api2.koprogo.com
```

### Étape 3 : Commit les changements

```bash
git add .env
git commit -m "chore: configure production environment"
git push origin main
```

### Étape 4 : Créer l'application Argo CD

```bash
cd ../../argocd

# Éditer application.yaml avec votre repo
nano application.yaml
# Remplacer YOUR_USERNAME par votre username GitHub

# Créer l'application
./argocd-helper.sh create-app
```

### Étape 5 : Accéder à l'UI

```bash
# Depuis un autre terminal (votre machine locale)
ssh -L 8080:localhost:8080 user@your-vps-ip

# Puis ouvrez dans votre navigateur
http://localhost:8080
```

**Login** :
- Username: `admin`
- Password: (celui affiché lors de l'installation)

## 🔄 Workflow GitOps

### Déploiement automatique

1. **Push code** vers `main`
2. **GitHub Actions** build les images → push vers `ghcr.io`
3. **Vous** mettez à jour `IMAGE_TAG` dans `deploy/production/.env`
4. **Commit + Push** le changement
5. **Argo CD** détecte le changement (< 3min)
6. **Auto-sync** tire les nouvelles images
7. **Redémarrage** automatique des containers

### Exemple : Déployer v1.2.3

```bash
# Option 1 : Via helper script
cd argocd
./argocd-helper.sh update-tag
# Entrer: v1.2.3

# Option 2 : Manuellement
cd deploy/production
sed -i 's/^IMAGE_TAG=.*/IMAGE_TAG=v1.2.3/' .env
git add .env
git commit -m "deploy: update to v1.2.3"
git push

# Argo CD sync automatiquement dans ~3 minutes
# Ou forcer le sync :
cd ../../argocd
./argocd-helper.sh sync
```

## 📊 Opérations courantes

### Vérifier le statut

```bash
./argocd-helper.sh status
```

Output :
```
Name:               koprogo-production
Project:            default
Server:             https://kubernetes.default.svc
Source:
- Repo:            https://github.com/user/koprogo.git
  Target:          main
  Path:            deploy/production
Health Status:      Healthy
Sync Status:        Synced
```

### Voir les logs en temps réel

```bash
./argocd-helper.sh logs
```

### Forcer un sync

```bash
./argocd-helper.sh sync
```

### Rollback vers une version précédente

```bash
./argocd-helper.sh rollback
```

Choisir le numéro de révision dans la liste affichée.

### Accéder à l'UI

```bash
./argocd-helper.sh ui
```

## 🎨 Interface Argo CD

### Fonctionnalités clés

1. **App View** : Vue graphique de tous les composants
2. **Sync Status** : État de synchronisation Git ↔ Cluster
3. **Health Status** : Santé de chaque service
4. **Diff** : Voir les changements avant de synchroniser
5. **History** : Historique des déploiements
6. **Rollback** : Retour en arrière en 1 clic

### Captures d'écran typiques

```
┌─────────────────────────────────────┐
│  koprogo-production                 │
│  ✅ Synced  |  ❤️ Healthy           │
├─────────────────────────────────────┤
│  ├─ backend                         │
│  │  └─ Image: ghcr.io/.../v1.2.3   │
│  │     ✅ Running  |  ❤️ Healthy    │
│  ├─ frontend                        │
│  │  └─ Image: ghcr.io/.../v1.2.3   │
│  │     ✅ Running  |  ❤️ Healthy    │
│  └─ postgres                        │
│     ✅ Running  |  ❤️ Healthy       │
└─────────────────────────────────────┘
```

## 🔐 Sécurité

### Changer le mot de passe admin

```bash
# Dans l'UI Argo CD
User Info > Update Password

# Ou via CLI
argocd account update-password --insecure
```

### Rendre Argo CD accessible en HTTPS

```bash
# Via reverse proxy (Nginx/Caddy)
server {
    listen 443 ssl;
    server_name argocd.koprogo.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## 🛠️ Dépannage

### Argo CD ne détecte pas les changements

```bash
# Forcer un refresh
argocd app get koprogo-production --refresh --insecure
```

### L'application est "OutOfSync"

```bash
# Voir la différence
argocd app diff koprogo-production --insecure

# Forcer la synchronisation
argocd app sync koprogo-production --insecure
```

### Les containers ne démarrent pas

```bash
# Vérifier les logs via Argo CD
./argocd-helper.sh logs

# Ou directement Docker
docker compose -f ../deploy/production/docker-compose.yml logs -f
```

### Erreur "Unable to create application"

Vérifier que :
1. Le repo GitHub est accessible (public ou token configuré)
2. Le chemin `deploy/production` existe dans le repo
3. L'URL du repo est correcte dans `application.yaml`

## 📚 Commandes utiles

```bash
# Lister les applications
argocd app list --insecure

# Voir l'historique des syncs
argocd app history koprogo-production --insecure

# Supprimer l'application (ne supprime pas les ressources)
argocd app delete koprogo-production --insecure

# Pause auto-sync
argocd app set koprogo-production --sync-policy none --insecure

# Reprendre auto-sync
argocd app set koprogo-production --sync-policy automated --insecure
```

## 🔄 Migration depuis docker-compose manuel

Si vous déployez actuellement avec `docker compose up` :

```bash
# 1. Arrêter l'ancien déploiement
docker compose -f docker-compose.vps.yml down

# 2. Sauvegarder les données PostgreSQL (si nécessaire)
docker run --rm -v koprogo_postgres_data:/data -v $(pwd):/backup alpine tar czf /backup/postgres-backup.tar.gz /data

# 3. Installer Argo CD (voir ci-dessus)

# 4. Restaurer les données (si nécessaire)
docker run --rm -v koprogo-production_postgres-data:/data -v $(pwd):/backup alpine tar xzf /backup/postgres-backup.tar.gz -C /

# 5. Créer l'application Argo CD

# 6. Argo CD prend le relais !
```

## 🎓 Ressources

- [Documentation Argo CD](https://argo-cd.readthedocs.io/)
- [Best Practices GitOps](https://www.gitops.tech/)
- [Argo CD Autopilot](https://argocd-autopilot.readthedocs.io/)

## 🆘 Support

Si vous rencontrez des problèmes :

1. Vérifier les logs : `./argocd-helper.sh logs`
2. Vérifier le statut : `./argocd-helper.sh status`
3. Consulter l'UI Argo CD
4. Vérifier les logs Docker : `docker compose logs`

---

**Prochaines étapes recommandées** :
- ✅ Configurer des notifications (Slack, Discord)
- ✅ Ajouter un environnement staging
- ✅ Configurer le RBAC Argo CD
- ✅ Mettre en place des health checks personnalisés
- ✅ Intégrer avec Prometheus/Grafana
