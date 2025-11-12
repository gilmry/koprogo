========================================
Mission KoproGo ASBL
========================================

:Version: 4.0
:Modèle: Progression par capacités et métriques
:Voir aussi: :doc:`VISION` | :doc:`ROADMAP_PAR_CAPACITES` | :doc:`ECONOMIC_MODEL`

Notre Raison d'Être
-------------------

La mission de KoproGo ASBL est de **résoudre les problèmes de société liés à la gestion des copropriétés et à l'isolement urbain** tout en adoptant des **pratiques technologiques à la pointe de l'écologie, de la sécurité, et du développement collaboratif**\ , alignées avec les standards du monde opensource.

Au-delà de la simple gestion administrative, KoproGo vise à **recréer du lien social** entre habitants d'un même immeuble via des modules communautaires optionnels, activables par chaque copropriété selon ses besoins.

Piliers de la Mission
---------------------

1. Mutualisation & Économies d'Échelle Inversées
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Objectif** : Démontrer que la croissance profite à **tous** les participants, pas aux actionnaires.

Le Concept de Démocratie Tarifaire
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Économies d'échelle traditionnelles** :

* Plus de clients → Marges accrues → Profits actionnaires ↑
* Prix restent fixes ou augmentent
* Usagers ne bénéficient jamais de l'échelle

**Démocratie tarifaire (KoproGo ASBL)** :

* Plus de participants → Coûts dilués → **Prix voté collectivement** ↓
* Surplus réinvesti dans le projet (features, infra, baisse tarifaire)
* Chaque nouveau participant **enrichit** les précédents
* **L'Assemblée Générale décide** de l'évolution du prix

Actions Concrètes
~~~~~~~~~~~~~~~~~

**1. Prix Fixe de Départ + Vote Démocratique**

**Prix de départ** (validé à la création ASBL):

* **Cloud géré**: **5€/mois** par copropriété
* **Self-hosted**: **Gratuit** (toujours)

**Évolution du prix**: Uniquement par **vote en Assemblée Générale** (ASBL → Coopérative)

**Principe**: Si les revenus dépassent largement les coûts, l'AG peut voter pour baisser le prix (ex: 5€ → 4€ → 3€ selon croissance et surplus).

**Exemple**: "Rapport AG: surplus 25.000€/an (marge 80%). Proposition CA: baisse à 4€/mois. Vote: 87% pour, 13% abstention. ✅ Adopté"

**2. Transparence Comptable Trimestrielle**

* **Publication coûts réels** infrastructure (serveurs, bande passante, stockage)
* **Calcul prix coûtant réel** vs prix facturé (tableau comparatif)
* **Dashboard public temps réel** : `/transparency` (nb copros, coûts, prix coûtant, surplus)
* **Si surplus > 25%** : l'AG peut voter baisse tarifaire ou redistribution

**3. Budget Participatif Annuel**

AG vote allocation surplus (si revenus > coûts+réserve) :

**Options vote** :

* Nouvelles features (vote priorités communauté)
* Amélioration infra (performance, sécurité)
* **Baisse tarifaire** (si surplus > 25%)
* Constitution réserve légale (3 mois coûts)
* Ristournes sociétaires (modèle coopératif)

**Exemple AG** (1.500 copros, prix 5€, surplus 72.000€/an) :

.. code-block:: text

   Rapport CA :
     • Revenus: 90.000€/an (1.500 copros × 5€/mois × 12)
     • Coûts infra: 2.160€/an
     • Coûts RH: 15.840€/an (2 devs temps partiel)
     • Surplus: 72.000€ (80% marge)

   Proposition CA :
     • 40% Baisse tarifaire (5€ → 3,50€) - Économies pour tous
     • 30% Features prioritaires (21.600€) - Vote communauté
     • 20% Réserve légale (14.400€) - Sécurité
     • 10% R&D PropTech (7.200€) - IA/Blockchain/IoT

   Vote AG (1 membre = 1 voix) :
     ✅ Adopté : 87% pour, 13% abstention

