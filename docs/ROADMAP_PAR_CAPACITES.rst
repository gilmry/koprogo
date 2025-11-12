=========================================================
KoproGo - Roadmap par Capacités
=========================================================

:Version: 4.0
:Modèle: Progression par jalons et métriques
:Auteurs: Gilles & Farah - Co-fondateurs KoproGo ASBL
:Statut: Document de référence stratégique

.. contents:: Table des matières
   :depth: 3
   :local:

=========================================================
Philosophie: Pas de Dates, Des Capacités
=========================================================

**Principe fondamental** : KoproGo avance quand les **conditions sont remplies**, pas selon un calendrier arbitraire.

Au lieu de promettre "Jalon 1 en décembre", nous disons:

**"Jalon 1 débloque 50-100 copropriétés quand Sécurité + GDPR basique sont implémentés"**

Pourquoi ce Modèle?
-------------------

**Dates fixes = pression artificielle**

* Stress d'équipe
* Qualité compromise
* Burnout fondateurs
* Promesses non tenues

**Capacités mesurables = progression saine**

* On livre quand c'est prêt
* Qualité préservée
* Équipe soutenable
* Confiance utilisateurs

=========================================================
Métriques de Progression: Les Deux Moteurs
=========================================================

Le succès de KoproGo se mesure par **deux métriques fondamentales**:

1. **Nombre de copropriétés hébergées**

   * Mesure: Adoption technique et proposition de valeur
   * Déclencheur: Features débloquent nouveaux paliers
   * Impact: Validation product-market fit

2. **Nombre de participants au projet**

   * Mesure: Contributeurs + sociétaires + bénévoles
   * Déclencheur: Vélocité de développement augmente
   * Impact: Viabilité communautaire

**Effet multiplicateur**: Plus de copropriétés → Plus de revenus → Plus de contributeurs → Plus de features → Plus de copropriétés → **Cercle vertueux**

Paliers de Croissance Infrastructure
-------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 20 15 15 15 20

   * - Palier
     - Capacité
     - Infrastructure
     - Coût/mois
     - Déclencheur
     - Temps Estimé
   * - **Nano**
     - 100 copros
     - VPS s1-2 (1c/2GB)
     - 6,30€
     - MVP lancé
     - Phase bootstrap
   * - **Micro**
     - 500 copros
     - VPS s1-4 (1c/4GB)
     - 17,70€
     - >80 copros
     - +3-6 mois
   * - **Small**
     - 1.500 copros
     - VPS b2-7 (2c/7GB)
     - 43,50€
     - >400 copros
     - +6-12 mois
   * - **Medium**
     - 3.000 copros
     - VPS b2-15 (4c/15GB)
     - 87€
     - >1.200 copros
     - +12-18 mois
   * - **Large**
     - 10.000+ copros
     - K8s cluster
     - 290€
     - >2.500 copros
     - +24-36 mois

**Principe clé**: L'infrastructure évolue **automatiquement** quand le seuil de 80% de capacité est atteint. Pas de date fixe, mais des **conditions mesurables**.

=========================================================
Jalons Produit: Features Débloquant la Croissance
=========================================================

Jalon 0: Fondations Techniques ✅
----------------------------------

**État**: Achevé (Automne 2025)

**Ce qui a été accompli**:

* ✅ Architecture hexagonale implémentée
* ✅ 73 endpoints API REST
* ✅ Tests E2E Playwright
* ✅ Load tests validés (99.74% success, 287 req/s)
* ✅ Documentation Sphinx publiée

**Capacité débloquée**: 10-20 early adopters (beta fermée)

**Conformité légale**: 30% (features CRUD de base)

**Ce que ça permet**:

* Valider l'architecture technique
* Tester avec premiers utilisateurs
* Identifier les besoins réels
* Construire une base solide pour la suite

Jalon 1: Sécurité & GDPR
-------------------------

**Débloque**: 50-100 copropriétés (beta publique possible)

**Issues critiques**:

* #39 : LUKS Encryption at-rest
* #40 : Backups automatisés GPG + S3
* #42 : GDPR basique (export + effacement)
* #48 : Authentification forte (itsme®)

**Livrables**:

* Données chiffrées au repos
* Backups quotidiens testés
* Conformité GDPR Articles 15 & 17
* Auth multi-facteur opérationnelle

**Conformité légale**: 40%

