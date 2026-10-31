============================================================
Modèle Économique KoproGo ASBL - Guide Complet
============================================================

:Version: 5.0
:Date: 2 novembre 2025
:Statut: ASBL en préparation (constitution prévue au Jalon 3, voir :doc:`GOVERNANCE`) ; projections non contractuelles
:License: Code AGPL-3.0 / Document CC BY-SA 4.0
:Voir aussi: :doc:`VISION` | :doc:`MISSION` | :doc:`ROADMAP_PAR_CAPACITES` | :doc:`GOVERNANCE`

----

Table des Matières
------------------


#. `Vision et Philosophie <#vision-et-philosophie>`_
#. `Structure Juridique ASBL <#structure-juridique-asbl>`_
#. `Modèle OpenCore <#modèle-opencore>`_
#. `Structure Tarifaire <#structure-tarifaire>`_
#. `Transparence Comptable <#transparence-comptable>`_
#. `Économies d'Échelle <#économies-dechelle>`_
#. `Viabilité Financière <#viabilité-financière>`_
#. `Impact Écologique <#impact-écologique>`_
#. `Comparaison Concurrence <#comparaison-concurrence>`_
#. `Exemples Open Source Réussis <#exemples-open-source-réussis>`_
#. `Opportunités de Soutien <#opportunités-de-soutien>`_
#. `Risques et Opportunités <#risques-et-opportunités>`_

----

🔄 Démocratie Tarifaire - Le Modèle qui Change Tout
----------------------------------------------------

Introduction - Pourquoi KoproGo est Différent
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

   *"Dans les SaaS classiques, l'échelle enrichit les actionnaires. Chez KoproGo, l'échelle enrichit **chaque participant**. Plus nous sommes nombreux, plus le surplus augmente, et **c'est la communauté qui décide** comment l'utiliser: baisse de prix, nouvelles features, ou réinvestissement. C'est démocratique, transparent, et inscrit dans nos statuts."*

Le Mécanisme de Démocratie Tarifaire
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**1. Prix Fixe de Départ** (validé à la création ASBL)

.. code-block:: text

   Prix cloud géré: 5€/mois par copropriété
   Self-hosted: Gratuit (toujours)

   Principe: Client cloud = Membre ASBL automatiquement
   - 5€/mois → Accès cloud + Droit de vote AG (1 voix)
   - 0€ self-hosted → Usage libre, mais pas de droit de vote
   - Membre self-hosted (5€/mois) → Droit de vote sans utiliser le cloud

**2. Coûts Infrastructure Réels** (recherche OVHcloud Nov 2025)

.. code-block:: text

   VPS Value (1 vCPU, 2GB RAM, 40GB NVMe): 7.02€/mois TTC
   Capacité: 2,000-3,000 copropriétés par VPS
   S3 Storage: 0.007€/GB/mois

   Coûts réels par copropriété (infrastructure seule):
     100 copros cloud (40):    8.20€/mois infra ÷ 40 = 0.205€/copro
     5,000 copros cloud (2000): 163€/mois infra ÷ 2000 = 0.082€/copro

   → Coût infrastructure diminue de 71% avec l'échelle

**3. Surplus Démocratique** (décidé par vote AG)

.. code-block:: text

   Exemple à 5,000 copropriétés (2,000 cloud, 3,000 self-hosted):

   Revenus: 2,000 copros × 5€ = 10,000€/mois (120,000€/an)
   Coûts infrastructure: 163€/mois (1,956€/an)
   Coûts RH (2-3 devs): 100,000€/an
   Total coûts: 101,956€/an

   Surplus: 18,044€/an (15% marge)

   Vote AG sur allocation surplus:
   ✅ Option 1: Baisse tarifaire (5€ → 4€ ou 3€)
   ✅ Option 2: Nouvelles features (vote priorités)
   ✅ Option 3: Réserve légale (sécurité)
   ✅ Option 4: R&D PropTech (IA, IoT, Blockchain)

   Décision: Assemblée Générale (1 membre = 1 voix)

Évolution Démocratique du Prix (Vote AG)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1
   :widths: 15 20 20 20 25

   * - Palier
     - Copros Cloud
     - Prix Facturé
     - Surplus Annuel
     - Décision AG Possible
   * - **Bootstrap**
     - 100
     - **5€/mois**
     - 18,044€/an
     - Prix maintenu (constitution réserve)
   * - **Viabilité**
     - 500
     - **5€/mois**
     - 90,220€/an
     - Prix maintenu (embauche devs)
   * - **Impact**
     - 1,000
     - **5€/mois** ou **4€** (si AG vote)
     - 180,440€/an
     - AG peut voter baisse à 4€
   * - **Leadership**
     - 2,000
     - **4€/mois** (après vote AG)
     - 192,088€/an
     - Prix ajusté selon surplus
   * - **Référence**
     - 5,000
     - **3€/mois** (après vote AG)
     - 218,044€/an
     - AG décide allocation surplus

**Principe** : Le prix **baisse uniquement par vote démocratique** en Assemblée Générale, pas automatiquement.

**Options de vote AG** (selon surplus disponible):
1. **Baisse tarifaire** (ex: 5€ → 4€ → 3€) si surplus > 25% pendant 2 trimestres
2. **Nouvelles features** (vote priorités communauté)
3. **Amélioration infrastructure** (performance, sécurité)
4. **R&D PropTech** (IA, IoT, Blockchain)
5. **Constitution réserve légale** (3-6 mois coûts)

Effet Concret pour les Premiers Participants
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Julie, syndic, copropriété #47 (rejoint en 2026)** :

.. code-block:: text

   Année 1 (2026): 5€/mois → 60€/an
     → Devient membre ASBL automatiquement (1 voix AG)

   Année 2 (2027): 5€/mois → 60€/an (500 copros, ASBL constituée)
     → Email: "Bonne nouvelle! 500 copropriétés, l'ASBL est créée. Votre voix compte!"
     → Participe à l'AG, vote budget et roadmap

   Année 3 (2028): 5€/mois → 60€/an (1,000 copros)
     → AG vote: surplus 25% → Proposition baisse à 4€
     → Email: "L'AG a voté: votre tarif passe à 4€/mois. Merci d'être parmi les pionniers!"

   Année 4 (2029): 4€/mois → 48€/an (2,000 copros, -20% vs année 1)
     → Économie: 12€/an vs début
     → Vote en AG sur allocation surplus (features vs réserve)

   Année 5 (2030): 3€/mois → 36€/an (5,000 copros, -40% vs année 1)
     → AG décide nouvelle baisse après consultation communauté
     → Économie: 24€/an vs début (40%)
     → **Économie vs SaaS concurrent** (50€/mois): 564€/an (99.4%)

Add-ons Optionnels (prix coûtant + marge raisonnable)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Tarif base** (inclus) :

* Self-hosted : **0€** (toujours gratuit, AGPL-3.0)
* Cloud géré : **0.40-1€/mois** selon palier dégressif

**Add-ons PropTech** (optionnels) :

**1. AI Assistant Pack** - **+2€/mois**

* Chatbot syndic 24/7
* OCR factures/PV automatique
* Prévisions budget ML
* **Coûts** : GPU inference (0.80€) + stockage modèles (0.50€) + marge infra (0.70€)

**2. IoT Sensors** - **Hardware coût + 1€/capteur/mois**

* Hardware : 15-45€/capteur (coût fabricant, **0€ marge**)
* Service : 1€/mois (stockage données + alertes)
* Pack 10 capteurs : 150-450€ hardware + 10€/mois service

**3. Blockchain Voting** - **+1€/mois**

* Votes AG immuables (Polygon Layer 2)
* 50 votes/mois inclus (≈2 AG/an)
* **Coûts** : Gas fees mutualisés (0.40€) + node infra (0.30€) + marge (0.30€)

