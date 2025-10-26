#!/bin/bash

# Script pour charger les variables d'environnement depuis .env
# Usage: source ./load-env.sh
# ⚠️  IMPORTANT: Utilisez "source" et non "./" pour que les variables soient exportées

# Vérifier si le script est sourcé
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "❌ Erreur: Ce script doit être sourcé, pas exécuté directement!"
    echo ""
    echo "Utilisation correcte:"
    echo "  source ./load-env.sh"
    echo "  # ou"
    echo "  . ./load-env.sh"
    echo ""
    echo "Puis vous pouvez utiliser:"
    echo "  terraform plan"
    echo "  terraform apply"
    echo "  terraform destroy"
    echo ""
    exit 1
fi

set -a  # Export automatiquement toutes les variables

ENV_FILE=".env"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== Chargement des variables d'environnement OVH ==="
echo ""

# Vérifier si .env existe
if [ ! -f "$SCRIPT_DIR/$ENV_FILE" ]; then
    echo "❌ Fichier $ENV_FILE non trouvé!"
    echo ""
    echo "Pour créer votre fichier .env:"
    echo "  1. Copiez le template: cp .env.example .env"
    echo "  2. Éditez .env et remplissez vos credentials OVH"
    echo "  3. Relancez: source ./load-env.sh"
    echo ""
    return 1 2>/dev/null || exit 1
fi

# Charger les variables depuis .env (ignore les commentaires et lignes vides)
while IFS='=' read -r key value || [ -n "$key" ]; do
    # Ignorer les commentaires et lignes vides
    if [[ ! "$key" =~ ^[[:space:]]*# ]] && [[ -n "$key" ]]; then
        # Supprimer les espaces autour de la clé
        key=$(echo "$key" | xargs)
        # Supprimer les espaces et guillemets autour de la valeur
        value=$(echo "$value" | xargs | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")

        # Exporter la variable
        if [[ -n "$value" ]]; then
            export "$key=$value"

            # Afficher les 4 premiers caractères seulement pour la sécurité
            if [[ "$key" =~ KEY|SECRET|PASSWORD ]]; then
                echo "✓ $key=${value:0:4}***"
            else
                echo "✓ $key=$value"
            fi
        fi
    fi
done < "$SCRIPT_DIR/$ENV_FILE"

set +a  # Désactiver l'export automatique

echo ""
echo "=== Variables chargées avec succès! ==="
echo ""

# Vérifier que les variables essentielles sont définies
MISSING=0
for var in OVH_ENDPOINT OVH_APPLICATION_KEY OVH_APPLICATION_SECRET OVH_CONSUMER_KEY; do
    if [ -z "${!var}" ]; then
        echo "⚠️  Variable manquante: $var"
        MISSING=1
    fi
done

if [ $MISSING -eq 1 ]; then
    echo ""
    echo "⚠️  Certaines variables ne sont pas définies dans $ENV_FILE"
    echo "Éditez le fichier et remplissez toutes les valeurs requises."
    return 1 2>/dev/null || exit 1
fi

echo "Prochaines étapes:"
echo "  terraform init      # Initialiser (si pas déjà fait)"
echo "  terraform validate  # Valider la configuration"
echo "  terraform plan      # Prévisualiser les changements"
echo "  terraform apply     # Déployer"
echo ""
