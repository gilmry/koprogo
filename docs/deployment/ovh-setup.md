# Configuration OVH

Guide détaillé pour configurer votre compte OVH Public Cloud avant le déploiement.

---

## 📋 Prérequis

1. **Compte OVH** : https://www.ovh.com/manager/public-cloud/
2. **Projet Public Cloud créé** (gratuit)
3. **Carte bancaire** (pour validation compte, pas de débit si vous restez en mode test)

---

## Étape 1 : Créer un Utilisateur OpenStack (REQUIS)

L'utilisateur OpenStack est **obligatoire** pour que Terraform puisse provisionner le VPS.

### 1.1 Accéder à la gestion des utilisateurs

1. **OVH Manager** → **Public Cloud**
2. Sélectionner votre **Projet**
3. Menu gauche → **Project Management** → **Users & Roles**

### 1.2 Créer l'utilisateur

1. Cliquer sur **Créer un utilisateur OpenStack**
2. **Nom** : `terraform-koprogo` (ou autre nom descriptif)

### 1.3 Sélectionner TOUS les rôles (CRITIQUE !)

**⚠️ IMPORTANT** : Cocher **TOUS** les rôles suivants, sinon Terraform échouera :

- ☑ **Administrator** (CRITIQUE - requis pour créer ressources)
- ☑ Compute Operator
- ☑ Network Operator
- ☑ Network Security Operator
- ☑ Image Operator
- ☑ Volume Operator
- ☑ ObjectStore Operator
- ☑ LoadBalancer Operator
- ☑ Backup Operator
- ☑ Infrastructure Supervisor
- ☑ KeyManager Operator
- ☑ KeyManager Read

### 1.4 Noter les credentials

Après création, OVH affiche :
- `OS_USERNAME` (format: `user-XXXXXXXXXXXX`)
- `OS_PASSWORD` (généré automatiquement, **à copier immédiatement** !)

**⚠️ Le mot de passe n'est affiché qu'une seule fois !** Notez-le dans un gestionnaire de mots de passe.

---

## Étape 2 : Télécharger le Fichier OpenRC (REQUIS)

Le fichier OpenRC contient la **région exacte** de votre projet OVH. C'est la source de vérité.

### 2.1 Télécharger le fichier

1. **OVH Manager** → **Public Cloud** → **Users & Roles**
2. Cliquer sur **...** (trois points) à côté de votre utilisateur
3. Sélectionner **Download OpenStack's RC file**
4. **Région** : Choisir **GRA9** (Gravelines, France - recommandé pour écologie)
5. Télécharger le fichier

### 2.2 Identifier la région

**Ouvrir le fichier téléchargé** et trouver la ligne :

```bash
export OS_REGION_NAME="GRA9"
```

**⚠️ CRITIQUE** : Utilisez **EXACTEMENT** cette région dans le script de déploiement. Ne PAS deviner !

### Régions disponibles

| Code | Datacenter | Localisation | CO₂/kWh |
|------|------------|--------------|---------|
| **GRA9** | Gravelines 9 | France | **60g** ⭐ |
| GRA11 | Gravelines 11 | France | **60g** ⭐ |
| SBG5 | Strasbourg 5 | France | **60g** ⭐ |
| BHS5 | Beauharnois 5 | Canada | 60g |
| DE1 | Francfort | Allemagne | 400g |
| UK1 | Londres | Royaume-Uni | 250g |
| WAW1 | Varsovie | Pologne | 700g |

**Recommandation ASBL** : GRA9 (France) pour écologie + GDPR + souveraineté.

---

## Étape 3 : Créer Credentials API OVH (OPTIONNEL)

**Requis seulement si** : Configuration DNS automatique souhaitée.

**Pas requis si** : Vous configurez le DNS manuellement ou utilisez une IP directe.

### 3.1 Créer le token API

1. Aller sur : https://www.ovh.com/auth/api/createToken
2. **Se connecter** avec votre compte OVH

### 3.2 Remplir le formulaire

**Application name** : `KoproGo Infrastructure`

