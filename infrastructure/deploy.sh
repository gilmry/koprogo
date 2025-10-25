#!/bin/bash
# KoproGo - Script de déploiement automatique
# Terraform + Ansible en une seule commande

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TERRAFORM_DIR="$SCRIPT_DIR/terraform"
ANSIBLE_DIR="$SCRIPT_DIR/ansible"

echo "=============================================="
echo "KoproGo - Déploiement Automatisé"
echo "=============================================="
echo ""

# ==========================================
# 1. Vérifications prérequis
# ==========================================
echo "🔍 Vérification des prérequis..."

if ! command -v terraform &> /dev/null; then
    echo "❌ Terraform n'est pas installé. Installez-le : https://www.terraform.io/downloads"
    exit 1
fi

if ! command -v ansible-playbook &> /dev/null; then
    echo "❌ Ansible n'est pas installé. Installez-le : pip install ansible"
    exit 1
fi

if [ -z "$OVH_APPLICATION_KEY" ] || [ -z "$OVH_APPLICATION_SECRET" ] || [ -z "$OVH_CONSUMER_KEY" ]; then
    echo "❌ Credentials OVH manquants. Exportez:"
    echo "   export OVH_ENDPOINT='ovh-eu'"
    echo "   export OVH_APPLICATION_KEY='...'"
    echo "   export OVH_APPLICATION_SECRET='...'"
    echo "   export OVH_CONSUMER_KEY='...'"
    exit 1
fi

echo "✅ Terraform $(terraform version -json | grep -o '"version":"[^"]*' | cut -d'"' -f4)"
echo "✅ Ansible $(ansible --version | head -n1 | awk '{print $2}')"
echo "✅ Credentials OVH configurés"
echo ""

# ==========================================
# 2. Configuration Terraform
# ==========================================
echo "📝 Configuration Terraform..."

if [ ! -f "$TERRAFORM_DIR/terraform.tfvars" ]; then
    echo "❌ Fichier terraform.tfvars manquant."
    echo "   Copiez terraform.tfvars.example et remplissez les valeurs:"
    echo "   cd terraform && cp terraform.tfvars.example terraform.tfvars"
    exit 1
fi

# ==========================================
# 3. Provisionner VPS avec Terraform
# ==========================================
echo "🚀 Provisionnement VPS OVH avec Terraform..."
cd "$TERRAFORM_DIR"

terraform init -upgrade

echo ""
echo "Plan Terraform:"
terraform plan

read -p "⚠️  Voulez-vous créer le VPS ? (yes/no) " -r
echo
if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
    echo "❌ Déploiement annulé"
    exit 1
fi

terraform apply -auto-approve

# Récupérer IP du VPS
VPS_IP=$(terraform output -raw vps_ip)
echo ""
echo "✅ VPS créé avec succès !"
echo "   IP: $VPS_IP"
echo ""

# ==========================================
# 4. Attendre que le VPS soit prêt
# ==========================================
echo "⏳ Attente que le VPS soit prêt (SSH)..."
sleep 30

MAX_RETRIES=30
RETRY=0
while [ $RETRY -lt $MAX_RETRIES ]; do
    if ssh -o StrictHostKeyChecking=no -o ConnectTimeout=5 ubuntu@$VPS_IP "echo SSH OK" &> /dev/null; then
        echo "✅ VPS prêt !"
        break
    fi
    echo "   Tentative $((RETRY+1))/$MAX_RETRIES..."
    sleep 10
    RETRY=$((RETRY+1))
done

if [ $RETRY -eq $MAX_RETRIES ]; then
    echo "❌ Impossible de se connecter au VPS via SSH"
    exit 1
fi

echo ""

# ==========================================
# 5. Configuration Ansible
# ==========================================
echo "📝 Configuration Ansible..."
cd "$ANSIBLE_DIR"

# Créer inventory dynamique
cat > inventory.ini << EOF
[koprogo]
koprogo-vps ansible_host=$VPS_IP ansible_user=ubuntu ansible_ssh_private_key_file=~/.ssh/id_rsa

[koprogo:vars]
ansible_ssh_common_args='-o StrictHostKeyChecking=no'
EOF

echo "✅ Inventory créé avec IP $VPS_IP"
echo ""

# ==========================================
# 6. Déployer KoproGo avec Ansible
# ==========================================
echo "🚀 Déploiement KoproGo avec Ansible..."

EXTRA_VARS=""
if [ -n "$KOPROGO_DOMAIN" ]; then
    EXTRA_VARS="-e domain=$KOPROGO_DOMAIN"
    if [ -n "$ACME_EMAIL" ]; then
        EXTRA_VARS="$EXTRA_VARS -e acme_email=$ACME_EMAIL"
    fi
    echo "🔒 SSL/HTTPS activé pour $KOPROGO_DOMAIN"
fi

ansible-playbook -i inventory.ini playbook.yml $EXTRA_VARS

echo ""
echo "=============================================="
echo "✅ Déploiement terminé avec succès !"
echo "=============================================="
echo ""
echo "📊 Informations de connexion:"
echo "   IP VPS:      $VPS_IP"
echo "   SSH:         ssh ubuntu@$VPS_IP"
echo "   API URL:     http://$VPS_IP/api/v1"
echo "   Health:      http://$VPS_IP/api/v1/health"
echo ""

if [ -n "$KOPROGO_DOMAIN" ]; then
    echo "   HTTPS URL:   https://$KOPROGO_DOMAIN/api/v1"
    echo ""
    echo "⚠️  N'oubliez pas de configurer votre DNS:"
    echo "   $KOPROGO_DOMAIN    A    $VPS_IP"
    echo ""
fi

echo "🔄 Auto-update: Activé (tous les jours à 3h)"
echo "💾 Backups:     Activés (tous les jours à 2h)"
echo "🏥 Monitoring:  Activé (health checks toutes les 5 min)"
echo ""
echo "📚 Logs:"
echo "   Updates:  ssh ubuntu@$VPS_IP 'tail -f /var/log/koprogo-update.log'"
echo "   Backups:  ssh ubuntu@$VPS_IP 'tail -f /var/log/koprogo-backup.log'"
echo "   Health:   ssh ubuntu@$VPS_IP 'tail -f /var/log/koprogo-health.log'"
echo ""
echo "🆘 Besoin d'aide ? https://github.com/gilmry/koprogo/issues"
echo "=============================================="