**4. Contributions Valorisées**

Qui contribue au bien commun en bénéficie davantage :

* **Contributeurs code/docs/traductions** : **-50%** tarif cloud (ex: 2,50€ au lieu de 5€)
* **Mainteneurs actifs** : **Gratuit à vie**
* **Principe** : Le bénévolat est récompensé concrètement

Impact par Paliers (objectif 5.000 copros)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Scénario Participation Croissante** (prix fixe 5€, évolution par vote AG):

.. code-block:: text

   Palier 1 : 100 copros → 5€/mois → Revenus: 60.000€/an
     Coûts infra : 6.300€/an
     Surplus : 53.700€ (89%) → Constitution réserve + 1er dev
     Prix maintenu à 5€ (phase bootstrap)

   Palier 2 : 500 copros → 5€/mois → Revenus: 300.000€/an
     Coûts infra : 21.200€/an
     Coûts RH : 72.000€/an (2 devs)
     Surplus : 206.800€ (69%)
     → AG vote baisse à 4€/mois (surplus > 25%)

   Palier 3 : 1.500 copros → 4€/mois → Revenus: 288.000€/an
     Coûts infra : 52.200€/an
     Coûts RH : 150.000€/an (5 ETP)
     Surplus : 85.800€ (30%)
     → Prix maintenu à 4€ (réinvestissement features)

   Palier 4 : 5.000 copros → 4€/mois → Revenus: 960.000€/an
     Coûts infra : 348.000€/an
     Coûts RH : 350.000€/an (10-15 ETP)
     Surplus : 262.000€ (27%)
     → AG vote baisse à 3€/mois (surplus > 25%)

Effet Cercle Vertueux
~~~~~~~~~~~~~~~~~~~~~~

* **Plus de participants** → Surplus augmente → AG peut voter baisse → Attractivité ↑
* **Attractivité ↑** → Nouveaux participants → Communauté ↑
* **Communauté ↑** → Contributions ↑ → Qualité produit ↑
* **Qualité ↑** → Satisfaction ↑ → Bouche-à-oreille ↑
* **Bouche-à-oreille ↑** → Nouveaux participants → **Cycle se répète**

Comparaison Modèles
~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 35 35

   * - Critère
     - SaaS Classique
     - KoproGo ASBL
   * - Prix/copro
     - 50-500€/mois fixe
     - **5€/mois** (voté démocratiquement)
   * - Évolution prix
     - Hausse annuelle
     - **Baisse si AG vote** (surplus > 25%)
   * - Bénéfice échelle
     - Actionnaires (90%)
     - **Tous participants (100%)**
   * - Gouvernance
     - Fermée (CEO)
     - **Ouverte (AG, 1=1 voix)**
   * - Transparence
     - Aucune
     - **Comptabilité publique**
   * - Contribution
     - Impossible
     - **Valorisée (-50% tarif)**

**Avantages du modèle**:

✅ **Simplicité**: Un seul prix, facile à comprendre (5€)
✅ **Démocratie**: La communauté décide quand et comment baisser
✅ **Transparence**: Comptabilité publique trimestrielle
✅ **Flexibilité**: L'AG choisit entre baisse, features, réserve, ristournes
✅ **Objectif de baisse**: Reste notre mission, mais décidé collectivement

.. note::
   **Détails complets** : Projections financières et transparence dans :doc:`ECONOMIC_MODEL` et :doc:`ROADMAP_PAR_CAPACITES`.

2. Résoudre un Problème Sociétal
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Transparence et Justice
~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Rendre la gestion de copropriété transparente, équitable et accessible à tous.

**Actions** :


* ✅ Calculs de charges vérifiables et auditables
* ✅ Historique complet des décisions (assemblées générales)
* ✅ Accès égalitaire aux documents (syndic et copropriétaires)
* ✅ Réduction des litiges par la transparence

**Impact** :


