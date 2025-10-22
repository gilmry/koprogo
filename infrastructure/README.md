# KoproGo Infrastructure

Infrastructure as Code (IaC) pour KoproGo - Plateforme SaaS de gestion de copropriété.

## 🎯 Vue d'ensemble

Cette infrastructure déploie un cluster Kubernetes (K3s) hautement disponible sur OVH Cloud avec :

- **Terraform** : Provisioning de l'infrastructure OVH
- **Ansible** : Configuration des serveurs et installation K3s
- **Helm** : Déploiement des applications
- **K3s** : Distribution Kubernetes légère

## 📁 Structure

```
infrastructure/
├── terraform/              # Infrastructure provisioning
│   ├── modules/           # Modules réutilisables
│   │   ├── ovh-instances/ # Compute instances
│   │   ├── networking/    # vRack, firewall, load balancer
│   │   ├── storage/       # Object storage (backups)
│   │   └── dns/           # DNS records
│   └── environments/      # Configurations par environnement
│       ├── dev/           # Développement (1 CP + 1 worker)
│       ├── staging/       # Staging (1 CP + 2 workers)
│       └── prod/          # Production (3 CP + 3 workers)
│
├── ansible/               # Configuration management
│   ├── roles/            # Rôles Ansible
│   │   ├── common/       # Configuration système de base
│   │   ├── security/     # Hardening sécurité
│   │   ├── k3s-server/   # Control plane K3s
│   │   └── k3s-agent/    # Workers K3s
│   ├── playbooks/        # Playbooks d'orchestration
│   └── inventory/        # Inventaires (générés par Terraform)
│
├── helm/                 # Charts Helm
│   ├── koprogo-api/     # API Backend Rust
│   ├── koprogo-frontend/# Frontend Astro
│   ├── postgresql-ha/   # PostgreSQL HA
│   ├── dragonfly/       # Cache DragonflyDB
│   ├── minio/           # Object storage
│   └── monitoring/      # Stack Prometheus/Grafana
│
└── scripts/             # Scripts d'automatisation
    ├── deploy.sh        # Déploiement complet
    ├── destroy.sh       # Destruction infrastructure
    ├── backup.sh        # Backup cluster
    └── rollback.sh      # Rollback applications
```

## 🚀 Démarrage Rapide

### Prérequis

- **Terraform** >= 1.5
- **Ansible** >= 2.15
- **kubectl** >= 1.28
- **Helm** >= 3.13
- Compte **OVH Cloud** avec API credentials

### 1. Configuration OVH

Obtenez vos credentials API OVH depuis https://api.ovh.com/createToken/

```bash
export OVH_ENDPOINT="ovh-eu"
export OVH_APPLICATION_KEY="your_app_key"
export OVH_APPLICATION_SECRET="your_app_secret"
export OVH_CONSUMER_KEY="your_consumer_key"
```

### 2. Configuration SSH

Générez une clé SSH pour les instances :

```bash
ssh-keygen -t rsa -b 4096 -f ~/.ssh/koprogo-dev -C "koprogo-dev"
```

### 3. Configuration Terraform

Copiez et personnalisez les variables :

```bash
cd terraform/environments/dev
cp terraform.tfvars.example terraform.tfvars
# Éditez terraform.tfvars avec vos valeurs
vim terraform.tfvars
```

### 4. Déploiement Automatique

Utilisez le script de déploiement automatique :

```bash
cd infrastructure
./scripts/deploy.sh dev
```

Ou déploiement manuel étape par étape :

```bash
# 1. Terraform
cd terraform/environments/dev
terraform init
terraform plan
terraform apply

# 2. Ansible
cd ../../ansible
ansible-playbook -i inventory/dev.yml playbooks/site.yml

# 3. Helm
cd ../helm
export KUBECONFIG=~/.kube/koprogo-dev
helm upgrade --install koprogo-api ./koprogo-api \
  --namespace koprogo \
  --create-namespace \
  --values ./koprogo-api/values-dev.yaml
```

### 5. Vérification

```bash
export KUBECONFIG=~/.kube/koprogo-dev

# Vérifier les nodes
kubectl get nodes

# Vérifier les pods
kubectl get pods -A

# Tester l'API
kubectl port-forward -n koprogo svc/koprogo-api 8080:8080
curl http://localhost:8080/api/v1/health
```

## 🏗️ Environnements

### Development (dev)

- **Coût** : ~30€/mois
- **Configuration** :
  - 1 control plane : b2-7 (2 vCPU, 7GB RAM)
  - 1 worker : b2-15 (4 vCPU, 15GB RAM)
- **Usage** : Tests et développement

```bash
./scripts/deploy.sh dev
```

### Staging (staging)

- **Coût** : ~90€/mois
- **Configuration** :
  - 1 control plane : b2-15 (4 vCPU, 15GB RAM)
  - 2 workers : b2-15 (4 vCPU, 15GB RAM)
- **Usage** : Pre-production testing

```bash
./scripts/deploy.sh staging
```

### Production (prod)

- **Coût** : ~270€/mois
- **Configuration** :
  - 3 control plane : b2-15 (4 vCPU, 15GB RAM) - HA
  - 3 workers : b2-30 (8 vCPU, 30GB RAM)
- **Usage** : Production avec haute disponibilité

