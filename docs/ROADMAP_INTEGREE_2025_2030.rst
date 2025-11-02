=========================================================
KoproGo - Vision Stratégique 2025-2030
=========================================================

:Version: 3.0
:Date: 2 novembre 2025
:Auteurs: Gilles & Farah - Co-fondateurs KoproGo ASBL
:Statut: Document de référence stratégique

.. contents:: Table des matières
   :depth: 3
   :local:

=========================================================
Vision : Démocratiser la Gestion de Copropriété
=========================================================

**"La technologie au service du bien commun, pas du profit"**

KoproGo est un projet open source de gestion de copropriété qui vise à **économiser 70 millions d'euros par an** aux 1,5 million de copropriétés belges en proposant une alternative éthique aux solutions propriétaires coûteuses (200-500€/mois).

Proposition de Valeur Unique
-----------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 30 30 20

   * - Aspect
     - Solutions Actuelles
     - KoproGo
     - Différence
   * - **Prix**
     - 200-500€/mois
     - **2-5€/mois**
     - **99% moins cher**
   * - **Modèle**
     - Propriétaire, opaque
     - **Open source** (AGPL-3.0)
     - Transparence totale
   * - **Structure**
     - Entreprise à profit
     - **ASBL → Coopérative**
     - Démocratique
   * - **Données**
     - Vendor lock-in
     - **Souveraineté totale**
     - Indépendance
   * - **Impact CO₂**
     - ~130 kg/an
     - **5 kg/an**
     - **96% réduction**
   * - **Infrastructure**
     - AWS/Azure (USA)
     - **OVH France**
     - Souverain

Horizon 2030
------------

En 2030, KoproGo aura atteint :

**Impact Économique**

* ✅ **5.000+ copropriétés** libérées du vendor lock-in
* ✅ **1M€ économisés/an** sur abonnements (vs marché)
* ✅ **750k€ économie circulaire/an** via SEL (30% adoption)
* ✅ **600k€ achats évités/an** via partage objets
* ✅ **2.000 coopérateurs** propriétaires du projet
* ✅ **10-15 emplois** stables et équitables créés

**Impact Environnemental**

* ✅ **1.109 tonnes CO₂ évitées/an** (infrastructure + features communautaires)
* ✅ **12.000 objets partagés** en circulation
* ✅ **6.250 objets réutilisés/an** via swap shop
* ✅ **Datacenter France** 87% moins carboné que moyenne mondiale

**Impact Social**

* ✅ **36.000h services échangés/an** entre voisins
* ✅ **5.000 annonces/an** tableau affichage communautaire
* ✅ **100% open source** pour toujours
* ✅ **Leader européen** ESS PropTech

=========================================================
Modèle de Croissance par Étapes
=========================================================

Le succès de KoproGo se mesure par **deux métriques fondamentales** :

1. **Nombre de copropriétés hébergées** → Valide la proposition de valeur technique
2. **Nombre de sociétaires coopératifs** → Valide la gouvernance démocratique

Paliers de Croissance Infrastructure
-------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 12 18 12 12 12 12 12 10

   * - Palier
     - Compute
     - SSD Chiffré
     - S3 Backup
     - Total/mois
     - Capacité
     - Coût/copro
     - Déclencheur
   * - **Nano**
     - VPS s1-2 (1c/2GB): 4,20€
     - 20GB: 2€
     - 10GB: 0,10€
     - **6,30€**
     - 100 copros
     - 0,063€
     - MVP lancé
   * - **Micro**
     - VPS s1-4 (1c/4GB): 7,20€
     - 100GB: 10€
     - 50GB: 0,50€
     - **17,70€**
     - 500 copros
     - 0,035€
     - >80 copros
   * - **Small**
     - VPS b2-7 (2c/7GB): 12€
     - 300GB: 30€
     - 150GB: 1,50€
     - **43,50€**
     - 1.500 copros
     - 0,029€
     - >400 copros
   * - **Medium**
     - VPS b2-15 (4c/15GB): 24€
     - 600GB: 60€
     - 300GB: 3€
     - **87€**
     - 3.000 copros
     - 0,029€
     - >1.200 copros
   * - **Large**
     - K8s cluster: 80€
     - 2TB: 200€
     - 1TB: 10€
     - **290€**
     - 10.000+ copros
     - 0,029€
     - >2.500 copros

**Principe clé** : L'infrastructure évolue **automatiquement** quand le seuil de 80% de capacité est atteint. Pas de date fixe, mais des conditions mesurables.

**Détail des coûts de stockage** :

* **SSD Chiffré (LUKS)** : Données dynamiques (PostgreSQL, documents uploadés, logs)

  * ~200MB/copro en moyenne (DB + documents)
  * Tarif OVH : ~0,10€/GB/mois
  * Réplication non comptée (incluse dans volume VPS)

* **S3 Object Storage** : Backups chiffrés GPG + archives

  * ~100MB/copro en moyenne (backups quotidiens rétention 30j)
  * Tarif OVH : ~0,01€/GB/mois
  * Lifecycle policy automatique (7 daily, 4 weekly, 12 monthly)

Évolution Structure Juridique
------------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 25 25 30

   * - Structure
     - Déclencheur
     - Gouvernance
     - Avantages
   * - **ASBL**
     - MVP production ready
     - CA 3 membres, AG annuelle
     - Crédibilité sociale, subventions
   * - **Pré-Coopérative**
     - 50+ utilisateurs réguliers
     - Préparation statuts
     - Test gouvernance
   * - **Coopérative SC**
     - 200+ sociétaires potentiels
     - 1 sociétaire = 1 voix
     - Tax shelter, ristournes
   * - **Agrément CNC**
     - 500+ sociétaires actifs
     - Audit externe validé
     - Crédibilité institutionnelle
   * - **B-Corp**
     - 1.000+ sociétaires
     - Certification impact
     - Leadership ESS européen

