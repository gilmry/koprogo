================================
Parcours Investisseur
================================

**Vous évaluez KoproGo pour un investissement ou une subvention ?**

Ce parcours vous présente l'essentiel en **5-10 minutes**.

.. contents:: Table des matières
   :depth: 2
   :local:

L'Histoire Humaine
==================

**Marie, 72 ans, retraitée** - Son syndic lance des travaux : 15 000€ sa quote-part. Elle conteste les devis excessifs. Un avocat coûte 2 000€. Sa pension : 1 200€/mois. Elle ne peut pas se défendre.

**Ahmed, 35 ans, intérimaire** - Trois mois de chômage technique. Résultat : 3 200€ d'impayés de charges. Les huissiers interviennent. Il risque la saisie.

**Sofiane, 40 ans, auto-entrepreneur** - La toiture doit être refaite : 12 000€ sa quote-part. Les banques refusent (pas de CDI). Les travaux sont bloqués.

**Ces situations ne sont pas exceptionnelles.** Elles reflètent les défis quotidiens de milliers de copropriétaires en Belgique.

Le Problème Sociétal
====================

**200 000 copropriétés belges** font face à :

* **Coûts élevés** : 200-500€/mois pour les solutions logicielles existantes
* **70M€/an dépensés** collectivement en Belgique
* **Empreinte carbone** importante (11,5g CO₂/requête en moyenne pour les solutions SaaS)
* **Manque de transparence** dans les calculs de charges
* **Litiges fréquents** faute de traçabilité
* **Exclusion financière** des copropriétaires en difficulté

La Solution Technique
=====================

**KoproGo** : Plateforme open-source de gestion de copropriété développée par une ASBL belge.

Architecture Ultra-Optimisée
-----------------------------

* **Backend** : Rust 1.83 + Actix-web (10x plus performant que Python/Node.js)
* **Frontend** : Astro + Svelte (PWA offline-first)
* **Base de données** : PostgreSQL 15
* **Infrastructure** : OVH France (datacenter bas carbone, 60g CO₂/kWh)

**Performance validée** (tests en production, Nov 2025) :

* **287 req/s** soutenus
* **752ms** latence P99 (1 vCPU)
* **0,12g CO₂/requête** (96% réduction vs solutions SaaS classiques)
* **99,74% uptime**

Fonctionnalités Opérationnelles
--------------------------------