**Ce que ça débloques**:

* **Confiance juridique**: Données protégées légalement
* **Beta publique**: On peut ouvrir sans risque GDPR
* **Premiers revenus**: Cloud géré devient possible
* **Réputation**: "Sécurité d'abord" attire utilisateurs exigeants

**Effort estimé selon force de travail**:

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/semaine
     - Durée estimée
     - Équipe
   * - **Solo bootstrap**
     - 10-20h
     - 2-3 mois
     - Gilles + Farah bénévole
   * - **Duo fondateurs**
     - 40-60h
     - 6-8 semaines
     - +1 dev backend Rust
   * - **Équipe projet**
     - 80-120h
     - 3-4 semaines
     - +DevOps +Security
   * - **Communauté active**
     - 200+h
     - 2-3 semaines
     - +10 contributeurs OSS

**Note**: La durée dépend de la force de travail disponible, pas d'une date arbitraire.

Jalon 2: Conformité Légale Belge
---------------------------------

**Débloque**: 200-500 copropriétés (production ouverte)

**Issues critiques**:

* #16 : Plan Comptable Normalisé Belge (PCB)
* #17 : État Daté (bloque ventes immobilières)
* #18 : Budget Prévisionnel Annuel
* #22 : Conseil de Copropriété (obligatoire >20 lots)
* #23 : Workflow Recouvrement Impayés

**Livrables**:

* Comptabilité conforme arrêté royal 12/07/2012
* Génération états datés automatique
* Budgets avec variance analysis
* Dashboard conseil avec alertes
* Relances automatiques 3 niveaux

**Conformité légale**: 80%

**Bloquants levés**:

* **État daté**: Permet ventes de lots (CRITIQUE pour adoption)
* **Conseil copropriété**: Débloque copros >20 lots (60% du marché belge)
* **Comptabilité conforme**: Crédibilité auprès syndics professionnels

**Ce que ça débloques**:

* **60% du marché**: Copropriétés >20 lots deviennent accessibles
* **Ventes immobilières**: État daté = feature critique pour notaires
* **Syndics professionnels**: Conformité légale = crédibilité
* **Croissance organique**: Bouche-à-oreille entre syndics

**Effort estimé selon force de travail**:

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/semaine
     - Durée estimée
     - Note
   * - **Solo bootstrap**
     - 10-20h
     - 4-6 mois
     - Complexité comptable
   * - **Duo fondateurs**
     - 40-60h
     - 8-12 semaines
     - Expertise comptable requise
   * - **Équipe projet**
     - 80-120h
     - 4-6 semaines
     - +Expert comptable copro
   * - **Communauté active**
     - 200+h
     - 3-4 semaines
     - Parallélisation possible

**Dépendances**: Jalon 1 (Sécurité) DOIT être terminé avant (données sensibles)

Jalon 3: Features Différenciantes
----------------------------------

**Débloque**: 500-1.000 copropriétés (différenciation marché)

**Issues importantes**:

* #46 : Voting Digital (scrutins AG conformes)
* #47 : PDF Generation étendue
* #49 : Module SEL (Système Échange Local)
* #26 : Partage d'Objets
* #52 : Contractor Backoffice

**Livrables**:

* Votes AG avec signature itsme®
* Templates PDF tous documents légaux
* Monnaie locale virtuelle intégrée
* Bibliothèque objets partagés
* Espace prestataires

**Conformité légale**: 90%

**Avantage compétitif**: Features communautaires uniques (mission ASBL)

**Ce que ça débloques**:

* **Différenciation**: SEL + Partage = unique sur le marché
* **Impact social**: Modules communautaires créent lien social
* **Impact écologique**: 790 tonnes CO₂/an évitées (partage objets)
* **Économie circulaire**: 750k€/an échanges SEL (30% adoption)
* **Marketing naturel**: "La plateforme avec communauté" = viral

**Effet multiplicateur**: Les features sociales amplifient l'impact écologique x16 !

**Effort estimé selon force de travail**:

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/semaine
     - Durée estimée
     - Parallélisation
   * - **Solo bootstrap**
     - 10-20h
     - 5-8 mois
     - Modules indépendants
   * - **Duo fondateurs**
     - 40-60h
     - 10-14 semaines
     - SEL + Voting // Partage
   * - **Équipe projet**
     - 80-120h
     - 5-7 semaines
     - 3 tracks parallèles
   * - **Communauté active**
     - 200+h
     - 3-4 semaines
     - 5 tracks parallèles