**Flexibilité** : Les transitions se font quand les conditions sont remplies, pas à date fixe.

=========================================================
Architecture Technique : Fondation de la Scalabilité
=========================================================

État Actuel : Fondations Solides
---------------------------------

✅ **Performance Validée** (Tests charge Nov 2025)

* Success Rate: **99.74%** (47.681 requêtes sur 3 min)
* Throughput: **287 req/s** soutenu
* Latence P50: **69ms** | P99: **752ms**
* RAM: **128MB max** (6.3% de 2GB allouée)
* CPU: **8% moyen**, pic 25%
* CO₂: **0.12g/req** (7-25x mieux que concurrents)

✅ **Architecture Technique**

* Architecture hexagonale (Ports & Adapters)
* 73 endpoints API REST opérationnels
* Backend Rust + Actix-web optimisé
* Frontend Astro + Svelte (PWA offline-first)
* PostgreSQL 15 avec migrations SQLx
* Tests : Unitaires + Intégration + BDD + E2E

Stack Technologique
-------------------

.. code-block:: yaml

   Backend:
     - Langue: Rust (10x plus efficace que Python/Node)
     - Framework: Actix-web (287 req/s sur 1 vCore)
     - Architecture: Hexagonale (DDD)
     - Database: PostgreSQL 15
     - Tests: Pyramid strategy (unit/integration/E2E/BDD)

   Frontend:
     - Framework: Astro + Svelte
     - Build: Static Site Generation
     - PWA: Offline-first avec IndexedDB
     - i18n: NL, FR, DE, EN

   Infrastructure:
     - Provider: OVH France (souverain, écologique)
     - Deployment: Docker Compose → K3s → K8s
     - GitOps: Terraform + Ansible
     - Monitoring: Prometheus + Grafana + Netdata

Optimisations Critiques
------------------------

**1. Utilisation RAM : 5% seulement**

Découverte clé des tests de charge : nous utilisons 128MB sur 2GB allouée. Cela signifie :

* **15x marge de sécurité** actuelle
* **Capacité réelle** : 1.000-1.500 copros sur VPS 4,20€/mois
* **Coût infrastructure** : 0,003-0,004€ par copro/mois

**2. PostgreSQL Connection Pool**

* Ajusté dynamiquement selon CPU cores
* 5 connections pour 1 core, 10 pour 2+ cores
* Indexes sur toutes foreign keys

**3. Cache Intelligent**

* Buildings/Owners : Cache 5 min (changent rarement)
* Expenses/Meetings : Cache 1 min (plus dynamiques)
* CDN Cloudflare pour assets statiques (gratuit)

**4. Workers Auto-Scaling**

* ``num_cpus::get()`` pour adaptation automatique
* Scale horizontal via HPA (Kubernetes phases)

**5. Architecture Stockage Hybride**

Découverte clé : Le **stockage devient le coût principal** au-delà de 500 copropriétés.

* **Données chaudes** (SSD chiffré LUKS) :

  * PostgreSQL database (~50MB/copro)
  * Documents uploadés récents (~100MB/copro)
  * Logs applicatifs (~50MB/copro)
  * Total : **~200MB/copro** sur SSD haute performance

* **Données froides** (S3 Object Storage) :

  * Backups GPG quotidiens (~100MB/copro)
  * Archives documents >6 mois
  * Lifecycle automatique : 7j → 30j → 1an
  * Total : **~100MB/copro** sur stockage économique

* **Optimisations stockage** :

  * Compression PostgreSQL (TOAST)
  * Deduplication backups (incremental)
  * Images optimisées WebP/AVIF
  * Purge logs >90 jours

**Impact économique** : Le stockage représente 30-70% des coûts selon échelle, mais reste 200x moins cher que solutions concurrentes grâce à l'efficacité compute.

=========================================================
Jalons Produit : Features Débloquant la Croissance
=========================================================

**Principe fondateur** : Les dates sont indicatives. Chaque jalon débloque un nombre de copropriétés supplémentaires hébergeables.

Jalon 0 : Fondations Techniques ✅
-----------------------------------

**État** : Achevé (Nov 2025)

**Débloque** : 10-20 early adopters (beta fermée)

* ✅ Architecture hexagonale implémentée
* ✅ 73 endpoints API REST
* ✅ Tests E2E Playwright
* ✅ Load tests validés (99.74% success)
* ✅ Documentation Sphinx publiée

**Conformité légale** : 30% (features CRUD de base)

Jalon 1 : Sécurité & GDPR
--------------------------

**Débloque** : 50-100 copropriétés (beta publique possible)

**Issues critiques** :

* #39 : LUKS Encryption at-rest
* #40 : Backups automatisés GPG + S3
* #42 : GDPR basique (export + effacement)
* #48 : Authentification forte (itsme®)

**Livrables** :

* Données chiffrées au repos
* Backups quotidiens testés
* Conformité GDPR Articles 15 & 17
* Auth multi-facteur opérationnelle

**Conformité légale** : 40%

**Timeline indicative** : 6-8 semaines

Jalon 2 : Conformité Légale Belge
----------------------------------

**Débloque** : 200-500 copropriétés (production ouverte)

**Issues critiques** :

* #16 : Plan Comptable Normalisé Belge (PCB)
* #17 : État Daté (bloque ventes immobilières)
* #18 : Budget Prévisionnel Annuel
* #22 : Conseil de Copropriété (obligatoire >20 lots)
* #23 : Workflow Recouvrement Impayés

