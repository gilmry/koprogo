Terraform - Infrastructure as Code
===================================

Terraform provisionne l'infrastructure OVH Public Cloud (VPS, réseau, SSH keys) de manière déclarative et reproductible.

**Localisation** : ``infrastructure/terraform/``

Vue d'ensemble
--------------

**Terraform** permet de définir l'infrastructure comme du code (IaC) :

- ✅ Reproductible : Même config = même infrastructure
- ✅ Versionné : Configuration dans Git
- ✅ Déclaratif : Décrire l'état désiré, Terraform fait le reste
- ✅ Idempotent : Plusieurs ``apply`` = même résultat
- ✅ Plan/Apply : Prévisualiser avant déployer

**Provider** : OVH via OpenStack API

Structure
---------

.. code-block:: text

   infrastructure/terraform/
   ├── main.tf              # Configuration principale (providers, resources)
   ├── variables.tf         # Variables configurables
   ├── load-env.sh          # Script chargement variables .env
   ├── .env.example         # Template credentials OVH
   └── .env                 # Credentials OVH (git ignored)

Prérequis
---------

**1. Compte OVH Public Cloud**

Créer projet OVH Public Cloud : https://www.ovh.com/manager/

**2. Credentials OVH API**

Générer tokens API : https://api.ovh.com/createToken/

