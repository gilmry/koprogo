============================
Jalons À Venir
============================

:Date: 2025-01-19
:État: Ce qui vient ensuite

.. contents:: Table des matières
   :depth: 2
   :local:

Introduction
============

Cette page présente les **jalons à venir** de KoproGo, organisés par ordre de priorité et capacités débloquées.

.. note::
   **Progression par Capacités**

   Nous livrons quand les capacités sont atteintes, pas selon des dates fixes. Chaque jalon débloque le suivant.

Prochaine Étape : Jalon 2 (Priorité Immédiate)
===============================================

Jalon 2 : Conformité Légale Belge
----------------------------------

**Débloque** : 200-500 copropriétés (production ouverte)

**Objectif** : Conformité légale complète pour le marché belge

**Conformité légale** : 80%

Issues Critiques
~~~~~~~~~~~~~~~~

⏳ **#17 : État Daté** (BLOQUANT pour ventes immobilières)

* Génération PDF conforme législation belge
* Signature syndic (certificat numérique)
* Validation par notaires (beta testing)
* API endpoints : Create, Read, Download PDF

**Impact** : Débloque ventes de lots (feature CRITIQUE pour adoption)

⏳ **#18 : Budget Prévisionnel Annuel**

* Création budgets avec catégories PCMN
* Variance analysis (budget vs réalisé)
* Approbation AG (workflow)
* Rapports PDF conformes

**Impact** : Obligation légale copropriétés belges

⏳ **#22 : Conseil de Copropriété** (>20 lots)

* Dashboard conseil avec alertes
* Gestion membres conseil
* Décisions conseil (historique)
* Rapports syndic

**Impact** : Débloque copros >20 lots (60% du marché belge)

⏳ **#23 : Workflow Recouvrement Complet**

* Intégration huissiers (API)
* Génération courriers recommandés
* Suivi procédures judiciaires
* Reporting impayés

**Impact** : Automatisation recouvrement, gain temps syndic

Effort Estimé
~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/sem
     - Durée estimée
     - Note
   * - **Solo bootstrap**
     - 10-20h
     - 4-6 mois
     - Complexité légale
   * - **Duo fondateurs**
     - 40-60h
     - 8-12 semaines
     - Expertise comptable requise
   * - **Équipe projet**
     - 80-120h
     - 4-6 semaines
     - +Expert comptable copro

**Dépendances** : Jalon 1 (Sécurité) DOIT être terminé avant (données sensibles)

Bloquants Levés
~~~~~~~~~~~~~~~

**État daté** : Permet ventes de lots (CRITIQUE pour adoption)

**Conseil copropriété** : Débloque copros >20 lots (60% du marché belge)

**Comptabilité conforme** : Crédibilité auprès syndics professionnels

**Résultat** : Production ouverte pour syndics professionnels

Jalons Moyen Terme (6-12 Mois)
===============================

Jalon 3 : Features Différenciantes
-----------------------------------

**Débloque** : 500-1 000 copropriétés (différenciation marché)

**Objectif** : Se différencier de la concurrence par l'impact social

**Conformité légale** : 90%

Issues Importantes
~~~~~~~~~~~~~~~~~~

⏳ **#46 : Voting Digital Basique**

* Scrutins AG avec signature itsme®
* Stockage PostgreSQL (non-blockchain)
* Conformité légale belge (suffisant)
* Traçabilité complète

.. note::
   **Blockchain Voting** (Jalon 7) ajoute immutabilité Polygon mais nécessite expertise blockchain + audit sécurité.

⏳ **#47 : PDF Generation Étendue**

* Templates tous documents légaux
* Procès-verbaux AG automatiques
* Convocations personnalisées
* Rapports syndic

⏳ **#49 : Module SEL** (Système d'Échange Local)

* Monnaie locale virtuelle (PostgreSQL)
* Échanges services entre voisins
* Historique transactions
* Dashboard communautaire

**Impact** : Économie circulaire 750k€/an (30% adoption)

⏳ **#26 : Partage d'Objets**

* Bibliothèque objets communautaire
* Réservations/emprunts
* Système de notation
* Calendrier disponibilité

**Impact** : 12 000 objets partagés/an, -500 tonnes CO₂/an

⏳ **#52 : Contractor Backoffice**

* Espace prestataires
* Devis en ligne
* Suivi chantiers
* Évaluations

**Impact** : Marketplace locale, économie de plateforme

⏳ **#110 : Achats Groupés d'Énergie** 🔥 (DIFFÉRENCIATEUR MAJEUR)