* Confiance restaurée entre syndics et copropriétaires
* Réduction de 50% des contestations de charges
* Gain de temps : 10h/mois par syndic

Lien Social et Dynamique Communautaire *(Modules Optionnels)*
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Combattre l'isolement urbain et recréer du lien entre voisins.

**Problème identifié** :


* 70% des habitants ne connaissent pas leurs voisins
* Ressources inexploitées (compétences, objets)
* Manque d'entraide locale
* Consommation individuelle excessive

**Actions** *(activables par copropriété)* :


* ✅ **SEL (Système d'Échange Local)** : Troc de compétences entre voisins (jardinage, bricolage, cours)
* ✅ **Bazar de troc** : Échange ou don d'objets entre habitants
* ✅ **Prêt d'objets** : Partage outils, échelles, tondeuse, etc.
* ✅ **Annuaire de compétences** : Qui sait faire quoi dans l'immeuble
* ✅ **Tableau d'affichage numérique** : Petites annonces locales, covoiturage, garde d'enfants

**Impact** *(objectif 2030, 30% adoption sur 5000 copros)* :


* Réduction isolement urbain : +30% de voisins connus
* Économie circulaire locale : 12,000 objets partagés (8 objets/copro actif)
* **Économie circulaire** : 750k€/an via échanges SEL
* **Consommation évitée** : 600k€ achats neufs grâce au partage
* **Impact carbone** : -790 tonnes CO₂/an (partage objets + réduction consommation)
* Entraide renforcée : 24,000 heures services échangés/an

**Note importante** : Ces modules sont **totalement optionnels** et configurables par le conseil de copropriété. Chaque immeuble décide librement d'activer ou non ces fonctionnalités selon sa culture et ses besoins.

Économie et Accessibilité
~~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Démocratiser l'accès à des outils de gestion professionnels.

**Actions** :


* ✅ Gratuit pour self-hosted (option toujours disponible)
* ✅ **Cloud géré** : 1,50-8€/mois selon taille et features (vs 200-500€/mois concurrents)
* ✅ Aucun coût de licence ni frais cachés
* ✅ Exportation données libre (CSV, JSON, PDF)
* ✅ **Grille tarifaire équitable** : alignée sur taille copropriété et frontières légales (détails :doc:`ROADMAP_INTEGREE_2025_2030`)

**Impact** :


* Économies : 1,600-9,500€/an par copropriété
* Budget réalloué vers travaux et entretien
* Accessibilité pour petites copropriétés (< 10 lots)

Souveraineté Numérique
~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Garantir la souveraineté des données des citoyens européens.

**Actions** :


* ✅ Hébergement local possible (self-hosted)
* ✅ Hébergement cloud en Europe (OVH GRA11, France)
* ✅ Conformité RGPD stricte (droit à l'oubli, portabilité)
* ✅ Pas de dépendance à des GAFAM

**Impact** :


* Données sous contrôle des utilisateurs
* Conformité réglementaire garantie
* Résilience face aux sanctions US (CLOUD Act)

3. Pratiques Technologiques à la Pointe
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Écologie et Performance
~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Minimiser l'empreinte carbone tout en maximisant les performances.

**Actions** :


* ✅ Architecture Rust (10x plus efficace que Python/Node.js)
* ✅ Datacenter bas carbone (GRA11 : 60g CO2/kWh)
* ✅ Latency P99 < 1s (752ms mesuré), throughput 287 req/s (expérience utilisateur optimale)
* ✅ Consommation : < 10W par VPS (2,000-3,000 copropriétés)

**Impact** *(infrastructure seule)* :


* **96% de réduction carbone** vs solutions actuelles
* ~50 tonnes CO₂/an économisées (5000 copros cloud, 2030)
* 0.12g CO2/requête (objectif < 0.5g largement dépassé grâce à Rust + datacenter bas carbone)
* **+ Features communautaires** : -790 tonnes CO₂/an supplémentaires (partage objets, réduction consommation)
* **Impact total 2030** : -840 tonnes CO₂/an (dépassement +57% vs objectif initial)

**Comparaison** :


* WordPress typique : 120 kg CO2/an (1 site)
* Solution SaaS moyenne : 50 kg CO2/an (1 copropriété)
* **KoproGo** : 0.0026 kg CO2/an par copropriété cloud

Sécurité et Conformité
~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Garantir la sécurité des données sensibles (RGPD, ePrivacy).

**Actions** :


* ✅ Chiffrement TLS 1.3 (SSL Let's Encrypt)
* ✅ JWT authentication avec rotation tokens
* ✅ Hashing passwords (Argon2id)
* ✅ Row-Level Security PostgreSQL (futur)
* ✅ Firewall UFW (ports 22, 80, 443 uniquement)
* ✅ Fail2ban (protection bruteforce SSH)
* ✅ GitOps : Patches sécurité en < 3 minutes

**Impact** :


* **0% d'instances obsolètes** (vs 70% self-hosted classique)
* Failles corrigées en < 3 minutes (vs semaines/mois)
* Conformité RGPD : Audit automatisé (sqlx compile-time checks)

**Approche GitOps pour l'Auto-Hébergement**

L'auto-hébergement traditionnel présente des défis de maintenance :


* De nombreuses instances ne bénéficient pas de mises à jour régulières
* Failles critiques peuvent persister
* Gestion technique parfois complexe

**Solution GitOps** :


* Service systemd vérifie GitHub toutes les 3 minutes
* Pull automatique des patches de sécurité
* Rollback automatique si health check échoue
* **100% des instances à jour** automatiquement

Performance et Scalabilité
~~~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Offrir des performances dignes de Google/Amazon sans leur infrastructure.

**Actions** :


* ✅ Actix-web (framework web le plus rapide au monde)
* ✅ PostgreSQL 15 avec indexes optimisés
* ✅ Connection pool configuré (8 connexions)
* ✅ Progressive Web App (offline-first)
* ✅ Benchmarks Criterion (régression detection)

**Impact** :


* Latency P99 : 752ms (charge soutenue, 1 vCPU VPS)
* Throughput : 287 req/s mesuré (charge soutenue)
* Memory : < 128MB par instance (8 GB RAM = 60+ instances)

4. Développement Collaboratif et Opensource
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Transparence du Code
~~~~~~~~~~~~~~~~~~~~

**Objectif** : Permettre à quiconque d'auditer, contribuer, et améliorer le code.

**Actions** :


* ✅ Licence AGPL-3.0 (copyleft fort)
* ✅ Code source public : https://github.com/gilmry/koprogo
* ✅ Contributions communautaires bienvenues
* ✅ Documentation exhaustive (Sphinx RST)

**Impact** :


* Confiance accrue (code auditable)
* Innovations communautaires (plugins, traductions)
* Formation développeurs (code exemplaire)

Standards Opensource
~~~~~~~~~~~~~~~~~~~~

**Objectif** : Suivre les meilleures pratiques du monde opensource.

**Actions** :


* ✅ Git + GitHub (versioning, issues, pull requests)
* ✅ CI/CD (GitHub Actions, tests automatiques)
* ✅ Semantic Versioning (v1.0.0, v1.1.0, v2.0.0)
* ✅ Changelog (CHANGELOG.md)
* ✅ Code of Conduct (CODE_OF_CONDUCT.md)

**Impact** :


* Contributions facilitées (workflow standard)
* Releases prévisibles (semantic versioning)
* Communauté respectueuse (code of conduct)

Gouvernance Évolutive : Solo Dev → Coopérative
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Garantir une gouvernance démocratique et transparente, évoluant avec la maturité du projet.

**Principe fondamental: Client = Membre**

Alignement total entre économique et gouvernance:

.. list-table:: Statut et Droits
   :header-rows: 1
   :widths: 25 25 25 25

   * - Statut
     - Accès Cloud
     - Paiement
     - Droit de Vote AG
   * - **Membre actif** (cloud)
     - ✅ Oui
     - 5€/mois
     - ✅ 1 voix
   * - **Membre inactif** (arrêt)
     - ❌ Non
     - 0€
     - ❌ 0 voix
   * - **Self-hosted**
     - N/A (autonome)
     - Gratuit
     - ⚠️ Consultation*

**Alignement total**:

* **Qui paie** → Qui décide (gouvernance)
* **Qui utilise** → Qui contribue (économique)
* **Qui bénéficie** → Qui vote (démocratie)

**Concrètement**:

* Acheter solution cloud → Membre ASBL/Coopérative automatiquement
* Paiement actif (5€/mois) → Droit de vote en AG
* Arrêt paiement → Perte accès cloud + Perte droit de vote

Pas de distinction "client" vs "membre", seulement **membre actif** (votant) ou **inactif** (sans droit de vote).

\*\ **Note self-hosted**: À clarifier si cotisation symbolique requise pour droit de vote, ou statut consultatif uniquement.

**Évolution progressive** (détails :doc:`ROADMAP_PAR_CAPACITES`) :


#. **Phase Bootstrap (2025)** : Solo dev bénévole (Gilmry)

   * Validation MVP et product-market fit
   * Premiers utilisateurs bêta (< 100 copros)
   * Développement bénévole (10-20h/semaine)

#. **Phase Fondateurs (2026)** : Noyau fondateur 2-3 personnes

   * Constitution structure légale (préparation ASBL)
   * Croissance 100 → 500 copropriétés
   * Premiers revenus cloud (autofinancement)

#. **Phase ASBL (Viabilité)** : Association Sans But Lucratif belge

   * ✅ Assemblée générale annuelle (décisions collectives)
   * ✅ Conseil d'administration élu (3-7 membres)
   * ✅ Comptes publics (bilans annuels)
   * ✅ Statuts ASBL belge (non-profit)
   * ✅ **Client cloud = Membre automatiquement** (droit de vote)
   * Croissance 500 → 5,000 copropriétés
   * Développement financé (1-2 ETP)

#. **Phase Coopérative (Leadership)** : Transformation optionnelle

   * **Si la communauté le souhaite** : ASBL → Coopérative agréée
   * Membres deviennent sociétaires (parts sociales)
   * Gouvernance renforcée (1 personne = 1 voix maintenue)
   * Éligibilité subventions économie sociale

**Impact gouvernance évolutive** :


* Aucun actionnaire, aucun profit (toutes phases)
* Excédents réinvestis dans le projet
* Décisions alignées avec la mission
* Protection contre la dérive commerciale
* Transparence radicale à chaque étape
* **Alignement économie-gouvernance** (qui paie = qui décide)

5. Utilisation de l'IA pour le Développement Collaboratif
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

IA Générative pour la Documentation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Produire une documentation exhaustive et maintenue à jour.

**Actions** :


* ✅ Claude Code pour génération RST
* ✅ Documentation Sphinx complète (backend, frontend, infrastructure)
* ✅ Diagrammes architecture (Mermaid, PlantUML)
* ✅ Exemples de code générés automatiquement

**Impact** :


* Documentation 100% synchronisée avec le code
* Onboarding développeurs : 1 jour (vs 1-2 semaines)
* Réduction time-to-contribution : 80%

IA pour les Tests et la Qualité
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Garantir une qualité de code maximale dès le premier commit.

**Actions** :


* ✅ Tests unitaires générés avec IA (TDD)
* ✅ Tests BDD Cucumber (Gherkin scenarios)
* ✅ Benchmarks Criterion (performance regression)
* ✅ Linting automatique (clippy, rustfmt)

**Impact** :


* Couverture tests : > 80% (objectif 100% domain layer)
* Bugs détectés avant production : 95%
* Code maintenable sur le long terme

Partage des Recettes IA
~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Démocratiser l'utilisation de l'IA pour le développement.

**Mission spécifique ASBL** :

..

   Promouvoir le partage du code et des pratiques IA à la pointe pour le développement collaboratif.


**Actions** :


* ✅ Tutoriels IA-assisted development (docs/)
* ✅ Prompts Claude Code documentés (CLAUDE.md)
* ✅ Workflow TDD + IA (tests first, puis implémentation)
* ✅ Recettes pour génération doc, tests, refactoring

**Impact** :


* Développeurs formés aux pratiques IA modernes
* Productivité × 3-5 (mesure interne)
* Code de qualité professionnelle dès le départ

**Philosophie "Code de la Bonne Manière Dès le Départ"** :

Au lieu de :


#. Code rapide et sale
#. Refactoring plus tard (jamais fait)
#. Dette technique accumulée

Avec l'IA :


#. Tests d'abord (TDD assisté par IA)
#. Code propre dès le départ (IA + pair programming)
#. Documentation synchronisée (IA + Sphinx)
#. Zéro dette technique

5. Pédagogie et Onboarding
^^^^^^^^^^^^^^^^^^^^^^^^^^

Documentation Pédagogique
~~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Former les développeurs aux bonnes pratiques (DDD, Hexagonal, Rust).

**Actions** :


* ✅ Guide Architecture Hexagonale (docs/backend/)
* ✅ Tutoriels Rust pour débutants
* ✅ Patterns DDD expliqués (Aggregates, Repositories, Services)
* ✅ Exemples concrets (Building, Unit, Expense)

**Impact** :


* Développeurs juniors formés en 1 mois
* Adoption Rust facilitée (courbe d'apprentissage réduite)
* Contribution possible après 1-2 semaines

Onboarding Contributeurs
~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Réduire la barrière d'entrée pour nouveaux contributeurs.

**Actions** :


* ✅ CONTRIBUTING.md (guide contribution)
* ✅ Setup automatisé (make dev, make test)
* ✅ Issues "good first issue" (débutants)
* ✅ Mentoring (Discord, GitHub Discussions)

**Impact** :


* Temps onboarding : 1 jour (vs 1-2 semaines classique)
* Première contribution : < 1 semaine
* Rétention contributeurs : 70% (objectif)

Formation Continue
~~~~~~~~~~~~~~~~~~

**Objectif** : Maintenir la communauté à jour sur les dernières pratiques.

**Actions** :


* ✅ Blog technique (Medium, Dev.to)
* ✅ Talks conférences (Rust Belgium, FOSDEM)
* ✅ Workshops IA + Rust (universités, écoles)
* ✅ Vidéos tutoriels (YouTube)

**Impact** :


* Visibilité projet : 10,000+ vues/an
* Contributeurs recrutés : 20-50/an
* Formation étudiants : 100-200/an

6. Standards et Conformité
^^^^^^^^^^^^^^^^^^^^^^^^^^

RGPD et ePrivacy
~~~~~~~~~~~~~~~~

**Objectif** : Conformité stricte RGPD et respect de la vie privée.

**Actions** :


* ✅ Data minimization (uniquement données nécessaires)
* ✅ Droit à l'oubli (DELETE /users/:id)
* ✅ Portabilité (export CSV, JSON)
* ✅ Consentement explicite (cookies, analytics)
* ✅ DPO désigné (Data Protection Officer)

**Impact** :


* Conformité 100% RGPD
* Audit CNIL/APD réussi
* Confiance utilisateurs restaurée

Accessibilité (WCAG 2.1)
~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Rendre l'application accessible à tous (handicap visuel, moteur).

**Actions** :


* ✅ Contraste AA (4.5:1 texte/fond)
* ✅ Navigation clavier (tab, enter, escape)
* ✅ Lecteurs d'écran (ARIA labels)
* ✅ Responsive mobile (< 576px)

**Impact** :


* Accessibilité 100% utilisateurs
* Conformité législation EU (European Accessibility Act 2025)

Internationalisation (i18n)
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Objectif** : Support multilingue (Belgique = 3 langues officielles).

**Actions** :


* ✅ svelte-i18n (frontend)
* ✅ Traductions nl, fr, de, en
* ✅ Dates/nombres localisés
* ✅ Fallback automatique (nl par défaut)

**Impact** :


* Adoption Belgique : Flandre + Wallonie + Bruxelles
* Expansion Europe facilitée (FR, DE, NL, ES, IT)

Mesure de l'Impact Mission
--------------------------

Indicateurs par Paliers de Croissance
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Notre succès se mesure par paliers, pas par années**

.. list-table:: Métriques par Palier
   :header-rows: 1
   :widths: 20 15 15 15 15 20

   * - Palier
     - Copros
     - Personnes
     - CO₂/an
     - Économies
     - Contributeurs
   * - **Validation**
     - 100
     - 2.000
     - -2t
     - 160k€
     - 10
   * - **Viabilité**
     - 500
     - 10.000
     - -15t
     - 850k€
     - 50
   * - **Impact**
     - 1.000
     - 20.000
     - -107t
     - 2M€
     - 100
   * - **Leadership**
     - 2.000
     - 40.000
     - -214t
     - 4,5M€
     - 200
   * - **Référence**
     - 5.000
     - 100.000
     - **-840t**
     - **9,35M€**
     - 500

**Impact Social par Palier** *(modules communautaires optionnels)* :

* **Validation** (100 copros): Tests des modules communautaires
* **Viabilité** (500 copros): 20% adoptent SEL/Partage → 100+ échanges/mois
* **Impact** (1.000 copros): 500+ objets partagés en circulation
* **Leadership** (2.000 copros): Économie circulaire établie
* **Référence** (5.000 copros): 1.000+ copros avec fonctions communautaires actives

**Performance Technique** (maintenue à tous les paliers) :

* Latency P99 : < 1s (752ms validé)
* Throughput : 287 req/s soutenu
* Uptime : > 99.9%
* Security : 0 CVE non patchées
* Coût cloud : 1,50-8€/mois vs 200-500€ concurrents

**Formation et Communauté** (croissance organique) :

* **Validation**: 10 contributeurs, 50 devs formés
* **Viabilité**: 50 contributeurs, 100 devs formés
* **Impact**: 100 contributeurs, 200 devs formés
* **Référence**: 500 contributeurs, 500+ devs formés

Conclusion : Mission Holistique
-------------------------------

KoproGo ne se contente pas de fournir un logiciel. Notre mission est **holistique** :

✅ **Résoudre un problème sociétal** (copropriétés + isolement urbain)
✅ **Adopter pratiques écologiques** (< 0.5g CO2/requête)
✅ **Garantir sécurité et conformité** (RGPD, GitOps)
✅ **Promouvoir opensource** (AGPL-3.0, communauté)
✅ **Former aux pratiques IA** (partage recettes)
✅ **Pédagogie et onboarding** (documentation exhaustive)
✅ **Standards éthiques** (ASBL, gouvernance transparente)
✅ **Recréer du lien social** (modules communautaires optionnels par immeuble)

**Notre engagement** : La technologie doit servir l'humain, la planète, et l'intérêt général. Les modules communautaires (SEL, bazar de troc, prêt d'objets) sont **optionnels** et permettent à chaque copropriété de créer sa propre dynamique sociale selon ses besoins et sa culture.

Même si tu préfères tes outils actuels, tu peux quand même bénéficier de KoproGo.

----

**Voir aussi** :

* :doc:`VISION` - Vision stratégique et problème sociétal
* :doc:`ROADMAP_PAR_CAPACITES` - Roadmap par capacités (sans dates fixes)
* :doc:`ECONOMIC_MODEL` - Modèle économique ASBL et viabilité financière
* :doc:`GOVERNANCE` - Gouvernance et structure ASBL
