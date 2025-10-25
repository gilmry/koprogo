# KoproGo - Déploiement VPS Automatisé (Terraform + Ansible)

Déploiement automatique de KoproGo sur OVH Cloud avec Terraform + Ansible + GitOps.

**Pour qui ?** Geeks qui veulent déployer KoproGo en 5 minutes avec auto-update automatique.

---

## 🎯 Ce que fait ce déploiement

1. **Terraform** : Provisionne un VPS OVH (1 vCPU, 2GB RAM, ~7€/mois)
2. **Ansible** : Configure le serveur (Docker, firewall, sécurité)
3. **GitOps** : Déploie KoproGo avec auto-update quotidien depuis GitHub
4. **Backups** : Backups PostgreSQL quotidiens automatiques
5. **Monitoring** : Health checks toutes les 5 minutes

**Résultat** : KoproGo tourne sur votre VPS et se met à jour automatiquement chaque nuit.

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
```

### Compte OVH Cloud

1. Créer un compte : https://www.ovh.com/manager/public-cloud/
2. Créer un projet Public Cloud
3. Obtenir credentials API :
   - Endpoint : `ovh-eu`
   - Application Key
   - Application Secret
   - Consumer Key

**Obtenir credentials OVH** :
```bash
# Aller sur : https://api.ovh.com/createToken/
# Droits requis : GET/POST/PUT/DELETE sur /cloud/*
# Récupérer : Application Key, Application Secret, Consumer Key
```

---

## 🚀 Déploiement Rapide (5 minutes)

### Étape 1 : Configurer credentials OVH

```bash
cd infrastructure/simple-vps

# Exporter credentials OVH
export OVH_ENDPOINT="ovh-eu"
export OVH_APPLICATION_KEY="votre_application_key"
export OVH_APPLICATION_SECRET="votre_application_secret"
export OVH_CONSUMER_KEY="votre_consumer_key"

# (Optionnel) Domaine pour SSL automatique
export KOPROGO_DOMAIN="koprogo.com"  # Laissez vide si pas de domaine
export ACME_EMAIL="admin@koprogo.com"
```

### Étape 2 : Configurer Terraform

```bash
cd terraform

# Copier et éditer variables
cp terraform.tfvars.example terraform.tfvars
nano terraform.tfvars

# Remplir :
# - ovh_service_name = "ID_DE_VOTRE_PROJET_OVH"
# - ssh_public_key_path = "~/.ssh/id_rsa.pub"
```

### Étape 3 : Provisionner VPS avec Terraform

```bash
# Initialiser Terraform
terraform init

# Voir ce qui va être créé
terraform plan

# Créer le VPS (prend ~2 minutes)
terraform apply

# Récupérer l'IP du VPS
terraform output vps_ip
# Exemple output: 51.75.xxx.xxx
```

### Étape 4 : Configurer Ansible

```bash
cd ../ansible

# Copier et éditer inventory
cp inventory.ini.example inventory.ini
nano inventory.ini

# Remplacer YOUR_VPS_IP par l'IP obtenue via terraform output
```

### Étape 5 : Déployer KoproGo avec Ansible

```bash
# Tester connexion SSH
ansible -i inventory.ini koprogo -m ping

# Déployer KoproGo (prend ~5-10 minutes)
ansible-playbook -i inventory.ini playbook.yml

# Si vous avez un domaine, ajouter variables :
ansible-playbook -i inventory.ini playbook.yml \
  -e "domain=koprogo.com" \
  -e "acme_email=admin@koprogo.com"
```

### Étape 6 : Vérifier déploiement

```bash
# Health check API
curl http://$(terraform -chdir=../terraform output -raw vps_ip)/api/v1/health

# Devrait retourner : {"status":"healthy"}

# Se connecter au VPS
ssh ubuntu@$(terraform -chdir=../terraform output -raw vps_ip)

# Sur le VPS, vérifier services
sudo su - koprogo
cd ~/koprogo
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
| VPS OVH Value (1 vCPU, 2GB RAM) | **7€ TTC/mois** |
| Domaine (optionnel) | ~12€/an (~1€/mois) |
| SSL Let's Encrypt | **0€** |
| **TOTAL** | **~7-8€/mois** |

**Capacité** : 1,000-1,500 copropriétés (validé par tests de charge)

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

### Terraform : "Error creating instance"

**Cause** : Credentials OVH incorrects ou quota dépassé.

**Fix** :
```bash
# Vérifier credentials
env | grep OVH

# Vérifier quota projet OVH
# https://www.ovh.com/manager/public-cloud/ → Quotas
```

### Ansible : "SSH connection failed"

**Cause** : VPS pas encore prêt ou IP incorrecte.

**Fix** :
```bash
# Attendre 1-2 minutes après terraform apply
sleep 60

# Tester SSH manuel
ssh ubuntu@$(cd terraform && terraform output -raw vps_ip)
```

### Health check échoue

**Cause** : Services Docker pas encore démarrés.

**Fix** :
```bash
# Attendre 30 secondes
sleep 30

# Vérifier logs
docker compose logs backend
```

---

## 📚 Ressources

- **Guide complet déploiement** : [docs/VPS_DEPLOYMENT.md](../../docs/VPS_DEPLOYMENT.md)
- **GitOps manuel** : [docs/DEPLOY_GITOPS.md](../../docs/DEPLOY_GITOPS.md)
- **Business plan** : [docs/BUSINESS_PLAN_BOOTSTRAP.md](../../docs/BUSINESS_PLAN_BOOTSTRAP.md)
- **Terraform OVH** : https://registry.terraform.io/providers/ovh/ovh/latest/docs
- **Ansible** : https://docs.ansible.com/

---

## 🤝 Support

**Problème de déploiement ?**

1. GitHub Issues : https://github.com/gilmry/koprogo/issues
2. Discord : [à créer]

---

**KoproGo ASBL** - Déploiement automatisé pour les geeks 🚀
