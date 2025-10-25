#!/bin/bash

# Script d'orchestration complète pour setup infrastructure OVH
# Guide l'utilisateur pas à pas pour:
# 1. Créer les credentials OVH API
# 2. Créer/configurer le projet Public Cloud
# 3. Créer l'utilisateur OpenStack avec les bons rôles
# 4. Télécharger le fichier OpenRC
# 5. Configurer le .env
# 6. Optionnel: Configurer un domaine et DNS
# 7. Déployer avec Terraform + Ansible

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TERRAFORM_DIR="$SCRIPT_DIR/terraform"
ANSIBLE_DIR="$SCRIPT_DIR/ansible"

# Couleurs
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                                                            ║"
echo "║     KoproGo - Setup Infrastructure OVH Cloud               ║"
echo "║     Guide pas à pas pour déploiement complet               ║"
echo "║                                                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# ============================================================================
# Vérifications préalables
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 0: Vérifications préalables                      │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

# Vérifier Terraform
if ! command -v terraform &> /dev/null; then
    echo -e "${RED}❌ Terraform n'est pas installé${NC}"
    echo ""
    echo "Installation:"
    echo "  Ubuntu/Debian: wget -O- https://apt.releases.hashicorp.com/gpg | sudo gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg && echo \"deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main\" | sudo tee /etc/apt/sources.list.d/hashicorp.list && sudo apt update && sudo apt install terraform"
    echo "  macOS: brew install terraform"
    echo ""
    exit 1
fi

# Vérifier Ansible
if ! command -v ansible &> /dev/null; then
    echo -e "${RED}❌ Ansible n'est pas installé${NC}"
    echo ""
    echo "Installation:"
    echo "  Ubuntu/Debian: sudo apt install -y ansible"
    echo "  macOS: brew install ansible"
    echo "  pip: pip3 install ansible"
    echo ""
    exit 1
fi

echo -e "${GREEN}✅ Terraform installé: $(terraform version | head -n1)${NC}"
echo -e "${GREEN}✅ Ansible installé: $(ansible --version | head -n1)${NC}"
echo ""

# ============================================================================
# Étape 1: Credentials OVH API
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 1: Configuration OVH API Credentials             │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

echo "Pour déployer sur OVH, vous avez besoin de credentials API OVH."
echo ""
echo -e "${YELLOW}📋 Instructions:${NC}"
echo "  1. Ouvrez: https://www.ovh.com/auth/api/createToken"
echo "  2. Connectez-vous à votre compte OVH"
echo "  3. Remplissez le formulaire:"
echo "     - Application name: KoproGo"
echo "     - Application description: KoproGo Infrastructure Management"
echo "     - Validity: Unlimited"
echo "     - Rights:"
echo "       GET    /cloud/*"
echo "       POST   /cloud/*"
echo "       PUT    /cloud/*"
echo "       DELETE /cloud/*"
echo "       GET    /domain/*"
echo "       POST   /domain/*"
echo "       PUT    /domain/*"
echo "  4. Cliquez sur 'Create keys'"
echo ""

read -p "Appuyez sur Entrée quand vous avez créé les credentials..."
echo ""

read -p "Application Key: " OVH_APPLICATION_KEY
read -p "Application Secret: " OVH_APPLICATION_SECRET
read -p "Consumer Key: " OVH_CONSUMER_KEY

echo ""
echo -e "${GREEN}✅ Credentials OVH API enregistrées${NC}"
echo ""

# ============================================================================
# Étape 2: Projet Public Cloud OVH
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 2: Projet Public Cloud OVH                       │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

echo "Vous avez besoin d'un projet Public Cloud OVH."
echo ""
echo -e "${YELLOW}📋 Instructions:${NC}"
echo "  1. Ouvrez: https://www.ovh.com/manager/public-cloud/"
echo "  2. Si vous n'avez pas de projet:"
echo "     - Cliquez sur 'Create a project'"
echo "     - Suivez les étapes de création"
echo "  3. Notez le 'Project ID' (Service Name)"
echo "     - Visible dans: Project Management > Project ID"
echo "     - Format: 32 caractères (ex: dd8822a8a417499bb97651ed4728a2ca)"
echo ""