**Livrables** :

* Comptabilité conforme arrêté royal 12/07/2012
* Génération états datés automatique
* Budgets avec variance analysis
* Dashboard conseil avec alertes
* Relances automatiques 3 niveaux

**Conformité légale** : 80%

**Bloquants levés** :

* **État daté** : Permet ventes de lots (CRITIQUE)
* **Conseil copropriété** : Débloque copros >20 lots (60% du marché)

**Timeline indicative** : 8-12 semaines après Jalon 1

Jalon 3 : Features Différenciantes
-----------------------------------

**Débloque** : 500-1.000 copropriétés (différenciation marché)

**Issues importantes** :

* #46 : Voting Digital (scrutins AG conformes)
* #47 : PDF Generation étendue
* #49 : Module SEL (Système Échange Local)
* #26 : Partage d'Objets
* #52 : Contractor Backoffice

**Livrables** :

* Votes AG avec signature itsme®
* Templates PDF tous documents légaux
* Monnaie locale virtuelle intégrée
* Bibliothèque objets partagés
* Espace prestataires

**Conformité légale** : 90%

**Avantage compétitif** : Features communautaires uniques (mission ASBL)

**Timeline indicative** : 10-14 semaines après Jalon 2

Jalon 4 : Automation & Intégrations
------------------------------------

**Débloque** : 1.000-2.000 copropriétés (scalabilité)

**Issues** :

* #19 : Convocations AG automatiques
* #20 : Carnet d'Entretien Digital
* #21 : GDPR complet (Articles 16, 18, 21)
* #24 : Module Devis Travaux
* #25 : Affichage Public Syndic
* #27 : Accessibilité WCAG 2.1 AA

**Livrables** :

* Workflow AG 100% automatisé
* Carnet maintenance avec alertes
* GDPR compliance totale
* Comparaison devis multi-entrepreneurs
* Page publique syndic (SEO)
* Accessibilité complète

**Conformité légale** : 95%

**Timeline indicative** : 12-16 semaines après Jalon 3

Jalon 5 : Mobile & API Publique
--------------------------------

**Débloque** : 2.000-5.000 copropriétés (expansion)

**Features** :

* PWA mobile responsive
* API publique v1 documentée (OpenAPI)
* Multi-langue NL/FR/DE/EN complet
* Intégrations comptables (Winbooks, Exact)
* Notifications intelligentes
* Analytics & Dashboards

**Livrables** :

* Progressive Web App installable
* SDK Python/JS/PHP
* Webhooks pour événements
* Export Winbooks/Exact Online
* Digest hebdomadaire personnalisé
* KPIs syndic temps réel

**Conformité légale** : 100%

**Timeline indicative** : 14-18 semaines après Jalon 4

Jalon 6 : Intelligence & Expansion
-----------------------------------

**Débloque** : 5.000-10.000 copropriétés (leadership)

**Features avancées** :

* IA Assistant Syndic (GPT-4/Claude)
* API Bancaire PSD2 (réconciliation auto)
* Marketplace Services Locaux
* Prédictions budgétaires (ML)
* Multi-region (Benelux)

**Livrables** :

* Chatbot réglementaire
* Import transactions bancaires
* Annuaire prestataires vérifiés
* Modèles ARIMA prévisions charges
* Adaptation législation NL/LU

**Timeline indicative** : 18-24 semaines après Jalon 5

Jalon 7 : Platform Economy
---------------------------

**Débloque** : 10.000+ copropriétés (scale planétaire)

**Vision long terme** :

* SDK multi-langages pour développeurs
* Store modules tiers (marketplace)
* Blockchain pour votes (immutabilité)
* Carbon Credits Trading
* White-label pour fédérations

**Timeline indicative** : 24+ mois après lancement

=========================================================
Modèle Économique Ultra-Optimisé
=========================================================

Découverte Clé : Compute Ultra-Efficace, Stockage Principal
------------------------------------------------------------

Les tests de performance révèlent deux insights critiques :

**1. Compute surdimensionné** (gain majeur) :

* **5% de la RAM** allouée (128 MB sur 2 GB)
* **8% CPU moyen** sous charge normale
* **Capacité théorique** : 1.000-1.500 copros sur VPS 4,20€/mois compute seul

**2. Stockage = nouveau goulot** (coût dominant) :

* **200MB/copro** pour données chaudes (SSD chiffré)
* **100MB/copro** pour backups (S3 Object Storage)
* **Total : 300MB/copro** → 30GB pour 100 copros, 150GB pour 500 copros

**Impact économique** :

* **Palier Nano** (100 copros) : Compute 67% / Stockage 33%
* **Palier Micro** (500 copros) : Compute 40% / Stockage 60%
* **Palier Small+** : Stockage devient 70%+ des coûts infra

Cela change la stratégie : **optimiser stockage = priorité au-delà de 500 copros**.

Stratégie Tarifaire Échelonnée par Taille et Features
------------------------------------------------------

**Principe** : Le prix reflète la **taille de la copropriété** et les **features débloquées** par les jalons produit.