* Workflow complet : Lancement campagne → Upload factures → Agrégation anonyme → Négociation courtier → Vote AG → Switch automatisé
* Upload factures PDF + OCR automatique (extraction consommation kWh)
* Chiffrement AES-256-GCM (données individuelles)
* Agrégation anonyme au niveau building (k-anonymat ≥ 5)
* Intégration CREG API (tarifs officiels belges)
* Consentement GDPR explicite (opt-in individuel)
* Génération contrats pré-remplis (PDF par unité)
* Certification CREG (label qualité)

**Impact** :

* **Économique** : 15-25% d'économies sur factures énergie (125 €/an/unité en moyenne)
* **Social** : Pouvoir de négociation collectif face à l'oligopole belge (Engie, Luminus, TotalEnergies >70% marché)
* **Écologique** : Incentive fournisseurs 100% renouvelables (2,000 tonnes CO₂/an évitées si 50% choix vert)
* **Traction** : **Unique sur le marché** (aucun outil SaaS ne propose regroupement spontané entre copropriétés)
* **ROI** : 4,067% (économie ÷ coût plateforme 5€/mois)
* **Législation favorable** : Belgique encourage achats groupés (Charte CREG 2013/2018)

.. note::
   **Pourquoi Jalon 3 (pas Jalon 6) ?**

   Contrairement à l'Issue #110 initiale (qui dépendait de l'IoT platform Jalon 6), cette implémentation utilise des **factures signées/authentifiées** (upload PDF + OCR), ce qui est :

   * ✅ Indépendant du matériel (universel)
   * ✅ Mise en œuvre rapide (2 semaines MVP)
   * ✅ GDPR-compliant par design
   * ✅ Pas de dépendance sur capteurs IoT temps réel

   **Résultat** : Déploiement dès Phase 2 (VPS) au lieu de Phase 3 (K8s + IoT).

Effort Estimé
~~~~~~~~~~~~~

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/sem
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

**Dépendances** : Jalon 2 (Conformité) recommandé mais non bloquant pour modules communautaires

Impact Attendu
~~~~~~~~~~~~~~

**Différenciation** : SEL + Partage = unique sur le marché

**Impact social** : Modules communautaires créent lien social

**Impact écologique** : 790 tonnes CO₂/an évitées (partage objets + SEL)

**Économie circulaire** : 750k€/an échanges SEL (30% adoption)

**Marketing naturel** : "La plateforme avec communauté" = viral

Jalon 4 : Automation & Intégrations
------------------------------------

**Débloque** : 1 000-2 000 copropriétés (scalabilité)

**Objectif** : Automatisation workflows, réduction temps syndic

**Conformité légale** : 95%

Issues
~~~~~~

⏳ **#19 : Convocations AG Automatiques**

* Workflow AG 100% automatisé
* Génération documents (ordre du jour, procès-verbal)
* Envoi emails/courriers personnalisés
* Rappels automatiques

**Impact** : Temps syndic réduit 50%

⏳ **#20 : Carnet d'Entretien Digital**

* Suivi maintenance (ascenseurs, chauffage, etc.)
* Alertes préventives
* Historique interventions
* Conformité légale (obligation)

⏳ **#21 : GDPR Complet** (Articles 16, 18, 21)

* Rectification données
* Limitation traitement
* Opposition traitement
* Audit automatisé

**Impact** : Conformité 100% GDPR

⏳ **#24 : Module Devis Travaux**

* Comparaison devis multi-entrepreneurs
* Workflow validation AG
* Suivi chantiers
* Paiements échelonnés

⏳ **#25 : Affichage Public Syndic**

* Pages publiques syndics (SEO)
* Présentation copropriété
* Contact direct
* Témoignages

**Impact** : Discovery organique via Google

⏳ **#27 : Accessibilité WCAG 2.1 AA**

* Navigation clavier
* Lecteurs d'écran (ARIA)
* Contraste AA (4.5:1)
* Responsive mobile

**Impact** : Conformité European Accessibility Act 2025

Effort Estimé
~~~~~~~~~~~~~

**6-10 semaines** (équipe projet 80-120h/sem) ou **6-10 mois** (solo bootstrap)

Jalons Long Terme (12-24 Mois)
===============================

Jalon 5 : Mobile & API Publique
--------------------------------

**Débloque** : 2 000-5 000 copropriétés (expansion)

**Features**

* Progressive Web App installable
* API publique v1 (OpenAPI)
* Multi-langue NL/FR/DE/EN complet
* Intégrations comptables (Winbooks, Exact)
* Notifications intelligentes
* Analytics & Dashboards

**Impact**

