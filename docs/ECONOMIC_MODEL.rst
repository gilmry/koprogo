
Modèle Économique KoproGo - Guide Complet
=========================================

**Version 4.0 - Janvier 2025**
**Statut**\ : ASBL Belge (Association Sans But Lucratif)
**License**\ : Code AGPL-3.0 / Document CC BY-SA 4.0

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
#. `Gouvernance ASBL <#gouvernance-asbl>`_
#. `Opportunités de Soutien <#opportunités-de-soutien>`_
#. `Risques et Opportunités <#risques-et-opportunités>`_

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
#. **🌱 Écologie**\ : Infrastructure bas carbone (0.12g CO₂/req)
#. **💚 Bénévolat**\ : Contribution par passion

Principes Fondamentaux du Modèle
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


#. **Prix d'entrée minimal**\ : 1€/mois par copropriété
#. **Quotas d'espace raisonnables**\ : Suffisants pour 90% des usages
#. **Dépassement à prix coûtant**\ : Coûts additionnels dilués avec toute la communauté
#. **Transparence totale**\ : Facture détaillée consultable par tous
#. **Option self-hosted gratuite**\ : Liberté totale pour les utilisateurs techniques

----

Structure Juridique ASBL
------------------------

Qu'est-ce qu'une ASBL ?
^^^^^^^^^^^^^^^^^^^^^^^

**ASBL** = Association Sans But Lucratif (loi belge du 27 juin 1921, réformée en 2019)

**Définition légale**\ : Une ASBL est une personne morale qui ne cherche pas à procurer un gain matériel à ses membres. Tous les bénéfices doivent être réinvestis dans l'objet social de l'association.

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


ASBL vs Startup
^^^^^^^^^^^^^^^

.. list-table::
   :header-rows: 1

   * - Aspect
     - Startup
     - ASBL KoproGo
   * - **Objectif**
     - Profit actionnaires
     - Impact social
   * - **Code**
     - Propriétaire
     - Open source (AGPL-3.0)
   * - **Prix**
     - Maximum acceptable
     - Prix coûtant + marge solidaire
   * - **Données**
     - Monétisées
     - Propriété utilisateur
   * - **Gouvernance**
     - CEO + VC
     - Assemblée Générale
   * - **Bénéfices**
     - Dividendes
     - Réinvestis mission
   * - **Financement**
     - Levées de fonds
     - Autofinancement + dons
   * - **Pression croissance**
     - Très élevée
     - Aucune
   * - **Dilution**
     - 30-50%
     - 0%


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
   ✅ Gestion copropriétaires (GDPR compliant)
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

Modèle Hybride 20/80
^^^^^^^^^^^^^^^^^^^^

**Objectif 2028**\ : Répartition utilisateurs


* **20% Cloud KoproGo**\ : 400 copropriétés × 1.20€ = 480€/mois
* **80% Self-hosted**\ : 1,600 copropriétés × 0€ = 0€ revenus (autonomes)

**Les revenus cloud financent**\ :


#. Développement (contributeurs, temps partiel)
#. Infrastructure (VPS + S3 + DNS)
#. Support (documentation, forum, email)
#. Réserves (6-12 mois de fonctionnement)

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

Option 2: Cloud KoproGo (1€/mois) ☁️
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Offre Standard: 1€/mois TTC par copropriété**

**Inclus dans l'offre de base**\ :

.. list-table::
   :header-rows: 1

   * - Ressource
     - Quota Standard
     - Usage Typique
   * - **Stockage documents**
     - 500 MB
     - ~500 fichiers PDF/photos (1 MB moyen)
   * - **Utilisateurs**
     - 50
     - Suffisant pour immeuble 20-30 lots
   * - **Requêtes API**
     - 100,000/mois
     - ~3,300 req/jour (~140 req/h)
   * - **Bande passante**
     - 5 GB/mois
     - Téléchargement documents, consultation
   * - **Backup automatique**
     - Quotidien
     - Rétention 7 jours
   * - **Support**
     - Email (72h)
     - Documentation complète + forum