**Dépendances**: Jalon 2 (Conformité) recommandé mais non bloquant pour modules communautaires

Jalon 4: Automation & Intégrations
-----------------------------------

**Débloque**: 1.000-2.000 copropriétés (scalabilité)

**Issues**:

* #19 : Convocations AG automatiques
* #20 : Carnet d'Entretien Digital
* #21 : GDPR complet (Articles 16, 18, 21)
* #24 : Module Devis Travaux
* #25 : Affichage Public Syndic
* #27 : Accessibilité WCAG 2.1 AA

**Livrables**:

* Workflow AG 100% automatisé
* Carnet maintenance avec alertes
* GDPR compliance totale
* Comparaison devis multi-entrepreneurs
* Page publique syndic (SEO)
* Accessibilité complète

**Conformité légale**: 95%

**Ce que ça débloques**:

* **Automation**: Temps syndic réduit de 50% (AG automatiques)
* **Accessibilité**: Conformité European Accessibility Act 2025
* **SEO**: Pages publiques syndics → discovery organique
* **Professionnalisation**: Outils niveau entreprise

**Effort estimé selon force de travail**:

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/semaine
     - Durée estimée
     - Focus
   * - **Solo bootstrap**
     - 10-20h
     - 6-10 mois
     - Automation prioritaire
   * - **Duo fondateurs**
     - 40-60h
     - 12-16 semaines
     - Workflow + GDPR
   * - **Équipe projet**
     - 80-120h
     - 6-8 semaines
     - 4 tracks parallèles
   * - **Communauté active**
     - 200+h
     - 4-5 semaines
     - 6 tracks parallèles

**Dépendances**: Jalon 3 recommandé pour avoir base utilisateurs suffisante (feedback)

Jalon 5: Mobile & API Publique
-------------------------------

**Débloque**: 2.000-5.000 copropriétés (expansion)

**Features**:

* PWA mobile responsive
* API publique v1 documentée (OpenAPI)
* Multi-langue NL/FR/DE/EN complet
* Intégrations comptables (Winbooks, Exact)
* Notifications intelligentes
* Analytics & Dashboards

**Livrables**:

* Progressive Web App installable
* SDK Python/JS/PHP
* Webhooks pour événements
* Export Winbooks/Exact Online
* Digest hebdomadaire personnalisé
* KPIs syndic temps réel

**Conformité légale**: 100%

**Ce que ça débloques**:

* **Écosystème**: API publique → développeurs tiers
* **Intégrations**: Winbooks/Exact → syndics professionnels
* **Mobile**: PWA → adoption copropriétaires
* **International**: Multi-langue → expansion EU

**Effort estimé selon force de travail**:

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/semaine
     - Durée estimée
     - Équipe recommandée
   * - **Équipe projet**
     - 80-120h
     - 14-18 semaines
     - +Mobile dev +API architect
   * - **Communauté active**
     - 200+h
     - 6-8 semaines
     - 8 tracks parallèles

**Note**: Ce jalon nécessite une équipe structurée (mobile + API expertise)

Jalon 6: Intelligence & Expansion
----------------------------------

**Débloque**: 5.000-10.000 copropriétés (leadership)

**Features avancées**:

* IA Assistant Syndic (GPT-4/Claude)
* API Bancaire PSD2 (réconciliation auto)
* Marketplace Services Locaux
* Prédictions budgétaires (ML)
* Multi-region (Benelux)

**Livrables**:

* Chatbot réglementaire
* Import transactions bancaires
* Annuaire prestataires vérifiés
* Modèles ARIMA prévisions charges
* Adaptation législation NL/LU

**Ce que ça débloques**:

* **IA Syndic**: Réponses réglementaires instantanées
* **PSD2**: Réconciliation bancaire automatique
* **Marketplace**: Économie de plateforme
* **Expansion**: Benelux → 3M copropriétés potentielles

**Effort estimé selon force de travail**:

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/semaine
     - Durée estimée
     - Équipe
   * - **Équipe élargie**
     - 200+h
     - 18-24 semaines
     - +Data scientist +MLOps

**Note**: Jalon ambitieux nécessitant maturité produit + équipe spécialisée

Jalon 7: Platform Economy
--------------------------

