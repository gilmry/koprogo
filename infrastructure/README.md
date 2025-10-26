# KoproGo - Déploiement VPS Automatisé (Terraform + Ansible)

Déploiement automatique de KoproGo sur OVH Public Cloud avec Terraform + Ansible + GitOps.

**Pour qui ?** Geeks qui veulent déployer KoproGo en production avec une commande.

---

## 🎯 Ce que fait ce déploiement

1. **Terraform** : Provisionne un VPS OVH (2 vCPU, 4GB RAM, ~14€/mois)
2. **Ansible** : Configure le serveur (Docker, Git, Firewall, Fail2ban)
3. **Traefik** : Reverse proxy avec SSL automatique (Let's Encrypt)
4. **Docker Compose** : Déploie Backend + Frontend + PostgreSQL
5. **DNS** : Configuration automatique via API OVH (optionnel)
6. **GitOps** : Auto-update quotidien depuis GitHub (3h du matin)
7. **Backups** : Backups PostgreSQL quotidiens (2h du matin)
8. **Monitoring** : Health checks toutes les 5 minutes

**Résultat** : KoproGo tourne sur votre VPS avec HTTPS et se met à jour automatiquement.

---

## 📋 Prérequis

### Outils installés sur votre machine

```bash
# Terraform 1.0+
terraform --version

# Ansible 2.9+
ansible --version

# Clé SSH générée
ls ~/.ssh/id_rsa.pub

# Si pas de clé SSH
ssh-keygen -t rsa -b 4096 -C "your_email@example.com"
```

### Compte OVH Cloud

1. **Créer un compte** : https://www.ovh.com/manager/public-cloud/
2. **Créer un projet Public Cloud**
3. **Créer un utilisateur OpenStack** (requis pour Terraform)
4. **Obtenir credentials OVH API** (optionnel, pour DNS automatique)

---

## 🚀 Déploiement Ultra-Rapide (1 commande)

### Déploiement complet automatisé

Depuis la racine du projet :

```bash
make setup-infra
```

Le script interactif vous guide à travers toutes les étapes :
1. **Détection automatique** : Si configuration existe déjà, propose de la réutiliser
2. Création des credentials OVH API (optionnel, pour DNS)
3. Création de l'utilisateur OpenStack avec les bons rôles
4. Téléchargement du fichier OpenRC (région GRA9)
5. Configuration du domaine (optionnel)
6. Déploiement Terraform (provisionne le VPS)
7. Configuration DNS automatique (si domaine configuré)
8. Déploiement Ansible (configure et déploie l'application)

**Durée totale** :
- Premier déploiement : ~20-30 minutes (dont 15-20 min d'attente automatique)
- Redéploiements suivants : ~15-20 minutes (config existante réutilisée)

### Réutilisation de configuration existante

Si vous avez déjà déployé une fois, le script détecte automatiquement votre fichier `.env` et propose de le réutiliser :

```bash
⚠️  Configuration existante détectée: infrastructure/terraform/.env

Voulez-vous:
  1) Utiliser la configuration existante (recommandé)
  2) Reconfigurer depuis le début

Votre choix (1/2): 1
```

Cela vous évite de re-saisir tous vos credentials OVH et OpenStack !

---

## 📖 Guide Détaillé Pas-à-Pas

Si vous préférez suivre le processus étape par étape :

### Étape 1 : Créer un utilisateur OpenStack (REQUIS)

1. **OVH Manager** → **Public Cloud** → **Projet Management** → **Users & Roles**
2. Cliquer sur **Créer un utilisateur OpenStack**
3. **Choisir TOUS les rôles suivants** (IMPORTANT !) :
   - ☑ **Administrator** (CRITIQUE pour Terraform)
   - ☑ Compute Operator
   - ☑ Network Operator
   - ☑ Network Security Operator
   - ☑ Image Operator
   - ☑ Volume Operator
   - ☑ ObjectStore Operator
   - ☑ LoadBalancer Operator
   - ☑ Backup Operator
   - ☑ Infrastructure Supervisor
   - ☑ KeyManager Operator
   - ☑ KeyManager Read

4. Créer l'utilisateur et **noter** :
   - `OS_USERNAME` (format: `user-XXXXXXXXXXXX`)
   - `OS_PASSWORD` (généré automatiquement, à copier immédiatement)

### Étape 2 : Télécharger le fichier OpenRC (REQUIS)

1. **OVH Manager** → **Public Cloud** → **Users & Roles**
2. Cliquer sur **...** à côté de votre utilisateur
3. Sélectionner **Download OpenStack's RC file**
4. **Ouvrir le fichier** et trouver la ligne :
   ```bash
   export OS_REGION_NAME="GRA9"
   ```
5. **Noter la région** (exemple: GRA9, GRA11, SBG5, etc.)

> **IMPORTANT** : Utilisez toujours la région exacte du fichier OpenRC !

### Étape 3 : Créer credentials OVH API (OPTIONNEL, pour DNS automatique)

**Seulement si vous voulez configurer automatiquement le DNS**

1. Aller sur : https://www.ovh.com/auth/api/createToken
2. **Application name** : `KoproGo Infrastructure`
3. **Application description** : `Terraform + Ansible deployment`
4. **Validity** : `Unlimited`
5. **Rights** :
   - `GET /domain/*`
   - `POST /domain/*`
   - `PUT /domain/*`
   - `DELETE /domain/*`
6. Cliquer sur **Create keys**
7. **Noter** :
   - `OVH_APPLICATION_KEY`
   - `OVH_APPLICATION_SECRET`
   - `OVH_CONSUMER_KEY`

### Étape 4 : Lancer le déploiement

```bash
# Depuis la racine du projet
make setup-infra
```

Le script vous demandera :
- Credentials OVH API (si DNS automatique souhaité)
- ID du projet OVH Cloud
- Username et password OpenStack
- Région OpenRC (ex: GRA9)
- Domaine (optionnel)
- Email pour SSL (si domaine configuré)

### Étape 5 : Vérifier le déploiement

Après le déploiement :

```bash
# Si vous avez configuré un domaine
curl https://votre-domaine.com/api/v1/health

# Sinon, utiliser l'IP du VPS
curl http://51.210.XXX.XXX:8080/api/v1/health

# Devrait retourner : {"status":"healthy","timestamp":"..."}
```

Se connecter au VPS :

```bash
# Récupérer l'IP
cd infrastructure/terraform
terraform output vps_ip

# SSH
ssh ubuntu@51.210.XXX.XXX

# Sur le VPS
sudo su - koprogo
cd ~/koprogo/deploy/production
docker compose ps
docker compose logs -f
```

---

## 🏗️ Architecture de Déploiement

Le déploiement utilise une stack complète avec reverse proxy et SSL automatique :

```
Internet (HTTPS)
      ↓
Traefik (Reverse Proxy + SSL Let's Encrypt)
      ↓
   ┌──────────────────────────────────────┐
   │         Docker Compose               │
   │                                      │
   │  ┌──────────┐  ┌──────────┐        │
   │  │ Frontend │  │ Backend  │        │
   │  │  (Astro  │  │  (Rust   │        │
   │  │  Svelte) │  │  Actix)  │        │
   │  └─────┬────┘  └────┬─────┘        │
   │        │            │               │
   │        └────────────┼──────────┐    │
   │                     │          │    │
   │              ┌──────▼──────┐   │    │
   │              │  PostgreSQL │   │    │
   │              │     15      │   │    │
   │              └─────────────┘   │    │
   └──────────────────────────────────────┘
```

**Composants** :

1. **Traefik** (Port 80/443)
   - Reverse proxy automatique
   - Gestion SSL Let's Encrypt
   - Redirection HTTP → HTTPS
   - Headers de sécurité

2. **Backend Rust** (Port interne 8080)
   - API REST (Actix-web)
   - Connexion PostgreSQL via pool
   - CORS configuré pour frontend

3. **Frontend Astro/Svelte** (Port interne 3000)
   - SSG (Static Site Generation)
   - Islands Architecture
   - Appels API vers backend

4. **PostgreSQL 15** (Port interne 5432)
   - Base de données persistante
   - Volume Docker monté
   - Backups quotidiens automatiques

**Fichiers déployés depuis** : `github.com/gilmry/koprogo/deploy/production`

**Variables d'environnement** : Générées automatiquement par Ansible via `env-production.j2`

---

## 🔄 GitOps Auto-Update

KoproGo se met à jour automatiquement depuis GitHub grâce à un service systemd qui vérifie les nouveaux commits **toutes les 3 minutes**.

### Comment ça marche ?

1. **Service systemd** : `koprogo-gitops.service` tourne en continu
2. **Vérification** : Check `git fetch` toutes les 3 minutes
3. **Détection** : Si nouveau commit détecté sur `main`
4. **Pull GitHub** : `git pull origin main`
5. **Pull Images** : Pull des nouvelles images Docker depuis GitHub Container Registry
6. **Rebuild** : `docker compose up -d --pull always`
7. **Health check** : Vérifie l'API sur l'URL publique HTTPS
8. **Permissions Git** : Correction automatique des permissions pour éviter les erreurs

### Logs GitOps

```bash
# Sur le VPS
tail -f /var/log/koprogo-gitops.log

# Status du service
sudo systemctl status koprogo-gitops
```

### Désactiver auto-update

```bash
# Arrêter le service
sudo systemctl stop koprogo-gitops
sudo systemctl disable koprogo-gitops
```

### Forcer une mise à jour manuelle

```bash
# Sur le VPS
cd /home/koprogo/koprogo
sudo -u koprogo ./deploy/production/gitops-deploy.sh deploy
```

---

## 💾 Backups

Backups PostgreSQL **quotidiens à 2h du matin**.

### Localisation backups

```bash
# Sur le VPS
ls -lh ~/koprogo/backups/
# koprogo_20250125_020000.sql.gz
# koprogo_20250126_020000.sql.gz
```

### Restaurer backup

```bash
cd ~/koprogo

# Restaurer le dernier backup
gunzip -c backups/koprogo_YYYYMMDD_HHMMSS.sql.gz | \
  docker compose exec -T postgres psql -U koprogo -d koprogo_db
```

### Rétention backups

Par défaut : **7 jours** (configurable dans `ansible/templates/backup.sh.j2`)

---

## 🔒 Sécurité

### Firewall UFW

- ✅ Port 22 (SSH)
- ✅ Port 80 (HTTP)
- ✅ Port 443 (HTTPS)
- ❌ Tout le reste bloqué

### Fail2ban

Protection contre brute-force SSH (installé automatiquement).

### SSL/TLS (HTTPS)

Si vous avez configuré un domaine, Traefik génère automatiquement certificat Let's Encrypt.

```bash
# Vérifier HTTPS
curl https://votre-domaine.com/api/v1/health
```

---

## 📊 Monitoring

### Health checks

Toutes les **5 minutes** : `curl http://localhost:8080/api/v1/health`

```bash
# Voir logs health checks
tail -f /var/log/koprogo-health.log
```

### Métriques système

```bash
# Sur le VPS
docker stats
htop
df -h
```

---

## 🛠️ Maintenance

### Restart services

```bash
cd ~/koprogo
docker compose restart
```

### Update manuel (sans attendre cron)

```bash
cd ~/koprogo
./scripts/auto-update.sh
```

### Voir logs

```bash
# Tous les services
docker compose logs -f

# Backend uniquement
docker compose logs -f backend

# PostgreSQL uniquement
docker compose logs -f postgres
```

### Cleanup Docker

```bash
# Supprimer images inutilisées
docker system prune -a
```

---

## 💰 Coûts

| Composant | Prix |
|-----------|------|
| VPS OVH d2-2 (2 vCPU, 4GB RAM, 25GB SSD) | **14€ TTC/mois** |
| Domaine (optionnel) | ~12€/an (~1€/mois) |
| SSL Let's Encrypt | **0€** |
| Bande passante | **0€** (250 Mbit/s inclus) |
| **TOTAL** | **~14-15€/mois** |

**Capacité estimée** :
- 2,000-3,000 copropriétés
- ~10,000-15,000 utilisateurs actifs
- P99 latency < 5ms (testé en charge)

**Pourquoi d2-2 ?**
- Production-ready (haute disponibilité)
- Performance adaptée au backend Rust + PostgreSQL
- Marge pour pics de charge

---

## 🌍 Écologie

**OVH Cloud France (Gravelines)** :
- Mix énergétique : **60g CO₂/kWh** (nucléaire 70% + renouvelables 25%)
- **7-25x moins de CO₂** qu'AWS/Azure (400-500g CO₂/kWh)
- Empreinte carbone : **0.12g CO₂/requête**

---

## 🧹 Désinstallation

```bash
# 1. Détruire VPS Terraform
cd terraform
terraform destroy

# 2. (Optionnel) Cleanup credentials
unset OVH_APPLICATION_KEY OVH_APPLICATION_SECRET OVH_CONSUMER_KEY
```

---

## 🆘 Troubleshooting

### Terraform : "No suitable endpoint could be found"

**Symptôme** :
```
Error: No suitable endpoint could be found in the service catalog
```

**Cause** : Région incorrecte ou non compatible avec votre fichier OpenRC

**Fix** :
1. **TOUJOURS** télécharger le fichier OpenRC depuis OVH Manager
2. Ouvrir le fichier et trouver : `export OS_REGION_NAME="GRA9"`
3. Utiliser EXACTEMENT cette région (GRA9, GRA11, SBG5, etc.)
4. Ne PAS deviner ou utiliser des régions aléatoires

```bash
# Vérifier le fichier OpenRC
grep OS_REGION_NAME openrc.sh
# export OS_REGION_NAME="GRA9"

# Utiliser cette région exacte dans setup-infra.sh
```

### Terraform : "Insufficient permissions"

**Symptôme** :
```
Error creating openstack_compute_instance_v2: Forbidden
```

**Cause** : Utilisateur OpenStack sans le rôle **Administrator**

**Fix** :
1. OVH Manager → Public Cloud → Users & Roles
2. Supprimer l'utilisateur actuel
3. Créer un nouvel utilisateur avec **TOUS** les rôles listés ci-dessus
4. **Surtout** : Cocher **Administrator** (CRITIQUE !)

### Terraform : "One of 'auth_url' or 'cloud' must be specified"

**Symptôme** :
```
Error: One of 'auth_url' or 'cloud' must be specified

  with provider["registry.terraform.io/terraform-provider-openstack/openstack"].ovh
```

**Cause** : Variables d'environnement OpenStack non chargées dans le shell

**Fix** : Utiliser `source` pour charger les variables (PAS `./`)
```bash
cd infrastructure/terraform

# ✅ CORRECT - Les variables sont exportées dans votre shell
source ./load-env.sh
terraform plan

# ✅ CORRECT - Raccourci avec "."
. ./load-env.sh
terraform apply

# ❌ FAUX - Crée un sous-shell, les variables sont perdues
./load-env.sh
terraform plan  # ❌ Erreur: variables non disponibles
```

Le script détecte maintenant si vous l'exécutez incorrectement :
```bash
$ ./load-env.sh
❌ Erreur: Ce script doit être sourcé, pas exécuté directement!

Utilisation correcte:
  source ./load-env.sh
  # ou
  . ./load-env.sh
```

### Ansible : "SSH connection failed"

**Cause** : VPS pas encore prêt ou clé SSH incorrecte

**Fix** :
```bash
# Attendre 1-2 minutes après terraform apply
sleep 120

# Tester SSH manuel
ssh -o StrictHostKeyChecking=no ubuntu@51.210.XXX.XXX

# Vérifier clé SSH
ls -la ~/.ssh/id_rsa.pub
```

### Ansible : "Failed to set permissions" (become_user error)

**Symptôme** :
```
Failed to set permissions on the temporary files Ansible needs to create
chmod: invalid mode: 'A+user:koprogo:rx:allow'
```

**Cause** : Problème d'ACL avec Ansible 2.16+ sur Ubuntu

**Fix** : Ce problème est déjà corrigé dans le playbook avec `become_method: su`

### DNS : Propagation lente

**Symptôme** : Le domaine ne pointe pas vers le VPS immédiatement

**Cause** : Propagation DNS normale (1-60 minutes)

**Fix** :
```bash
# Vérifier la configuration DNS (peut montrer ancienne IP)
nslookup votre-domaine.com

# Forcer requête vers les DNS OVH
nslookup votre-domaine.com dns200.anycast.me

# Attendre 5-10 minutes et retester
```

### Ansible : Health check échoue pendant le déploiement

**Symptôme** :
```
TASK [Check API health (public HTTPS endpoint)] ****************************
FAILED - RETRYING: [koprogo-vps]: Check API health (10 retries left).
fatal: [koprogo-vps]: FAILED! => {"status": -1, "msg": "SSL certificate problem"}
```

**Cause** :
- Certificat Let's Encrypt pas encore généré (DNS pas propagé)
- Services Docker pas encore prêts
- Configuration Traefik incorrecte

**Fix** :
Le playbook Ansible utilise maintenant l'URL publique HTTPS avec jusqu'à 10 retries (100 secondes).
Si ça échoue quand même :

```bash
# 1. Vérifier que le DNS pointe vers le VPS
dig your-domain.com

# 2. Se connecter au VPS et vérifier les services
ssh ubuntu@VPS_IP
sudo docker ps -a

# 3. Vérifier les logs Traefik pour le certificat SSL
sudo docker logs koprogo-traefik | grep -i "cert"

# 4. Tester le health check manuellement
curl -k https://api.your-domain.com/api/v1/health

# 5. Si tout fonctionne manuellement, le déploiement Ansible a réussi
# Le health check peut juste avoir échoué pour timeout
```

### Health check échoue après déploiement

**Cause** : Services Docker pas encore démarrés ou erreur de déploiement

**Fix** :
```bash
# Se connecter au VPS
ssh ubuntu@VPS_IP

# Vérifier les services
sudo su - koprogo
cd ~/koprogo/deploy/production
docker compose ps

# Vérifier les logs
docker compose logs backend
docker compose logs frontend
docker compose logs postgres
docker compose logs traefik

# Redémarrer si nécessaire
docker compose restart

# Si problème de build, forcer le rebuild
docker compose down
docker compose up -d --force-recreate
```

### Traefik : Certificat SSL pas généré

**Symptôme** : HTTPS ne fonctionne pas, erreur de certificat

**Cause** : DNS pas encore propagé ou domaine incorrect

**Fix** :
```bash
# Vérifier que le DNS pointe vers le VPS
nslookup votre-domaine.com

# Vérifier les logs Traefik
docker compose logs traefik

# Vérifier le fichier acme.json
ls -la /home/koprogo/koprogo/deploy/production/letsencrypt/acme.json

# Si vide, attendre propagation DNS puis redémarrer Traefik
docker compose restart traefik
```

---

## 📚 Ressources

### Documentation KoproGo

- **Lessons Learned** : [LESSONS-LEARNED.md](./LESSONS-LEARNED.md) - Tous les problèmes rencontrés et solutions
- **Guide Terraform** : [terraform/README.md](./terraform/README.md)
- **Guide Ansible** : [ansible/README.md](./ansible/README.md)
- **GitOps manuel** : [docs/DEPLOY_GITOPS.md](../../docs/DEPLOY_GITOPS.md)
- **Business plan** : [docs/BUSINESS_PLAN_BOOTSTRAP.md](../../docs/BUSINESS_PLAN_BOOTSTRAP.md)

### Documentation externe

- **Terraform OpenStack Provider** : https://registry.terraform.io/providers/terraform-provider-openstack/openstack/latest/docs
- **OVH Public Cloud** : https://help.ovhcloud.com/csm/en-public-cloud-compute-getting-started
- **Ansible** : https://docs.ansible.com/ansible/latest/
- **Traefik** : https://doc.traefik.io/traefik/
- **Let's Encrypt** : https://letsencrypt.org/docs/

---

## 🤝 Support

**Problème de déploiement ?**

1. GitHub Issues : https://github.com/gilmry/koprogo/issues
2. Discord : [à créer]

---

**KoproGo ASBL** - Déploiement automatisé pour les geeks 🚀