**Application description** : `Terraform + Ansible deployment`

**Validity** : `Unlimited`

**Rights** :
- `GET /domain/*`
- `POST /domain/*`
- `PUT /domain/*`
- `DELETE /domain/*`

### 3.3 Générer et noter les credentials

Après validation, OVH affiche :
- `OVH_APPLICATION_KEY` (format: `XXXXXXXXXXXX`)
- `OVH_APPLICATION_SECRET` (format: `XXXXXXXXXXXX`)
- `OVH_CONSUMER_KEY` (format: `XXXXXXXXXXXX`)

**⚠️ Noter ces 3 clés**, elles seront demandées par `make setup-infra`.

---

## Étape 4 : Récapitulatif des Credentials

Avant de lancer `make setup-infra`, vous devez avoir :

### REQUIS (Terraform)
- ✅ `OS_USERNAME` (utilisateur OpenStack)
- ✅ `OS_PASSWORD` (mot de passe OpenStack)
- ✅ `OS_REGION_NAME` (depuis fichier OpenRC)
- ✅ `OS_PROJECT_ID` (ID du projet OVH)

### OPTIONNEL (DNS automatique)
- ⚪ `OVH_APPLICATION_KEY`
- ⚪ `OVH_APPLICATION_SECRET`
- ⚪ `OVH_CONSUMER_KEY`
- ⚪ Nom de domaine (ex: `koprogo.com`)

---

## Étape 5 : Lancer le Déploiement

Une fois tous les credentials obtenus :

```bash
# Depuis la racine du projet KoproGo
make setup-infra
```

Le script détecte automatiquement si vous avez déjà une configuration et propose de la réutiliser.

---

## 🔐 Sécurité des Credentials

### Où sont stockés les credentials ?

Les credentials sont stockés dans : `infrastructure/terraform/.env`

**⚠️ Ce fichier est dans .gitignore** - Ne JAMAIS le committer !

### Réutilisation de configuration

Si vous redéployez, le script `make setup-infra` détecte le fichier `.env` existant :

```
⚠️  Configuration existante détectée: infrastructure/terraform/.env

Voulez-vous:
  1) Utiliser la configuration existante (recommandé)
  2) Reconfigurer depuis le début

Votre choix (1/2):
```

Choisir **1** vous évite de re-saisir tous les credentials.

---

## 🆘 Problèmes Courants

### "Insufficient permissions" (Terraform)

**Cause** : Utilisateur OpenStack sans le rôle **Administrator**

**Solution** :
1. OVH Manager → Public Cloud → Users & Roles
2. **Supprimer** l'utilisateur actuel
3. **Créer un nouvel utilisateur** avec **TOUS** les rôles (surtout Administrator)

### "No suitable endpoint could be found"

**Cause** : Région incorrecte ou non compatible

**Solution** :
1. **Télécharger à nouveau** le fichier OpenRC depuis OVH Manager
2. Vérifier la ligne : `export OS_REGION_NAME="GRA9"`
3. Utiliser **EXACTEMENT** cette région dans `make setup-infra`

### "Application key invalid" (DNS)

**Cause** : Credentials API OVH incorrects ou expirés

**Solution** :
1. Régénérer les credentials : https://www.ovh.com/auth/api/createToken
2. Vérifier que les 3 clés (APPLICATION_KEY, APPLICATION_SECRET, CONSUMER_KEY) sont correctes
3. Relancer `make setup-infra` et choisir "2) Reconfigurer"

---

## 📚 Prochaine Étape

Une fois la configuration OVH terminée, continuer vers : **[Terraform + Ansible](terraform-ansible.md)**

---

## 🔗 Ressources

- **OVH Manager** : https://www.ovh.com/manager/public-cloud/
- **API OVH** : https://www.ovh.com/auth/api/createToken
- **Documentation OVH** : https://help.ovhcloud.com/csm/en-public-cloud-compute-getting-started

---

**Dernière mise à jour** : Octobre 2025

**KoproGo ASBL** 🚀