**Débloque**: 10.000+ copropriétés (scale planétaire)

**Vision long terme**:

* SDK multi-langages pour développeurs
* Store modules tiers (marketplace)
* Blockchain pour votes (immutabilité)
* Carbon Credits Trading
* White-label pour fédérations

**Ce que ça débloques**:

* **Écosystème complet**: Développeurs tiers créent valeur
* **Expansion EU**: France, Espagne, Italie (3M+ copros)
* **Leadership**: Référence européenne PropTech ESS

**Effort estimé**: Organisation mature, 10-15 ETP

=========================================================
Progression Conditionnelle: Force de Travail
=========================================================

Le Facteur Multiplicateur
--------------------------

**Principe clé**: La vélocité dépend de la force de travail disponible.

.. list-table::
   :header-rows: 1
   :widths: 20 15 20 15 30

   * - Configuration
     - Heures/sem
     - Jalons/an
     - Coût RH
     - Financement
   * - **Solo**
     - 10-20h
     - 2-3
     - 0€
     - Bénévolat
   * - **Duo**
     - 40-60h
     - 4-5
     - 18k€
     - Premiers revenus
   * - **Équipe**
     - 80-120h
     - 6-8
     - 60k€
     - Revenus cloud
   * - **Communauté**
     - 200+h
     - 10-12
     - 120k€
     - Surplus ASBL

**Scénarios de Croissance Réalistes**

**Scénario 1: Bootstrap Solo** (Gilles + Farah bénévole, 20h/sem total)

.. code-block:: text

   Mois 0-3 : Jalon 1 (Sécurité)
     → 50-100 copros débloquées
     → Premiers revenus: 200-300€/mois

   Mois 4-9 : Jalon 2 (Conformité)
     → 200-500 copros débloquées
     → Revenus: 1.000-1.500€/mois
     → Peut embaucher 1 dev temps partiel

   Mois 10-18 : Jalon 3 (Différenciation)
     → 500-1.000 copros
     → Revenus: 2.500-3.500€/mois
     → Duo fondateurs possibles

   Total: 18-24 mois pour atteindre 1.000 copropriétés

**Scénario 2: Duo Fondateurs** (40-60h/sem total)

.. code-block:: text

   Mois 0-2 : Jalon 1 (Sécurité)
     → 50-100 copros
     → Revenus: 200-300€/mois

   Mois 3-5 : Jalon 2 (Conformité)
     → 200-500 copros
     → Revenus: 1.000-1.500€/mois

   Mois 6-9 : Jalon 3 (Différenciation)
     → 500-1.000 copros
     → Revenus: 2.500-3.500€/mois

   Mois 10-14 : Jalon 4 (Automation)
     → 1.000-2.000 copros
     → Revenus: 5.000-7.000€/mois
     → Équipe 3-5 personnes possible

   Total: 14-18 mois pour atteindre 2.000 copropriétés

**Scénario 3: Équipe Projet** (80-120h/sem total)

.. code-block:: text

   Mois 0-1 : Jalon 1 (Sécurité)
     → 50-100 copros

   Mois 2-3 : Jalon 2 (Conformité)
     → 200-500 copros

   Mois 4-6 : Jalon 3 (Différenciation)
     → 500-1.000 copros

   Mois 7-9 : Jalon 4 (Automation)
     → 1.000-2.000 copros

   Mois 10-14 : Jalon 5 (Mobile + API)
     → 2.000-5.000 copros
     → Revenus: 15.000-25.000€/mois
     → Organisation 10+ personnes

   Total: 14-18 mois pour atteindre 5.000 copropriétés

**Principe**: Chaque scénario est valide. La rapidité dépend des ressources, pas de promesses marketing.

Effet Boule de Neige
---------------------

**Phase 1: Traction Initiale** (0-100 copros)

* Bénévolat fondateurs
* Feedback qualitatif
* Itérations rapides
* Coûts minimaux (< 10€/mois)

**Phase 2: Premiers Revenus** (100-500 copros)

* Revenus: 200-1.500€/mois
* Autofinancement partiel
* Contributions OSS augmentent
* Infrastructure scale up (20€/mois)

**Phase 3: Équipe Viable** (500-1.000 copros)

* Revenus: 1.500-3.500€/mois
* 1er emploi temps partiel possible
* Communauté active (10+ contributeurs)
* Infrastructure mature (50€/mois)

