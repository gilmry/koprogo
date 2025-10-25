# KoproGo - Quick Start Déploiement

Déployez KoproGo sur OVH en **5 minutes** avec Terraform + Ansible + GitOps.

---

## 🚀 TL;DR - One-Liner

```bash
# 1. Exporter credentials OVH
export OVH_ENDPOINT="ovh-eu"
export OVH_APPLICATION_KEY="..."
export OVH_APPLICATION_SECRET="..."
export OVH_CONSUMER_KEY="..."

# 2. Configurer Terraform
cd infrastructure/terraform
cp terraform.tfvars.example terraform.tfvars
nano terraform.tfvars  # Remplir ovh_service_name

# 3. Déployer TOUT (VPS + KoproGo)
cd ..
./deploy.sh
```

**C'est tout !** ✅

---

## 📋 Prérequis

**Outils** (sur votre machine locale) :
```bash
terraform --version  # 1.0+
ansible --version    # 2.9+
ssh-keygen -t rsa    # Clé SSH générée
```

**Compte OVH Cloud** :
- Créer projet : https://www.ovh.com/manager/public-cloud/
- Obtenir credentials : https://api.ovh.com/createToken/
  - Droits : `GET/POST/PUT/DELETE` sur `/cloud/*`
  - Récupérer : Application Key, Secret, Consumer Key

---

## 📖 Guide Complet

Voir [README.md](README.md) pour documentation complète.

---

## 🎯 Ce qui sera déployé

- ✅ VPS OVH (1 vCPU, 2GB RAM, ~7€/mois)
- ✅ Docker + Docker Compose
- ✅ Firewall UFW (SSH, HTTP, HTTPS)
- ✅ KoproGo backend + PostgreSQL
- ✅ Auto-update quotidien (GitHub → GitOps)
- ✅ Backups PostgreSQL quotidiens
- ✅ Health checks toutes les 5 min

**Datacenter** : Gravelines, France (60g CO₂/kWh)

---

## 🔒 SSL/HTTPS (Optionnel)

Si vous avez un domaine :

```bash
export KOPROGO_DOMAIN="koprogo.com"
export ACME_EMAIL="admin@koprogo.com"

# Lancer deploy.sh (SSL activé automatiquement)
./deploy.sh
```

N'oubliez pas de configurer DNS :
```
koprogo.com    A    <VPS_IP>
```

---

## 📊 Après déploiement

### Vérifier santé

```bash
# Health check API
curl http://<VPS_IP>/api/v1/health

# SSH vers VPS
ssh ubuntu@<VPS_IP>

# Logs Docker
docker compose logs -f
```

### Monitoring

```bash
# Sur le VPS
tail -f /var/log/koprogo-update.log   # Auto-updates
tail -f /var/log/koprogo-backup.log   # Backups
tail -f /var/log/koprogo-health.log   # Health checks
```

---

## 🆘 Aide

**Problème ?** Voir [README.md](README.md) section Troubleshooting.

**Support** : https://github.com/gilmry/koprogo/issues

---

**KoproGo ASBL** - Déploiement ultra-rapide 🚀