**Services Cloud inclus**\ :

.. code-block::

   ✅ Hébergement géré OVH France (datacenter bas carbone)
   ✅ Sauvegardes quotidiennes automatiques
   ✅ Mises à jour gratuites (rolling updates sans downtime)
   ✅ Support email (délai 48-72h)
   ✅ SSL/TLS inclus (Let's Encrypt)
   ✅ Monitoring uptime (99.7%+ garanti)
   ✅ Exports données (CSV, JSON, SQL)
   ✅ GDPR compliance (données EU)

**Pour qui?**


* Petites et moyennes copropriétés (5-30 lots)
* Usage standard: gestion charges, assemblées, documents
* ~90% des utilisateurs restent dans les quotas de base
* Copropriétés sans compétences techniques

Dépassement de Quotas: Prix Coûtant Mutualisé
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Philosophie**\ : Nous ne faisons **aucun profit** sur les dépassements. Les coûts additionnels sont calculés au **prix coûtant réel** et **mutualisés entre tous les utilisateurs** du cloud KoproGo.

Calcul du Prix Coûtant
~~~~~~~~~~~~~~~~~~~~~~

Le prix coûtant est calculé mensuellement et communiqué publiquement:

**Formule**\ :

.. code-block::

   Prix coûtant = (Coût infrastructure total + Coût bande passante + Coût stockage S3) / Nombre total copropriétés cloud

**Exemple Octobre 2025**\ :

.. code-block::

   Infrastructure VPS OVH (d2-2): 7€/mois
   Stockage S3 OVH (200 GB):     2€/mois (0.01€/GB)
   Bande passante (500 GB):      0€ (inclus)
   Support (bénévole):           0€
   Total coûts:                  9€/mois

   Nombre copropriétés cloud:    100
   Prix coûtant de base:         0.09€/copro/mois
   Marge ASBL (maintenance):     0.91€/copro/mois (91%)

Grille Tarifaire Dépassement (Prix Coûtant)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1

   * - Ressource
     - Coût Unitaire
     - Exemple Dépassement
     - Coût Additionnel
   * - **Stockage +100 MB**
     - 0.001€/GB/mois
     - 600 MB total
     - +0.10€/mois
   * - **Utilisateurs +10**
     - 0€
     - 60 users total
     - **Gratuit**
   * - **Requêtes API +50k**
     - 0€
     - 150k req/mois
     - **Gratuit**
   * - **Bande passante +1 GB**
     - 0.002€/GB
     - 6 GB/mois
     - +0.02€/mois


**Important**\ : Les quotas utilisateurs et requêtes API n'ont **aucun coût marginal** pour l'infrastructure, donc **aucun surcoût** en cas de dépassement.

Exemples Concrets de Tarification
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Cas 1: Copropriété 10 lots (usage léger)**


* Stockage: 200 MB (sous quota)
* Utilisateurs: 15 (sous quota)
* Requêtes: 30,000/mois (sous quota)
* **Coût total: 1.00€/mois** ✅

**Cas 2: Copropriété 50 lots (usage normal)**


* Stockage: 800 MB *(+300 MB)*
* Utilisateurs: 80 *(+30 users, gratuit)*
* Requêtes: 180,000/mois *(gratuit)*
* **Coût total: 1.30€/mois** (1€ base + 0.30€ stockage)

**Cas 3: Grande copropriété 100 lots (usage intensif)**


* Stockage: 2 GB *(+1.5 GB)*
* Utilisateurs: 150 *(+100 users, gratuit)*
* Requêtes: 500,000/mois *(gratuit)*
* Bande passante: 12 GB *(+7 GB)*
* **Coût total: 2.64€/mois** (1€ + 1.50€ stockage + 0.14€ BP)

**Comparaison avec concurrent propriétaire**\ :


* Solution SaaS classique: 200-500€/mois pour 100 lots
* **KoproGo: 2.64€/mois** (soit **99% d'économie**\ )

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

Chaque mois, l'ASBL publie un **rapport financier public** détaillant:


#. **Coûts infrastructure réels** (factures OVH)
#. **Nombre de copropriétés hébergées**
#. **Utilisation ressources** (stockage, BP, CPU)
#. **Prix coûtant calculé**
#. **Répartition revenus** (maintenance, développement, réserves)

**Accès**\ : Tableau de bord public sur `koprogo.com/transparence <https://koprogo.com/transparence>`_

**Exemple Format**\ :

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
   - Uptime: 99.94%
   - Latency P99: 3.2ms
   - CO2: 0.12g/req
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

Évolution Tarifs avec l'Échelle
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Plus de copropriétés = Prix plus bas**

.. list-table::
   :header-rows: 1

   * - Année
     - Copros Cloud
     - Coût Infra
     - Prix/Copro Possible
   * - **2025**
     - 100
     - 10€/mois
     - 1.00€
   * - **2026**
     - 400
     - 20€/mois
     - 0.70€
   * - **2028**
     - 1,000
     - 30€/mois
     - 0.50€
   * - **2030**
     - 2,000
     - 40€/mois
     - 0.40€


Réinvestissement Démocratique
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

L'\ **Assemblée Générale ASBL** vote chaque année pour:


* **Baisser le prix de base** (si réserves suffisantes)
* **Améliorer les quotas** (plus de stockage inclus)
* **Investir dans de nouvelles features**
* **Constituer des réserves** (sécurité)

**C'est la communauté qui décide, pas des actionnaires.**

----

Viabilité Financière
--------------------

Budget Prévisionnel 2025-2030
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Hypothèses conservatrices**\ :


* Croissance: 100 copros (2025) → 2,000 copros (2030)
* Répartition: 20% cloud, 80% self-hosted
* Prix moyen cloud: 1.20€/mois (avec dépassements)

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


**Réserves cumulées 2030**\ : ~15,000€ (soit 25 mois de fonctionnement)

Unit Economics Cloud ASBL
^^^^^^^^^^^^^^^^^^^^^^^^^

**LTV (Lifetime Value)**\ :

.. code-block::

   1€/copro/mois × durée vie moyenne
   - Churn: 5%/an (très faible, besoin réel)
   - Durée vie = 1 / 0.05 = 20 ans
   - LTV = 1€ × 12 mois × 20 ans = 240€ par copro

   Conservateur (10 ans): LTV = 120€

**CAC (Customer Acquisition Cost)**\ :

.. code-block::

   0€ marketing → CAC = 0€
   Temps bénévole si compté: ~5€ réaliste

**LTV/CAC**\ :

.. code-block::

   240€ / 5€ = 48:1 (exceptionnel)
   Target SaaS classique: 3:1
   KoproGo ASBL: 48:1 ✅

**Payback Period**\ :

.. code-block::

   CAC / MRR par copro = 5€ / 1€ = 5 mois
   Target SaaS: < 12 mois
   KoproGo: 5 mois ✅

**Gross Margin**\ :

.. code-block::

   Revenus 1€/copro/mois
   Coûts variables: ~0.01€/copro (compute)
   Marge brute: 99% ✅

**Conclusion Unit Economics**\ : Très sains, scalabilité énorme, pas de pression croissance.

Scénarios de Crise
^^^^^^^^^^^^^^^^^^

**Scénario 1: Chute revenus cloud (-50%)**


* Impact: Réduction développement à 0.5 ETP
* Solution: Appel communauté, campagne dons

**Scénario 2: Augmentation coûts infra (+100%)**


* Impact: Augmentation prix 1€ → 1.50€
* Vote Assemblée Générale requis

**Scénario 3: Pic usage (×10)**


* Impact: Migration K3s anticipée (Phase 2)
* Financement: Réserves cumulées

----

Impact Écologique
-----------------

Comparaison Carbone
^^^^^^^^^^^^^^^^^^^

**Solution classique (SaaS WordPress)**\ :


* Serveur dédié par client: 50W × 8760h = 438 kWh/an
* Datacenter standard: 438 kWh × 0.3 kg CO2/kWh = **131 kg CO2/an**

**KoproGo Cloud (mutualisé)**\ :


* VPS partagé: 10W / 1,000 copros = 0.01W par copro
* Datacenter bas carbone (GRA11): 0.01W × 8760h × 0.06 kg CO2/kWh = **0.0053 kg CO2/an**
* **Réduction: 99.996%** 🌱

Politique Green IT
^^^^^^^^^^^^^^^^^^


#. **Datacenter bas carbone**\ : OVH GRA11 (60g CO2/kWh vs 300g moyenne)
#. **Mutualisation maximale**\ : 1,000+ copros sur 1 VPS
#. **Architecture Rust**\ : 10x moins de CPU que Python/Node.js
#. **Progressive Web App**\ : Cache local, moins de requêtes réseau
#. **Backup intelligent**\ : Déduplication, compression

**Mesures concrètes**\ :


* 0.12g CO₂/req (OVH France, mix 60g CO₂/kWh)
* 5.8x moins d'émissions que Hetzner DE
* 7-25x moins que AWS/Azure US

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
     - **1-3€**
     - 0.5-∞ GB
     - Email 72h
     - ✅ EU/Local
     - **0.005 kg**
   * - **KoproGo Self-Hosted**
     - **0€**
     - ∞
     - Communauté
     - ✅ Total
     - **0.001 kg**


**Économie moyenne**\ : **1,600-9,500€/an par copropriété** (soit 95-99% de réduction)

Économies Réalisées
^^^^^^^^^^^^^^^^^^^

**Exemple: Copropriété 20 lots sur 3 ans**

.. list-table::
   :header-rows: 1

   * - Poste
     - Propriétaire
     - KoproGo Cloud
     - Économie
   * - Licence
     - 3,000€
     - 12€
     - −99.6%
   * - Formation
     - 800€
     - 0€
     - −100%
   * - Migration
     - 500€
     - 0€
     - −100%
   * - **Total**
     - **10,400€**
     - **36€**
     - **−99.65%**


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

Gouvernance ASBL
----------------

Assemblée Générale (AG)
^^^^^^^^^^^^^^^^^^^^^^^

**Composition**\ : Tous les membres de l'ASBL

**Membres fondateurs**\ :


* Architecte logiciel (fondateur)
* Étudiante informatique (fondatrice)
* +1 membre externe (diversité: juriste ou comptable bénévole)

**Cotisation membres**\ : 0€ (ASBL accessible à tous)

**Pouvoirs de l'AG**\ :


* Modification des statuts
* Nomination/révocation administrateurs
* Approbation comptes annuels
* Dissolution de l'ASBL
* **Vote prix et affectation excédents**

**Fréquence**\ : 1x/an minimum (obligatoire) + AG extraordinaires si besoin

Conseil d'Administration (CA)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**Composition**\ : 3 administrateurs minimum (loi belge)

**Administrateurs KoproGo**\ :


* Architecte logiciel (Président)
* Étudiante informatique (Secrétaire)
* Membre externe (Trésorier)

**Mandat**\ : 4 ans renouvelables

**Rémunération**\ : **0€** (bénévolat pur en phase bootstrap)

**Pouvoirs du CA**\ :


* Gestion quotidienne de l'ASBL
* Décisions stratégiques (roadmap, investissements)
* Recrutement/indemnisation contributeurs (si trésorerie suffisante)
* Représentation de l'ASBL

**Fréquence réunions**\ : Trimestrielles (4x/an) + ad-hoc si urgent

Gestion Journalière
^^^^^^^^^^^^^^^^^^^

**Délégation**\ : Le CA peut déléguer la gestion journalière

**KoproGo**\ : Architecte logiciel = gestionnaire journalier délégué


* Décisions opérationnelles (infrastructure, déploiements)
* Engagement dépenses < 500€ (au-delà: validation CA)
* Représentation ASBL (contrats fournisseurs)

Obligations Légales ASBL
^^^^^^^^^^^^^^^^^^^^^^^^

**Comptabilité simplifiée** (si revenus < 500k€/an):


* Livre journal des recettes/dépenses
* Inventaire annuel actifs/passifs
* Budget prévisionnel annuel

**Comptes annuels**\ :


* Dépôt à la Banque Nationale de Belgique (BNB)
* Délai: 6 mois après clôture exercice
* Accessibles au public (transparence)

**TVA**\ : Exonération possible si activités à caractère social, éducatif ou culturel

**Impôt sur les Sociétés (ISOC)**\ : Exonération si activités conformes à l'objet social non-lucratif

Rémunération dans l'ASBL
^^^^^^^^^^^^^^^^^^^^^^^^

**Principes légaux belges**\ :


#. **Administrateurs**\ : Pas de rémunération (sauf remboursement frais réels)
#. **Bénévoles**\ : Indemnités forfaitaires autorisées (max ~40€/jour, 2,000€/an)
#. **Salariés**\ : Rémunération normale possible si justifiée et approuvée par AG

**Timeline KoproGo**\ :


* **Années 1-3**\ : **0€ rémunération** (bénévolat pur)
* **Année 4**\ : **Indemnités forfaitaires** si trésorerie > 10k€ (500€/mois max)
* **Année 5+**\ : **Salaires partiels** si trésorerie > 30k€ (1,500€/mois brut mi-temps)

Dissolution ASBL
^^^^^^^^^^^^^^^^

En cas de fin de mission:


#. **Décision AG**\ : Dissolution votée à majorité qualifiée (2/3)
#. **Liquidation**\ : Remboursement dettes, vente actifs
#. **Boni de liquidation**\ : **Interdit de distribuer aux membres**
#. **Attribution**\ : Actifs donnés à ASBL similaire ou d'utilité publique

**KoproGo**\ : En cas de dissolution, code source reste AGPL-3.0 (open-source perpétuel), infrastructure donnée à autre ASBL tech sociale (ex: Framasoft).

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
* ✅ Performance tech: Rust, 0.12g CO₂/req
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

2. Tendance Anti-Vendor Lock-In
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Contexte**\ : 2025-2030 = décennie décentralisation, souveraineté numérique

**Upside**\ :


* ✅ GDPR natif: Données EU, conformité totale
* ✅ Souveraineté: OVH France, pas de CLOUD Act
* ✅ Écologie: 0.12g CO₂/req, mix français bas carbone
* ✅ Éthique: ASBL non-profit vs SaaS profit-driven

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

**Le coût de KoproGo ne dépassera jamais 5€/mois par copropriété**\ , quel que soit le succès du projet. Tout excédent sera réinvesti dans le développement, la communauté, ou redistribué via baisse de prix.

L'Équipe (Bénévole Jusqu'à Viabilité)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

**2 personnes, 0.25 FTE équivalent**\ :


#. 
   **Architecte Logiciel** (10-20h/semaine en side-project)


   * Emploi temps plein ailleurs (sécurité financière)
   * Architecture + développement core Rust
   * DevOps + infrastructure OVH
   * Vision produit long-terme

#. 
   **Étudiante en Informatique** (5-10h/semaine bénévole)


   * Formation pratique sur projet réel
   * Maintenance, documentation, tests
   * Community management GitHub
   * Contribution au CV professionnel

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

----

**KoproGo ASBL - Janvier 2025**

*"Impact social avant profit. Qualité avant vitesse. Pérennité avant croissance."*

**Transparence**\ : Ce document est public. Les comptes annuels de l'ASBL KoproGo seront publiés sur GitHub et le site web, conformément aux obligations légales belges et à notre engagement de transparence radicale.

**License document**\ : CC BY-SA 4.0 (Creative Commons Attribution-ShareAlike)