**Phase 4: Organisation Stable** (1.000-5.000 copros)

* Revenus: 3.500-25.000€/mois
* Équipe 3-10 personnes
* ASBL structurée
* Infrastructure scalable (100-300€/mois)

**Phase 5: Leadership ESS** (5.000+ copros)

* Revenus: 25.000+€/mois
* Organisation 10-15 ETP
* Coopérative possible
* Expansion européenne

=========================================================
Vision Holistique: Tout S'Articule
=========================================================

L'Architecture des Dépendances
-------------------------------

**Pourquoi cette séquence de jalons?**

.. code-block:: text

   Jalon 0: Fondations
     ↓ Nécessaire pour
   Jalon 1: Sécurité
     ↓ Débloque confiance → Beta publique
     ↓ Sans ça: Pas d'utilisateurs réels
   Jalon 2: Conformité
     ↓ Débloque légalité → Production ouverte
     ↓ Sans ça: Illégal pour copros >20 lots (60% marché)
   Jalon 3: Différenciation
     ↓ Débloque impact social → Viralité
     ↓ Sans ça: Clone des concurrents
   Jalon 4: Automation
     ↓ Débloque efficacité → Scale
     ↓ Sans ça: Support non soutenable
   Jalon 5: Mobile + API
     ↓ Débloque écosystème → Expansion
     ↓ Sans ça: Platform monocanal
   Jalon 6-7: Intelligence & Platform
     ↓ Leadership européen ESS PropTech

**Chaque jalon est une fondation pour le suivant.**

Les Trois Piliers Interconnectés
---------------------------------

**1. Technique → Économique → Social**

.. code-block:: text

   Architecture Rust (Technique)
     ↓ Permet
   Infrastructure 99% moins chère (Économique)
     ↓ Permet
   Prix cassés 2-8€/mois (Social)
     ↓ Permet
   Adoption massive (Technique)
     → Cercle vertueux

**2. Conformité → Crédibilité → Adoption**

.. code-block:: text

   GDPR + PCN Belge (Conformité)
     ↓ Donne
   Crédibilité syndics pro (Crédibilité)
     ↓ Génère
   Bouche-à-oreille (Adoption)
     ↓ Finance
   Nouvelles features (Conformité++)
     → Cercle vertueux

**3. Communauté → Impact → Viralité**

.. code-block:: text

   Features SEL + Partage (Communauté)
     ↓ Créent
   Lien social + Économie 750k€ (Impact)
     ↓ Génèrent
   Témoignages utilisateurs (Viralité)
     ↓ Attirent
   Nouveaux membres (Communauté++)
     → Cercle vertueux

**L'interconnexion est la clé de la viabilité.**

Métriques de Succès Holistiques
--------------------------------

**Au lieu de KPIs 2030, des paliers mesurables:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 20 20 20

   * - Palier
     - Copros
     - Participants
     - Impact CO₂
     - Économie
   * - **Validation**
     - 100
     - 10
     - -2t/an
     - 20k€/an
   * - **Viabilité**
     - 500
     - 50
     - -15t/an
     - 100k€/an
   * - **Impact**
     - 1.000
     - 100
     - -107t/an
     - 350k€/an
   * - **Leadership**
     - 2.000
     - 200
     - -214t/an
     - 750k€/an
   * - **Référence**
     - 5.000
     - 500
     - -840t/an
     - 2,35M€/an

**Chaque palier valide la proposition de valeur à une échelle supérieure.**

=========================================================
Avantages du Modèle par Capacités
=========================================================

Pour l'Équipe
-------------

✅ **Pas de pression calendaire**

* On livre quand c'est prêt
* Qualité préservée
* Burnout évité

✅ **Planification réaliste**

* Effort estimé selon force disponible
* Scenarios multiples (solo/duo/équipe)
* Flexibilité assumée

✅ **Motivation intrinsèque**

* Objectifs mesurables
* Progrès visible
* Impact concret

Pour les Utilisateurs
---------------------

✅ **Transparence totale**

* Pas de promesses vides
* Conditions de déblocage claires
* Dashboard progression public

✅ **Participation possible**

* Contributeurs accélèrent les jalons
* Feedback oriente les priorités
* Communauté co-construit

✅ **Confiance renforcée**

* Livraisons quand prêt = qualité
* Pas de "coming soon" éternel
* Roadmap honnête

