# KoproGo VPS Deployment Guide

**Guide complet de déploiement de KoproGo sur OVH Public Cloud avec Terraform + Ansible**

Déploiement automatisé avec GitOps, SSL automatique, DNS automatique, backups et monitoring.

---

## 🎯 Qui utilise ce guide ?

Ce guide est pour **deux cas d'usage** :

### 1. **Cloud ASBL (Hébergement Géré)** ☁️
L'ASBL KoproGo utilise ce guide pour maintenir son infrastructure cloud multi-tenant (1€/copro/mois).

### 2. **Self-Hosting (Gratuit)** 🔓
Copropriétés ou syndics qui veulent héberger leur propre instance KoproGo.

---

## 🚀 TL;DR - Déploiement en 1 Commande

Depuis la racine du projet KoproGo :

```bash
make setup-infra
```

Le script interactif vous guide à travers :
1. ✅ Création credentials OVH API (optionnel, pour DNS automatique)
2. ✅ Création utilisateur OpenStack avec tous les rôles nécessaires
3. ✅ Téléchargement fichier OpenRC (région GRA9)
4. ✅ Configuration domaine (optionnel)
5. ✅ Déploiement Terraform (provisionne le VPS)
6. ✅ Configuration DNS automatique via API OVH
7. ✅ Déploiement Ansible (Docker, Traefik, KoproGo, PostgreSQL)

