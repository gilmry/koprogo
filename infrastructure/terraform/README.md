# Déploiement Terraform OVH - KoproGo

Configuration Terraform pour déployer KoproGo sur un VPS OVH Cloud.

## Architecture déployée

- **VPS**: OVH Cloud Instance d2-2 (1 vCPU, 2GB RAM, 40GB NVMe SSD)
- **OS**: Ubuntu 22.04 LTS
- **Région**: GRA11 (Gravelines, France) - datacenter bas carbone (60g CO2/kWh)
- **Prix**: ~7€ TTC/mois
- **Providers**: OVH + OpenStack (OVH Cloud utilise OpenStack)

## Prérequis

1. **Compte OVH Cloud**
   - Créer un compte sur https://www.ovh.com
   - Créer un projet Public Cloud
   - Noter l'ID du projet (service_name)

2. **Clé SSH**
   ```bash
   # Si vous n'avez pas de clé SSH, créez-en une:
   ssh-keygen -t rsa -b 4096 -C "votre-email@example.com"
   ```

3. **Terraform installé**
   ```bash
   # Vérifier l'installation
   terraform version

   # Si non installé: https://www.terraform.io/downloads
   ```

## Note technique importante

Cette configuration utilise le **provider OpenStack** pour créer les instances, car OVH Cloud est basé sur OpenStack. Le provider OVH (`ovh/ovh`) est utilisé pour d'autres ressources OVH si nécessaire, mais les instances compute sont créées via `openstack_compute_instance_v2`.

**Pourquoi?** Le provider OVH v0.51+ ne supporte plus `ovh_cloud_project_instance`. OVH recommande d'utiliser directement le provider OpenStack pour la gestion des instances compute.

## Configuration OVH

### 1. Créer les credentials API OVH

Visitez: https://www.ovh.com/auth/api/createToken?GET=/*&POST=/*&PUT=/*&DELETE=/*

- **Application name**: `koprogo-terraform`
- **Application description**: `Terraform deployment for KoproGo`
- **Validity**: `Unlimited` (recommandé) ou durée limitée pour plus de sécurité

Notez les credentials générés:
- `Application Key`
- `Application Secret`
- `Consumer Key`

### 2. Configuration des variables d'environnement

**Méthode recommandée**: Utiliser un fichier `.env` (ignoré par git):

```bash
cd infrastructure/terraform

# Créer votre fichier .env depuis le template
cp .env.example .env
nano .env  # Éditer et remplir vos credentials

# Charger les variables
source ./load-env.sh
```

📖 **Guide complet**: Voir [ENV_SETUP.md](./ENV_SETUP.md) pour tous les détails et méthodes alternatives.

**Alternative** - Script interactif (méthode originale):

```bash
source ./setup-ovh.sh
```

Le script vous demandera vos credentials et créera automatiquement:
- Les variables d'environnement OVH
- Le fichier `terraform.tfvars`

### 3. Configuration manuelle (alternative)

**Variables d'environnement:**

```bash
export OVH_ENDPOINT="ovh-eu"
export OVH_APPLICATION_KEY="votre_application_key"
export OVH_APPLICATION_SECRET="votre_application_secret"
export OVH_CONSUMER_KEY="votre_consumer_key"
```

**Fichier terraform.tfvars:**

```bash
cp terraform.tfvars.example terraform.tfvars
# Éditer terraform.tfvars avec vos valeurs
```

## Test de la configuration (sans déployer)

Script de test complet qui valide tout sans déployer:

```bash
./test-config.sh
```

Ce script vérifie:
1. ✓ Installation de Terraform
2. ✓ Présence de la clé SSH
3. ✓ Variables d'environnement OVH
4. ✓ Fichier terraform.tfvars
5. ✓ Validation de la configuration
6. ✓ Formatage du code
7. ✓ Génération du plan (dry-run)

## Déploiement

### Étape par étape

```bash
cd infrastructure/terraform

# 1. Initialiser Terraform (télécharge le provider OVH)
terraform init

# 2. Valider la configuration
terraform validate

# 3. Prévisualiser les changements (DRY RUN - ne déploie rien)
terraform plan

# 4. Appliquer (DÉPLOIEMENT RÉEL)
terraform apply
# Tapez "yes" pour confirmer

# 5. Voir les outputs (IP du VPS, commande SSH)
terraform output
```

### Commande rapide

```bash
# Tout en une commande (avec auto-approbation - ATTENTION!)
terraform apply -auto-approve
```

⚠️  **Attention**: `-auto-approve` déploie sans demander confirmation. À utiliser uniquement si vous êtes sûr!

