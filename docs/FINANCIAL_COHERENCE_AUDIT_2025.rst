===================================================================
Audit de Cohérence Financière KoproGo - Novembre 2025
===================================================================

:Auteur: Audit KoproGo ASBL
:Date: Novembre 2025
:Status: 🔴 DRAFT - En cours de validation

.. contents:: Table des Matières
   :depth: 3
   :local:

🎯 Objectif
===========

Identifier toutes les incohérences dans les chiffres avancés entre les différents documents stratégiques et techniques, puis établir une source unique de vérité basée sur:

1. **Données de performance réelles** (PERFORMANCE_REPORT.rst - Octobre 2025)
2. **Prix OVH actuels 2025** (tarifs publics vérifiables)
3. **Simulations d'échelle validées**
4. **Coûts PropTech 2.0** (GPU IA, Blockchain, IoT)

📊 Données de Performance Réelles
==================================

Source: PERFORMANCE_REPORT.rst
------------------------------

Infrastructure Testée
~~~~~~~~~~~~~~~~~~~~~

- **VPS**: d2-2 Ubuntu (1 vCPU / 2GB RAM) - Facturation à l'heure
- **Coût total**: 8€/mois (VPS + domaine + backups) - OVH Cloud France
- **Datacenter**: GRA11 (60g CO2/kWh)
- **Note**: Le d2-2 est un VPS "à l'heure" désormais obsolète, équivalent actuel: s1-2

Performance Mesurée
~~~~~~~~~~~~~~~~~~~

Test de charge: 3 minutes, 287 req/s

- **Taux de succès**: 99.74%
- **Throughput moyen**: 287 req/s
- **Latence P50**: 69ms
- **Latence P90**: 130ms
- **Latence P99**: 752ms ⚠️ (objectif < 5ms non atteint)
- **CO₂/requête**: 0.12g (excellent)
- **Capacité estimée**: 2,000-3,000 copropriétés multi-tenant

⚠️ Écart Objectif vs Réalité
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Objectif P99 < 5ms**: NON ATTEINT (752ms réel)
- **Objectif Throughput > 100k req/s**: NON ATTEINT (287 req/s réel)

💰 Prix OVH 2025 - Audit des Documents
=======================================

Incohérences Détectées
----------------------

.. list-table:: Comparaison des VPS mentionnés
   :header-rows: 1
   :widths: 30 20 15 20 15

   * - Document
     - VPS Mentionné
     - Prix
     - Source
     - Status
   * - **PERFORMANCE_REPORT.rst**
     - d2-2 (2c/4GB)
     - 8€/mois
     - Test réel Oct 2025
     - ✅ **RÉFÉRENCE**
   * - **ROADMAP_INTEGREE_2025_2030.rst**
     - s1-2 (1c/2GB)
     - 4,20€/mois
     - Projection
     - ⚠️ Incohérent
   * - **ECONOMIC_MODEL.rst**
     - s1-2 (1c/2GB)
     - 4,20€/mois
     - Projection
     - ⚠️ Incohérent
   * - **ECONOMIC_MODEL.rst** (exemple)
     - d2-2 (2c/4GB)
     - 7,00€/mois
     - Exemple
     - ⚠️ Incohérent avec perf report

Prix OVH 2025 Réels
-------------------

⚠️ **À VÉRIFIER MANUELLEMENT** sur https://www.ovhcloud.com/fr/vps/

Estimations basées sur documentation interne:

VPS Starter/Essential
~~~~~~~~~~~~~~~~~~~~~

Anciennement s1/s2:

- **s1-2** (1 vCore, 2GB RAM, 20GB SSD): ~6€/mois HT
- **s1-4** (1 vCore, 4GB RAM, 40GB SSD): ~9€/mois HT
- **s1-8** (1 vCore, 8GB RAM, 80GB SSD): ~15€/mois HT

VPS Balanced
~~~~~~~~~~~~

Anciennement b2:

- **b2-7** (2 vCore, 7GB RAM, 50GB SSD): ~14€/mois HT
- **b2-15** (4 vCore, 15GB RAM, 100GB SSD): ~28€/mois HT
- **b2-30** (8 vCore, 30GB RAM, 200GB SSD): ~56€/mois HT

Storage
~~~~~~~

