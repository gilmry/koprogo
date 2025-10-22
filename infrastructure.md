# KoproGo Infrastructure

Ce répertoire contient l'Infrastructure as Code (IaC) pour KoproGo.

## Structure

```
infrastructure/
├── terraform/       # Provisioning OVH
├── ansible/         # Configuration & K3s
├── helm/           # Déploiement applicatif
└── scripts/        # Automatisation
```

## Prérequis

- Terraform >= 1.5
- Ansible >= 2.15
- kubectl >= 1.28
- helm >= 3.13
- Compte OVH avec API credentials

## Quick Start

```bash
# 1. Configuration OVH
export OVH_ENDPOINT="ovh-eu"
export OVH_APPLICATION_KEY="your_app_key"
export OVH_APPLICATION_SECRET="your_app_secret"
export OVH_CONSUMER_KEY="your_consumer_key"

# 2. Déploiement
cd infrastructure
./scripts/deploy.sh dev

# 3. Accès au cluster
export KUBECONFIG=~/.kube/koprogo-dev
kubectl get nodes
```

Voir [infrastructure/README.md](infrastructure/README.md) pour plus de détails.