**4. Energy Buying Groups** - **0€ (gratuit)**

* Mission ASBL (intérêt général)
* Financé par partenariats fournisseurs
* Aucune commission sur contrats

**Précision Mutualisation Add-ons** :

.. code-block:: text

   AI Assistant: 1 GPU 50€/mois sert TOUS les usagers add-on
     • 100 copros add-on: 50€ ÷ 100 = 0.50€ coût réel → Facturé 2€ (4x pour R&D)
     • 1,000 copros add-on: 50€ ÷ 1,000 = 0.05€ coût réel → Baisse future possible à 1€

   Blockchain: 1 node 20€/mois pour tous
     • 100 copros: 0.20€ coût réel → Facturé 1€
     • 1,000 copros: 0.02€ coût réel → Baisse future possible à 0.50€

Projections 2025-2030 Révisées
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Scénario Conservateur** (20% adoption add-ons) :

.. code-block:: text

   Année 3 (2028): 2,000 copros
     • Base 0.60€: 14,400€/an
     • AI (400 copros): 9,600€/an
     • IoT (200 copros): 24,000€/an
     • Blockchain (300 copros): 3,600€/an
     Total: 51,600€/an (vs 24,000€ sans add-ons)

**Scénario Optimiste** (40% adoption add-ons) :

.. code-block:: text

   Année 5 (2030): 5,000 copros
     • Base 0.40€: 24,000€/an
     • AI (2,000 copros): 48,000€/an
     • IoT (1,000 copros): 120,000€/an
     • Blockchain (1,500 copros): 18,000€/an
     Total: 210,000€/an

Coûts Infrastructure PropTech
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

⚠️ **IMPORTANT**: Architecture 100% OVH France (Gravelines) pour souveraineté des données

.. code-block:: text

   AI Assistant (OVH AI Endpoints - serverless @ Gravelines):
     • Pay-as-you-go inference (40+ modèles LLM open-source)
     • Estimé: 50-100€/mois selon utilisation
     • ⚠️ Tarifs non publics - nécessite devis commercial OVH
     • Alternative GPU instance H100 PCIe: 2,000€/mois (trop cher)

   Blockchain node (Polygon RPC sur VPS OVH):
     • s1-2 VPS (1 vCore, 2GB RAM) @ 7.20€/mois TTC
     • Ou service managé Polygon: 20-30€/mois (estim.)

   IoT ingestion (MQTT broker):
     • Inclus dans backend principal (pas de surcoût VPS)
     • Ou VPS dédié s1-2 si scaling: 7.20€/mois TTC

   Stockage time-series IoT (TimescaleDB):
     • Extension PostgreSQL managée OVH Database
     • Inclus dans PostgreSQL principal ou instance dédiée
     • Estimé: 15-25€/mois selon volumétrie

   Total infra PropTech (estimé conservateur):
     • Minimal: 50€ AI + 7€ Blockchain + 0€ MQTT + 15€ TimescaleDB = 72€/mois (864€/an)
     • Réaliste: 80€ AI + 20€ Blockchain + 7€ MQTT + 20€ TimescaleDB = 127€/mois (1,524€/an)
     • ⚠️ À VALIDER avec devis OVH commercial (surtout AI Endpoints)

Transparence - Facture Détaillée Add-ons
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Dashboard public** `/transparency` affiche :

* Coûts réels PropTech (GPU, node, stockage)
* Nombre utilisateurs par add-on
* Prix coûtant calculé vs Prix facturé
* Surplus généré → Allocation (R&D modèles, audits smart contracts)

**Rapports trimestriels** :

* Détail coûts par add-on
* Évolution adoption (graphiques)
* Investissements R&D (nouvelles features IA, smart contracts audités)

Budget Participatif - Allocation Surplus
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Règle ASBL KoproGo** (inscrite statuts) :

* **Réserve légale minimum** : 3 mois coûts infra
* **Surplus au-delà réserve** : Voté en AG annuelle
* **Options vote** :

  #. Nouvelles features (développement)
  #. Amélioration infra (performance, sécurité)
  #. Baisse tarifaire anticipée (redistribution)
  #. R&D PropTech (IA, IoT, Blockchain)

**Exemple AG 2028** (1,500 copros, surplus 8,640€) :

.. code-block:: text

   Proposition CA:
     • 40% Features prioritaires (vote communauté): 3,456€
     • 30% Réserve légale (sécurité): 2,592€
     • 20% R&D IA/Blockchain: 1,728€
     • 10% Baisse anticipée palier 2k: 864€ (remboursé pro-rata)

   Vote AG (1 membre = 1 voix):
     ✅ Adopté: 87% pour, 13% abstention

Transparence Comptable - Dashboard Public
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**URL publique** : `https://koprogo.com/transparency` (accessible sans login)

**Métriques temps réel** :

* Nombre participants actifs
* Coûts infrastructure (serveurs, bande passante, stockage)
* Prix coûtant réel calculé
* Prix facturé actuel
* Surplus généré (€ et %)
* Prochain palier dégressif (countdown participants)
* Historique baisses tarifaires

**Rapports trimestriels** (PDF téléchargeable) :

* Bilan comptable complet
* Détail coûts par poste
* Allocation surplus
* Décisions AG
* Roadmap investissements

.. warning::
   **Garantie transparence radicale** : Si surplus > 25% pendant 2 trimestres consécutifs, baisse tarifaire **obligatoire** ou redistribution **automatique** (inscrit statuts).

----

Vision et Philosophie
---------------------

🎯 Principe Fondamental
^^^^^^^^^^^^^^^^^^^^^^^

KoproGo ASBL adopte un **modèle économique solidaire** basé sur la **mutualisation des coûts** et la **transparence absolue**. Notre objectif n'est pas le profit, mais la viabilité financière du projet au service de l'intérêt général.

..

   "Nous construisons un bien commun numérique, pas une licorne."


Valeurs ASBL
^^^^^^^^^^^^


#. **🏛️ Intérêt Général**\ : Mission sociale avant profit privé
#. **🔓 Open Source**\ : Code AGPL-3.0, transparence totale, auditabilité
#. **🐢 Durabilité**\ : Lent mais solide, vision 10+ ans
#. **⚡ Excellence**\ : Qualité technique sans compromis
#. **🤝 Communauté**\ : Gouvernance partagée, décisions collectives
#. **🌱 Écologie**\ : Infrastructure bas carbone (CO₂/req à mesurer)
#. **💚 Bénévolat**\ : Contribution par passion

Principes Fondamentaux du Modèle
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


#. **Prix d'entrée minimal**\ : 1€/mois par copropriété
#. **Quotas d'espace raisonnables**\ : Suffisants pour 90% des usages
#. **Dépassement à prix coûtant**\ : Coûts additionnels dilués avec toute la communauté
#. **Transparence totale**\ : Facture détaillée consultable par tous
#. **Option self-hosted gratuite**\ : Liberté totale pour les utilisateurs techniques

----

Structure Juridique Évolutive
------------------------------

Évolution de la Gouvernance (2025-2030+)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. note::
   **Détails complets** : Pour le calendrier détaillé et les jalons de chaque phase, voir :doc:`ROADMAP_PAR_CAPACITES` (Section "Évolution Gouvernance et Structure Juridique").

KoproGo adopte une **approche progressive** alignée sur la maturité du projet :

#. **Phase Bootstrap (Nov 2025 - Déc 2026)** : Solo dev bénévole

   * Statut : Projet personnel opensource
   * Gouvernance : Gilmry (solo developer)
   * Financement : Bénévolat (10-20h/semaine)
   * Objectif : MVP, validation product-market fit, < 100 copros