.. code-block:: text

   Droits requis:
   - GET /cloud/project/*
   - POST /cloud/project/*
   - PUT /cloud/project/*
   - DELETE /cloud/project/*

**3. Credentials OpenStack**

Télécharger ``openrc.sh`` depuis OVH Manager → Horizon → Project → API Access → Download OpenStack RC File.

**4. Terraform CLI**

.. code-block:: bash

   # Linux
   wget https://releases.hashicorp.com/terraform/1.9.0/terraform_1.9.0_linux_amd64.zip
   unzip terraform_1.9.0_linux_amd64.zip
   sudo mv terraform /usr/local/bin/

   # macOS
   brew install terraform

   # Vérifier
   terraform --version

Configuration
-------------

Variables d'environnement
^^^^^^^^^^^^^^^^^^^^^^^^^^

**Créer .env** :

.. code-block:: bash

   cd infrastructure/terraform
   cp .env.example .env
   nano .env  # Éditer avec vos credentials

**Exemple .env** :

.. code-block:: bash

   # OVH API Credentials
   OVH_ENDPOINT=ovh-eu
   OVH_APPLICATION_KEY=your_app_key
   OVH_APPLICATION_SECRET=your_app_secret
   OVH_CONSUMER_KEY=your_consumer_key

   # OpenStack Credentials (depuis openrc.sh)
   OS_AUTH_URL=https://auth.cloud.ovh.net/v3
   OS_PROJECT_ID=your_project_id
   OS_PROJECT_NAME=your_project_name
   OS_USERNAME=your_username
   OS_PASSWORD=your_password
   OS_REGION_NAME=GRA11
   OS_IDENTITY_API_VERSION=3

**Charger variables** :

.. code-block:: bash

   # IMPORTANT: Utilisez "source" (pas "./")
   source ./load-env.sh

   # Output:
   # ✓ OVH_ENDPOINT=ovh-eu
   # ✓ OVH_APPLICATION_KEY=abcd***
   # ✓ OVH_APPLICATION_SECRET=wxyz***
   # ...

Variables Terraform
^^^^^^^^^^^^^^^^^^^

**variables.tf** : Variables configurables

.. code-block:: hcl

   variable "ovh_endpoint" {
     description = "OVH API endpoint (ovh-eu, ovh-ca, etc.)"
     type        = string
     default     = "ovh-eu"
   }

   variable "ovh_service_name" {
     description = "ID du projet OVH Cloud"
     type        = string
   }

   variable "instance_name" {
     description = "Nom de l'instance VPS"
     type        = string
     default     = "koprogo-vps"
   }

   variable "region" {
     description = "Région OVH (GRA11 = Gravelines, bas carbone)"
     type        = string
     default     = "GRA11"  # 60g CO2/kWh
   }

   variable "ssh_public_key_path" {
     description = "Chemin vers votre clé SSH publique"
     type        = string
     default     = "~/.ssh/id_rsa.pub"
   }

   variable "domain" {
     description = "Nom de domaine (optionnel)"
     type        = string
     default     = ""
   }

**Personnaliser** :

.. code-block:: bash

   # Créer fichier terraform.tfvars
   cat > terraform.tfvars <<EOF
   ovh_service_name = "your_project_id"
   instance_name = "koprogo-prod"
   region = "GRA11"
   ssh_public_key_path = "~/.ssh/koprogo.pub"
   domain = "koprogo.com"
   EOF

Providers
---------

OVH Provider
^^^^^^^^^^^^

Provider OVH pour gérer ressources OVH (domaine, DNS, etc.).

.. code-block:: hcl

   terraform {
     required_version = ">= 1.0"
     required_providers {
       ovh = {
         source  = "ovh/ovh"
         version = "~> 0.51"
       }
     }
   }

   provider "ovh" {
     endpoint = var.ovh_endpoint
     # Credentials via env: OVH_APPLICATION_KEY, etc.
   }

**Documentation** : https://registry.terraform.io/providers/ovh/ovh/

OpenStack Provider
^^^^^^^^^^^^^^^^^^

Provider OpenStack pour gérer compute (VPS).

.. code-block:: hcl

   terraform {
     required_providers {
       openstack = {
         source  = "terraform-provider-openstack/openstack"
         version = "~> 2.1"
       }
     }
   }

   provider "openstack" {
     alias  = "ovh"
     region = var.region
     # Credentials via env: OS_AUTH_URL, OS_USERNAME, etc.
   }

**Documentation** : https://registry.terraform.io/providers/terraform-provider-openstack/openstack/

Ressources
----------

SSH Key Pair
^^^^^^^^^^^^

Clé SSH pour accès VPS.

.. code-block:: hcl

   resource "openstack_compute_keypair_v2" "koprogo_key" {
     provider   = openstack.ovh
     name       = "${var.instance_name}-key"
     public_key = file(var.ssh_public_key_path)
   }

**Explication** :

- Lit votre clé SSH publique (``~/.ssh/id_rsa.pub``)
- Upload sur OVH OpenStack
- Associée au VPS au démarrage

VPS Instance
^^^^^^^^^^^^

Instance VPS d2-2 (2 vCPU, 4GB RAM, 25GB SSD).

.. code-block:: hcl

   resource "openstack_compute_instance_v2" "koprogo_vps" {
     provider = openstack.ovh
     name     = var.instance_name

     # VPS Value d2-2
     flavor_name = "d2-2"

     # Ubuntu 22.04 LTS
     image_name = "Ubuntu 22.04"

     # SSH Key
     key_pair = openstack_compute_keypair_v2.koprogo_key.name

     # Network configuration
     network {
       name = "Ext-Net"  # Réseau public
     }

     # Metadata
     metadata = {
       project     = "koprogo"
       environment = "production"
     }
   }

**Flavor d2-2 Specs** :

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Ressource
     - Valeur
   * - **vCPUs**
     - 2 vCPUs Intel Xeon
   * - **RAM**
     - 4 GB DDR4
   * - **Stockage**
     - 25 GB SSD NVMe
   * - **Réseau**
     - 100 Mbps illimité
   * - **Prix**
     - ~6.96€ TTC/mois
   * - **OS**
     - Ubuntu 22.04 LTS

**Images disponibles** :

.. code-block:: bash

   # Lister images
   openstack image list

   # Images communes:
   # - Ubuntu 22.04
   # - Ubuntu 20.04
   # - Debian 11
   # - Debian 12

**Flavors disponibles** :

.. code-block:: bash

   # Lister flavors
   openstack flavor list

   # Flavors OVH:
   # - d2-2: 2 vCPU, 4GB RAM (6.96€/mois)
   # - d2-4: 4 vCPU, 8GB RAM (13.92€/mois)
   # - d2-8: 8 vCPU, 16GB RAM (27.84€/mois)

Outputs
-------

Outputs Terraform affichent infos utiles après ``apply``.

.. code-block:: hcl

   output "vps_ip" {
     description = "IP publique du VPS KoproGo"
     value       = openstack_compute_instance_v2.koprogo_vps.access_ip_v4
   }

   output "vps_id" {
     description = "ID de l'instance VPS"
     value       = openstack_compute_instance_v2.koprogo_vps.id
   }

   output "ssh_command" {
     description = "Commande SSH pour se connecter"
     value       = "ssh ubuntu@${openstack_compute_instance_v2.koprogo_vps.access_ip_v4}"
   }

**Exemple output** :

.. code-block:: text

   Outputs:

   ssh_command = "ssh ubuntu@51.178.12.34"
   vps_id = "e23f9a8b-1234-5678-90ab-cdef12345678"
   vps_ip = "51.178.12.34"

Commandes Terraform
-------------------

Workflow Standard
^^^^^^^^^^^^^^^^^

.. code-block:: bash

   cd infrastructure/terraform

   # 1. Charger variables d'environnement
   source ./load-env.sh

   # 2. Initialiser (télécharge providers)
   terraform init

   # 3. Valider configuration
   terraform validate

   # 4. Formater code
   terraform fmt

   # 5. Prévisualiser changements
   terraform plan

   # 6. Appliquer (créer infrastructure)
   terraform apply

   # 7. Afficher outputs
   terraform output

   # 8. Détruire infrastructure (⚠️ DESTRUCTIF)
   terraform destroy

Init
^^^^

Initialise Terraform (première fois).

.. code-block:: bash

   terraform init

**Ce que fait init** :

- Télécharge providers (OVH, OpenStack)
- Crée fichier ``.terraform.lock.hcl`` (lock versions)
- Crée répertoire ``.terraform/`` (cache providers)

Plan
^^^^

Prévisualise les changements sans les appliquer.

.. code-block:: bash

   terraform plan

**Output exemple** :

.. code-block:: text

   Terraform will perform the following actions:

     # openstack_compute_keypair_v2.koprogo_key will be created
     + resource "openstack_compute_keypair_v2" "koprogo_key" {
         + name       = "koprogo-vps-key"
         + public_key = (sensitive value)
       }

     # openstack_compute_instance_v2.koprogo_vps will be created
     + resource "openstack_compute_instance_v2" "koprogo_vps" {
         + name        = "koprogo-vps"
         + flavor_name = "d2-2"
         + image_name  = "Ubuntu 22.04"
       }

   Plan: 2 to add, 0 to change, 0 to destroy.

Apply
^^^^^

Applique les changements (crée infrastructure).

.. code-block:: bash

   terraform apply

   # Ou sans confirmation:
   terraform apply -auto-approve

**Durée** : ~2-3 minutes pour créer VPS.

**Output** :

.. code-block:: text

   Apply complete! Resources: 2 added, 0 changed, 0 destroyed.

   Outputs:

   ssh_command = "ssh ubuntu@51.178.12.34"
   vps_ip = "51.178.12.34"

Destroy
^^^^^^^

Détruit toute l'infrastructure (⚠️ DESTRUCTIF).

.. code-block:: bash

   terraform destroy

   # Ou sans confirmation:
   terraform destroy -auto-approve

**⚠️ Attention** : Supprime VPS et TOUTES les données !

State
^^^^^

Terraform stocke l'état de l'infrastructure dans ``terraform.tfstate``.

.. code-block:: bash

   # Lister ressources
   terraform state list

   # Voir détails ressource
   terraform state show openstack_compute_instance_v2.koprogo_vps

   # Rafraîchir state
   terraform refresh

**⚠️ IMPORTANT** : ``terraform.tfstate`` contient secrets (IPs, IDs). **NE PAS commiter dans Git**.

Gestion State
-------------

State Local
^^^^^^^^^^^

Par défaut, state stocké localement (``terraform.tfstate``).

**Avantages** :

- ✅ Simple (pas de setup)
- ✅ Rapide

**Inconvénients** :

- ❌ Pas de partage (équipe)
- ❌ Risque perte (si supprimé)
- ❌ Pas de locking (conflits concurrent)

**Git ignore** :

.. code-block:: bash

   # infrastructure/terraform/.gitignore
   .terraform/
   *.tfstate
   *.tfstate.backup
   .env

State Remote (Recommandé)
^^^^^^^^^^^^^^^^^^^^^^^^^^

State stocké sur backend distant (S3, Terraform Cloud, etc.).

**Exemple S3** :

.. code-block:: hcl

   terraform {
     backend "s3" {
       bucket         = "koprogo-terraform-state"
       key            = "production/terraform.tfstate"
       region         = "eu-west-3"
       encrypt        = true
       dynamodb_table = "koprogo-terraform-locks"
     }
   }

**Avantages** :

- ✅ Partagé (équipe)
- ✅ Versioning (rollback possible)
- ✅ Locking (évite conflits)
- ✅ Backup automatique

Sécurité
--------

Credentials
^^^^^^^^^^^

**NE JAMAIS commiter credentials** :

.. code-block:: bash

   # infrastructure/terraform/.gitignore
   .env
   *.tfvars  # Sauf si valeurs publiques
   *.tfstate

**Best practices** :

1. ✅ Variables d'environnement (``.env``)
2. ✅ Secrets manager (HashiCorp Vault)
3. ✅ CI/CD secrets (GitHub Secrets)
4. ❌ Hard-coded dans ``*.tf``

State File
^^^^^^^^^^

``terraform.tfstate`` contient données sensibles :

- IP publique VPS
- IDs ressources
- Metadata

**Protection** :

.. code-block:: bash

   # Chiffrer state si backend S3
   backend "s3" {
     encrypt = true
   }

Permissions
^^^^^^^^^^^

**Principe Least Privilege** : Donner uniquement droits nécessaires.

**OVH API tokens** :

.. code-block:: text

   # Droits minimaux:
   GET /cloud/project/{projectId}/*
   POST /cloud/project/{projectId}/instance
   PUT /cloud/project/{projectId}/instance/*
   DELETE /cloud/project/{projectId}/instance/*

Troubleshooting
---------------

Erreur Authentication
^^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   Error: authentication failed: Unable to authenticate

**Solution** :

.. code-block:: bash

   # Vérifier variables chargées
   echo $OVH_APPLICATION_KEY
   echo $OS_AUTH_URL

   # Recharger .env
   source ./load-env.sh

   # Vérifier credentials OVH API
   curl -X GET "https://eu.api.ovh.com/1.0/auth/currentCredential" \
     -H "X-Ovh-Application: $OVH_APPLICATION_KEY" \
     -H "X-Ovh-Consumer: $OVH_CONSUMER_KEY"

Erreur Quota Dépassé
^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   Error: Error creating instance: Quota exceeded for instances

**Solution** :

.. code-block:: bash

   # Vérifier quotas
   openstack quota show

   # Demander augmentation quotas via OVH Support

State Lock
^^^^^^^^^^

.. code-block:: text

   Error: Error locking state

**Solution** :

.. code-block:: bash

   # Forcer unlock (si sûr qu'aucun autre apply)
   terraform force-unlock <lock_id>

Provider Version Conflict
^^^^^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: text

   Error: Inconsistent dependency lock file

**Solution** :

.. code-block:: bash

   # Mettre à jour lock file
   terraform init -upgrade

CI/CD Terraform
---------------

GitHub Actions Example
^^^^^^^^^^^^^^^^^^^^^^

.. code-block:: yaml

   # .github/workflows/terraform.yml
   name: Terraform Apply

   on:
     push:
       branches: [main]
       paths:
         - 'infrastructure/terraform/**'

   jobs:
     terraform:
       runs-on: ubuntu-latest
       defaults:
         run:
           working-directory: infrastructure/terraform

       env:
         OVH_ENDPOINT: ${{ secrets.OVH_ENDPOINT }}
         OVH_APPLICATION_KEY: ${{ secrets.OVH_APPLICATION_KEY }}
         OVH_APPLICATION_SECRET: ${{ secrets.OVH_APPLICATION_SECRET }}
         OVH_CONSUMER_KEY: ${{ secrets.OVH_CONSUMER_KEY }}
         OS_AUTH_URL: ${{ secrets.OS_AUTH_URL }}
         OS_PROJECT_ID: ${{ secrets.OS_PROJECT_ID }}
         OS_USERNAME: ${{ secrets.OS_USERNAME }}
         OS_PASSWORD: ${{ secrets.OS_PASSWORD }}
         OS_REGION_NAME: ${{ secrets.OS_REGION_NAME }}

       steps:
         - uses: actions/checkout@v3

         - name: Setup Terraform
           uses: hashicorp/setup-terraform@v2
           with:
             terraform_version: 1.9.0

         - name: Terraform Init
           run: terraform init

         - name: Terraform Validate
           run: terraform validate

         - name: Terraform Plan
           run: terraform plan

         - name: Terraform Apply
           if: github.ref == 'refs/heads/main'
           run: terraform apply -auto-approve

**Secrets GitHub** :

.. code-block:: bash

   # Ajouter secrets: Settings → Secrets → Actions
   OVH_ENDPOINT
   OVH_APPLICATION_KEY
   OVH_APPLICATION_SECRET
   OVH_CONSUMER_KEY
   OS_AUTH_URL
   OS_PROJECT_ID
   OS_USERNAME
   OS_PASSWORD
   OS_REGION_NAME

Mise à jour Infrastructure
---------------------------

Modifier Ressource
^^^^^^^^^^^^^^^^^^

.. code-block:: bash

   # Éditer main.tf
   nano main.tf

   # Changer flavor (scaling vertical)
   flavor_name = "d2-4"  # 4 vCPU, 8GB RAM

   # Prévisualiser
   terraform plan

   # Appliquer
   terraform apply

⚠️ **Attention** : Certains changements détruisent VPS (rebuild). Terraform affiche ``forces replacement``.

Scaling Vertical
^^^^^^^^^^^^^^^^

**Augmenter VPS** :

.. code-block:: hcl

   # main.tf
   resource "openstack_compute_instance_v2" "koprogo_vps" {
     # Passer de d2-2 à d2-4
     flavor_name = "d2-4"  # 4 vCPU, 8GB RAM (~14€/mois)
   }

.. code-block:: bash

   terraform apply

**Downtime** : ~2-5 minutes (rebuild VPS).

Import Ressource Existante
^^^^^^^^^^^^^^^^^^^^^^^^^^^

Si VPS créé manuellement, l'importer dans Terraform :

.. code-block:: bash

   # Obtenir ID instance
   openstack server list

   # Importer dans state
   terraform import openstack_compute_instance_v2.koprogo_vps <instance_id>

Coûts
-----

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Ressource
     - Coût Mensuel
     - Coût Annuel
   * - **VPS d2-2**
     - 6.96€ TTC
     - 83.52€
   * - **IP Publique**
     - Inclus
     - Inclus
   * - **Bande Passante**
     - Illimité
     - Illimité
   * - **TOTAL**
     - **6.96€ TTC**
     - **~84€**

**Évolution** :

- VPS d2-4 (4 vCPU, 8GB) : ~14€/mois
- VPS d2-8 (8 vCPU, 16GB) : ~28€/mois

Références
----------

- Terraform OVH Provider : https://registry.terraform.io/providers/ovh/ovh/
- Terraform OpenStack Provider : https://registry.terraform.io/providers/terraform-provider-openstack/openstack/
- Terraform Docs : https://developer.hashicorp.com/terraform/docs
- OVH Public Cloud : https://www.ovhcloud.com/fr/public-cloud/
- OVH API : https://api.ovh.com/