read -p "Appuyez sur Entrée quand vous êtes prêt..."
echo ""

read -p "Project ID (Service Name): " OVH_SERVICE_NAME

echo ""
echo -e "${GREEN}✅ Project ID enregistré: $OVH_SERVICE_NAME${NC}"
echo ""

# ============================================================================
# Étape 3: Utilisateur OpenStack
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 3: Utilisateur OpenStack avec rôles appropriés   │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

echo "Vous devez créer un utilisateur OpenStack avec les bons rôles."
echo ""
echo -e "${YELLOW}📋 Instructions:${NC}"
echo "  1. Ouvrez: https://www.ovh.com/manager/public-cloud/"
echo "  2. Sélectionnez votre projet"
echo "  3. Allez dans: Project Management > Users & Roles"
echo "  4. Cliquez sur 'Create User'"
echo "  5. Description: koprogo-deploy"
echo "  6. Cochez TOUS les rôles suivants:"
echo ""
echo -e "${YELLOW}     ☑ Administrator${NC} ${GREEN}(IMPORTANT!)${NC}"
echo "     ☑ Compute Operator"
echo "     ☑ Network Operator"
echo "     ☑ Network Security Operator"
echo "     ☑ Image Operator"
echo "     ☑ Volume Operator"
echo "     ☑ ObjectStore Operator"
echo "     ☑ LoadBalancer Operator"
echo "     ☑ Backup Operator"
echo "     ☑ Infrastructure Supervisor"
echo "     ☑ KeyManager Operator"
echo "     ☑ KeyManager Read"
echo ""
echo "  7. Cliquez sur 'Confirm'"
echo "  8. IMPORTANT: Notez le mot de passe généré (affiché une seule fois!)"
echo "  9. Notez le nom d'utilisateur (format: user-XXXXXXXXXXXX)"
echo ""

read -p "Appuyez sur Entrée quand vous avez créé l'utilisateur..."
echo ""

read -p "Username OpenStack: " OS_USERNAME
read -s -p "Password OpenStack: " OS_PASSWORD
echo ""

echo ""
echo -e "${GREEN}✅ Utilisateur OpenStack configuré${NC}"
echo ""

# ============================================================================
# Étape 4: Fichier OpenRC et Région
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 4: Téléchargement OpenRC et Région               │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

echo "Le fichier OpenRC contient les informations de connexion OpenStack."
echo ""
echo -e "${YELLOW}📋 Instructions:${NC}"
echo "  1. Dans OVH Manager > Project Management > Users & Roles"
echo "  2. Cliquez sur le bouton '...' à côté de votre utilisateur"
echo "  3. Sélectionnez 'Download OpenStack's RC file'"
echo "  4. Ouvrez le fichier téléchargé et trouvez les lignes:"
echo "     export OS_REGION_NAME=\"GRAxx\""
echo "     export OS_TENANT_NAME=\"xxxxxxxxxxxxx\""
echo "  5. Notez la région et le tenant name"
echo ""

read -p "Appuyez sur Entrée quand vous avez le fichier OpenRC..."
echo ""

echo "Régions OVH disponibles:"
echo "  - GRA5, GRA7, GRA9, GRA11 (Gravelines, France)"
echo "  - SBG5, SBG7 (Strasbourg, France)"
echo "  - BHS5 (Beauharnois, Canada)"
echo "  - WAW1, WAW2 (Warsaw, Pologne)"
echo "  - UK1 (London, UK)"
echo "  - DE1 (Frankfurt, Allemagne)"
echo ""

read -p "Région depuis le fichier OpenRC (ex: GRA9): " OS_REGION_NAME
read -p "OS_TENANT_NAME depuis le fichier OpenRC: " OS_TENANT_NAME

echo ""
echo -e "${GREEN}✅ Région configurée: $OS_REGION_NAME${NC}"
echo -e "${GREEN}✅ Tenant configuré: $OS_TENANT_NAME${NC}"
echo ""

# ============================================================================
# Étape 5: Configuration Domaine (Optionnel)
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 5: Configuration Domaine et DNS (Optionnel)      │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

