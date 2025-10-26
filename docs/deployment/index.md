# Déploiement KoproGo

Guide complet pour déployer KoproGo sur OVH Public Cloud avec Terraform, Ansible et GitOps automatique.

---

## 🚀 TL;DR - Déploiement en 1 Commande

Depuis la racine du projet :

```bash
make setup-infra
```

**Durée totale** : ~20-30 minutes (dont 15-20 min d'attente automatique)

Le script interactif vous guide automatiquement à travers toutes les étapes nécessaires.

---

## 🎯 Qui utilise ce guide ?

### 1. **Cloud ASBL (Hébergement Géré)** ☁️
L'ASBL KoproGo utilise ce guide pour maintenir son infrastructure cloud multi-tenant (1€/copro/mois).

### 2. **Self-Hosting (Gratuit)** 🔓
Copropriétés ou syndics qui veulent héberger leur propre instance KoproGo.

---

## 📚 Documentation Complète

### [1. Configuration OVH](ovh-setup.md)
- Création compte OVH Public Cloud
- Création utilisateur OpenStack (requis pour Terraform)
- Téléchargement fichier OpenRC
- Création credentials API OVH (optionnel, pour DNS automatique)

### [2. Terraform + Ansible](terraform-ansible.md)
- Architecture de déploiement
- Provisionnement VPS avec Terraform
- Configuration serveur avec Ansible
- Traefik + SSL Let's Encrypt
- Docker Compose (Backend + Frontend + PostgreSQL)

### [3. GitOps Auto-Update](gitops.md)
- Service systemd qui tourne en continu
- Vérification toutes les 3 minutes
- Pull automatique depuis GitHub
- Health checks HTTPS publics
- Rollback automatique si échec

### [4. Troubleshooting](troubleshooting.md)
- Problèmes Terraform courants
- Problèmes Ansible courants
- Problèmes DNS et SSL
- Problèmes GitOps
- Commandes de debug

---

## 🏗️ Architecture Déployée

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

### Composants

1. **Traefik** (Port 80/443) - Reverse proxy + SSL automatique
2. **Backend Rust** (Port interne 8080) - API REST Actix-web
3. **Frontend Astro/Svelte** (Port interne 3000) - SSG + Islands
4. **PostgreSQL 15** (Port interne 5432) - Base de données
5. **GitOps systemd** - Auto-update toutes les 3 minutes

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

### ✅ Performance Validée
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

Voir [Configuration OVH](ovh-setup.md)

---

## 🔄 Workflow de Déploiement

1. **Configuration OVH** - Créer utilisateur OpenStack, télécharger OpenRC
2. **Lancer `make setup-infra`** - Script automatique
3. **Terraform provisionne VPS** - ~5 min
4. **Ansible configure serveur** - ~10 min
5. **Health check HTTPS** - Vérification automatique
6. **GitOps activé** - Auto-update toutes les 3 minutes

---

## 🛠️ Maintenance

### Vérifier le déploiement

```bash
# Health check
curl https://votre-domaine.com/api/v1/health

# Se connecter au VPS
ssh ubuntu@$(cd infrastructure/terraform && terraform output -raw vps_ip)

# Voir les services
sudo su - koprogo
cd ~/koprogo/deploy/production
docker compose ps
docker compose logs -f
```

### Forcer une mise à jour

```bash
# Sur le VPS
sudo /home/koprogo/koprogo/deploy/production/gitops-deploy.sh deploy
```

### Backups

Backups PostgreSQL **quotidiens à 2h du matin** (rétention 7 jours).

```bash
# Sur le VPS
ls -lh ~/koprogo/backups/
```

---

## 🔒 Sécurité

- **Firewall UFW** : Ports 22, 80, 443 ouverts, reste bloqué
- **Fail2ban** : Protection brute-force SSH
- **SSL/TLS** : Certificat Let's Encrypt automatique
- **Health checks** : Vérification HTTPS toutes les 3 minutes

---

## 🧹 Désinstallation

```bash
cd infrastructure/terraform
source ./load-env.sh
terraform destroy
```

---

## 📚 Ressources

### Documentation KoproGo

- **[Configuration OVH](ovh-setup.md)** - Setup compte et credentials
- **[Terraform + Ansible](terraform-ansible.md)** - Détails techniques
- **[GitOps](gitops.md)** - Service systemd et auto-update
- **[Troubleshooting](troubleshooting.md)** - Résolution de problèmes
- **[Lessons Learned](../../infrastructure/LESSONS-LEARNED.md)** - Historique des problèmes

### Documentation externe

- **Terraform OpenStack** : https://registry.terraform.io/providers/terraform-provider-openstack/openstack/latest/docs
- **OVH Public Cloud** : https://help.ovhcloud.com/csm/en-public-cloud-compute-getting-started
- **Ansible** : https://docs.ansible.com/ansible/latest/
- **Traefik** : https://doc.traefik.io/traefik/
- **Let's Encrypt** : https://letsencrypt.org/docs/

---

## 🤝 Support

**Problème de déploiement ?**

1. Consulter [Troubleshooting](troubleshooting.md)
2. Consulter [Lessons Learned](../../infrastructure/LESSONS-LEARNED.md)
3. GitHub Issues : https://github.com/gilmry/koprogo/issues

---

**Dernière mise à jour** : Octobre 2025
**Version** : 3.0 (Terraform + Ansible + GitOps systemd)

**KoproGo ASBL** - Déploiement automatisé pour les geeks 🚀