#. **Phase Fondateurs (Jan 2026 - Juin 2027)** : Noyau fondateur

   * Statut : Association de fait (2-3 fondateurs)
   * Gouvernance : Décisions consensuelles fondateurs
   * Financement : Autofinancement (premiers revenus cloud)
   * Objectif : 100 → 500 copropriétés, préparation ASBL

#. **Phase ASBL (Juil 2027 - Déc 2029)** : Association Sans But Lucratif

   * Statut : ASBL de droit belge constituée (cible de la phase)
   * Gouvernance : AG annuelle + CA élu (3-7 membres)
   * Financement : Revenus cloud + services B2B
   * Objectif : 500 → 5,000 copropriétés, viabilité long terme

#. **Phase Coopérative (2030+)** : Transformation optionnelle

   * Statut : **Si vote favorable AG** → Coopérative agréée
   * Gouvernance : Utilisateurs = sociétaires (1 personne = 1 voix)
   * Avantages : Subventions économie sociale, implication renforcée
   * Décision : Soumise à l'Assemblée Générale 2029

Qu'est-ce qu'une ASBL ?
^^^^^^^^^^^^^^^^^^^^^^^^

**ASBL** = Association Sans But Lucratif (loi belge du 27 juin 1921, réformée en 2019)

**Définition légale**\ : Une ASBL est une personne morale qui ne cherche pas à procurer un gain matériel à ses membres. Tous les bénéfices doivent être réinvestis dans l'objet social de l'association.

**Calendrier constitution ASBL KoproGo** : Juillet 2027 (J2-Launch, 500 copros, revenus stabilisés)

Avantages du Statut ASBL
^^^^^^^^^^^^^^^^^^^^^^^^


* ✅ **Non-lucratif**\ : Tous les bénéfices réinvestis dans le projet
* ✅ **Exonération TVA**\ : Activités à caractère social (sous conditions)
* ✅ **Transparence**\ : Assemblée générale annuelle, comptes publiés
* ✅ **Gouvernance**\ : Conseil d'administration bénévole
* ✅ **Mission sociale**\ : Démocratiser l'accès à la gestion de copropriété
* ✅ **Confiance**\ : Statut non-profit = légitimité et confiance accrues

Constitution de l'ASBL
^^^^^^^^^^^^^^^^^^^^^^

**Étapes de création**\ :


#. **Statuts**\ : Rédaction (objet social, gouvernance) - 1 semaine - 0€
#. **Acte authentique**\ : Passage devant notaire - 1 jour - ~250€
#. **Publication Moniteur**\ : Annonce légale - 2-4 semaines - ~200€
#. **Numéro d'entreprise**\ : BCE automatique - Immédiat - 0€
#. **Compte bancaire ASBL**\ : Ouverture compte - 1 semaine - 0-10€/mois

**Total création**\ : ~450-500€ + 1-2 mois

Objet Social ASBL KoproGo
^^^^^^^^^^^^^^^^^^^^^^^^^

..

   "L'association a pour objet la **promotion de l'accès démocratique aux outils numériques de gestion de copropriété**\ , par le développement, la maintenance et la diffusion de logiciels libres et open-source, ainsi que la fourniture de services d'hébergement et de support à prix coûtant.

   L'ASBL poursuit un but d'\ **intérêt général** et d'\ **éducation populaire** en:


   * Rendant accessible la technologie de gestion immobilière à tous
   * Favorisant la transparence par l'open-source
   * Formant des bénévoles aux pratiques professionnelles
   * Réduisant l'empreinte écologique de l'hébergement numérique"