echo "Voulez-vous configurer un domaine pour votre application?"
echo "Si oui, le DNS sera automatiquement configuré pour pointer vers votre VPS."
echo "Le frontend sera accessible sur: votre-domaine.com"
echo "L'API sera accessible sur: api.votre-domaine.com"
echo ""

read -p "Configurer un domaine? (y/N): " CONFIGURE_DOMAIN

KOPROGO_DOMAIN=""
KOPROGO_FRONTEND_DOMAIN=""
KOPROGO_API_DOMAIN=""
ACME_EMAIL=""
CONFIGURE_DNS_AUTO="no"

if [[ "$CONFIGURE_DOMAIN" =~ ^[Yy]$ ]]; then
    echo ""
    read -p "Nom de domaine principal (ex: koprogo.com): " KOPROGO_DOMAIN
    read -p "Email pour certificat SSL (Let's Encrypt): " ACME_EMAIL

    # Extrapoler automatiquement les sous-domaines
    KOPROGO_FRONTEND_DOMAIN="$KOPROGO_DOMAIN"
    KOPROGO_API_DOMAIN="api.$KOPROGO_DOMAIN"

    echo ""
    echo -e "${GREEN}✅ Configuration domaines:${NC}"
    echo "   Frontend: https://$KOPROGO_FRONTEND_DOMAIN"
    echo "   API:      https://$KOPROGO_API_DOMAIN"
    echo ""

    echo ""
    echo "Le domaine est-il géré chez OVH?"
    echo "(Si oui, le DNS sera configuré automatiquement via l'API OVH)"
    read -p "Domaine géré chez OVH? (y/N): " OVH_DOMAIN

    if [[ "$OVH_DOMAIN" =~ ^[Yy]$ ]]; then
        CONFIGURE_DNS_AUTO="yes"
        echo ""
        echo -e "${YELLOW}⚠️  Le DNS sera configuré automatiquement après le déploiement${NC}"
        echo "   - $KOPROGO_FRONTEND_DOMAIN → VPS IP"
        echo "   - $KOPROGO_API_DOMAIN → VPS IP"
    else
        echo ""
        echo -e "${YELLOW}⚠️  Vous devrez configurer manuellement le DNS:${NC}"
        echo "  1. Créez deux enregistrements A:"
        echo "     - $KOPROGO_FRONTEND_DOMAIN → VPS IP"
        echo "     - $KOPROGO_API_DOMAIN → VPS IP"
        echo "  2. L'IP du VPS sera affichée après le déploiement"
    fi

    echo ""
    echo -e "${GREEN}✅ Domaines configurés${NC}"
fi

echo ""

# ============================================================================
# Étape 6: Génération fichier .env
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 6: Génération du fichier de configuration        │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

echo "Génération du fichier .env avec toutes les configurations..."
echo ""

cat > "$TERRAFORM_DIR/.env" <<EOF
# OVH Terraform Credentials
# Generated by setup-infra.sh on $(date)

# ═══════════════════════════════════════════════════════════
# OVH API Credentials (for OVH provider)
# ═══════════════════════════════════════════════════════════

# OVH API Endpoint
OVH_ENDPOINT=ovh-eu

# OVH API Credentials
OVH_APPLICATION_KEY=$OVH_APPLICATION_KEY
OVH_APPLICATION_SECRET=$OVH_APPLICATION_SECRET
OVH_CONSUMER_KEY=$OVH_CONSUMER_KEY

# OVH Cloud Project ID
OVH_SERVICE_NAME=$OVH_SERVICE_NAME

# ═══════════════════════════════════════════════════════════
# OpenStack Credentials (for OpenStack provider - REQUIRED)
# ═══════════════════════════════════════════════════════════

# OpenStack Auth URL
OS_AUTH_URL=https://auth.cloud.ovh.net/v3
OS_IDENTITY_API_VERSION=3
OS_USER_DOMAIN_NAME=Default
OS_PROJECT_DOMAIN_NAME=Default

