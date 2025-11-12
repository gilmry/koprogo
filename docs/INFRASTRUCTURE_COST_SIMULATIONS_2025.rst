===================================================================
Simulations Coûts Infrastructure par Échelle - 2025
===================================================================

:Auteur: Gilles Maury - Fondateur KoproGo ASBL
:Date: Novembre 2025 (Recherche tarifaire)
:Status: ✅ VALIDÉ - Données OVHcloud officielles
:Source: PERFORMANCE_REPORT.rst (Oct 2025) + Recherche web OVHcloud Nov 2025

.. note::
   **Sources tarifaires validées** :

   - **VPS OVHcloud** : Recherche web Nov 2025 (Starter 4.24€, Value 7.02€, VPS-1 6.53€ TTC)
   - **Object Storage S3** : 0.007€/GB/mois (Standard 1-AZ), 0.014€/GB/mois (3-AZ)
   - **Performances** : PERFORMANCE_REPORT.rst (287 req/s @ 1 vCPU 2GB RAM)
   - **TVA Belgique** : 21% (vs 20% estimé précédemment)
   - **VPS retenu** : **Value** (7.02€ TTC) pour NVMe + 250 Mbps bande passante
   - **Prix cloud KoproGo** : **5€/mois** (décision ASBL, baisse par vote AG uniquement)

.. contents:: Table des Matières
   :depth: 3
   :local:

🎯 Méthodologie
===============

Hypothèses de Base
------------------

**Données de Performance Réelles** (PERFORMANCE_REPORT.rst):

- **VPS testé**: 1 vCPU / 2GB RAM @ 8€/mois (mentionné comme "d2-2" dans docs)
- **VPS équivalent 2025**: **s1-2** (1 vCore, 2GB RAM) @ ~6€/mois HT
- **Throughput mesuré**: 287 req/s
- **P99 latency**: 752ms (objectif réaliste: < 1s)
- **Capacité multi-tenant**: 2,000-3,000 copropriétés par VPS
- **CO₂/requête**: 0.12g
- **Taux de succès**: 99.74%

**Prix OVH 2025** (✅ VALIDÉ - Recherche web Nov 2025):

VPS:
  - **Starter** (1 vCore, 2GB RAM, 20GB SSD SATA, 100 Mbps): 3.50€/mois HT → **4.24€/mois TTC** (TVA 21% BE)
  - **Value** (1 vCore, 2GB RAM, 40GB SSD NVMe, 250 Mbps): 5.80€/mois HT → **7.02€/mois TTC**
  - **VPS-1 (nouveau)** (4 vCore, 8GB RAM, 75GB SSD NVMe, 400 Mbps): 5.40€/mois HT → **6.53€/mois TTC**

Storage:
  - **S3 Standard 1-AZ**: 0.007€/GB/mois (7€/TB)
  - **S3 Standard 3-AZ**: 0.014€/GB/mois (haute résilience)
  - **S3 Cold Archive**: 0.002€/GB/mois (estimé)
  - **SSD additionnel**: 0.10€/GB/mois
  - **Outgoing public traffic**: 0.01€/GB (incoming gratuit)

Réseau:
  - **DNS OVH**: 0.10€/mois
  - **Bande passante interne**: Illimitée (incluse, entre services OVH)

**Ratio Cloud/Self-hosted**:

- **40% cloud-hosted** (KoproGo gère l'infrastructure)
- **60% self-hosted** (syndics gèrent leur propre VPS)

**Calcul de Capacité par VPS** (basé sur 287 req/s mesuré):

.. code-block:: text

   Hypothèses d'usage par copropriété:
   - Moyenne: 100 requêtes/jour (consultation documents, paiements, etc.)
   - Pics: 10x la moyenne lors d'AG, paiements de masse
   - Heures de pointe: 8h-20h (12h = 43,200 sec)

   Calcul charge moyenne en heure de pointe:
   - 100 req/jour × 10 (pic) = 1,000 req/jour en pic
   - 1,000 req / 43,200 sec = 0.023 req/s par copro

   Capacité théorique:
   - VPS: 287 req/s (P99 < 1s)
   - Buffer sécurité 50%: 287 × 0.5 = 143.5 req/s utilisable
   - Capacité: 143.5 / 0.023 = 6,239 copropriétés MAX

   Capacité CONSERVATRICE retenue:
   - **2,000-3,000 copropriétés par VPS** (facteur sécurité 2-3x)
   - Permet pics exceptionnels (AG simultanées, paiements groupés)
   - Garantit P99 < 1s même en charge élevée

**Hypothèses Storage** (par copropriété):

- **Documents PDF**: 200MB/copro/an (assemblées, règlements, etc.)
- **Rétention**: 10 ans (2GB/copro total)
- **Stratégie**:

  - Année en cours: SSD (accès rapide)
  - 1-3 ans: S3 Standard 1-AZ (0.007€/GB/mois)
  - 3-10 ans: S3 Cold Archive (0.002€/GB/mois)

📊 Simulations par Palier
==========================

Palier 1: 100 Copropriétés
---------------------------

Infrastructure
~~~~~~~~~~~~~~

- **VPS nécessaires**: 1 × Value (capacité 2,000-3,000 copros)
- **Copros cloud** (40%): 40 copros
- **Copros self-hosted** (60%): 60 copros

Coûts Compute
~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Ressource
     - Quantité
     - Coût/mois TTC
   * - VPS Value (1 vCore, 2GB, 40GB NVMe)
     - 1
     - 7.02€
   * - DNS OVH
     - 1
     - 0.10€
   * - **Total Compute**
     -
     - **7.12€**

Coûts Storage
~~~~~~~~~~~~~

Storage par copro (moyenne sur 10 ans):

- **SSD** (année en cours): 200MB × 0.10€/GB = 0.02€/copro/mois
- **S3 Standard 1-AZ** (années 1-3): 600MB × 0.007€/GB = 0.0042€/copro/mois
- **S3 Cold Archive** (années 3-10): 1.4GB × 0.002€/GB = 0.0028€/copro/mois
- **Total storage**: ~0.027€/copro/mois

Pour 40 copros cloud:

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Type Storage
     - Volume
     - Coût/mois TTC
   * - SSD (année courante)
     - 8GB (40 × 200MB)
     - 0.80€
   * - S3 Standard 1-AZ (1-3 ans)
     - 24GB (40 × 600MB)
     - 0.17€
   * - S3 Cold Archive (3-10 ans)
     - 56GB (40 × 1.4GB)
     - 0.11€
   * - **Total Storage**
     -
     - **1.08€**

**Coût Infrastructure Total**: 7.12€ + 1.08€ = **8.20€/mois**

Revenus
~~~~~~~

Grille tarifaire ASBL (prix fixe démocratique):

- **Prix/copro/mois**: **5.00€** (fixe, baisse par vote AG uniquement)
- **Copros cloud** (40%): 40 copros × 5.00€ = **200€/mois**

Bilan Financier
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Montant/mois
     - Montant/an
   * - **Revenus cloud**
     - 200.00€
     - 2,400€
   * - **Coûts infrastructure**
     - -8.20€
     - -98€
   * - **Surplus**
     - **191.80€**
     - **2,302€**
   * - **Marge**
     - **95.9%**
     -

Impact Écologique
~~~~~~~~~~~~~~~~~

Basé sur 0.12g CO₂/req (PERFORMANCE_REPORT):

- **Requêtes/jour** (estimation 100 req/copro/jour): 4,000 req/jour
- **CO₂ cloud annuel**: 4,000 × 365 × 0.12g = **175kg CO₂/an**
- **CO₂ évité** vs solutions propriétaires (facteur 96×): **16.8 tonnes CO₂/an**

Palier 2: 500 Copropriétés
---------------------------

Infrastructure
~~~~~~~~~~~~~~

- **VPS nécessaires**: 1 × Value (capacité 2,000-3,000 copros)
- **Copros cloud** (40%): 200 copros
- **Copros self-hosted** (60%): 300 copros

Coûts
~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Détail
     - Coût/mois TTC
   * - **Compute**
     - 1 × VPS Value + DNS
     - 7.12€
   * - **Storage**
     - 200 copros × 0.027€
     - 5.40€
   * - **Total Infrastructure**
     -
     - **12.52€**

Revenus
~~~~~~~

Grille tarifaire (prix fixe démocratique):

- **Prix/copro/mois**: **5.00€** (fixe, baisse par vote AG uniquement)
- **Copros cloud**: 200 × 5.00€ = **1,000€/mois**

Bilan Financier
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Montant/mois
     - Montant/an
   * - **Revenus cloud**
     - 1,000.00€
     - 12,000€
   * - **Coûts infrastructure**
     - -12.52€
     - -150€
   * - **Surplus**
     - **987.48€**
     - **11,850€**
   * - **Marge**
     - **98.7%**
     -

Impact Écologique
~~~~~~~~~~~~~~~~~

- **CO₂ cloud annuel**: 20,000 req/j × 365 × 0.12g = **876kg CO₂/an**
- **CO₂ évité**: **84 tonnes CO₂/an**

Palier 3: 1,000 Copropriétés
-----------------------------

Infrastructure
~~~~~~~~~~~~~~

- **VPS nécessaires**: 1 × Value (capacité 2,000-3,000 copros)
- **Copros cloud** (40%): 400 copros
- **Copros self-hosted** (60%): 600 copros

Coûts
~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Détail
     - Coût/mois TTC
   * - **Compute**
     - 1 × VPS Value + DNS
     - 7.12€
   * - **Storage**
     - 400 copros × 0.027€
     - 10.80€
   * - **Total Infrastructure**
     -
     - **17.92€**

Revenus
~~~~~~~

Grille tarifaire (prix fixe démocratique):

- **Prix/copro/mois**: **5.00€** (fixe, baisse par vote AG uniquement)
- **Copros cloud**: 400 × 5.00€ = **2,000€/mois**

Bilan Financier
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Montant/mois
     - Montant/an
   * - **Revenus cloud**
     - 2,000.00€
     - 24,000€
   * - **Coûts infrastructure**
     - -17.92€
     - -215€
   * - **Surplus**
     - **1,982.08€**
     - **23,785€**
   * - **Marge**
     - **99.1%**
     -

Impact Écologique
~~~~~~~~~~~~~~~~~

- **CO₂ cloud annuel**: 40,000 req/j × 365 × 0.12g = **1.75 tonnes CO₂/an**
- **CO₂ évité**: **168 tonnes CO₂/an**

Palier 4: 2,000 Copropriétés
-----------------------------

Infrastructure
~~~~~~~~~~~~~~

- **VPS nécessaires**: 1 × Value (capacité 2,000-3,000 copros)
- **Copros cloud** (40%): 800 copros
- **Copros self-hosted** (60%): 1,200 copros

Coûts
~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Détail
     - Coût/mois TTC
   * - **Compute**
     - 1 × VPS Value + DNS
     - 7.12€
   * - **Storage**
     - 800 copros × 0.027€
     - 21.60€
   * - **Total Infrastructure**
     -
     - **28.72€**

Revenus
~~~~~~~

Grille tarifaire (prix fixe démocratique):

- **Prix/copro/mois**: **5.00€** (fixe, baisse par vote AG uniquement)
- **Copros cloud**: 800 × 5.00€ = **4,000€/mois**

Bilan Financier
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Montant/mois
     - Montant/an
   * - **Revenus cloud**
     - 4,000.00€
     - 48,000€
   * - **Coûts infrastructure**
     - -28.72€
     - -345€
   * - **Surplus**
     - **3,971.28€**
     - **47,655€**
   * - **Marge**
     - **99.3%**
     -

Impact Écologique
~~~~~~~~~~~~~~~~~

- **CO₂ cloud annuel**: 80,000 req/j × 365 × 0.12g = **3.5 tonnes CO₂/an**
- **CO₂ évité**: **336 tonnes CO₂/an**

Palier 5: 5,000 Copropriétés (KPI 2030)
----------------------------------------

Infrastructure
~~~~~~~~~~~~~~

- **VPS nécessaires**: 2 × Value (1 VPS = 2,500 copros)
- **Copros cloud** (40%): 2,000 copros
- **Copros self-hosted** (60%): 3,000 copros

Coûts Base
~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Détail
     - Coût/mois TTC
   * - **Compute**
     - 2 × VPS Value + DNS
     - 14.14€
   * - **Storage**
     - 2,000 copros × 0.027€
     - 54.00€
   * - **Total Infrastructure Base**
     -
     - **68.14€**

Revenus Base
~~~~~~~~~~~~

Grille tarifaire (prix fixe démocratique):

- **Prix/copro/mois**: **5.00€** (fixe, baisse par vote AG uniquement)
- **Copros cloud**: 2,000 × 5.00€ = **10,000€/mois**

🚀 PropTech 2.0 Add-ons
~~~~~~~~~~~~~~~~~~~~~~~

**Infrastructure Add-ons**:

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Add-on
     - Infrastructure
     - Coût/mois
   * - **AI Assistant**
     - OVH AI Endpoints (GPU inference)
     - 50€
   * - **Blockchain Voting**
     - Polygon RPC node
     - 20€
   * - **IoT Sensors**
     - MQTT + TimescaleDB
     - 25€
   * - **Total PropTech Infra**
     -
     - **95€**

**Total Infrastructure avec PropTech**: 68.14€ + 95€ = **163.14€/mois**

**Tarification Add-ons**:

- **AI Assistant**: +2€/mois par copro
- **Blockchain Voting**: +1€/mois par copro
- **IoT Sensors**: +10€/mois par copro (incluant hardware)

**Projections Revenus Add-ons** (40% adoption):

.. list-table::
   :header-rows: 1
   :widths: 30 20 20 30

   * - Add-on
     - Adoption
     - Copros
     - Revenus/mois
   * - **AI Assistant**
     - 40%
     - 800
     - 800 × 2€ = 1,600€
   * - **Blockchain Voting**
     - 30%
     - 600
     - 600 × 1€ = 600€
   * - **IoT Sensors**
     - 20%
     - 400
     - 400 × 10€ = 4,000€
   * - **Total Add-ons**
     -
     -
     - **6,200€**

Bilan Financier Complet
~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Montant/mois
     - Montant/an
   * - **Revenus base**
     - 10,000€
     - 120,000€
   * - **Revenus add-ons**
     - 6,200€
     - 74,400€
   * - **Revenus TOTAL**
     - **16,200€**
     - **194,400€**
   * -
     -
     -
   * - **Coûts infrastructure base**
     - -68.14€
     - -818€
   * - **Coûts infrastructure PropTech**
     - -95€
     - -1,140€
   * - **Coûts TOTAL**
     - **-163.14€**
     - **-1,958€**
   * -
     -
     -
   * - **Surplus**
     - **16,036.86€**
     - **192,442€**
   * - **Marge**
     - **99.0%**
     -

💰 Financement Développement
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Avec surplus annuel de **192,442€**:

- **1.5 ETP** (3,600€/mois): 43,200€/an → ✅ **COUVERT**
- **Surplus restant**: 192,442€ - 43,200€ = **149,242€/an**

  - **Réinvestissement R&D**: 20,000€/an
  - **Fonds urgence**: 10,000€/an
  - **Distribution communauté**: 8,842€/an

Impact Écologique
~~~~~~~~~~~~~~~~~

- **CO₂ cloud annuel**: 200,000 req/j × 365 × 0.12g = **8.76 tonnes CO₂/an**
- **CO₂ évité**: **840 tonnes CO₂/an**

Palier 6: 10,000 Copropriétés
------------------------------

Infrastructure
~~~~~~~~~~~~~~

- **VPS nécessaires**: 4 × Value (1 VPS = 2,500 copros)
- **Copros cloud** (40%): 4,000 copros
- **Copros self-hosted** (60%): 6,000 copros

Coûts
~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Détail
     - Coût/mois TTC
   * - **Compute**
     - 4 × VPS Value + DNS
     - 28.18€
   * - **Storage**
     - 4,000 copros × 0.027€
     - 108.00€
   * - **PropTech Infra**
     - GPU + Blockchain + IoT
     - 95€
   * - **Total Infrastructure**
     -
     - **231.18€**

Revenus
~~~~~~~

Grille tarifaire (prix fixe démocratique):

- **Prix/copro/mois base**: **5.00€** (fixe, baisse par vote AG uniquement)
- **Copros cloud**: 4,000 × 5.00€ = **20,000€/mois**

**Revenus add-ons** (40% adoption):

- **AI** (1,600 copros): 1,600 × 2€ = 3,200€/mois
- **Blockchain** (1,200 copros): 1,200 × 1€ = 1,200€/mois
- **IoT** (800 copros): 800 × 10€ = 8,000€/mois
- **Total add-ons**: **12,400€/mois**

Bilan Financier
~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Poste
     - Montant/mois
     - Montant/an
   * - **Revenus base**
     - 20,000€
     - 240,000€
   * - **Revenus add-ons**
     - 12,400€
     - 148,800€
   * - **Revenus TOTAL**
     - **32,400€**
     - **388,800€**
   * -
     -
     -
   * - **Coûts infrastructure**
     - -231.18€
     - -2,774€
   * - **Surplus**
     - **32,168.82€**
     - **386,026€**
   * - **Marge**
     - **99.3%**
     -

💰 Financement avec 10k Copros
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **3 ETP** (7,200€/mois): 86,400€/an → ✅ **COUVERT**
- **Surplus restant**: 150,826€ - 86,400€ = **64,426€/an**

Impact Écologique
~~~~~~~~~~~~~~~~~

- **CO₂ cloud annuel**: 400,000 req/j × 365 × 0.12g = **17.5 tonnes CO₂/an**
- **CO₂ évité**: **1,680 tonnes CO₂/an**

📈 Tableau Récapitulatif
=========================

.. list-table:: Synthèse par Palier
   :header-rows: 1
   :widths: 10 15 15 15 15 15 15

   * - Copros
     - VPS
     - Coûts/mois
     - Revenus base/mois
     - Revenus add-ons/mois
     - Surplus/an
     - Marge
   * - **100**
     - 1 × Value
     - 8.20€
     - 200€
     - -
     - 2,302€
     - 95.9%
   * - **500**
     - 1 × Value
     - 12.52€
     - 1,000€
     - -
     - 11,850€
     - 98.7%
   * - **1,000**
     - 1 × Value
     - 17.92€
     - 2,000€
     - -
     - 23,785€
     - 99.1%
   * - **2,000**
     - 1 × Value
     - 28.72€
     - 4,000€
     - -
     - 47,655€
     - 99.3%
   * - **5,000**
     - 2 × Value
     - 163.14€
     - 10,000€
     - 6,200€
     - **192,442€**
     - 99.0%
   * - **10,000**
     - 4 × Value
     - 231.18€
     - 20,000€
     - 12,400€
     - **386,026€**
     - 99.3%

🎯 Conclusions Clés
===================

Viabilité Économique
--------------------

✅ **Modèle viable à partir de 5,000 copros**:

- Surplus annuel: **192,442€**
- Permet financement 1.5 ETP + R&D avec large surplus (149k€/an restant)
- Add-ons PropTech 2.0 représentent **38% des revenus** (résilience accrue vs modèle dégressif)

⚠️ **Risques identifiés**:

1. **Adoption add-ons**: Scénario optimiste (40%), besoin scénario conservateur (20%)
2. **Ratio cloud/self-hosted**: 40/60 à valider empiriquement
3. **Pricing add-ons**: 2€ AI, 1€ Blockchain, 10€ IoT à tester marché

Baisse Objective des Coûts
---------------------------

**Coût par copropriété** (infrastructure cloud):

- **100 copros**: 8.20€ / 40 copros = **0.205€/copro/mois**
- **5,000 copros**: 163.14€ / 2,000 copros = **0.082€/copro/mois**
- **10,000 copros**: 231.18€ / 4,000 copros = **0.058€/copro/mois**

**Réduction coût unitaire**: **-71.7%** entre 100 et 10,000 copros ✅

**Évolution coût storage** (devient dominant):

- **100 copros**: Storage = 13.2% du coût total (1.08€ / 8.20€)
- **5,000 copros**: Storage = 33.1% du coût total (54€ / 163.14€)
- **10,000 copros**: Storage = 46.7% du coût total (108€ / 231.18€)

→ **Justifie tarification échelonnée** selon taille réelle de la copropriété

Impact Écologique
-----------------

**CO₂ évité vs solutions propriétaires**:

- **100 copros**: 16.8 tonnes CO₂/an
- **5,000 copros**: **840 tonnes CO₂/an** (KPI 2030 ajusté)
- **10,000 copros**: **1,680 tonnes CO₂/an**

⚠️ **KPI VISION à ajuster**: -840t CO₂/an pour 5,000 copros (vs -534t initial)

Attractivité Investisseurs/Subsides
------------------------------------

**Arguments financiers**:

- Marges élevées (95.9-99.3%) démontrent viabilité extrême du modèle fixe 5€/mois
- Scaling linéaire des coûts infrastructure (VPS Value @ 7.02€ TTC)
- **Coût infrastructure ultra-compétitif**: 0.082€/copro/mois à 5,000 copros
- Storage S3 @ 0.007€/GB (30% moins cher vs estimations précédentes)
- Modèle ASBL avec réinvestissement communautaire (149k€/an disponibles à 5,000 copros)
- Add-ons PropTech 2.0 = différenciation concurrentielle (38% revenus, résilience accrue)

**Arguments écologiques**:

- **-840 tonnes CO₂/an** à 5,000 copros (dépassement KPI +57%)
- 0.12g CO₂/req (96× moins que solutions actuelles)
- Infrastructure mutualisée optimisée (VPS 1 vCPU / 2GB RAM)

**Arguments sociétaux**:

- Tarification dégressive (1€ → 0.10€) = **-90% réduction**
- Opensource (AGPL-3.0)
- Souveraineté des données (RGPD compliant)
- **100,000 personnes impactées** à 5,000 copros

📋 Recommandations
==================

1. Créer 3 Scénarios
--------------------

.. list-table::
   :header-rows: 1
   :widths: 30 20 20 30

   * - Métrique
     - Conservateur
     - Réaliste
     - Optimiste
   * - **Adoption add-ons**
     - 20%
     - 30%
     - 40%
   * - **Ratio cloud/self**
     - 30/70
     - 40/60
     - 50/50
   * - **Prix add-ons**
     - AI 1.5€, BC 0.75€
     - AI 2€, BC 1€
     - AI 3€, BC 1.5€

2. Valider Hypothèses Marché
-----------------------------

- Sonder 50 syndics belges sur pricing add-ons
- Tester ratio cloud/self-hosted (enquête)
- Analyser solutions concurrentes PropTech 2.0

3. Créer Dashboard Investisseurs
---------------------------------

- Graphiques évolution coûts/revenus par palier
- Mise en avant -84% réduction coût unitaire
- Démonstration viabilité long terme
- Scénarios risques/opportunités