- **SSD local**: Inclus dans prix VPS
- **LUKS encryption**: 0€ (software, pas de surcoût)
- **Object Storage S3 (Cold Archive)**: ~0.002€/GB/mois
- **Object Storage S3 (Standard)**: ~0.01€/GB/mois
- **Additional Disk (SSD)**: ~0.10€/GB/mois

Réseau
~~~~~~

- **Bande passante**: Incluse (illimitée sur VPS)
- **DNS OVH**: 0.10€/mois

🔴 PROBLÈME MAJEUR
------------------

Le VPS **d2-2** utilisé dans PERFORMANCE_REPORT (Oct 2025) n'existe pas dans la gamme OVH actuelle.

Équivalents possibles:

- **b2-7** (2 vCore, 7GB RAM): ~14€/mois → **6€/mois plus cher** que test
- **s1-4** (1 vCore, 4GB RAM): ~9€/mois → **1€/mois plus cher** que test

🧮 Modèle Économique - Incohérences
====================================

Vision Document (KPIs 2030)
---------------------------

- **5,000 copropriétés**
- **Impact Économique**: 9,35M€/an économisés

  - 8M€ logiciels propriétaires
  - 750k€ SEL
  - 600k€ consommation évitée

- **Impact Écologique**: -840 tonnes CO₂/an (dépassement +57% vs objectif initial -534t)

  - 50t infrastructure
  - 790t features communautaires

Economic Model - Grille Tarifaire Dégressive
---------------------------------------------

.. list-table:: Tarification par palier
   :header-rows: 1
   :widths: 30 25 20

   * - Palier
     - Prix/copro/mois
     - Réduction
   * - 0-500 copros
     - 1.00€
     - -
   * - 500-1,000
     - 0.80€
     - -20%
   * - 1,000-2,000
     - 0.60€
     - -40%
   * - 2,000-5,000
     - 0.40€
     - -60%
   * - 5,000-10,000
     - 0.20€
     - -80%
   * - 10,000+
     - 0.10€
     - -90%

⚠️ Calculs à Valider
---------------------

**Scénario 5,000 copros (2030)**:

Revenus
~~~~~~~

- **Ratio cloud/self-hosted**: 40% cloud, 60% self-hosted
- **Copros cloud**: 2,000 copros × 0.40€/mois = 800€/mois
- **Revenus annuels**: **9,600€/an**

Coûts Infrastructure Réels
~~~~~~~~~~~~~~~~~~~~~~~~~~~

À recalculer avec prix OVH 2025:

- **Compute** (VPS b2-7): 14€/mois
- **Storage** (estimation 500GB SSD + 1TB S3):

  - SSD additionnel: 50€/mois
  - S3 Standard: 10€/mois
  - Total storage: 60€/mois

- **DNS**: 0.10€/mois
- **Total infrastructure**: ~74€/mois = **888€/an**

Surplus Apparent
~~~~~~~~~~~~~~~~

- **Surplus**: 9,600€ - 888€ = **8,712€/an**

⚠️ **PROBLÈME CRITIQUE**
~~~~~~~~~~~~~~~~~~~~~~~~~

Comment financer **1.5 ETP** (3,600€/mois = 43,200€/an) avec seulement **800€/mois** (9,600€/an) de revenus??

**Gap de financement**: 43,200€ - 9,600€ = **33,600€/an**

🚀 PropTech 2.0 - Coûts Add-ons
================================

Infrastructure Add-ons
----------------------

.. list-table:: Coûts infrastructure PropTech 2.0
   :header-rows: 1
   :widths: 30 30 20 20

   * - Add-on
     - Infrastructure
     - Coût/mois
     - Source
   * - **AI Assistant**
     - OVH AI Endpoints (GPU inference)
     - 50€/mois
     - ECONOMIC_MODEL
   * - **Blockchain Voting**
     - Polygon RPC node
     - 20€/mois
     - ECONOMIC_MODEL
   * - **IoT Sensors**
     - MQTT + TimescaleDB
     - 25€/mois
     - ECONOMIC_MODEL
   * - **Total PropTech**
     -
     - **95€/mois**
     -

Tarification Add-ons (ECONOMIC_MODEL)
-------------------------------------