# OpenStack Project (from OpenRC file)
OS_PROJECT_ID=$OVH_SERVICE_NAME
OS_TENANT_ID=$OVH_SERVICE_NAME
OS_TENANT_NAME=$OS_TENANT_NAME

# OpenStack User Credentials
OS_USERNAME=$OS_USERNAME
OS_PASSWORD=$OS_PASSWORD

# Region (from OpenRC file)
OS_REGION_NAME=$OS_REGION_NAME

# ═══════════════════════════════════════════════════════════
# Domain Configuration (Optional)
# ═══════════════════════════════════════════════════════════

KOPROGO_DOMAIN=$KOPROGO_DOMAIN
KOPROGO_FRONTEND_DOMAIN=$KOPROGO_FRONTEND_DOMAIN
KOPROGO_API_DOMAIN=$KOPROGO_API_DOMAIN
ACME_EMAIL=$ACME_EMAIL
CONFIGURE_DNS_AUTO=$CONFIGURE_DNS_AUTO
EOF

echo -e "${GREEN}✅ Fichier .env créé: $TERRAFORM_DIR/.env${NC}"
echo ""

# Mettre à jour terraform.tfvars
echo "Mise à jour de terraform.tfvars..."

cat > "$TERRAFORM_DIR/terraform.tfvars" <<EOF
# Configuration OVH KoproGo
# Generated by setup-infra.sh on $(date)

ovh_service_name    = "$OVH_SERVICE_NAME"
instance_name       = "koprogo-production"
region              = "$OS_REGION_NAME"
ssh_public_key_path = "~/.ssh/id_rsa.pub"
EOF

echo -e "${GREEN}✅ Fichier terraform.tfvars créé${NC}"
echo ""

# ============================================================================
# Étape 7: Résumé et confirmation
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 7: Résumé de la configuration                    │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

echo "Configuration complète:"
echo ""
echo "  ${YELLOW}OVH API:${NC}"
echo "    Application Key: ${OVH_APPLICATION_KEY:0:8}***"
echo "    Consumer Key: ${OVH_CONSUMER_KEY:0:8}***"
echo "    Project ID: $OVH_SERVICE_NAME"
echo ""
echo "  ${YELLOW}OpenStack:${NC}"
echo "    Username: $OS_USERNAME"
echo "    Region: $OS_REGION_NAME"
echo ""

if [ -n "$DOMAIN" ]; then
    echo "  ${YELLOW}Domaine:${NC}"
    echo "    Domain: $DOMAIN"
    echo "    Email SSL: $ACME_EMAIL"
    echo "    DNS Auto: $CONFIGURE_DNS_AUTO"
    echo ""
fi

echo "  ${YELLOW}Infrastructure:${NC}"
echo "    Instance: koprogo-production"
echo "    Type: d2-2 (2 vCPU, 4GB RAM)"
echo "    OS: Ubuntu 22.04"
echo ""

read -p "Confirmer et lancer le déploiement? (y/N): " CONFIRM