Grille Tarifaire par Segment
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1
   :widths: 15 12 12 15 15 15 16

   * - Segment
     - Lots
     - Beta (J1)
     - Launch (J2)
     - Growth (J3)
     - Scale (J4+)
     - Features Clés
   * - **Micro**
     - 1-5
     - Gratuit
     - 1,50€
     - 2€
     - 2€
     - CRUD basique, GDPR
   * - **Petit**
     - 6-20
     - Gratuit
     - 2€
     - 2,50€
     - 2,50€
     - + PCN, État daté, Budget
   * - **Moyen**
     - 21-50
     - Gratuit
     - 3€
     - 3,50€
     - 3,50€
     - + **Conseil Copro**, Voting
   * - **Grand**
     - 51-100
     - Gratuit
     - 4€
     - 5€
     - 5€
     - + SEL, Partage, Analytics
   * - **XL**
     - 100+
     - Gratuit
     - 5€
     - 7€
     - 8€
     - + IA, API, Multi-langue

**Frontières Business** :

* **<20 lots** : Conseil Copropriété non obligatoire (loi belge)
* **≥20 lots** : **Conseil Copropriété obligatoire** → Feature exclusive Jalon 2 (#22)
* **≥50 lots** : Complexité accrue → Features avancées justifient premium
* **≥100 lots** : Besoins entreprise → Full platform avec API/IA

Évolution Tarifaire par Phase
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1
   :widths: 15 20 25 20 20

   * - Phase
     - Jalon Requis
     - Features Incluses
     - Prix Moyen
     - Positionnement
   * - **Beta**
     - Jalon 1 (Sécurité)
     - Auth forte, GDPR basique, Backups
     - **Gratuit**
     - Test & Feedback
   * - **Launch**
     - Jalon 2 (Conformité)
     - + PCN, État daté, Budget, **Conseil**
     - **2,80€**
     - Production ouverte
   * - **Growth**
     - Jalon 3 (Différenciation)
     - + Voting, SEL, Partage, PDF étendu
     - **3,50€**
     - Compétitif avancé
   * - **Scale**
     - Jalon 4+ (Automation)
     - + AG auto, Devis, WCAG, Mobile
     - **4,20€**
     - Full platform

Simulation Revenus par Mix Client
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1
   :widths: 15 12 12 12 12 12 12 13

   * - Palier
     - Micro
     - Petit
     - Moyen
     - Grand
     - XL
     - Total Copros
     - Revenus/mois
   * - **Launch (J2)**
     - 40×1,5€
     - 35×2€
     - 20×3€
     - 4×4€
     - 1×5€
     - **100**
     - **211€**
   * - **Growth (J3)**
     - 150×2€
     - 200×2,5€
     - 100×3,5€
     - 40×5€
     - 10×7€
     - **500**
     - **1.420€**
   * - **Scale (J4)**
     - 250×2€
     - 350×2,5€
     - 250×3,5€
     - 100×5€
     - 50×8€
     - **1.000**
     - **3.025€**

**Hypothèses Mix** : Répartition réaliste marché belge (65% petites, 20% moyennes, 10% grandes, 5% XL)

Projections par Palier (Modèle Tarifaire Échelonné)
----------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 10 12 10 10 10 10 12 13 13

   * - Copros
     - Mix Client
     - Prix Moyen
     - Revenus/mois
     - Compute
     - Stockage
     - Total Infra
     - Marge
     - Excédent/an
   * - **100**
     - Voir simulation
     - 2,11€
     - **211€**
     - 4,20€
     - 2,10€
     - **6,30€**
     - **97%**
     - **2.450€**
   * - **500**
     - 30%/40%/20%/7%/3%
     - 2,84€
     - **1.420€**
     - 7,20€
     - 10,50€
     - **17,70€**
     - **98.8%**
     - **16.830€**
   * - **1.000**
     - 25%/35%/25%/10%/5%
     - 3,03€
     - **3.025€**
     - 12€
     - 31,50€
     - **43,50€**
     - **98.6%**
     - **35.775€**
   * - **2.000**
     - 20%/35%/28%/12%/5%
     - 3,25€
     - **6.500€**
     - 24€
     - 63€
     - **87€**
     - **98.7%**
     - **76.950€**
   * - **5.000**
     - 15%/30%/30%/15%/10%
     - 3,80€
     - **19.000€**
     - 80€
     - 210€
     - **290€**
     - **98.5%**
     - **224.500€**

**Évolution du Mix Client** :

* **100 copros** : Majorité petites (40% micro, 35% petit) - early adopters
* **500 copros** : Équilibre (30% micro, 40% petit, 20% moyen)
* **1.000+ copros** : Shift vers moyennes/grandes (complexité croissante)
* **5.000 copros** : 10% XL (grandes résidences) - clients premium

**Analyse Économique** :

* **Stockage = 33-72% des coûts** selon palier (croissance linéaire)
* **Prix moyen évolue** : 2,11€ → 3,80€ avec montée en gamme client
* **Marge maintenue** : 97-99% grâce à compute ultra-efficace
* **ROI Infrastructure** : Pour chaque 1€ investi, 65-90€ de revenus générés
* **Alignement business** : Prix reflète valeur (Conseil >20 lots, IA >100 lots)

Justification Tarifaire par Segment
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Alignement Valeur/Features** :

.. list-table::
   :header-rows: 1
   :widths: 15 15 30 20 20

   * - Segment
     - Prix (Scale)
     - Besoin Principal
     - Feature Critique
     - Économie vs Marché
   * - **Micro (1-5)**
     - 2€
     - Gestion simple
     - CRUD + GDPR
     - 48€/an (vs 600€)
   * - **Petit (6-20)**
     - 2,50€
     - Conformité légale
     - PCN + État daté
     - 30€/an (vs 1.200€)
   * - **Moyen (21-50)**
     - 3,50€
     - **Conseil obligatoire**
     - **Conseil Copro** (#22)
     - 42€/an (vs 1.800€)
   * - **Grand (51-100)**
     - 5€
     - Engagement communauté
     - SEL + Voting + Analytics
     - 60€/an (vs 2.400€)
   * - **XL (100+)**
     - 8€
     - Automation complète
     - IA + API + Multi-région
     - 96€/an (vs 3.600€)

**Seuils Légaux Belges** :

* **<20 lots** : Conseil Copropriété **facultatif** → Tarif basic (Article 577-8/3)
* **≥20 lots** : Conseil Copropriété **OBLIGATOIRE** → Tarif +1€ justified (Article 577-8/4)
* **Logique** : Feature #22 (Conseil) développée spécialement pour ce seuil légal

**Comparaison Concurrentielle** :

* **Vilogi** : 50-300€/mois → KoproGo 95-98% moins cher
* **Apronet** : 80-400€/mois → Même avec premium (8€), économie 90%+
* **Excel** : Gratuit mais coût temps → KoproGo = automation valeur

Allocation Excédents (Modèle Coopératif)
-----------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 15 55

   * - Poste
     - %
     - Utilisation
   * - **Réserves**
     - 30%
     - Sécurité financière, imprévus
   * - **Ristournes sociétaires**
     - 30%
     - Redistribution démocratique
   * - **Investissement R&D**
     - 20%
     - Nouvelles features, innovations
   * - **Solidarité**
     - 10%
     - Copros précaires (tarif solidaire)
   * - **Formation**
     - 10%
     - Workshops, documentation, support

=========================================================
Analyse de Marché et Positionnement
=========================================================

Taille du Marché Belge (Grille Tarifaire Alignée)
---------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 12 12 15 15 15 16

   * - Segment KoproGo
     - Volume BE
     - Prix Marché
     - Prix KoproGo
     - Économie/an
     - Marché Total
     - Potentiel 0,33%
   * - **Micro (1-5)**
     - 650.000
     - 30-50€
     - **2€**
     - 456-576€
     - 234-390M€
     - 15,6M€
   * - **Petit (6-20)**
     - 325.000
     - 50-100€
     - **2,50€**
     - 570-1.170€
     - 195-390M€
     - 9,75M€
   * - **Moyen (21-50)**
     - 300.000
     - 100-200€
     - **3,50€**
     - 1.158-2.358€
     - 360-720M€
     - 12,6M€
   * - **Grand (51-100)**
     - 150.000
     - 150-300€
     - **5€**
     - 1.740-3.540€
     - 270-540M€
     - 9M€
   * - **XL (100+)**
     - 75.000
     - 200-500€
     - **8€**
     - 2.304-5.904€
     - 180-450M€
     - 7,2M€
   * - **TOTAL**
     - **1.500.000**
     - -
     - Moy: 3,50€
     - -
     - **1.239-2.490M€**
     - **54,15M€**

**Part de marché réaliste** : 0,33% (5.000 copros) = **54M€ économisés/an** pour utilisateurs

**Insights Marché** :

* **65% du marché** = copros <20 lots (975.000) → Cible prioritaire Micro/Petit
* **20% du marché** = 21-50 lots (300.000) → **Conseil obligatoire** = feature différenciante
* **15% du marché** = >50 lots (225.000) → Premium justifié par complexité
* **Seuil 20 lots** : Frontière légale ET business (Article 577-8/4)

Analyse Concurrentielle
------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 15 20 20 30

   * - Concurrent
     - Prix/mois
     - Forces
     - Faiblesses
     - Différenciation KoproGo
   * - **Vilogi**
     - 50-300€
     - Leader BE
     - Cher, propriétaire
     - 95% moins cher, open source
   * - **Apronet**
     - 80-400€
     - Intégrations
     - Complex, lock-in
     - Simple, souverain
   * - **Excel**
     - 0€
     - Gratuit
     - Manuel, erreurs
     - Automatisé, collaboratif
   * - **Bexio**
     - 30-150€
     - Comptabilité
     - Généraliste
     - Spécialisé copropriété

**Avantage compétitif durable** :

1. **Prix imbattable** : 99% moins cher (infrastructure optimisée)
2. **Open source** : Impossibilité de fermeture/rachat
3. **Mission sociale** : ASBL/Coopérative (pas de profit)
4. **Souveraineté** : Données en France, GDPR natif
5. **Features uniques** : SEL, partage, communauté

=========================================================
Impact Social et Environnemental
=========================================================

Impact Économique Direct (Horizon 5.000 copros)
------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Métrique
     - Valeur
     - Impact
   * - **Économies abonnement**
     - 1M€/an
     - 200€/copro/an économisés vs marché
   * - **Emplois créés**
     - 10-15 ETP
     - CDI équitables (salaires justes)
   * - **Parts coopérateurs**
     - 100.000€
     - 2.000 × 50€ (capital démocratique)
   * - **Réinvestissement local**
     - 100%
     - Aucun dividende externe

Impact Features Communautaires (SEL + Partage)
-----------------------------------------------

**Hypothèse** : 30% des copropriétés activent les modules communautaires (Jalon 3+)

.. list-table::
   :header-rows: 1
   :widths: 25 20 25 30

   * - Feature
     - Adoption
     - Impact/copro/an
     - Impact Total (1.500 copros)
   * - **SEL (Monnaie locale)**
     - 30% (1.500 copros)
     - 500€ échangés
     - **750.000€** économie circulaire
   * - **Partage d'Objets**
     - 30% (1.500 copros)
     - 8 objets partagés
     - **12.000 objets** en circulation
   * - **Skills Directory**
     - 20% (1.000 copros)
     - 3h services/mois
     - **36.000h/an** compétences échangées
   * - **Swap Shop (Troc)**
     - 25% (1.250 copros)
     - 5 items/an
     - **6.250 objets** réutilisés

**Calcul Impact SEL** :

* **500€ échangés/copro/an** : Moyenne services entre voisins (bricolage, garde enfants, cours)
* **1.500 copros actives** (30% de 5.000) × 500€ = **750k€ économie circulaire/an**
* **Multiplicateur** : Chaque euro en SEL = 1€ non dépensé dans économie classique
* **Valeur sociale** : Renforcement lien social, réduction isolement

**Calcul Impact Partage Objets** :

* **8 objets partagés/copro** : Moyenne (perceuse, échelle, tondeuse, livres, jeux, vélos)
* **12.000 objets en circulation** → Évite 12.000 achats neufs
* **Valeur économique** : 50€/objet moyen → **600k€ achats évités**
* **Impact écologique** : 240 tonnes CO₂ évitées (20kg CO₂/objet fabriqué moyen)

Impact Environnemental (Infrastructure + Features)
---------------------------------------------------

**Impact Direct Infrastructure**

.. list-table::
   :header-rows: 1
   :widths: 25 20 25 30

   * - Métrique
     - KoproGo
     - Moyenne Marché
     - Réduction
   * - **CO₂/requête**
     - 0,12g
     - 3g
     - **-96%**
   * - **CO₂/copro/an**
     - 5 kg
     - 130 kg
     - **-96%**
   * - **CO₂ total (5k copros)**
     - 25 tonnes
     - 650 tonnes
     - **-625 tonnes**
   * - **Datacenter**
     - France (60g/kWh)
     - Monde (450g/kWh)
     - **-87%**

**Impact Indirect Features Partage** (30% adoption)

.. list-table::
   :header-rows: 1
   :widths: 30 25 20 25

   * - Source
     - Calcul
     - CO₂ évité/an
     - Équivalent
   * - **Partage objets**
     - 12.000 objets × 20kg
     - **240 tonnes**
     - 1.200 vols Paris-NY
   * - **SEL (services locaux)**
     - 750k€ × 0,2kg/€
     - **150 tonnes**
     - Réduction déplacements
   * - **Swap Shop (réutilisation)**
     - 6.250 items × 15kg
     - **94 tonnes**
     - 470 tonnes déchets évités
   * - **Total Features**
     - -
     - **484 tonnes**
     - 2x impact infrastructure

**Impact Environnemental Total** :

* **Infrastructure** : 625 tonnes CO₂ évitées (optimisation tech)
* **Features communautaires** : 484 tonnes CO₂ évitées (partage/réutilisation)
* **TOTAL** : **1.109 tonnes CO₂/an** évitées à 5.000 copropriétés
* **Multiplicateur** : Les features sociales doublent l'impact écologique !

**Facteurs écologiques** :

* Mix électrique France nucléaire (faible carbone)
* Architecture Rust ultra-optimisée (5% RAM)
* OVH engagé compensation carbone
* **Features partage** : Économie circulaire intégrée dans le produit

Impact Social
-------------

**Transparence**

* Code source ouvert (AGPL-3.0)
* Comptes publiés annuellement
* Roadmap et décisions publiques

**Démocratie**

* 1 coopérateur = 1 voix
* Gouvernance participative
* Élections CA annuelles

**Solidarité**

* 10% excédents pour copros précaires
* Tarif solidaire (50% réduction)
* Formation gratuite

**Lien Social & Communauté** (Impact Mesurable)

.. list-table::
   :header-rows: 1
   :widths: 30 25 45

   * - Feature
     - Métrique
     - Impact Social
   * - **SEL (Monnaie locale)**
     - 750k€ échangés/an
     - Création monnaie locale, économie circulaire, autonomie
   * - **Partage objets**
     - 12.000 objets partagés
     - Réduction consommation, entraide, tissu social
   * - **Skills Directory**
     - 36.000h services/an
     - Valorisation compétences, intergénérationnel
   * - **Swap Shop (Troc)**
     - 6.250 objets réutilisés
     - Lutte gaspillage, solidarité, gratuité
   * - **Notice Board**
     - 5.000 annonces/an
     - Communication voisinage, événements locaux

**Bénéfices Sociaux Indirects** :

* **Réduction isolement** : Interactions régulières entre voisins
* **Intergénérationnel** : Seniors valorisés (compétences) + jeunes aidés (services)
* **Cohésion résidentielle** : Sentiment appartenance communauté
* **Autonomie locale** : Moins dépendance économie globalisée
* **Résilience** : Réseau entraide en cas de crise

=========================================================
Équipe et Compétences
=========================================================

Évolution par Palier de Croissance
-----------------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 10 25 15 35

   * - Palier Copros
     - Taille
     - Composition
     - Budget RH
     - Focus
   * - **<100**
     - 2
     - Gilles + Farah (fondateurs)
     - 0€
     - Dev + vision
   * - **100-500**
     - 3
     - +Dev backend Rust
     - 18k€/an
     - Features + revue code
   * - **500-1k**
     - 5
     - +Frontend +DevOps
     - 60k€/an
     - Platform + scalabilité
   * - **1k-2k**
     - 7
     - +Support +Community
     - 120k€/an
     - Croissance + engagement
   * - **2k-5k**
     - 10
     - +Data +Sales
     - 200k€/an
     - Expansion + intelligence
   * - **5k+**
     - 15
     - +International
     - 350k€/an
     - Leadership ESS

**Principe** : Recrutement quand excédents > 2x salaire annuel (sécurité)

Compétences Critiques
----------------------

**Phase Fondation (0-100 copros)** :

* ✅ Rust backend (Gilles + Farah)
* ✅ Frontend Svelte (Farah)
* ✅ Architecture (Gilles)
* ⚠️ Juridique copropriété (formation continue)
* ⚠️ Comptabilité belge (apprendre PCB)

**Phase Croissance (100-1.000 copros)** :

* 🔜 Backend Rust senior (revue code, mentoring)
* 🔜 UI/UX designer (expérience utilisateur)
* 🔜 DevOps/SRE (infra K8s, monitoring)

**Phase Expansion (1.000-5.000 copros)** :

* 🔜 Support client (SLA, satisfaction)
* 🔜 Community manager (engagement sociétaires)
* 🔜 Data analyst (insights business)
* 🔜 Business developer (partenariats)

=========================================================
Risques et Mitigations
=========================================================

Matrice des Risques
--------------------

.. list-table::
   :header-rows: 1
   :widths: 20 12 12 25 25

   * - Risque
     - Probabilité
     - Impact
     - Mitigation
     - Plan B
   * - **Burn-out fondateurs**
     - Moyenne
     - Fatal
     - Coopérative, équipe tôt
     - Pause, contributeurs OSS
   * - **Adoption lente**
     - Forte
     - Moyen
     - Prix cassé, freemium
     - Pivot B2B syndics pro
   * - **Conformité légale**
     - Moyenne
     - Fort
     - Priorité absolue Jalon 2
     - Avocat spécialisé
   * - **Concurrent agressif**
     - Faible
     - Moyen
     - Open source protection
     - Focus différenciation
   * - **Scalabilité tech**
     - Très faible
     - Faible
     - Architecture validée
     - K8s ready dès conception
   * - **Financement**
     - Faible
     - Moyen
     - Bootstrap viable démontré
     - Subventions ESS, CNC

Facteurs Critiques de Succès
-----------------------------

1. **Conformité légale belge 100%** → Sans cela, 0 adoption (Jalon 2 critique)
2. **Prix imbattable** (2-5€ vs 200-500€) → Seul argument suffisant
3. **Excellence technique** maintenue → Performance = économies infra
4. **Communauté engagée** → Sociétaires actifs = pérennité
5. **Impact mesurable** → Économies + CO₂ documentés = preuve mission

=========================================================
Indicateurs de Succès (KPIs)
=========================================================

KPIs par Palier de Croissance
------------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 12 12 12 12 12 12

   * - Métrique
     - 100 copros
     - 500 copros
     - 1k copros
     - 2k copros
     - 5k copros
     - Cible
   * - **MRR**
     - 200€
     - 1.500€
     - 3.000€
     - 8.000€
     - 25.000€
     - Croissance
   * - **Sociétaires**
     - 0
     - 50
     - 200
     - 500
     - 2.000
     - Engagement
   * - **NPS**
     - >50
     - >60
     - >70
     - >75
     - >80
     - Satisfaction
   * - **Churn/an**
     - <15%
     - <10%
     - <7%
     - <5%
     - <3%
     - Rétention
   * - **Uptime**
     - 98%
     - 99%
     - 99.5%
     - 99.9%
     - 99.99%
     - Fiabilité

KPIs Techniques
---------------

.. list-table::
   :header-rows: 1
   :widths: 25 12 12 12 12 12 13

   * - Métrique
     - 100
     - 500
     - 1k
     - 2k
     - 5k
     - Objectif
   * - **Latency P99**
     - <200ms
     - <150ms
     - <120ms
     - <100ms
     - <80ms
     - Rapidité
   * - **Throughput**
     - 100 r/s
     - 200 r/s
     - 400 r/s
     - 800 r/s
     - 1500 r/s
     - Capacité
   * - **Coût/copro**
     - 0,042€
     - 0,014€
     - 0,012€
     - 0,012€
     - 0,016€
     - Efficacité
   * - **CO₂/req**
     - 0,15g
     - 0,12g
     - 0,10g
     - 0,08g
     - 0,06g
     - Écologie

KPIs Impact (Infra + Features Communautaires)
----------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 12 12 12 12 12

   * - Métrique
     - 100
     - 500
     - 1k
     - 2k
     - 5k
   * - **Économies abonnement/an**
     - 20k€
     - 100k€
     - 200k€
     - 400k€
     - 1M€
   * - **Économie SEL (30%)**
     - 15k€
     - 75k€
     - 150k€
     - 300k€
     - 750k€
   * - **Achats évités partage**
     - 12k€
     - 60k€
     - 120k€
     - 240k€
     - 600k€
   * - **CO₂ infra évité/an**
     - 1t
     - 5t
     - 10t
     - 20t
     - 50t
   * - **CO₂ features évité/an**
     - 1t
     - 10t
     - 97t
     - 194t
     - 484t
   * - **CO₂ TOTAL évité/an**
     - 2t
     - 15t
     - 107t
     - 214t
     - **534t**
   * - **Objets partagés**
     - 240
     - 1.200
     - 2.400
     - 4.800
     - 12.000
   * - **Heures services échangées**
     - 720
     - 3.600
     - 7.200
     - 14.400
     - 36.000
   * - **Emplois créés**
     - 0
     - 1
     - 3
     - 5
     - 10

**Note** : Impact features communautaires (SEL, partage) atteint à partir du Jalon 3 (500+ copros)

=========================================================
Plan d'Action Immédiat
=========================================================

Priorités Court Terme (Jalon 1)
--------------------------------

**Semaine 1-2** : Sécurité Infrastructure

* [ ] Issue #39 : LUKS Encryption at-rest
* [ ] Issue #40 : Backups GPG + S3
* [ ] Setup VPS OVH s1-2 (4,20€/mois)
* [ ] Monitoring Netdata gratuit

**Semaine 3-4** : GDPR & Auth

* [ ] Issue #42 : GDPR basique (export + effacement)
* [ ] Issue #48 : Inscription itsme® (démarrer, 2-4 sem délai)
* [ ] Privacy policy + CGU v1.0
* [ ] Tests sécurité complets

**Semaine 5-6** : ASBL & Beta

* [ ] Finaliser statuts ASBL
* [ ] RDV notaire (450€)
* [ ] Landing page koprogo.be
* [ ] Liste 20 beta-testers

**Résultat attendu** : Jalon 1 atteint → 50-100 copros débloquées

Priorités Moyen Terme (Jalon 2)
--------------------------------

**Bloc 1 : Plan Comptable** (bloquant)

* [ ] Issue #16 : PCB classes 1-8
* [ ] Tests comptabilité conforme
* [ ] Documentation utilisateur

**Bloc 2 : Documents Légaux**

* [ ] Issue #17 : État Daté (CRITIQUE ventes)
* [ ] Issue #18 : Budget Prévisionnel
* [ ] Issue #22 : Conseil Copropriété (>20 lots)

**Bloc 3 : Automation**

* [ ] Issue #23 : Workflow Recouvrement
* [ ] Tests E2E workflows complets
* [ ] Formation beta-testers

**Résultat attendu** : Jalon 2 atteint → 200-500 copros débloquées

=========================================================
Innovations Clés
=========================================================

1. **Architecture Ultra-Efficace**

   * Rust : 10x plus efficace que alternatives
   * 5% RAM utilisée : Optimisation extrême
   * 4,20€/mois pour 100 copros : Imbattable

2. **Modèle Coopératif**

   * Utilisateurs = Propriétaires
   * Ristournes sur excédents (30%)
   * Gouvernance démocratique (1 = 1 voix)

3. **Impact Environnemental**

   * 96% réduction CO₂
   * Datacenter France nucléaire
   * Architecture optimisée

4. **Features Communautaires**

   * SEL intégré (monnaie locale)
   * Partage objets entre voisins
   * Skills directory

5. **Transparence Totale**

   * Code open source (AGPL-3.0)
   * Comptes publics annuels
   * Roadmap ouverte

=========================================================
Conclusion : Un Projet Viable et Impactant
=========================================================

Preuves de Viabilité
--------------------

✅ **Technique** : 99.74% success rate, 287 req/s sur 1 vCore, 5% RAM utilisée
✅ **Économique** : 98% marge brute maintenue, modèle tarifaire échelonné 2-8€/mois
✅ **Juridique** : ASBL immédiate, coopérative préparée, conformité légale belge
✅ **Impact** : 534t CO₂ évitées/an (infra + features), 2,35M€ économies totales
✅ **Marché** : 1,5M copros belges, 0,33% part = succès, seuil 20 lots = différenciation

Vision 2030 Réalisée
--------------------

Le succès de KoproGo ne se mesure **pas en dates** mais en **jalons atteints** :

* **100 copropriétés** → Validation product-market fit
* **500 copropriétés** → Viabilité économique + Features communautaires débloquées
* **1.000 copropriétés** → Impact social mesurable (107t CO₂/an)
* **2.000 copropriétés** → Leadership ESS PropTech (214t CO₂/an)
* **5.000 copropriétés** → Référence européenne (534t CO₂/an)

Chaque palier débloque le suivant. Pas de pression calendaire, mais des **conditions objectives**.

.. note::
   **"La technologie au service du bien commun, pas du profit"**

   En 2030 - ou quand 5.000 copropriétés seront atteintes - KoproGo aura :

   **Impact Économique Total**

   * Économisé **1M€/an** aux utilisateurs (vs abonnements marché)
   * Généré **750k€ économie circulaire/an** via SEL (30% adoption)
   * Évité **600k€ achats/an** via partage objets
   * **Total : 2,35M€/an** réinjectés dans l'économie locale

   **Impact Environnemental Total**

   * Évité **50t CO₂/an** infrastructure (optimisation Rust)
   * Évité **484t CO₂/an** features communautaires (partage/réutilisation)
   * **Total : 534 tonnes CO₂/an** évitées (10x impact initial !)
   * **12.000 objets partagés** en circulation permanente

   **Impact Social Mesurable**

   * **36.000h services échangés/an** entre voisins (SEL + Skills)
   * **6.250 objets réutilisés/an** (swap shop, économie circulaire)
   * **5.000 annonces/an** (notice board, vie communautaire)
   * Créé **10-15 emplois** équitables dans économie sociale
   * Construit une **coopérative** de 2.000 sociétaires actifs

   **Démonstration**

   * Prouvé qu'**open source + ESS = modèle viable**
   * Prouvé que **features sociales 10x l'impact écologique**
   * Inspiré réplication modèle autres secteurs

=========================================================
Documents de Référence
=========================================================

* :doc:`VISION` - Notre mission sociale
* :doc:`MISSION` - Objectifs et principes
* :doc:`ECONOMIC_MODEL` - Modèle économique détaillé
* :doc:`PERFORMANCE_REPORT` - Tests de charge Nov 2025
* :doc:`GOVERNANCE` - Structure ASBL/Coopérative
* :doc:`ROADMAP` - Roadmap technique 2025-2026

---

*Vision Stratégique KoproGo v3.0 - Novembre 2025*
*Document vivant - Mise à jour par palier de croissance*
*Contact : contact@koprogo.com - GitHub : github.com/gilmry/koprogo*