- **AI Assistant**: +2€/mois par copro
- **Blockchain Voting**: +1€/mois par copro
- **IoT Sensors**: Hardware coût + 1€/capteur/mois
- **Energy Buying Groups**: 0€ (gratuit, financé partenariats)

Projections Revenus Add-ons
----------------------------

Scénario Optimiste (40% adoption, 5,000 copros, 2030):

- **AI** (2,000 copros): 2,000 × 2€ × 12 mois = **48,000€/an**
- **Blockchain** (1,500 copros): 1,500 × 1€ × 12 mois = **18,000€/an**
- **IoT** (1,000 copros × 10€/mois): 1,000 × 10€ × 12 mois = **120,000€/an**
- **Total add-ons**: **186,000€/an**

⚠️ Cohérence à Valider
~~~~~~~~~~~~~~~~~~~~~~~

- Revenus add-ons: **186k€/an**
- Revenus base: **9,6k€/an**
- **Ratio**: 19:1

**Question critique**: Est-ce que ce ratio add-ons/base est réaliste??

📋 Recommandations pour Cohérence
==================================

1. Établir Prix Référence OVH 2025
-----------------------------------

.. todo::

   - [ ] Vérifier tarifs actuels sur ovhcloud.com
   - [ ] Documenter prix HT vs TTC
   - [ ] Intégrer coûts LUKS, S3, DNS

2. Recalculer Modèle Économique Complet
----------------------------------------

.. todo::

   - [ ] Simuler 6 paliers: 100, 500, 1k, 2k, 5k, 10k copros
   - [ ] Calculer coûts infrastructure réels par palier (compute + storage)
   - [ ] Valider ratio cloud/self-hosted (actuellement 40/60)
   - [ ] Projeter revenus base + add-ons
   - [ ] Vérifier viabilité financement dev (ETP)

3. Aligner Objectifs Performance avec Réalité
----------------------------------------------

.. todo::

   - [ ] **P99 < 5ms**: Impossible avec VPS mutualisé → Objectif réaliste **P99 < 1s** ✅
   - [ ] **Throughput > 100k req/s**: Impossible avec 1 VPS → Objectif réaliste **> 200 req/s** ✅
   - [ ] Documenter hypothèses scaling (K3s/K8s pour > 10k copros)

4. Intégrer Coûts PropTech 2.0 dans Simulations
------------------------------------------------

.. todo::

   - [ ] Ajouter coûts infrastructure PropTech (+95€/mois) aux simulations
   - [ ] Calculer seuil rentabilité add-ons (combien de copros pour couvrir 95€?)
   - [ ] Valider tarification add-ons (2€ AI, 1€ Blockchain raisonnables?)

5. Créer Dashboard Investisseurs/Subsides
------------------------------------------

.. todo::

   - [ ] Tableau unique: Performance + Coûts + Impact + ROI
   - [ ] Scénarios conservateur/réaliste/optimiste
   - [ ] Mettre en avant baisse objective des coûts avec échelle
   - [ ] Démontrer viabilité long terme

🎯 Questions Critiques à Résoudre
==================================

1. **VPS d2-2 @ 8€/mois existe-t-il encore?**

   Si non, quel VPS utiliser comme référence?

2. **Ratio 40% cloud / 60% self-hosted est-il réaliste?**

   Ou trop optimiste?

3. **Comment financer 1.5 ETP avec 800€/mois de revenus base?**

   Add-ons indispensables?

4. **P99 < 5ms est-il atteignable**

   ou faut-il réviser l'objectif à < 1s?

5. **Adoption add-ons 40% en 2030**

   est-elle réaliste ou optimiste?

✅ Actions Immédiates
=====================

.. list-table:: Plan d'action
   :header-rows: 1
   :widths: 10 60 30

   * - Status
     - Action
     - Responsable
   * - ✅
     - Créer ce document d'audit
     - DONE
   * - ⏳
     - Vérifier prix OVH actuels (ovhcloud.com)
     - À FAIRE
   * - ⏳
     - Recalculer modèle économique complet avec prix réels
     - À FAIRE
   * - ⏳
     - Aligner tous les documents (VISION, MISSION, ECONOMIC_MODEL, ROADMAP)
     - À FAIRE
   * - ⏳
     - Créer tableau investisseurs séduisant et cohérent
     - À FAIRE