```bash
./scripts/deploy.sh prod
```

## 📊 Architecture

### High Availability (Production)

```
┌─────────────────────────────────────────────┐
│           Load Balancer (OVH)               │
│         (HTTP/HTTPS Traffic)                │
└────────────────┬────────────────────────────┘
                 │
         ┌───────┴───────┐
         │               │
    ┌────▼────┐    ┌────▼────┐    ┌─────────┐
    │ Control │    │ Control │    │ Control │
    │ Plane 1 │◄──►│ Plane 2 │◄──►│ Plane 3 │
    │ (etcd)  │    │ (etcd)  │    │ (etcd)  │
    └─────────┘    └─────────┘    └─────────┘
         │               │              │
    ┌────┴───────────────┴──────────────┴────┐
    │                                         │
┌───▼────┐        ┌─────────┐        ┌──────▼──┐
│Worker 1│        │Worker 2 │        │Worker 3 │
│        │        │         │        │         │
│Longhorn│◄──────►│Longhorn │◄──────►│Longhorn │
└────────┘        └─────────┘        └─────────┘
```

### Network Architecture

- **Private Network** : vRack 10.0.0.0/24
- **Cluster CIDR** : 10.42.0.0/16
- **Service CIDR** : 10.43.0.0/16
- **Firewall** : UFW avec règles strictes
- **Load Balancer** : OVH LB pour ingress

## 🔒 Sécurité

### Implémentations

- ✅ SSH key-only authentication
- ✅ Fail2ban pour protection brute-force
- ✅ UFW firewall avec règles strictes
- ✅ TLS/SSL avec Let's Encrypt
- ✅ Network policies Kubernetes
- ✅ Pod Security Standards
- ✅ RBAC strict
- ✅ Secrets chiffrés
- ✅ Audit logging (auditd)

### Best Practices

1. **Rotation des secrets** : Changer les tokens K3s régulièrement
2. **Updates** : Unattended upgrades activé
3. **Backups** : Automatiques quotidiens
4. **Monitoring** : Prometheus + Alertmanager
5. **Logs centralisés** : Loki + Grafana

## 📈 Monitoring & Observabilité

### Stack Monitoring

- **Prometheus** : Métriques
- **Grafana** : Dashboards
- **Loki** : Logs centralisés
- **Kepler** : Métriques CO2
- **Alertmanager** : Alertes

### Dashboards Inclus

- Cluster Overview
- Node Metrics
- Pod Resources
- API Performance
- Database Metrics
- CO2 Footprint

## 🔄 Backup & Disaster Recovery

### Stratégie de Backup

- **Etcd** : Backup automatique toutes les 6h
- **PostgreSQL** : Backup quotidien avec rétention 30 jours
- **Persistent Volumes** : Snapshots Longhorn quotidiens
- **Configuration** : GitOps avec versioning

### Restoration

```bash
# Restore depuis backup
./scripts/restore.sh prod 2024-01-15-backup

# Rollback application
./scripts/rollback.sh koprogo-api v1.2.0
```

## 🛠️ Commandes Utiles

```bash
# Déploiement
./scripts/deploy.sh [env]

# Destruction (ATTENTION!)
./scripts/destroy.sh [env]

# Backup manuel
./scripts/backup.sh [env]

# Mise à jour application
cd helm
helm upgrade koprogo-api ./koprogo-api \
  --namespace koprogo \
  --values ./koprogo-api/values-prod.yaml

# Scaling manuel
kubectl scale deployment koprogo-api \
  -n koprogo \
  --replicas=10

# Logs
kubectl logs -n koprogo -l app.kubernetes.io/name=koprogo-api -f

# Shell dans pod
kubectl exec -it -n koprogo <pod-name> -- /bin/bash
```

## 🐛 Troubleshooting

### Problèmes Courants

**1. Terraform apply échoue**

```bash
# Vérifier les credentials OVH
echo $OVH_APPLICATION_KEY

# Réinitialiser l'état
terraform init -reconfigure
```

**2. Ansible ne peut pas se connecter**

```bash
# Tester connectivité SSH
ansible all -i inventory/dev.yml -m ping

# Vérifier la clé SSH
ssh -i ~/.ssh/koprogo-dev ubuntu@<ip>
```

**3. Pods en CrashLoopBackOff**

```bash
# Voir les logs
kubectl logs -n koprogo <pod-name>

# Décrire le pod
kubectl describe pod -n koprogo <pod-name>

# Vérifier les events
kubectl get events -n koprogo --sort-by='.lastTimestamp'
```

**4. Performance dégradée**

```bash
# Vérifier les ressources
kubectl top nodes
kubectl top pods -A

# Vérifier le HPA
kubectl get hpa -n koprogo

# Forcer un scale up
kubectl scale deployment koprogo-api -n koprogo --replicas=5
```

## 📚 Documentation Additionnelle

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Architecture détaillée
- [SECURITY.md](./SECURITY.md) - Politiques de sécurité (TODO)
- [RUNBOOK.md](./RUNBOOK.md) - Procédures opérationnelles (TODO)

## 🤝 Support

Pour toute question ou problème :

- **Email** : ops@koprogo.io
- **Issues** : GitHub Issues
- **Slack** : #infrastructure (interne)

## 📄 Licence

MIT License - Voir [LICENSE](../LICENSE)