**Durée totale** : ~20-30 minutes (dont 15-20 min d'attente automatique)

---

## 🏗️ Architecture de Déploiement

```
Internet (HTTPS)
      ↓
Traefik (Reverse Proxy + SSL Let's Encrypt)
      ↓
   ┌──────────────────────────────────────┐
   │     Docker Compose (VPS OVH)         │
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

   Datacenter France (Gravelines GRA9)
   60g CO₂/kWh (nucléaire 70% + renouvelables 25%)
   0.12g CO₂/requête
```

### Composants Déployés

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

5. **GitOps Auto-Update**
   - Pull automatique depuis GitHub (3h du matin)
   - Rebuild et redémarrage automatique
   - Rollback automatique si health check échoue

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

## 🌍 Pourquoi OVH Cloud France ?

### ✅ Écologie Exceptionnelle
- Datacenter France (mix énergétique **60g CO₂/kWh**)
- **0.12g CO₂/requête** (7-25x mieux que AWS/Azure)
- Nucléaire (70%) + Renouvelables (25%)
- **Champion mondial** de l'écologie cloud

### ✅ Souveraineté & GDPR
- Données hébergées en France
- GDPR-native, conformité totale
- Pas de CLOUD Act américain
- Support en français

### ✅ Performance Validée (Tests Réels)
- **287 req/s** soutenus, 99.74% uptime
- P50: 69ms, P90: 130ms, P99: 752ms
- Anti-DDoS inclus
- 1 Gbps network

---

## 📋 Prérequis

### Sur Votre Machine

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

## 📖 Guide Détaillé Pas-à-Pas

### Étape 1 : Créer un Utilisateur OpenStack (REQUIS)

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

> **⚠️ CRITIQUE** : Le rôle **Administrator** est absolument nécessaire pour que Terraform puisse créer des ressources !

### Étape 2 : Télécharger le Fichier OpenRC (REQUIS)

1. **OVH Manager** → **Public Cloud** → **Users & Roles**
2. Cliquer sur **...** à côté de votre utilisateur
3. Sélectionner **Download OpenStack's RC file**
4. **Ouvrir le fichier** et trouver la ligne :
   ```bash
   export OS_REGION_NAME="GRA9"
   ```
5. **Noter la région** (exemple: GRA9, GRA11, SBG5, etc.)

> **⚠️ IMPORTANT** : Utilisez toujours la région exacte du fichier OpenRC ! Ne PAS deviner.

### Étape 3 : Créer Credentials OVH API (OPTIONNEL, pour DNS automatique)

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

### Étape 4 : Lancer le Déploiement

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

### Étape 5 : Vérifier le Déploiement

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

## 🔄 GitOps Auto-Update

KoproGo se met à jour automatiquement tous les jours à **3h du matin** depuis GitHub.

### Comment ça marche ?

1. **Cron job** : Exécute `~/koprogo/scripts/auto-update.sh` quotidiennement
2. **Backup** : Sauvegarde la DB avant update
3. **Pull GitHub** : `git pull origin main`
4. **Rebuild** : `docker compose up -d --build`
5. **Health check** : Vérifie `/api/v1/health`
6. **Rollback automatique** : Si health check échoue

### Logs auto-update

```bash
# Sur le VPS
tail -f /var/log/koprogo-update.log
```

### Désactiver auto-update

```bash
# Supprimer cron job
crontab -e -u koprogo
# Commenter ou supprimer la ligne : 0 3 * * * ...
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
cd ~/koprogo/deploy/production

# Restaurer le dernier backup
gunzip -c ../../backups/koprogo_YYYYMMDD_HHMMSS.sql.gz | \
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

# Vérifier certificat
openssl s_client -connect votre-domaine.com:443 -servername votre-domaine.com
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
cd ~/koprogo/deploy/production
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

# Frontend uniquement
docker compose logs -f frontend

# PostgreSQL uniquement
docker compose logs -f postgres

# Traefik uniquement
docker compose logs -f traefik
```

### Cleanup Docker

```bash
# Supprimer images inutilisées
docker system prune -a

# Voir utilisation disque
docker system df
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

### Terraform : "Variables not loaded"

**Symptôme** :
```
Error: Missing required argument
```

**Cause** : Variables d'environnement non chargées

**Fix** : Utiliser `source` pour charger les variables
```bash
# ✅ CORRECT
source ./load-env.sh

# ❌ FAUX (crée une nouvelle sous-shell)
./load-env.sh

# Ou utiliser le script de déploiement
cd infrastructure/terraform
./deploy.sh
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

### Health check échoue

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

## 🧹 Désinstallation

```bash
# 1. Détruire VPS Terraform
cd infrastructure/terraform
terraform destroy

# 2. (Optionnel) Cleanup credentials
unset OVH_APPLICATION_KEY OVH_APPLICATION_SECRET OVH_CONSUMER_KEY
unset OS_USERNAME OS_PASSWORD

# 3. (Optionnel) Supprimer fichiers locaux
rm -f .env terraform.tfvars
cd ../ansible
rm -f inventory.ini
```

---

## 📚 Ressources

### Documentation KoproGo

- **Infrastructure README** : [`infrastructure/README.md`](../infrastructure/README.md)
- **Lessons Learned** : [`infrastructure/LESSONS-LEARNED.md`](../infrastructure/LESSONS-LEARNED.md) - Tous les problèmes rencontrés et solutions
- **GitOps manuel** : [DEPLOY_GITOPS.md](./DEPLOY_GITOPS.md)
- **Business plan** : [BUSINESS_PLAN_BOOTSTRAP.md](./BUSINESS_PLAN_BOOTSTRAP.md)

### Documentation externe

- **Terraform OpenStack Provider** : https://registry.terraform.io/providers/terraform-provider-openstack/openstack/latest/docs
- **OVH Public Cloud** : https://help.ovhcloud.com/csm/en-public-cloud-compute-getting-started
- **Ansible** : https://docs.ansible.com/ansible/latest/
- **Traefik** : https://doc.traefik.io/traefik/
- **Let's Encrypt** : https://letsencrypt.org/docs/

---

## 🤝 Support

**Problème de déploiement ?**

1. Consulter **Troubleshooting** ci-dessus
2. Consulter [`infrastructure/LESSONS-LEARNED.md`](../infrastructure/LESSONS-LEARNED.md)
3. GitHub Issues : https://github.com/gilmry/koprogo/issues

---

## 🎓 Leçons Clés

1. **TOUJOURS télécharger le fichier OpenRC** - C'est la source de vérité pour la région
2. **Utiliser le provider OpenStack** - Plus stable que le provider OVH natif
3. **Rôle Administrator requis** - Pour l'utilisateur OpenStack
4. **Source vs Execute** - `source ./load-env.sh` pas `./load-env.sh`
5. **Automation complète** - Le script `setup-infra.sh` réduit drastiquement les erreurs
6. **Documentation visuelle** - Screenshots + guide interactif = succès
7. **Validation préalable** - Vérifier les prérequis avant de commencer
8. **Become method** - Toujours utiliser `become_method: su` avec Ansible

---

**Dernière mise à jour** : Octobre 2025
**Version** : 2.0 (Terraform + Ansible automatisé)

---

**KoproGo ASBL** - Déploiement automatisé pour les geeks 🚀