if [[ ! "$CONFIRM" =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${YELLOW}❌ Déploiement annulé${NC}"
    echo ""
    echo "Configuration sauvegardée dans:"
    echo "  - $TERRAFORM_DIR/.env"
    echo "  - $TERRAFORM_DIR/terraform.tfvars"
    echo ""
    echo "Pour déployer plus tard:"
    echo "  cd infrastructure/terraform"
    echo "  source ./load-env.sh"
    echo "  terraform init"
    echo "  terraform apply"
    exit 0
fi

echo ""

# ============================================================================
# Étape 8: Déploiement Terraform
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 8: Déploiement Infrastructure (Terraform)        │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

cd "$TERRAFORM_DIR"

# Charger les variables d'environnement
echo "Chargement des variables d'environnement..."
set -a
source .env
set +a

# Initialiser Terraform si nécessaire
if [ ! -d ".terraform" ]; then
    echo "Initialisation Terraform..."
    terraform init
fi

echo ""
echo "Déploiement de l'infrastructure avec Terraform..."
echo ""

terraform apply -auto-approve

# Récupérer l'IP du VPS
VPS_IP=$(terraform output -raw vps_ip)

echo ""
echo -e "${GREEN}✅ Infrastructure déployée avec succès!${NC}"
echo -e "${GREEN}   VPS IP: $VPS_IP${NC}"
echo ""

# ============================================================================
# Étape 9: Configuration DNS (si demandé)
# ============================================================================

if [ "$CONFIGURE_DNS_AUTO" = "yes" ] && [ -n "$KOPROGO_DOMAIN" ]; then
    echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ Étape 9: Configuration DNS automatique                 │${NC}"
    echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
    echo ""

    echo "Configuration du DNS pour:"
    echo "  - $KOPROGO_FRONTEND_DOMAIN -> $VPS_IP"
    echo "  - $KOPROGO_API_DOMAIN -> $VPS_IP"
    echo ""

    # Copier le script DNS Python depuis ansible/files
    cp "$SCRIPT_DIR/ansible/files/configure-ovh-dns.py" /tmp/configure-dns.py
    chmod +x /tmp/configure-dns.py

    # Installer python3-ovh si nécessaire
    if ! python3 -c "import ovh" 2>/dev/null; then
        echo "Installation du module Python OVH..."
        pip3 install ovh 2>/dev/null || sudo pip3 install ovh
    fi

    # Exécuter la configuration DNS pour les deux domaines
    export TARGET_IP="$VPS_IP"
    export OVH_ENDPOINT="${OVH_ENDPOINT:-ovh-eu}"
    DNS_SUCCESS=true

    # Configuration du domaine frontend
    echo "Configuration de $KOPROGO_FRONTEND_DOMAIN..."
    export DOMAIN="$KOPROGO_FRONTEND_DOMAIN"
    if ! python3 /tmp/configure-dns.py; then
        DNS_SUCCESS=false
    fi

    echo ""

    # Configuration du domaine API
    echo "Configuration de $KOPROGO_API_DOMAIN..."
    export DOMAIN="$KOPROGO_API_DOMAIN"
    if ! python3 /tmp/configure-dns.py; then
        DNS_SUCCESS=false
    fi

    echo ""
    if [ "$DNS_SUCCESS" = "true" ]; then
        echo -e "${GREEN}✅ DNS configuré automatiquement pour les deux domaines${NC}"
        echo ""
        echo "Propagation DNS:"
        echo "  - Peut prendre 1-60 minutes"
        echo "  - Vérifier: dig $KOPROGO_FRONTEND_DOMAIN"
        echo "  - Vérifier: dig $KOPROGO_API_DOMAIN"
        echo ""
    else
        echo ""
        echo -e "${YELLOW}⚠️  Échec configuration DNS automatique${NC}"
        echo ""
        echo "Configuration manuelle requise:"
        echo "  1. Connectez-vous à votre gestionnaire DNS OVH"
        echo "  2. Créez deux enregistrements A:"
        echo "     - $KOPROGO_FRONTEND_DOMAIN -> $VPS_IP"
        echo "     - $KOPROGO_API_DOMAIN -> $VPS_IP"
        echo "  3. TTL: 60 seconds (ou minimum disponible)"
        echo ""
    fi

    rm -f /tmp/configure-dns.py
fi

# ============================================================================
# Étape 10: Configuration Ansible
# ============================================================================

echo -e "${BLUE}┌─────────────────────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│ Étape 10: Configuration et Déploiement Application     │${NC}"
echo -e "${BLUE}└─────────────────────────────────────────────────────────┘${NC}"
echo ""

cd "$ANSIBLE_DIR"

echo "Configuration de l'inventaire Ansible..."

# Créer l'inventaire avec ou sans domaine
cat > inventory.ini <<EOF
# Ansible Inventory for KoproGo
# Generated by setup-infra.sh on $(date)
# VPS IP: $VPS_IP

[koprogo]
koprogo-vps ansible_host=$VPS_IP ansible_user=ubuntu ansible_ssh_private_key_file=~/.ssh/id_rsa

[koprogo:vars]
# Domain configuration
EOF

if [ -n "$KOPROGO_DOMAIN" ]; then
    echo "domain=$KOPROGO_DOMAIN" >> inventory.ini
    echo "frontend_domain=$KOPROGO_FRONTEND_DOMAIN" >> inventory.ini
    echo "api_domain=$KOPROGO_API_DOMAIN" >> inventory.ini
    echo "acme_email=$ACME_EMAIL" >> inventory.ini
fi

echo ""
echo -e "${GREEN}✅ Inventaire Ansible créé${NC}"
echo ""

echo "Test de connexion SSH..."
if ssh -o ConnectTimeout=10 -o StrictHostKeyChecking=no ubuntu@$VPS_IP "echo 'OK'" &>/dev/null; then
    echo -e "${GREEN}✅ Connexion SSH OK${NC}"
else
    echo -e "${YELLOW}⚠️  Connexion SSH en attente...${NC}"
    echo "   Le VPS peut prendre 1-2 minutes à démarrer complètement"
    echo "   Attente de 60 secondes..."
    sleep 60

    if ssh -o ConnectTimeout=10 -o StrictHostKeyChecking=no ubuntu@$VPS_IP "echo 'OK'" &>/dev/null; then
        echo -e "${GREEN}✅ Connexion SSH OK${NC}"
    else
        echo -e "${RED}❌ Impossible de se connecter au VPS${NC}"
        echo ""
        echo "Vérifiez:"
        echo "  1. Le VPS est démarré (OVH Manager > Public Cloud > Instances)"
        echo "  2. Votre clé SSH est correcte (~/.ssh/id_rsa.pub)"
        echo "  3. Test manuel: ssh ubuntu@$VPS_IP"
        echo ""
        echo "Pour déployer plus tard:"
        echo "  cd infrastructure/ansible"
        echo "  ansible-playbook -i inventory.ini playbook.yml"
        exit 1
    fi
fi

echo ""
echo "Déploiement de l'application avec Ansible..."
echo "(Cela peut prendre 10-20 minutes)"
echo ""

# Exporter les variables pour Ansible
export KOPROGO_DOMAIN="$KOPROGO_DOMAIN"
export KOPROGO_FRONTEND_DOMAIN="$KOPROGO_FRONTEND_DOMAIN"
export KOPROGO_API_DOMAIN="$KOPROGO_API_DOMAIN"
export ACME_EMAIL="$ACME_EMAIL"

ansible-playbook -i inventory.ini playbook.yml

echo ""

# ============================================================================
# Résumé final
# ============================================================================

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                                                            ║"
echo "║              🎉 Déploiement Terminé! 🎉                    ║"
echo "║                                                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

if [ -n "$DOMAIN" ]; then
    echo -e "${GREEN}🌐 Application disponible:${NC}"
    echo "   https://$DOMAIN/api/v1"
    echo "   https://$DOMAIN/api/v1/health"
else
    echo -e "${GREEN}🌐 Application disponible:${NC}"
    echo "   http://$VPS_IP:8080/api/v1"
    echo "   http://$VPS_IP:8080/api/v1/health"
fi

echo ""
echo -e "${YELLOW}📊 Informations utiles:${NC}"
echo "   IP du VPS: $VPS_IP"
echo "   SSH: ssh ubuntu@$VPS_IP"
echo "   Logs: ssh ubuntu@$VPS_IP 'cd /home/koprogo/koprogo && docker compose logs -f'"
echo ""
echo -e "${YELLOW}🔄 Maintenance:${NC}"
echo "   Auto-update: Tous les jours à 3h (cron)"
echo "   Backups: Tous les jours à 2h (cron)"
echo "   Health checks: Toutes les 5 minutes (cron)"
echo ""
echo -e "${YELLOW}📚 Documentation:${NC}"
echo "   infrastructure/terraform/README.md"
echo "   infrastructure/ansible/README.md"
echo ""

if [ "$CONFIGURE_DNS_AUTO" = "yes" ] && [ -n "$DOMAIN" ]; then
    echo -e "${YELLOW}⏳ Note DNS:${NC}"
    echo "   La propagation DNS peut prendre 1-60 minutes"
    echo "   Vérifier: dig $DOMAIN"
    echo ""
fi

echo -e "${GREEN}✅ Setup infrastructure terminé avec succès!${NC}"
echo ""