## Accès au VPS déployé

Après le déploiement, Terraform affiche:

```
Outputs:

ssh_command = "ssh ubuntu@X.X.X.X"
vps_id = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
vps_ip = "X.X.X.X"
```

**Connexion SSH:**

```bash
# Utiliser la commande affichée
ssh ubuntu@X.X.X.X

# Ou récupérer l'IP
terraform output -raw vps_ip
ssh ubuntu@$(terraform output -raw vps_ip)
```

## Gestion de l'infrastructure

### Voir l'état actuel

```bash
terraform show
terraform output
```

### Modifier la configuration

1. Éditer les fichiers `.tf` ou `terraform.tfvars`
2. Prévisualiser: `terraform plan`
3. Appliquer: `terraform apply`

### Détruire l'infrastructure

```bash
# Supprimer toutes les ressources créées
terraform destroy

# Avec auto-confirmation (ATTENTION!)
terraform destroy -auto-approve
```

## Variables configurables

| Variable | Description | Défaut | Obligatoire |
|----------|-------------|--------|-------------|
| `ovh_service_name` | ID du projet OVH Cloud | - | ✓ |
| `instance_name` | Nom de l'instance VPS | `koprogo-vps` | ✗ |
| `region` | Région OVH | `GRA11` | ✗ |
| `ssh_public_key_path` | Chemin de la clé SSH publique | `~/.ssh/id_rsa.pub` | ✗ |
| `domain` | Nom de domaine (optionnel) | `""` | ✗ |

### Régions disponibles

- **GRA11** (Gravelines, France) - Datacenter bas carbone ⚡ 60g CO2/kWh
- **SBG5** (Strasbourg, France)
- **WAW1** (Varsovie, Pologne)
- **UK1** (Londres, UK)
- **DE1** (Francfort, Allemagne)

## Sécurité

### Bonnes pratiques

1. **Ne committez JAMAIS** `terraform.tfvars` contenant des credentials
2. **Utilisez des variables d'environnement** pour les secrets
3. **Limitez la validité** des tokens API OVH
4. **Rotez régulièrement** les credentials
5. **Utilisez Terraform Cloud** ou un backend S3 pour le state en production

### Fichier .gitignore

Déjà configuré dans le projet:

```gitignore
# Terraform
*.tfstate
*.tfstate.backup
.terraform/
terraform.tfvars
```

## Coûts estimés

- **VPS d2-2**: ~7€/mois TTC
- **Trafic réseau**: Illimité (inclus)
- **IP publique**: Incluse
- **Stockage**: 40GB NVMe inclus

**Total**: ~7€/mois pour l'infrastructure de base

## Troubleshooting

### Erreur: "Invalid credentials"

```bash
# Vérifier les variables d'environnement
echo $OVH_APPLICATION_KEY
echo $OVH_ENDPOINT

# Re-sourcer le script de config
source ./setup-ovh.sh
```

### Erreur: "SSH key not found"

```bash
# Vérifier la présence de la clé
ls -l ~/.ssh/id_rsa.pub

# Créer une clé si nécessaire
ssh-keygen -t rsa -b 4096
```

### Erreur: "Region not available"

Vérifier les régions disponibles sur votre projet OVH Cloud.
Modifier `region` dans `terraform.tfvars`.

### Terraform state locked

```bash
# Forcer le unlock (si vous êtes sûr qu'aucun autre process ne tourne)
terraform force-unlock <LOCK_ID>
```

## Prochaines étapes

Après le déploiement, vous devez:

1. **Configurer le VPS**:
   ```bash
   ssh ubuntu@$(terraform output -raw vps_ip)

   # Mettre à jour le système
   sudo apt update && sudo apt upgrade -y

   # Installer Docker
   curl -fsSL https://get.docker.com | sh
   sudo usermod -aG docker ubuntu
   ```

2. **Déployer KoproGo**:
   - Cloner le repository
   - Configurer les variables d'environnement
   - Lancer Docker Compose

3. **Configurer le DNS** (si vous avez un domaine):
   - Pointer votre domaine vers l'IP du VPS
   - Configurer le SSL avec Let's Encrypt

## Ressources

- [Documentation OVH Cloud](https://docs.ovh.com/fr/public-cloud/)
- [Terraform OVH Provider](https://registry.terraform.io/providers/ovh/ovh/latest/docs)
- [API OVH](https://api.ovh.com/)
- [KoproGo Documentation](../../README.md)

## Support

Pour toute question:
- Issues GitHub: https://github.com/yourusername/koprogo/issues
- Documentation OVH: https://help.ovh.com/