Pour les Investisseurs / Subventions
-------------------------------------

✅ **Risques maîtrisés**

* Progression par paliers validés
* Métriques objectives
* Pas de promesses irréalistes

✅ **Scalabilité prouvée**

* Chaque palier démontre viabilité
* Infrastructure évolue avec adoption
* Modèle économique validé à chaque étape

✅ **Impact mesurable**

* CO₂ évité calculé par palier
* Économies générées documentées
* Social impact tracé

=========================================================
Plan d'Action: Prochaines Étapes
=========================================================

**Au lieu de "Semaine 1-2", des conditions à remplir:**

Prochaine Étape: Compléter Jalon 1
-----------------------------------

**Objectif**: Débloquer 50-100 copropriétés (beta publique)

**Conditions de déblocage**:

1. **Sécurité Infrastructure** ✅

   * [x] LUKS Encryption at-rest implémenté
   * [x] Backups GPG + S3 configurés
   * [x] Monitoring Prometheus + Grafana
   * [x] Tests sécurité passés

2. **GDPR Basique** (En cours)

   * [ ] Export données utilisateur (Article 15)
   * [ ] Droit à l'oubli (Article 17)
   * [ ] Privacy policy v1.0
   * [ ] Tests GDPR automatisés

3. **Auth Forte** (Prochaine priorité)

   * [ ] Inscription itsme® (délai 2-4 semaines)
   * [ ] Intégration API itsme®
   * [ ] Fallback email/password
   * [ ] Tests auth E2E

**Effort restant estimé**:

* Solo (20h/sem): 4-6 semaines
* Duo (60h/sem): 2-3 semaines

**Déblocage automatique quand**: Tests GDPR + Auth forte passent tous

Ensuite: Attaquer Jalon 2
--------------------------

**Ne commence QUE quand Jalon 1 est complet** (données sécurisées)

**Conditions de déblocage**:

1. **Plan Comptable Belge**

   * PCMN complet (90 comptes pré-seedés) ✅
   * Tests conformité arrêté royal ✅
   * Documentation comptable

2. **État Daté** (Feature critique)

   * Génération PDF conforme
   * Signature syndic
   * Tests notaires (beta)

3. **Conseil Copropriété**

   * Dashboard conseil >20 lots
   * Alertes réglementaires
   * Workflow validation

**Effort estimé**:

* Solo: 4-6 mois
* Duo: 8-12 semaines
* Équipe: 4-6 semaines

=========================================================
Conclusion: Progression Saine et Soutenable
=========================================================

Ce Que Nous Promettons
-----------------------

✅ **Qualité avant vitesse**

* Chaque jalon est solide
* Tests exhaustifs
* Documentation complète

✅ **Transparence totale**

* Conditions de déblocage publiques
* Progression mesurable
* Pas de dates artificielles

✅ **Flexibilité assumée**

* Vélocité dépend de la force de travail
* Scenarios multiples (solo/équipe/communauté)
* Aucune pression calendaire

Ce Que Nous NE Promettons PAS
------------------------------

❌ **Dates fixes**

* "Livré en décembre" → Fausse promesse
* Préférons: "Livré quand sécurité validée"

❌ **Roadmap rigide**

* Priorités peuvent changer selon feedback
* Jalons peuvent fusionner/splitter
* Communauté influence direction

❌ **Croissance à tout prix**

* Privilégions qualité sur quantité
* Pas de dette technique
* Équipe soutenable

Notre Engagement
----------------

**"Nous livrons quand c'est prêt, pas quand le calendrier le dit."**

Chaque jalon débloque des capacités mesurables. Chaque palier valide la viabilité. Chaque feature crée de l'impact réel.

**La patience et la qualité battent toujours la vitesse et les promesses vides.**

=========================================================
Documents de Référence
=========================================================

* :doc:`VISION` - Vision macro et problème sociétal
* :doc:`MISSION` - Mission holistique et valeurs
* :doc:`ECONOMIC_MODEL` - Viabilité économique
* :doc:`GOVERNANCE` - Structure ASBL évolutive
* :doc:`PERFORMANCE_REPORT` - Validation technique

---

*Roadmap par Capacités KoproGo v4.0*
*Modèle de progression mesurable et soutenable*
*Contact : contact@koprogo.com - GitHub : github.com/gilmry/koprogo*