* **73 endpoints REST API** (CRUD complet)
* **11 entités du domaine** (Building, Unit, Owner, Expense, etc.)
* **Multi-owner support** (pourcentages de propriété)
* **Multi-role support** (syndic, copropriétaire, comptable)
* **Comptabilité belge** (PCMN conforme AR 12/07/2012)
* **Workflow factures** (Draft → Approval → Paid)
* **Recouvrement automatisé** (4 niveaux d'escalade)

Le Modèle Économique
=====================

Modèle OpenCore
---------------

* **Self-hosted** : Gratuit (AGPL-3.0, toujours)
* **Cloud managé** : 5€/mois par copropriété

Viabilité Financière
---------------------

.. list-table:: Projections par Palier
   :header-rows: 1
   :widths: 15 15 20 20 30

   * - Palier
     - Copros
     - Revenus/mois
     - Coûts/mois
     - Surplus
   * - **100**
     - 100
     - 200€
     - 20€
     - 180€ (autofinancement)
   * - **500**
     - 500
     - 1 000€
     - 500€
     - 500€ (1 dev temps partiel)
   * - **2 000**
     - 2 000
     - 4 000€
     - 1 500€
     - 2 500€ (2 ETP)
   * - **5 000**
     - 5 000
     - 10 000€
     - 3 000€
     - **7 000€** (5 ETP + R&D)

*Note : Coûts = Infrastructure + RH*

**Pas de levée de fonds nécessaire** : Croissance organique autofinancée par revenus cloud.

Allocation du Surplus (votée en AG)
------------------------------------

* **30%** Fonds de Solidarité (aide financière membres)
* **40%** Développement features & R&D
* **30%** Réserves légales

**Si surplus > 25%** : l'Assemblée Générale peut voter une baisse tarifaire (ex: 5€ → 4€ → 3€).

Gouvernance ASBL
================

Structure Juridique
-------------------

* **ASBL belge** sans actionnaires
* **1 membre = 1 voix** (Assemblée Générale)
* **Prix voté démocratiquement** par l'AG
* **Transparence comptable** : bilans trimestriels publics
* **Transition optionnelle** : ASBL → Coopérative (2028+)

Équipe & Contributeurs
----------------------

**Aujourd'hui (Nov 2025)**

* Solo dev (Gilles Maury) + IA assistée
* 10-20h/semaine (bénévole)
* Communauté : ~10 contributeurs actifs

**Objectif 2026-2027**

* 1-2 ETP (devs Rust/Svelte)
* 50-100 contributeurs actifs
* Communauté : 500-1 000 membres

La Roadmap
==========

Progression par Capacités (Pas Dates Fixes)
--------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 15 20 20 30

   * - Jalon
     - Copros
     - État
     - Durée Estimée
     - Débloque
   * - **0-1**
     - 100
     - **70% fait**
     - 2-4 sem
     - Beta publique
   * - **2**
     - 500
     - Prochaine priorité
     - 4-12 sem
     - Production ouverte
   * - **3**
     - 1 000
     - Moyen terme
     - 5-14 sem
     - Différenciation marché
   * - **4-5**
     - 2 000-5 000
     - Long terme
     - 6-18 mois
     - Mobile + API publique
   * - **6-7**
     - 10 000+
     - Très long terme
     - 24-48 mois
     - PropTech 2.0 (IA, IoT)

**Philosophie** : Nous livrons quand c'est prêt, pas selon un calendrier. La durée dépend de la force de travail (clients + contributeurs).

L'Impact Sociétal
=================

Impact par Palier
-----------------

.. list-table::
   :header-rows: 1
   :widths: 15 20 20 20 25

   * - Palier
     - Économies/an
     - CO₂ évité/an
     - Aidés/an
     - Impact Social
   * - **100**
     - 80k€
     - -2 tonnes
     - 3-5
     - Beta publique
   * - **500**
     - 400k€
     - -15 tonnes
     - 10-15
     - Production ouverte
   * - **2 000**
     - 1,6M€
     - -214 tonnes
     - 30-40
     - Référence belge
   * - **5 000**
     - **4M€**
     - **-840 tonnes**
     - **40-60**
     - Leadership EU

Fonds de Solidarité
-------------------

**Budget attendu** (5 000 copros) : ~45k€/an (30% surplus)

**Aide financière** :

* **Prêts 0%** pour impayés de charges
* **Aide litiges** (2 000€ max) pour contestation AG
* **Crédits solidaires** travaux urgents (1-2% vs 4-6% banques)
* **Subventions urgence** pour situations extrêmes

**Impact** : 40-60 copropriétaires aidés/an, 15-20 litiges évités/an, économie 200k€/an pour les copropriétés.

Métriques de Succès
===================

KPIs Techniques
---------------

* **Performance** : Latency P99 < 1s, Throughput > 200 req/s
* **Fiabilité** : Uptime > 99,9%
* **Écologie** : CO₂/requête < 0,5g
* **Sécurité** : 0 CVE non patchées, GDPR 100% conforme

KPIs Business
-------------

* **Croissance** : Nombre de copropriétés actives
* **Rétention** : Taux de désabonnement < 5%
* **NPS** : Net Promoter Score > 50
* **Contribution** : Nombre de contributeurs actifs

KPIs Impact
-----------

* **Économies** : €/an économisés collectivement
* **Écologie** : Tonnes CO₂/an évitées
* **Solidarité** : Nombre de personnes aidées/an
* **Communauté** : Adoption modules communautaires (SEL, partage)

Risques & Mitigation
=====================

Risques Identifiés
------------------

**Technique**

* Complexité architecture hexagonale
* **Mitigation** : Documentation exhaustive, tests 100%

**Marché**

* Concurrence établie (acteurs en place)
* **Mitigation** : Différenciation prix (99% moins cher) + impact social

**Adoption**

* Résistance changement syndics
* **Mitigation** : Modules compatibles avec outils existants, migration progressive

**Financement**

* Croissance lente = revenus insuffisants
* **Mitigation** : Self-hosting gratuit toujours possible, communauté contributeurs

**Légal**

* Conformité législation belge
* **Mitigation** : Expert comptable copropriété, PCMN conforme

Opportunités
============

Subventions Potentielles
-------------------------

* **Économie sociale** : ASBL → Coopérative éligible subventions
* **Écologie** : 96% réduction CO₂ → Green IT grants
* **Innovation** : PropTech 2.0 → R&D européenne
* **Social** : Fonds de Solidarité → fondations philanthropiques

Partenariats Stratégiques
--------------------------

* **Syndics professionnels** : Beta testing, feedback produit
* **Universités** : Stages, mémoires, R&D
* **Fédérations copropriétés** : White-label, déploiement massif
* **ONG** : Impact social, solidarité financière

Expansion Géographique
-----------------------

* **Belgique** : 200 000 copropriétés (marché primaire)
* **France** : 750 000 copropriétés (expansion 2028+)
* **Benelux** : NL, LU (adaptation législative)
* **Europe** : 150M personnes en copropriété (vision long terme)

Documents Complets
==================

Pour approfondir :

* :doc:`INVESTOR_EXECUTIVE_SUMMARY_2025` - Résumé exécutif détaillé
* :doc:`economic-model/modele-economique` - Modèle économique complet
* :doc:`roadmap/roadmap-2025-2030` - Roadmap technique détaillée
* :doc:`INFRASTRUCTURE_COST_SIMULATIONS_2025` - Simulations coûts infrastructure
* :doc:`PERFORMANCE_REPORT` - Rapport de performance validée
* :doc:`vision-strategie/fonds-solidarite` - Fonds de Solidarité détaillé

Contact
=======

**Intéressé par un investissement ou une subvention ?**

* **Email** : contact@koprogo.com
* **GitHub** : https://github.com/gilmry/koprogo
* **Discussions** : https://github.com/gilmry/koprogo/discussions

Nous sommes ouverts aux partenariats stratégiques et aux subventions alignées avec notre mission sociétale.

----

*Parcours Investisseur - KoproGo ASBL*

*Dernière mise à jour : 2025-01-19*