KoproGo vs Startup (Trajectoires Comparées)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Aspect
     - Startup Classique
     - KoproGo (Solo→Fondateurs→ASBL→Coop)
   * - **Objectif**
     - Profit actionnaires
     - Impact social + viabilité
   * - **Phase Bootstrap**
     - Levée seed (100-500k€)
     - Bénévolat solo dev (0€)
   * - **Gouvernance initiale**
     - CEO + investisseurs
     - Solo dev → Fondateurs consensuels
   * - **Gouvernance mature**
     - Board + actionnaires
     - AG démocratique (ASBL/Coop)
   * - **Code**
     - Propriétaire (secret)
     - Open source (AGPL-3.0, auditable)
   * - **Prix**
     - Maximum acceptable (ROI)
     - Prix coûtant + marge solidaire
   * - **Données**
     - Monétisées (revente)
     - Propriété utilisateur (RGPD)
   * - **Bénéfices**
     - Dividendes actionnaires
     - Réinvestis mission (0 dividende)
   * - **Financement**
     - Levées (Series A/B/C)
     - Autofinancement progressif
   * - **Pression croissance**
     - Très élevée (exit 5-7 ans)
     - Aucune (croissance organique)
   * - **Dilution fondateurs**
     - 30-50% (voire plus)
     - 0% (pas d'actionnaires)
   * - **Exit strategy**
     - Acquisition ou IPO
     - Aucune (pérennité long terme)


----

Modèle OpenCore
---------------

Qu'est-ce qu'OpenCore ?
^^^^^^^^^^^^^^^^^^^^^^^

Le **cœur** du produit est **100% open-source (AGPL-3.0)**\ , et les **services d'hébergement cloud** sont **payants à prix coûtant** pour financer l'ASBL.

Fonctionnalités Core (100% Open Source)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**License**\ : AGPL-3.0 (copyleft fort)

.. code-block::

   ✅ Gestion immeubles (CRUD complet)
   ✅ Gestion lots/unités
   ✅ Gestion copropriétaires (conçue pour le RGPD)
   ✅ Gestion charges et répartition
   ✅ Suivi paiements
   ✅ Assemblées générales (convocations, PV, votes)
   ✅ Gestion documents (upload, versioning)
   ✅ API REST complète
   ✅ Frontend complet (Astro + Svelte)
   ✅ Infrastructure as Code (Docker Compose, Traefik)
   ✅ Self-hosting (installation 1-click)
   ✅ Exports données (CSV, JSON, SQL)

**Aucune feature fermée, aucun code propriétaire.**

Pourquoi Open Source le Core ?
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


* ✅ **Adoption large**\ : 0 friction, téléchargement libre
* ✅ **Confiance maximale**\ : Code auditable par tous
* ✅ **Contributions communauté**\ : Features gratuites
* ✅ **Anti-lock-in**\ : USP majeur vs concurrence
* ✅ **SEO/Visibilité**\ : GitHub stars, crédibilité
* ✅ **Sécurité**\ : Failles détectées rapidement

Modèle Hybride Cloud + Self-Hosted
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. note::
   **Évolution du modèle** : Avec la tarification échelonnée, le modèle n'est plus un simple 20/80 mais évolue selon le mix client (micro/petit/moyen/grand/XL). Voir :doc:`ROADMAP_PAR_CAPACITES` pour les projections détaillées.

**Répartition utilisateurs** (évolue selon la croissance) :


* **Cloud KoproGo géré** : Tarifs échelonnés 1,50-8€ selon taille (détails ci-dessous)
* **Self-hosted** : Gratuit à vie, aucun revenu (autonomie totale)

**Les revenus cloud financent**\ :


#. Développement (contributeurs, puis temps partiel, puis ETP)
#. Infrastructure (VPS + SSD chiffré + S3 + DNS + bande passante)
#. Support (documentation, forum, email)
#. Réserves (6-12 mois de fonctionnement)
#. Investissement features communautaires (SEL, partage objets)

----

Structure Tarifaire
-------------------

Option 1: Self-Hosted (Gratuit à Vie) 🔓
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Coût: 0€** (uniquement coût serveur personnel)

.. list-table::
   :header-rows: 1

   * - Avantage
     - Description
   * - **Gratuit à vie**
     - Aucun frais de licence, aucun abonnement
   * - **Souveraineté totale**
     - Données sous votre contrôle exclusif
   * - **Personnalisation**
     - Modification code source (AGPL-3.0)
   * - **Pas de limites**
     - Stockage, utilisateurs, requêtes illimités
   * - **GitOps automatique**
     - Mises à jour sécurité en 3 minutes


**Prérequis techniques**\ :


* VPS: 1 vCPU, 2 GB RAM, 40 GB SSD (~7€/mois OVH)
* OS: Ubuntu 22.04 LTS
* Compétences: Terminal Linux, Git, Docker

**Installation automatique**\ :

.. code-block:: bash

   git clone https://github.com/gilmry/koprogo.git
   cd koprogo
   make setup-infra  # Terraform + Ansible (20-30 min)

**Capacité Self-Hosted**\ :


* 1,000-1,500 copropriétés (charge légère)
* 50,000-100,000 utilisateurs
* Stockage local: 40 GB (40,000 documents)
* Performance: P99 < 5ms maintenue

**Pour qui?**


* Copropriétés avec un résident informaticien/DevOps
* Syndics ayant déjà un VPS/serveur
* Utilisateurs exigeant souveraineté totale des données

Option 2: Cloud KoproGo (Tarifs Échelonnés) ☁️
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Grille tarifaire échelonnée selon taille et features**

.. note::
   **Nouveau modèle de pricing** : À partir de 2025, KoproGo adopte une grille tarifaire échelonnée basée sur la taille de la copropriété et les fonctionnalités activées. Pour le détail complet des paliers et de l'évolution des prix, voir :doc:`ROADMAP_PAR_CAPACITES`.

**Tarifs 2025 (Launch - J2)** :

* **Micro (1-5 lots)** : 1,50€/mois
* **Petit (6-20 lots)** : 2-2,50€/mois
* **Moyen (21-50 lots)** : 3-3,50€/mois (inclut Conseil de Copropriété obligatoire)
* **Grand (51-100 lots)** : 4-5€/mois
* **XL (100+ lots)** : 5-8€/mois

Cette tarification échelonnée permet de :

* Maintenir un prix d'entrée bas pour les petites copropriétés (1,50€)
* Aligner les prix sur les frontières légales belges (Article 577-8/4 : 20 lots = Conseil obligatoire)
* Refléter les coûts réels de stockage et compute par segment
* Rester compétitif face aux solutions propriétaires (200-500€/mois)

**Quotas et services inclus** (adaptés selon la taille) :

.. note::
   Les quotas de stockage et utilisateurs évoluent selon le segment (micro/petit/moyen/grand/XL) pour refléter les besoins réels. Voir :doc:`ROADMAP_PAR_CAPACITES` pour les détails par segment.

.. list-table::
   :header-rows: 1

   * - Ressource
     - Micro (1-5)
     - Petit (6-20)
     - Moyen (21-50)
     - Grand/XL (51+)
   * - **Stockage documents**
     - 100-200 MB
     - 200-500 MB
     - 500 MB-1 GB
     - 1-5 GB
   * - **Utilisateurs**
     - 10-20
     - 20-50
     - 50-100
     - 100-500
   * - **Requêtes API**
     - Illimité
     - Illimité
     - Illimité
     - Illimité
   * - **Bande passante**
     - Incluse
     - Incluse
     - Incluse
     - Incluse
   * - **Backup automatique**
     - Quotidien (7j)
     - Quotidien (7j)
     - Quotidien (30j)
     - Quotidien (90j)
   * - **Support**
     - Email (72h)
     - Email (48h)
     - Email (48h)
     - Email (24h)


**Services Cloud inclus**\ :

.. code-block::

   ✅ Hébergement géré OVH France (datacenter bas carbone)
   ✅ Sauvegardes quotidiennes automatiques
   ✅ Mises à jour gratuites (rolling updates sans downtime)
   ✅ Support email (délai 48-72h)
   ✅ SSL/TLS inclus (Let's Encrypt)
   ✅ Monitoring uptime (objectif 99.7%+, non garanti)
   ✅ Exports données (CSV, JSON, SQL)
   ✅ Conçu pour le RGPD (données EU)

**Pour qui?**


* **Micro/Petit** : Petites copropriétés (1-20 lots) avec usage léger
* **Moyen** : Copropriétés moyennes (21-50 lots) avec Conseil de Copropriété
* **Grand/XL** : Grandes copropriétés (51+ lots) avec usage intensif
* Copropriétés sans compétences techniques souhaitant une solution clé en main

Philosophie Tarifaire : Prix Coûtant + Marge Minimale
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Principe fondamental**\ : Les tarifs sont calculés au **prix coûtant réel** (infrastructure compute + storage) avec une **marge minimale ASBL** pour financer le développement et les réserves.

Calcul du Prix Coûtant par Segment
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. note::
   Pour le calcul détaillé des coûts par palier (Nano/Micro/Petit/Moyen/Grand) incluant compute, SSD chiffré, et S3 backup, voir :doc:`ROADMAP_PAR_CAPACITES` (Section "Coûts Infrastructure Réels").

**Composantes du coût** (exemple palier Micro, 100 copros) :

.. code-block::

   Compute (VPS s1-2):           4,20€/mois
   SSD chiffré (20 GB):          2,00€/mois (0.10€/GB LUKS)
   S3 backup (10 GB):            0,10€/mois (0.01€/GB)
   DNS OVH:                      0,10€/mois
   Bande passante:               0€ (incluse)
   Support (bénévole Phase 1):   0€
   -------------------------------------------
   Total coûts:                  6,40€/mois

   100 copropriétés cloud:       100 × 2€ = 200€/mois
   Coût réel par copro:          0,064€/copro/mois
   Marge ASBL (développement):   1,936€/copro/mois (96,8%)

**Évolution des coûts** : Le storage (SSD + S3) devient dominant (60-70%) au-delà de 500 copropriétés, d'où la tarification échelonnée selon la taille.

**Important**\ : Les requêtes API et utilisateurs n'ont **aucun coût marginal** infrastructure, donc **illimités sans surcoût**.

Exemples Concrets de Tarification (Launch 2025 - J2)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cas 1: Micro copropriété 4 lots**


* Segment: Micro (1-5 lots)
* Stockage: ~100 MB (factures, PV)
* Utilisateurs: 8 copropriétaires
* **Coût total: 1,50€/mois** ✅
* **Économie vs concurrent** : 198,50€/mois (99,25%)

**Cas 2: Petite copropriété 15 lots**


* Segment: Petit (6-20 lots)
* Stockage: ~300 MB (documents 5 ans)
* Utilisateurs: 25 copropriétaires
* **Coût total: 2,50€/mois**
* **Économie vs concurrent** : 197,50€/mois (98,75%)

**Cas 3: Copropriété moyenne 35 lots**


* Segment: Moyen (21-50 lots) - **Conseil Copropriété obligatoire**
* Stockage: ~600 MB (historique complet)
* Utilisateurs: 70 copropriétaires + conseil
* Features: CRUD + Conseil + Voting
* **Coût total: 3,50€/mois**
* **Économie vs concurrent** : 196,50€/mois (98,25%)

**Cas 4: Grande copropriété 80 lots**


* Segment: Grand (51-100 lots)
* Stockage: ~1,5 GB (10 ans archives)
* Utilisateurs: 150 copropriétaires
* Features: Complètes + Community
* **Coût total: 5€/mois**
* **Économie vs concurrent** : 295-495€/mois (98-99%)

**Comparaison synthétique** :

* Solution SaaS classique 50 lots : **200-300€/mois**
* **KoproGo 50 lots : 4-5€/mois** (soit **98-99% d'économie**\ )

Services Additionnels (Futurs)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Pour grandes copropriétés et syndics professionnels:

.. list-table::
   :header-rows: 1

   * - Service
     - Prix estimé
     - Description
   * - **Déploiement assisté**
     - 200-500€ one-time
     - Installation serveur client
   * - **Formation syndic**
     - 800€
     - Formation 1 jour
   * - **Support premium**
     - +5€/mois
     - Réponse 24h, téléphone
   * - **Intégration comptable**
     - 300€ setup
     - API Odoo, Sage, etc.
   * - **Programme Sponsor**
     - 100€/an
     - Logo, influence roadmap


----

Transparence Comptable
----------------------

Facture Publique Mensuelle
^^^^^^^^^^^^^^^^^^^^^^^^^^

Une fois constituée, l'ASBL publiera chaque mois un **rapport financier public** détaillant:


#. **Coûts infrastructure réels** (factures OVH)
#. **Nombre de copropriétés hébergées**
#. **Utilisation ressources** (stockage, BP, CPU)
#. **Prix coûtant calculé**
#. **Répartition revenus** (maintenance, développement, réserves)

**Accès**\ : Tableau de bord public sur `koprogo.com/transparence <https://koprogo.com/transparence>`_

**Exemple Format** (valeurs fictives, à titre d'illustration)\ :

.. code-block:: markdown

   ## Rapport Financier Octobre 2025

   ### Coûts Infrastructure
   - VPS OVH (d2-2): 7.00€
   - S3 OVH (200 GB): 2.00€
   - DNS OVH: 0.10€
   - Total: 9.10€

   ### Revenus
   - 100 copropriétés × 1€: 100.00€
   - Dépassements stockage: 15.00€
   - Total: 115.00€

   ### Affectation Excédent (105.90€)
   - Réserve sécurité (50%): 54.60€
   - Développement (30%): 32.76€
   - Infrastructure K3s (10%): 10.92€
   - Fonds urgence (10%): 7.62€

   ### Statistiques
   - Uptime: XX.XX%
   - Latency P99: X.Xms
   - CO2: X.XXg/req (mesuré)
   - Support tickets: 3 (résolus en 48h)

Engagement Transparence
^^^^^^^^^^^^^^^^^^^^^^^

**KoproGo s'engage à**\ :


#. **Comptes publics annuels**\ : Publiés sur GitHub + site web
#. **Budget prévisionnel**\ : Partagé avec communauté en début d'année
#. **Rapport d'activité**\ : Annuel, détaillant usage des fonds
#. **Dashboard temps réel**\ : Revenus, coûts, trésorerie (màj trimestrielle)

**Exemples de transparence open source**\ :


* Mozilla Foundation: Publie budget complet + salaires dirigeants
* Wikimedia: Dashboard financier public temps réel
* Document Foundation: Comptes annuels + rapports activité

----

Économies d'Échelle
-------------------

Comment les Coûts Diminuent avec la Croissance
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

Plus d'utilisateurs = coût par copropriété qui diminue. Infrastructure fixe jusqu'à un seuil, croissance progressive ensuite.

Coûts Réels OVH (2025)
~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1

   * - Ressource
     - Prix
   * - VPS Value
     - 5,80€/mois (1 vCore, 2GB RAM)
   * - VPS Essential
     - 12€/mois (2 vCore, 4GB RAM)
   * - VPS Elite
     - 27€/mois (8 vCore, 32GB RAM)
   * - Object Storage S3
     - 0,007€/GB/mois
   * - Bande passante
     - Gratuite (incluse)
   * - DNS
     - 0,10€/mois


Scénario Croissance
~~~~~~~~~~~~~~~~~~~

**100 copropriétés**\ :


* Infrastructure: 6,25€/mois
* Coût par copro: 0,063€/mois
* Revenus: 100€/mois
* Excédent: 93,75€/mois

**500 copropriétés**\ :


* Infrastructure: 13,85€/mois
* Coût par copro: 0,028€/mois (−55%)
* Revenus: 500€/mois
* Excédent: 486,15€/mois

**2,000 copropriétés**\ :


* Infrastructure: 34,10€/mois
* Coût par copro: 0,017€/mois (−73%)
* Revenus: 2,000€/mois
* Excédent: 1,965,90€/mois

**Le coût par copropriété diminue de 73% entre 100 et 2,000 utilisateurs.**

Évolution Tarifs par Phase
^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. note::
   **Détails complets** : Pour l'évolution des tarifs par segment (Micro/Petit/Moyen/Grand/XL) et par phase (Beta/Launch/Growth/Scale), voir :doc:`ROADMAP_PAR_CAPACITES` (Grille Tarifaire Échelonnée).

**Principe** : Les prix **baissent progressivement** grâce aux économies d'échelle, tout en maintenant une tarification équitable selon la taille.

.. list-table::
   :header-rows: 1

   * - Phase
     - Année
     - Copros Cloud
     - Prix Moyen
     - Mix Client
   * - **Beta (J1)**
     - 2025
     - 100
     - 2,00€/mois
     - Majoritairement Micro/Petit (early adopters)
   * - **Launch (J2)**
     - 2026
     - 500
     - 2,50€/mois
     - Équilibré Petit/Moyen
   * - **Growth (J3)**
     - 2027-2028
     - 2000
     - 3,00€/mois
     - Montée en gamme Moyen/Grand
   * - **Scale (J4)**
     - 2030
     - 5000
     - 3,50€/mois
     - Mix mature avec plus de Premium

**Exemple concret** : Une copropriété Petit (6-20 lots) paie 2,50€ en Launch (2026) et pourrait voir ce prix baisser à 2€ en Scale (2030) grâce aux économies d'échelle.

Réinvestissement Démocratique
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

L'\ **Assemblée Générale ASBL** vote chaque année l'affectation des excédents :


* **Baisser les prix** (si réserves suffisantes et coûts infrastructure baissent)
* **Améliorer les quotas** (plus de stockage/backup inclus)
* **Investir features communautaires** (SEL, partage objets, annuaire compétences)
* **Investir dans nouvelles features** (mobile app, real-time, AI assistant)
* **Constituer des réserves** (6-12 mois fonctionnement minimum)

**C'est la communauté qui décide, pas des actionnaires.**

----

Viabilité Financière
--------------------

Budget Prévisionnel 2025-2030
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. note::
   **Projections détaillées** : Pour les projections financières complètes avec mix client, coûts infrastructure (compute + storage), et évolution des revenus, voir :doc:`ROADMAP_PAR_CAPACITES` (Section "Projections Financières Complètes").

**Hypothèses actualisées**\ :


* Croissance: 100 copros (2025) → 5,000 copros (2030)
* Répartition cloud: évolution du mix petit→moyen→grand
* Prix moyen cloud évolue: 2€/mois (J1-Beta) → 3,50€/mois (J4-Scale)
* **Coûts infrastructure** : compute + SSD chiffré + S3 backup
* Storage devient dominant (60-70% coûts au-delà de 500 copros)

.. list-table::
   :header-rows: 1

   * - Année
     - Copros Cloud
     - Revenus/an
     - Coûts Infra
     - Développement
     - Excédent
   * - **2025**
     - 20
     - 288€
     - 120€
     - 0€ (bénévole)
     - +168€
   * - **2026**
     - 80
     - 1,152€
     - 240€
     - 0€ (bénévole)
     - +912€
   * - **2027**
     - 200
     - 2,880€
     - 360€
     - 1,200€ (0.5 ETP)
     - +1,320€
   * - **2028**
     - 400
     - 5,760€
     - 480€
     - 2,400€ (1 ETP)
     - +2,880€
   * - **2030**
     - 1,000
     - 14,400€
     - 600€
     - 3,600€ (1.5 ETP)
     - +10,200€


**Réserves cumulées 2030**\ : ~150k€ (détails :doc:`ROADMAP_PAR_CAPACITES`)

**Impact économique et écologique 2030** :

* **Économies générées** : 8M€ (logiciels propriétaires) + 750k€ (SEL) + 600k€ (consommation évitée) = **9,35M€/an**
* **Impact carbone** : -840 tonnes CO₂/an (50t infrastructure + 790t features communautaires, dépassement +57% vs objectif initial)

Unit Economics Cloud ASBL (Tarif Moyen)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**LTV (Lifetime Value)**\ :

.. code-block::

   Prix moyen: 3€/copro/mois (mix client équilibré Phase J3-J4)
   - Churn: 5%/an (très faible, besoin réel de gestion copro)
   - Durée vie = 1 / 0.05 = 20 ans
   - LTV = 3€ × 12 mois × 20 ans = 720€ par copro

   Conservateur (10 ans): LTV = 360€

**CAC (Customer Acquisition Cost)**\ :

.. code-block::

   0€ marketing (croissance organique, bouche-à-oreille)
   Temps bénévole si valorisé: ~5-10€ réaliste

   CAC effectif: 5-10€

**LTV/CAC**\ :

.. code-block::

   720€ / 10€ = 72:1 (exceptionnel)
   Target SaaS classique: 3:1
   KoproGo ASBL: 72:1 ✅

**Payback Period**\ :

.. code-block::

   CAC / MRR par copro = 10€ / 3€ = 3,3 mois
   Target SaaS: < 12 mois
   KoproGo: 3,3 mois ✅

**Gross Margin**\ :

.. code-block::

   Revenus moyens: 3€/copro/mois
   Coûts variables (compute + storage): ~0,10€/copro
   Marge brute: 96,7% ✅

   Note: Storage devient dominant (60-70%) au-delà de 500 copros,
   d'où la tarification échelonnée selon taille réelle.

**Conclusion Unit Economics**\ : Exceptionnellement sains, scalabilité énorme, aucune pression croissance, viabilité long terme assurée.

Scénarios de Crise
^^^^^^^^^^^^^^^^^^

**Scénario 1: Chute revenus cloud (-50%)**


* Impact: Réduction développement à temps partiel
* Solution: Appel communauté, campagne dons, gel nouvelles features
* Marge sécurité: Réserves 6-12 mois

**Scénario 2: Augmentation coûts infra (+100%)**


* Impact: Ajustement tarifs (+0,50-1€ selon segment)
* Exemple: Petit 2,50€ → 3€, Moyen 3,50€ → 4€
* Vote Assemblée Générale requis + transparence totale

**Scénario 3: Pic usage inattendu (×10)**


* Impact: Migration infrastructure anticipée (K3s Phase 2)
* Financement: Réserves cumulées + ajustement temporaire tarifs
* Transition progressive sans interruption service

----

Impact Écologique
-----------------

.. note::
   **Impact complet** : Pour une analyse détaillée de l'impact écologique incluant les features communautaires (SEL, partage objets) et leurs bénéfices carbone (10x l'infrastructure), voir :doc:`ROADMAP_PAR_CAPACITES` et :doc:`MISSION`.

Comparaison Carbone Infrastructure
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Solution classique (SaaS WordPress)**\ :


* Serveur dédié par client: 50W × 8760h = 438 kWh/an
* Datacenter standard: 438 kWh × 0.3 kg CO2/kWh = **131 kg CO2/an**

**KoproGo Cloud (mutualisé)**\ :


* VPS partagé: 10W / 1,000 copros = 0.01W par copro
* Datacenter bas carbone (GRA11): 0.01W × 8760h × 0.06 kg CO2/kWh = **0.0053 kg CO2/an**
* **Réduction infrastructure: 99.996%** 🌱

**Impact Total avec Features Communautaires (2030)** :


* Infrastructure: -50 tonnes CO₂/an (5000 copros cloud)
* **Features communautaires** (30% adoption) : -790 tonnes CO₂/an (partage objets, réduction consommation)
* **Impact total : -840 tonnes CO₂/an** (effet multiplicateur 16x, dépassement +57% vs objectif initial)

Politique Green IT
^^^^^^^^^^^^^^^^^^


#. **Datacenter bas carbone**\ : OVH GRA11 (60g CO2/kWh vs 300g moyenne)
#. **Mutualisation maximale**\ : 1,000+ copros sur 1 VPS
#. **Architecture Rust**\ : 10x moins de CPU que Python/Node.js
#. **Progressive Web App**\ : Cache local, moins de requêtes réseau
#. **Backup intelligent**\ : Déduplication, compression

**Mesures concrètes**\ :


* CO₂/req : estimation réseau, non mesurée (OVH GRA11 Gravelines, mix 60g CO₂/kWh)
* 5.8x moins d'émissions que datacenters Allemagne (350g CO₂/kWh)
* 7-25x moins que AWS/Azure US (475-700g CO₂/kWh)
* **100% souveraineté française** (données hébergées à Gravelines)

----

Comparaison Concurrence
-----------------------

Marché Solutions Propriétaires
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Solution
     - Prix/mois
     - Stockage
     - Support
     - Souveraineté
     - CO2/an
   * - **Vilogi**
     - 200-500€
     - 5-50 GB
     - Phone 9-18h
     - ❌ Cloud US
     - ~50 kg
   * - **Apronet**
     - 150-300€
     - 10 GB
     - Email 48h
     - ❌ Cloud FR
     - ~40 kg
   * - **Homeasy**
     - 100-200€
     - 2 GB
     - Chatbot
     - ❌ Cloud BE
     - ~30 kg
   * - **KoproGo Cloud**
     - **1,50-8€**
     - 0.1-5 GB
     - Email 24-72h
     - ✅ EU/Local
     - **0.005 kg**
   * - **KoproGo Self-Hosted**
     - **0€**
     - ∞
     - Communauté
     - ✅ Total
     - **0.001 kg**


**Économie moyenne**\ :

* Micro (1-5 lots) : **198,50€/mois** (99,25% réduction vs 200€)
* Petit (6-20 lots) : **197,50€/mois** (98,75% réduction vs 200€)
* Moyen (21-50 lots) : **246,50€/mois** (98,60% réduction vs 250€)
* Grand/XL (51-100+ lots) : **395-495€/mois** (98-99% réduction vs 400-500€)

Économies Réalisées (Exemples Réels)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Exemple 1: Petite copropriété 15 lots sur 3 ans**

.. list-table::
   :header-rows: 1

   * - Poste
     - Solution Propriétaire
     - KoproGo Cloud (Petit)
     - Économie
   * - Licence/Abonnement
     - 7,200€ (200€×36 mois)
     - 90€ (2,50€×36 mois)
     - −99%
   * - Formation
     - 800€
     - 0€ (doc self-service)
     - −100%
   * - Migration
     - 500€
     - 0€ (import CSV)
     - −100%
   * - **Total 3 ans**
     - **8,500€**
     - **90€**
     - **−98,94%**

**Exemple 2: Copropriété moyenne 40 lots sur 3 ans**

.. list-table::
   :header-rows: 1

   * - Poste
     - Solution Propriétaire
     - KoproGo Cloud (Moyen)
     - Économie
   * - Licence/Abonnement
     - 10,800€ (300€×36 mois)
     - 126€ (3,50€×36 mois)
     - −98,8%
   * - Formation
     - 1,200€
     - 0€
     - −100%
   * - Migration
     - 800€
     - 0€
     - −100%
   * - **Total 3 ans**
     - **12,800€**
     - **126€**
     - **−99,02%**


----

Exemples Open Source Réussis
----------------------------

Red Hat
^^^^^^^


* **Activité**\ : Distribution Linux enterprise (RHEL)
* **Modèle**\ : OS gratuit + support/certification payant
* **Résultat**\ : Acquis par IBM pour **34 milliards USD** (2019)
* **Leçon**\ : Open source + services B2B = viable à très grande échelle

WordPress / Automattic
^^^^^^^^^^^^^^^^^^^^^^


* **Activité**\ : CMS open source (43% du web)
* **Modèle**\ : Self-hosted gratuit + WordPress.com payant
* **Résultat**\ : **7.5 milliards USD** valorisation
* **Leçon**\ : Freemium + hébergement managé = millions d'utilisateurs

GitLab
^^^^^^


* **Activité**\ : Plateforme DevOps
* **Modèle**\ : Core gratuit + features enterprise payantes
* **Résultat**\ : **6 milliards USD** IPO (2021)
* **Leçon**\ : Transparence + fonctionnalités avancées = confiance entreprises

Odoo (Belge)
^^^^^^^^^^^^


* **Activité**\ : ERP open source
* **Modèle**\ : Community gratuite + Enterprise + SaaS
* **Résultat**\ : Leader ERP PME, **7+ millions utilisateurs**
* **Leçon**\ : Open source local peut devenir leader mondial

Signal
^^^^^^


* **Activité**\ : Messagerie chiffrée
* **Modèle**\ : 100% gratuit, 0€ revenus commerciaux, dons
* **Résultat**\ : **40+ millions utilisateurs**\ , alternative éthique aux GAFAM
* **Leçon**\ : Impact social > profit = possible et viable

Mozilla Foundation
^^^^^^^^^^^^^^^^^^


* **Activité**\ : Firefox, Thunderbird
* **Modèle**\ : Logiciels gratuits + services + dons
* **Résultat**\ : **$500M/an budget**\ , rentable depuis 20+ ans
* **Leçon**\ : ASBL tech peut être pérenne et impactante

Wikimedia
^^^^^^^^^


* **Activité**\ : Wikipedia
* **Modèle**\ : Contenu gratuit + dons + services
* **Résultat**\ : **300M+ utilisateurs**\ , **$150M/an**\ , 0 publicité
* **Leçon**\ : Bien commun peut se financer par la communauté

Blender Foundation
^^^^^^^^^^^^^^^^^^


* **Activité**\ : Logiciel 3D professionnel
* **Modèle**\ : Logiciel gratuit + cloud rendering payant
* **Résultat**\ : Utilisé par **Hollywood**\ , **$3M/an**\ , **50+ devs salariés**
* **Leçon**\ : Excellence technique + communauté = financement viable

Framasoft (ASBL France)
^^^^^^^^^^^^^^^^^^^^^^^


* **Activité**\ : Suite outils open-source
* **Modèle**\ : Services gratuits + dons + support
* **Résultat**\ : **1M+ utilisateurs**\ , 0€ publicité, **100% éthique**
* **Leçon**\ : ASBL française prouve viabilité modèle francophone

----

.. note::
   **Gouvernance ASBL**

   Pour les détails complets sur la gouvernance de l'ASBL KoproGo, consultez le document dédié :

   📋 `GOVERNANCE.md <../GOVERNANCE.html>`_ - Structure ASBL, membres, Conseil d'Administration, Assemblée Générale, mainteneurs techniques, processus de décision, et évolution de la gouvernance.

   Ce document couvre : membres de l'ASBL, cotisation (120€/an), Conseil d'Administration (1-10 administrateurs), mainteneurs techniques, contributeurs externes (DCO), financements, transparence financière, propriété intellectuelle, Code of Conduct, et résolution de conflits.

----

Opportunités de Soutien
-----------------------

Pourquoi Soutenir KoproGo ?
^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Impact social mesurable**\ :


* 1.5 million de copropriétés en Belgique
* 8 millions € d'économies potentielles d'ici 2030
* −500 tonnes CO₂/an d'ici 2030
* Lien social via modules communautaires

**Modèle économique prouvé**\ : Red Hat (34 mds), WordPress (7.5 mds), GitLab (6 mds)

**Différenciation forte**\ :


* **Légal**\ : AGPL protège contre fork propriétaire
* **Éthique**\ : ASBL = mission sociale > profit
* **Technique**\ : Rust + GitOps = performance + fiabilité
* **Écologique**\ : 96% réduction CO₂
* **Local**\ : Belge, RGPD-first, souveraineté UE

**Traction sans financement**\ :


* 0€ levés, 100% autofinancé
* Break-even projeté: mois 2
* Croissance organique: 10-20%/mois

Formes de Soutien Possibles
^^^^^^^^^^^^^^^^^^^^^^^^^^^

A. Partenariat Stratégique (Non-Financier)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


* Beta-testing de la plateforme
* Feedback sur features prioritaires
* Études de cas et témoignages
* Avantages: accès gratuit à vie, influence roadmap

B. Sponsoring ASBL
~~~~~~~~~~~~~~~~~~


* 
  **Programme Copropriété Sponsor**\ : 100€/an


  * Logo sur site web KoproGo
  * Priorité support (email 24h)
  * Influence roadmap (vote features)
  * Quota cloud étendu (10 GB stockage)

* 
  **Grandes entreprises/fondations**\ : 1,000-10,000€/an


  * Partenariat stratégique
  * Co-développement features
  * Études de cas communes

C. Subventions Publiques
~~~~~~~~~~~~~~~~~~~~~~~~

**Cibles**\ :


* **Horizon Europe** (EU): 10-50k€/projet si éligible
* **Digital Wallonia** (BE): 5-20k€/an
* **Innoviris** (Bruxelles): Projets innovants
* **Fondation Roi Baudouin**\ : Projets d'intérêt général
* **Mozilla Foundation**\ , **Sloan Foundation**\ : Grants open-source

D. Services B2B (Revenus Futurs)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1

   * - Service
     - Volume An 3
     - Revenus Projetés
   * - Déploiement
     - 50/an
     - 15,000€
   * - Formation
     - 20/an
     - 16,000€
   * - Support premium
     - 100 clients
     - 6,000€
   * - Intégration API
     - 10/an
     - 5,000€
   * - **Total services B2B**
     - 
     - **42,000€**


**Total avec cloud An 3**\ : ~126,000€/an

Avantages Fiscaux Donateurs
^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Si reconnaissance "établissement d'utilité publique"** (après 3-5 ans d'activité):

**Donateurs particuliers**\ :


* Déduction fiscale 45% du don (min 40€/an)
* Exemple: Don 100€ = 45€ réduction impôt

**Donateurs entreprises**\ :


* Déduction à 120% du don (sponsoring déductible)
* Exemple: Don 1,000€ = 1,200€ déduction bénéfice imposable

Roadmap Financement
^^^^^^^^^^^^^^^^^^^

**Phase 1: Bootstrap (2025) - 0€ externe**


* Développement bénévole (10-20h/semaine)
* Infrastructure minimale (10€/mois)
* Objectif: 100 premiers utilisateurs

**Phase 2: Sponsoring initial (2026) - 10,000-30,000€**


* Syndics partenaires, subventions régionales
* Temps développeur partiel (2j/semaine)
* Objectif: 500 copropriétés, communauté active

**Phase 3: Services B2B (2027+) - Autofinancement**


* Revenus récurrents: 126,000€/an
* 2-3 développeurs temps plein
* Objectif: 2,000+ copropriétés, viabilité long terme

----

Risques et Opportunités
-----------------------

Risques et Mitigations
^^^^^^^^^^^^^^^^^^^^^^

1. Croissance Ultra-Lente
~~~~~~~~~~~~~~~~~~~~~~~~~

**Risque**\ : Croissance 5-10 copros/mois (vs 50-100 startup avec marketing)

**Impact**\ : Faible (acceptable pour ASBL side-project)

**Mitigation**\ :


* ✅ Pas de stress: Aucune pression investisseurs
* ✅ Qualité > Quantité: Meilleure rétention (churn 3-5% vs 10-15%)
* ✅ Excellence produit: NPS > 60 = bouche-à-oreille naturel
* ✅ Rentable immédiatement: Break-even Mois 2

2. Temps Équipe Limité
~~~~~~~~~~~~~~~~~~~~~~

**Risque**\ : 10-20h/semaine = vélocité 4x plus lente qu'une startup

**Impact**\ : Moyen (features livrées lentement)

**Mitigation**\ :


* ✅ Communauté OSS: Contributors externes (traductions, bugfixes)
* ✅ Automation maximum: CI/CD, tests auto
* ✅ Focus ruthless: 20% features = 80% valeur (Pareto)
* ✅ Documentation self-service: Réduit support

3. Bénévolat Non Rémunéré (3-4 ans)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Risque**\ : Démotivation contributeurs

**Impact**\ : Moyen (risque abandon)

**Mitigation**\ :


* ✅ Passion intrinsèque: Mission sociale
* ✅ Apprentissage: Formation pratique sur CV
* ✅ Flexibilité: Job externe + KoproGo passion
* ✅ Transparence: Promesse rémunération si trésorerie

4. Concurrence Agressive
~~~~~~~~~~~~~~~~~~~~~~~~

**Risque**\ : Vilogi/Septeo baissent prix ou copient OpenCore

**Impact**\ : Très faible (incompatible avec leur modèle)

**Mitigation**\ :


* ✅ License AGPL-3.0: Forks doivent rester open-source
* ✅ Impossible à copier: Authenticité ASBL vs greenwashing
* ✅ First-mover: Première solution OpenCore copropriété
* ✅ Performance tech: Rust, sobriété visée
* ✅ Communauté loyale: Open-source = confiance

Opportunités
^^^^^^^^^^^^

1. Communauté Open-Source = Croissance Gratuite
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Upside**\ :


* ✅ Features gratuites: Contributors externes développent
* ✅ Evangelists: Ambassadeurs promeuvent naturellement
* ✅ Crédibilité: GitHub stars = preuve sociale
* ✅ Acquisition $0: Bouche-à-oreille tech, SEO organique

**Exemple**\ : Plausible Analytics (bootstrap, OSS) : 15k stars → 10k+ clients sans marketing

2. Souveraineté des Données et Portabilité
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Contexte**\ : 2025-2030 = décennie décentralisation, souveraineté numérique

**Upside**\ :


* ✅ GDPR natif: Données EU, conformité totale
* ✅ Souveraineté: OVH France, protection des données
* ✅ Données portables: Export CSV/JSON/SQL inclus
* ✅ Écologie: mix français bas carbone, CO₂/req à mesurer
* ✅ Éthique: ASBL non-profit, réinvestissement communauté

3. Subventions & Dons (Si Utilité Publique)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Upside**\ :


* ✅ Subventions EU: Horizon Europe, Digital Europe Programme
* ✅ Subventions BE: Innoviris, Digital Wallonia
* ✅ Dons particuliers: Déduction fiscale 45%
* ✅ Dons entreprises: Déduction 120% (RSE)

**Exemple**\ : Blender Foundation : $1M/an en dons + $2M subventions → 50+ devs

4. Partenariats Institutionnels
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Upside**\ :


* ✅ Bailleurs sociaux: Logements publics BE/FR (milliers copros)
* ✅ Associations copropriétaires: ARC, UNPI
* ✅ Universités: Cas d'étude, contributions
* ✅ Certifications: Labels open-source, B Corp

----

Conclusion
----------

Points Clés
^^^^^^^^^^^


#. **Le modèle open source + services fonctionne**\ : Preuves à 34 milliards USD
#. **KoproGo résout un vrai problème**\ : 1.5M copropriétés, 95-99% d'économies, 96% réduction carbone
#. **Structure ASBL = impact social**\ : Bénéfices réinvestis, transparence, démocratie
#. **Traction sans financement**\ : Bootstrap réussi, break-even mois 2
#. **Opportunités multiples**\ : Partenariats, sponsoring, subventions, services B2B

Notre Engagement
^^^^^^^^^^^^^^^^

**Le coût de KoproGo restera accessible et transparent** :

* Grille tarifaire échelonnée : 1,50€ (micro) à 8€ (XL 100+ lots)
* Prix alignés sur la valeur apportée et les coûts réels (compute + storage)
* Toujours **95-99% moins cher** que les solutions propriétaires (200-500€/mois)
* Tout excédent sera réinvesti dans le développement, la communauté, ou redistribué via amélioration des services
* Option self-hosted **gratuite à vie** toujours disponible

L'Équipe et Évolution Organisationnelle
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. note::
   **Évolution progressive** : L'équipe évolue avec la maturité du projet (voir :doc:`ROADMAP_PAR_CAPACITES`).

**Phase Bootstrap (Nov 2025 - Déc 2026)** : Solo dev bénévole


* **Gilmry** (10-20h/semaine side-project)

  * Emploi temps plein ailleurs (sécurité financière)
  * Architecture + développement core Rust
  * DevOps + infrastructure OVH
  * Vision produit long-terme

**Phase Fondateurs (Jan 2026 - Jun 2027)** : Noyau fondateur (2-3 personnes)


* **Gilmry** + 1-2 co-fondateurs
* Contribution bénévole ou temps partiel rémunéré (si revenus suffisants)
* Spécialisations : Backend, Frontend, Community/Support
* Constitution structure légale (préparation ASBL)

**Phase ASBL (Jul 2027 - Déc 2029)** : Équipe structurée


* CA élu (3-7 administrateurs bénévoles)
* 1-2 développeurs salariés (temps partiel puis temps plein)
* Contributeurs communautaires (opensource)
* Budget : Revenus cloud + services B2B

**Phase Coopérative (2030+)** : Organisation mature


* Sociétaires (utilisateurs-membres)
* 2-3 salariés permanents
* Contributeurs réguliers (10-20 personnes)
* Gouvernance démocratique renforcée

Contact et Collaboration
^^^^^^^^^^^^^^^^^^^^^^^^

**GitHub**\ : https://github.com/gilmry/koprogo

**Opportunités**\ :


* Beta-testeurs (syndics, copropriétés)
* Sponsors ASBL (entreprises, fondations)
* Contributeurs open source (développeurs)
* Partenaires institutionnels (subventions)

----

**L'open source n'est pas seulement idéaliste, c'est pragmatique.** Les plus grandes réussites technologiques des 20 dernières années sont open source. KoproGo combine l'impact social d'une ASBL avec la viabilité du modèle OpenCore éprouvé.

**Nous ne construisons pas une licorne. Nous construisons un bien commun durable.** 🏛️🔓🌱

Même si tu préfères continuer avec tes solutions actuelles, tu peux quand même bénéficier de KoproGo.

----

**KoproGo ASBL - Janvier 2025**

*"Impact social avant profit. Qualité avant vitesse. Pérennité avant croissance."*

**Transparence**\ : Ce document est public. Les comptes annuels de l'ASBL KoproGo seront publiés sur GitHub et le site web, conformément aux obligations légales belges et à notre engagement de transparence radicale.

**License document**\ : CC BY-SA 4.0 (Creative Commons Attribution-ShareAlike)

----

**Voir aussi** :

* :doc:`ROADMAP_PAR_CAPACITES` - Roadmap stratégique complète avec projections financières détaillées et évolution tarifaire
* :doc:`VISION` - Vision sociétale et impact écologique
* :doc:`MISSION` - Mission ASBL et valeurs fondamentales
* :doc:`GOVERNANCE` - Structure de gouvernance et processus de décision