* **Écosystème** : API publique → développeurs tiers
* **Intégrations** : Winbooks/Exact → syndics professionnels
* **Mobile** : PWA → adoption copropriétaires
* **International** : Multi-langue → expansion EU

**Effort estimé** : 14-18 semaines (équipe structurée avec mobile dev + API architect)

Jalons Très Long Terme (24-48 Mois)
====================================

Jalon 6 : Intelligence & Expansion (PropTech 2.0)
--------------------------------------------------

**Débloque** : 5 000-10 000 copropriétés (leadership)

.. warning::
   **PropTech 2.0 Zone** : Modules avancés nécessitant **maturité technique complète + équipe 3-4 ETP minimum**

**Features Avancées**

* ⚠️ IA Assistant Syndic (GPT-4/Claude via OVH)
* ⚠️ API Bancaire PSD2 (réconciliation auto)
* ⚠️ IoT Sensors (monitoring temps réel, maintenance prédictive, DPE automatisé)
* Marketplace Services Locaux
* Prédictions budgétaires (ML)
* Multi-region (Benelux)

.. note::
   **IoT Sensors vs Achats Groupés d'Énergie**

   Les capteurs IoT (Jalon 6) servent au **monitoring temps réel** (fuites eau, surconsommations, anomalies), pas aux achats groupés d'énergie.

   **Achats groupés** (Jalon 3) : Basés sur factures annuelles uploadées (PDF + OCR), pas besoin de capteurs temps réel.

**Prérequis CRITIQUES**

✅ Base utilisateurs stable (>2 000 copros)
✅ Revenus >10 000€/mois pour R&D
✅ Équipe structurée : +Data scientist, +IoT engineer, +FinTech expert

**Effort estimé** : 18-24 semaines (équipe 3-4 ETP)

Jalon 7 : Platform Economy (PropTech 2.0)
------------------------------------------

**Débloque** : 10 000+ copropriétés (scale planétaire)

.. warning::
   **PropTech 2.0 Expérimental** : Features blockchain nécessitant **équipe 10-15 ETP + audits sécurité externes**

**Vision Long Terme**

* SDK multi-langages (Python, JS, PHP, Ruby)
* Store modules tiers (marketplace plugins)
* ⚠️ **Blockchain Voting** (votes AG immutables, Polygon)
* ⚠️ **Carbon Credits Trading** (tokenisation CO₂)
* White-label pour fédérations
* Interopérabilité EU (API standards CEN)

**Prérequis CRITIQUES**

✅ Organisation mature (10-15 ETP)
✅ Revenus >50 000€/mois
✅ Budget audits sécurité (50-100k€/audit)
✅ Agrément trading carbone (FSMA, AMF)

**Effort estimé** : 24-36 semaines (organisation mature)

Résumé des Priorités
=====================

.. list-table:: Ordre de Priorité des Jalons
   :header-rows: 1
   :widths: 10 20 20 20 30

   * - Jalon
     - Priorité
     - Copros Débloquées
     - Durée Estimée
     - Prérequis
   * - **2**
     - **IMMÉDIATE**
     - 200-500
     - 4-12 semaines
     - Jalon 1 terminé
   * - **3**
     - Moyen terme
     - 500-1 000
     - 5-14 semaines
     - Jalon 2 recommandé
   * - **4**
     - Moyen terme
     - 1 000-2 000
     - 6-10 semaines
     - Jalon 3 recommandé
   * - **5**
     - Long terme
     - 2 000-5 000
     - 14-18 semaines
     - Équipe structurée
   * - **6**
     - Très long terme
     - 5 000-10 000
     - 18-24 semaines
     - 3-4 ETP + R&D
   * - **7**
     - Expérimental
     - 10 000+
     - 24-36 semaines
     - 10-15 ETP + audits

Conclusion
==========

**Prochaine Action** : Compléter Jalon 2 (Conformité Légale Belge)

Ce jalon est **bloquant** pour 60% du marché belge (copros >20 lots) et critique pour les ventes immobilières (état daté).

**Après Jalon 2** : Production ouverte pour syndics professionnels → Croissance organique → Jalons 3-4 accessibles

**Vision Long Terme** : PropTech 2.0 (Jalons 6-7) ne démarre qu'après **maturité complète du produit** et **équipe structurée**.

----

**Voir Aussi**

* :doc:`jalons-atteints` - Ce qui est déjà fait
* :doc:`roadmap-2025-2030` - Roadmap complète 2025-2030
* :doc:`../ROADMAP_PAR_CAPACITES` - Roadmap détaillée par capacités

----

*Jalons À Venir - Documentation KoproGo ASBL*

*Dernière mise à jour : 2025-01-19*
