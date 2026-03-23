#!/bin/bash

# Script de déploiement complet
# Usage: ./deploy.sh

set -e

cd "$(dirname "$0")"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Déploiement Terraform OVH - KoproGo                       ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Charger les variables
echo "📦 Chargement des variables d'environnement..."
source ./load-env.sh

echo ""
echo "🚀 Déploiement de l'infrastructure..."
echo ""

# Déployer
terraform apply -auto-approve

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Déploiement terminé!                                      ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Afficher les outputs
echo "📊 Informations du VPS:"
terraform output

echo ""
echo "Prochaines étapes:"
echo "  1. Configurer Ansible: cd ../ansible && ./setup-inventory.sh"
echo "  2. Déployer l'app: ansible-playbook -i inventory.ini playbook.yml"
echo ""
