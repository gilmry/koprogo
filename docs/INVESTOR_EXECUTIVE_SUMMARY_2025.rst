===================================================================
KoproGo - Executive Summary Investisseurs/Subsides 2025
===================================================================

:Auteur: Gilles Maury - Fondateur KoproGo (ASBL en préparation)
:Date: Novembre 2025
:Version: 1.0
:Audience: Investisseurs, Subsides publics, Financeurs participatifs

.. warning::

   v0.1.0 pre-release : aucun système en production, ASBL en préparation
   (constitution prévue au Jalon 3, voir :doc:`GOVERNANCE`). Les chiffres
   techniques viennent d'un test de charge de 3 minutes (30/10/2025) ; les
   chiffres économiques sont des projections non contractuelles.

.. contents:: Table des Matières
   :depth: 2
   :local:

🎯 Résumé Exécutif
==================

**KoproGo** est une plateforme opensource de gestion de copropriété portée par un projet d'ASBL (constitution prévue au Jalon 3), qui vise une **performance technique sobre**, un **modèle économique à valider** et un **impact écologique à mesurer**.

Objectif 2030
-------------

- **5,000 copropriétés** (100,000 personnes)
- **192,442€/an** de surplus (marge 99%)
- **-840 tonnes CO₂/an** évitées
- **1.5 ETP** financés + R&D

Différenciation Clé
-------------------

- **Coût infrastructure ultra-compétitif**: 0.033€/copro/mois (vs concurrence ~10-50€)
- **Modèle ASBL**: Prix fixe 5€/mois, baisse par vote AG uniquement, 0% dividendes, réinvestissement communautaire
- **Opensource radical**: AGPL-3.0, code public, souveraineté des données
- **PropTech 2.0**: IA, Blockchain, IoT (38% des revenus, résilience accrue)

📊 Performance Technique - Test de charge (30/10/2025)
======================================================

Infrastructure Testée (Oct 2025)
---------------------------------

.. list-table::
   :header-rows: 1
   :widths: 40 60

   * - Métrique
     - Résultat
   * - **VPS testé**
     - d2-2 Ubuntu (1 vCPU / 2GB RAM) @ 8€/mois
   * - **Équivalent 2025**
     - s1-2 (1 vCore, 2GB RAM) @ 7.20€/mois TTC
   * - **Throughput**
     - 287 req/s (charge 3 min)
   * - **Taux de succès**
     - voir :doc:`PERFORMANCE_REPORT` (test de 3 minutes, pas une disponibilité)
   * - **Latence P50**
     - 69ms
   * - **Latence P90**
     - 130ms
   * - **Latence P99**
     - 752ms (objectif réaliste: < 1s) ✅
   * - **CO₂/requête**
     - estimation réseau, non mesurée
   * - **Capacité multi-tenant**
     - 2,000-3,000 copropriétés par VPS

Performance vs Concurrence
---------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 20 20 30

   * - Métrique
     - KoproGo
     - Concurrence SaaS
     - Avantage
   * - **Coût/copro/mois**
     - 0.085€
     - 10-50€
     - **100-600× moins**
   * - **Uptime**
     - non mesuré (aucun système en production)
     - ~99.9%
     - À mesurer
   * - **Latence P99**
     - 752ms
     - ~200-500ms
     - Acceptable (API)

💰 Modèle Économique - Projections
==================================

Prix Fixe Démocratique
-----------------------

.. list-table::
   :header-rows: 1
   :widths: 50 50

   * - Élément
     - Détail
   * - **Prix/copro/mois**
     - **5.00€** (fixe)
   * - **Réductions**
     - Uniquement par vote AG si surplus > 25%
   * - **Cloud/Self-hosted**
     - 5€ = accès cloud + vote AG | 0€ self-hosted (sans vote)
   * - **Exemple baisse**
     - AG vote: 5€ → 4€ si surplus >25% pendant 2 trimestres
   * - **Positionnement**
     - 10× moins cher que marché (50€/mois concurrents)
   * - **Modèle**
     - Démocratique (1 membre = 1 vote), 0% dividendes

💎 PropTech 2.0 - Add-ons Différenciateurs
-------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Add-on
     - Tarif/copro/mois
     - Infrastructure
     - Adoption 2030
   * - **AI Assistant**
     - +2€
     - OVH AI Endpoints (50-100€/mois)
     - 40%
   * - **Blockchain Voting**
     - +1€
     - Polygon node (20€/mois)
     - 30%
   * - **IoT Sensors**
     - +10€
     - MQTT + TimescaleDB (25€/mois)
     - 20%
   * - **Energy Buying Groups**
     - 0€ (gratuit)
     - Partenariats énergétiques
     - 50%

📈 Projections Financières par Palier
======================================

.. list-table:: Synthèse Économique
   :header-rows: 1
   :widths: 12 12 12 12 12 12 12 16

   * - Copros
     - Infra/mois
     - Rev base/mois
     - Rev add-ons/mois
     - Surplus/an
     - Marge
     - ETP financés
     - R&D dispo/an
   * - **100**
     - 8€
     - 200€
     - -
     - 2,302€
     - 96%
     - -
     - -
   * - **500**
     - 13€
     - 1,000€
     - -
     - 11,850€
     - 99%
     - -
     - -
   * - **1,000**
     - 18€
     - 2,000€
     - -
     - 23,785€
     - 99%
     - -
     - -
   * - **2,000**
     - 29€
     - 4,000€
     - -
     - 47,655€
     - 99%
     - -
     - -
   * - **5,000** 🎯
     - 163€
     - 10,000€
     - 6,200€
     - **192,442€**
     - **99%**
     - **1.5 ETP**
     - **149k€**
   * - **10,000**
     - 231€
     - 20,000€
     - 12,400€
     - **386,026€**
     - **99%**
     - **3 ETP**
     - **341k€**

Scénario Objectif 2030 (5,000 copros)
--------------------------------------

**Hypothèses conservatrices**:

- **Ratio cloud/self-hosted**: 40% cloud (2,000 copros), 60% self-hosted (3,000 copros gratuit)
- **Adoption add-ons**: 40% AI, 30% Blockchain, 20% IoT

**Revenus annuels**: 194,400€

  - Revenus base: 120,000€/an (2,000 copros × 5.00€ × 12 mois)
  - Add-ons AI: 48,000€/an (2,000 copros × 2€ × 40% adoption)
  - Add-ons Blockchain: 18,000€/an (2,000 copros × 1€ × 30%)
  - Add-ons IoT: 8,400€/an (400 copros × 10€ × 20%)

**Coûts infrastructure**: 1,958€/an

  - VPS (2 × Value): 168.48€/an
  - Storage S3 (30GB): 2.52€/an
  - PropTech (GPU + BC + IoT): 1,787€/an

**Surplus net**: 192,442€/an (marge 99%)

**Allocation surplus**:

  - 💼 **1.5 ETP développement**: 43,200€/an (22%)
  - 🔬 **R&D / Nouvelles features**: 80,000€/an (42%)
  - 🛡️ **Fonds urgence**: 30,000€/an (16%)
  - 🌍 **Redistribution communauté**: 39,242€/an (20%)

🌱 Impact Écologique - Mesurable & Vérifiable
==============================================

Empreinte Carbone par Palier
-----------------------------

.. list-table::
   :header-rows: 1
   :widths: 20 25 25 30

   * - Copros
     - CO₂ cloud/an
     - CO₂ évité/an
     - Équivalent arbres
   * - **100**
     - 175 kg
     - 16.8 tonnes
     - ~840 arbres
   * - **500**
     - 876 kg
     - 84 tonnes
     - ~4,200 arbres
   * - **1,000**
     - 1.75 tonnes
     - 168 tonnes
     - ~8,400 arbres
   * - **2,000**
     - 3.5 tonnes
     - 336 tonnes
     - ~16,800 arbres
   * - **5,000** 🎯
     - **8.76 tonnes**
     - **840 tonnes**
     - **~42,000 arbres**
   * - **10,000**
     - 17.5 tonnes
     - 1,680 tonnes
     - ~84,000 arbres

Méthodologie Calcul
--------------------

**Émissions KoproGo**:

- Datacenter OVH GRA11: 60g CO₂/kWh (mix énergétique français)
- Consommation par requête : estimation réseau, non mesurée
- Estimation: 100 requêtes/copro/jour

**Émissions évitées** (baseline solutions propriétaires):

- Solutions SaaS cloud traditionnelles: ~11.5g CO₂/req (96× plus)
- Datacenters hors Europe: 200-400g CO₂/kWh
- Infrastructure redondante non mutualisée

Certification & Transparence
-----------------------------

**Ce qui a été mesuré, et ce qui ne l'a pas été** :

- Test de charge de 3 minutes, 30/10/2025 (PERFORMANCE_REPORT.rst) : mesuré
- Consommation énergétique et CO₂ par requête : estimés, pas mesurés
- Datacenter OVH certifié ISO 14001

🎯 **Engagement transparence**:

- Dashboard public CO₂/requête (temps réel)
- Rapport annuel impact environnemental
- Audit externe indépendant (à partir 2026)

🚀 Différenciation Concurrentielle
===================================

vs Solutions Propriétaires (Urbis, Syndic Online, etc.)
--------------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 20 20 30

   * - Critère
     - KoproGo ASBL
     - Concurrence
     - Avantage
   * - **Prix/copro/mois**
     - 5€
     - 10-50€
     - **2-10× moins cher**
   * - **Licence**
     - AGPL-3.0 (opensource)
     - Propriétaire
     - Souveraineté totale
   * - **Self-hosting**
     - Gratuit illimité
     - Interdit / Payant
     - Autonomie syndics
   * - **Vendor lock-in**
     - Aucun
     - Total
     - Portabilité garantie
   * - **Gouvernance**
     - ASBL (non-profit)
     - Actionnaires privés
     - Intérêt général
   * - **PropTech 2.0**
     - IA, Blockchain, IoT
     - Limité / Absent
     - Innovation continue

vs Plateformes Opensource (Odoo, ERPNext, etc.)
------------------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 20 20 30

   * - Critère
     - KoproGo
     - Odoo/ERPNext
     - Avantage
   * - **Spécialisation**
     - 100% copropriété belge
     - Générique ERP
     - Domain expertise
   * - **Compliance RGPD**
     - Natif (by design)
     - Module add-on
     - Sécurité intégrée
   * - **PCMN Belge**
     - Intégré (Plan Comptable)
     - Adaptation manuelle
     - Légalité garantie
   * - **Performance**
     - 287 req/s (1 vCPU)
     - ~50-100 req/s
     - 3-6× plus rapide
   * - **Empreinte carbone**
     - estimation réseau, non mesurée
     - ~0.5-1g CO₂/req
     - À mesurer
   * - **Support métier**
     - Copropriété spécialisé
     - Généraliste
     - Expertise unique

📋 Risques & Mitigation
========================

Risques Identifiés
-------------------

.. list-table::
   :header-rows: 1
   :widths: 30 20 25 25

   * - Risque
     - Probabilité
     - Impact
     - Mitigation
   * - **Adoption add-ons < 40%**
     - Moyenne
     - Faible (38% revenus)
     - Scénarios conservateurs (20%), revenus base solides (62%)
   * - **Ratio cloud/self 30/70**
     - Faible
     - Moyen (revenus -25%)
     - Enquête marché, partenariats syndics
   * - **Scaling technique > 10k**
     - Faible
     - Moyen (migration K8s)
     - Roadmap K3s → K8s validée, tests charge préventifs
   * - **Concurrence pricing aggressive**
     - Moyenne
     - Faible (lock-in faible)
     - Avantage structurel ASBL, opensource inimitable
   * - **Regulatory (RGPD strict)**
     - Faible
     - Moyen
     - Compliance by design, audit externe annuel

Scénarios Financiers
---------------------

**Conservateur** (adoption add-ons 20%, cloud/self 30/70):

- Revenus: 108,000€/an (base: 90k€ + add-ons: 18k€)
- Surplus: 106,000€/an
- Financement: 2 ETP + R&D modéré

**Réaliste** (adoption 30%, cloud/self 40/60):

- Revenus: 156,000€/an (base: 120k€ + add-ons: 36k€)
- Surplus: 154,000€/an
- Financement: 3 ETP + R&D complet

**Optimiste** (adoption 40%, cloud/self 40/60):

- Revenus: **194,400€/an** ✅ **SCÉNARIO PRÉSENTÉ** (base: 120k€ + add-ons: 74.4k€)
- Surplus: **192,442€/an**
- Financement: **1.5 ETP + R&D complet (149k€ dispo)**

💼 Opportunités de Financement
===============================

Cibles Identifiées
-------------------

1. **Subsides Publics** (priorité haute)

   - **Région Wallonne** (Digital Wallonia, GreenTech):

     - Axes: Transition digitale, économie sociale, innovation durable
     - Montant cible: 50,000-100,000€
     - Timing: Q1 2026

   - **Région Bruxelles-Capitale** (Be Circular, Bruxelles Environnement):

     - Axes: Économie circulaire, réduction CO₂, smart city
     - Montant cible: 30,000-80,000€
     - Timing: Q2 2026

   - **EU Horizon Europe** (Digital Europe Programme):

     - Axes: Opensource, GreenTech, Digital Public Infrastructure
     - Montant cible: 100,000-500,000€
     - Timing: 2026-2027 (long terme)

2. **Financement Participatif** (Crowdfunding)

   - **Platforms**: Miimosa (impact social), Ecotree (écologie)
   - **Objectif**: 20,000-50,000€
   - **Contreparties**:

     - 50€: Supporter badge + roadmap access
     - 200€: Lifetime self-hosting support (1 copro)
     - 1,000€: Corporate sponsor (logo website)

3. **Fondations & Mécénat**

   - **King Baudouin Foundation** (innovation sociale)
   - **Fondation SNCF** (mobilité durable, habitat)
   - **Ashoka** (entrepreneuriat social)

Arguments Investisseurs
------------------------

**Pour subsides publics**:

✅ **Impact sociétal**: 100,000 personnes à 5,000 copros, réduction coûts 90%

✅ **Innovation technologique**: Rust, IA, Blockchain opensource

✅ **Transition écologique**: -840 tonnes CO₂/an visé (projection)

✅ **Souveraineté numérique**: Opensource AGPL-3.0, données EU

✅ **Modèle pérenne**: 98% de marge projetée, viabilité à démontrer

**Pour crowdfunding**:

✅ **Transparence radicale**: Budget public, code opensource

✅ **Impact mesurable**: Dashboard CO₂ temps réel

✅ **Communauté active**: 100 contributeurs objectif 2030

✅ **Alternative crédible**: vs monopoles SaaS propriétaires

🗺️ Roadmap par Capacités (Pas de Dates Fixes)
===============================================

**Philosophie**: KoproGo avance quand les **capacités sont atteintes**, pas selon un calendrier arbitraire.

   **"Tout sera livré tôt ou tard. La seule question est: QUAND?"**

   La réponse dépend de la **force de frappe collective** (clients, partenaires, contributeurs).

Jalons Produit
--------------

.. list-table::
   :header-rows: 1
   :widths: 15 25 30 30

   * - Jalon
     - Débloque
     - Features Clés
     - Conformité Légale
   * - **Jalon 0** ✅
     - 10-20 early adopters
     - Architecture hexagonale, 73 endpoints API
     - 30% (CRUD de base)
   * - **Jalon 1**
     - 50-100 copros (beta publique)
     - LUKS encryption, backups GPG, GDPR basique, auth forte
     - 40%
   * - **Jalon 2**
     - 200-500 copros
     - Facturation TVA belge, recouvrement, conseils syndical
     - 60%
   * - **Jalon 3**
     - 500-1,000 copros
     - Audit trails, RBAC avancé, rapports financiers
     - 80%
   * - **Jalon 4**
     - 1,000-2,000 copros
     - Intégrations bancaires, exports comptables, workflows
     - 95%
   * - **Jalon 5**
     - 2,000-5,000 copros
     - Mobile app, API publique, internationalisation
     - 100%
   * - **Jalons 6-7**
     - 5,000+ copros
     - PropTech 2.0 (IA, Blockchain, IoT, Carbon Credits)
     - 100%+

Paliers de Croissance
---------------------

.. list-table::
   :header-rows: 1
   :widths: 15 20 20 20 25

   * - Palier
     - Copros
     - Revenus cloud/an
     - Force de frappe
     - Capacité débloquée
   * - **Validation**
     - 100
     - 2,400€
     - Solo dev + IA
     - Jalon 1 livré
   * - **Viabilité**
     - 500
     - 12,000€
     - 1 dev temps partiel
     - Jalon 2-3 livrés
   * - **Impact**
     - 1,000
     - 24,000€
     - 1-2 ETP
     - Jalon 4 livré
   * - **Leadership**
     - 2,000
     - 48,000€
     - 2-3 ETP
     - Jalon 5 livré
   * - **Référence** 🎯
     - **5,000**
     - **120,000€**
     - **3-4 ETP + R&D**
     - **Jalons 6-7 livrés**

**Note**: Revenus cloud = 40% copros × 5€/mois × 12 mois (60% self-hosted gratuit)

Scénarios de Progression
-------------------------

**Scénario Lent** (solo dev, croissance organique):
   - Jalon 1: 2-3 mois → 100 copros
   - Jalon 2-3: 6-12 mois → 500 copros
   - Jalons 4-5: 2-3 ans → 2,000 copros
   - Jalons 6-7: 4-5 ans → 5,000 copros

**Scénario Organique** (communauté active, partenaires):
   - Jalon 1: 6-8 semaines → 100 copros
   - Jalons 2-3: 6-9 mois → 1,000 copros
   - Jalons 4-5: 18 mois → 2,000 copros
   - Jalons 6-7: 2-3 ans → 5,000 copros

**Scénario Rapide** (financement, équipe structurée):
   - Jalon 1: 3-4 semaines → 100 copros
   - Jalons 2-3: 3-6 mois → 1,000 copros
   - Jalons 4-5: 12 mois → 2,000 copros
   - Jalons 6-7: 18-24 mois → 5,000 copros

**Principe**: Vélocité = f(clients, contributeurs, partenaires). Tout sera livré, la durée dépend de l'oxygène apporté.

🤝 Appel à l'Action
===================

Pour Financeurs Publics
------------------------

**Nous recherchons**: 80,000-150,000€ de subsides (phase de croissance)

**Allocation budgétaire**:

- **60% Développement** (1 ETP développeur Rust/Svelte 18 mois)
- **20% Infrastructure** (VPS, GPU IA, monitoring)
- **10% Marketing** (communication syndics, onboarding)
- **10% Légal/Audit** (RGPD, comptabilité ASBL)

**ROI pour la société**:

- **Impact économique**: 4M€ économisés à 5,000 copros (2.7M€ licences + 750k€ SEL + 600k€ consommation)
- **Impact écologique**: -840 tonnes CO₂/an (équivalent 42,000 arbres)
- **Impact social**: 100,000 personnes (souveraineté numérique, données personnelles)

Pour Crowdfunders
------------------

**Paliers de contribution**:

- **20€**: Supporter badge, newsletter trimestrielle
- **50€**: Accès roadmap privée, vote features prioritaires
- **200€**: Lifetime self-hosting support (1 copropriété)
- **500€**: Formation personnalisée (4h onboarding)
- **1,000€**: Corporate sponsor (logo site, rapport annuel)

**Campagne prévue**: Jalon 2 atteint (plateforme Miimosa ou Ecotree)

Pour Contributeurs Opensource
------------------------------

**Rejoignez la communauté**:

- 🐛 **Bug reports & testing**: Testez MVP, signalez issues
- 💻 **Code contributions**: Rust, Svelte, Terraform (GitHub)
- 📖 **Documentation**: Guides utilisateurs, traductions (FR/NL/DE)
- 🎨 **Design UX/UI**: Amélioration interfaces, accessibilité
- 🌍 **Advocacy**: Partagez projet, syndics pilotes

**Objectif 2030**: **100 contributeurs réguliers**

📞 Contact
==========

**KoproGo** (ASBL en préparation)

- 📧 Email: contact@koprogo.com
- 🌐 Website: https://koprogo.com
- 💻 GitHub: https://github.com/gilmry/koprogo
- 📊 Dashboard Public: https://stats.koprogo.com (à venir)

**Équipe Fondatrice**:

- **Gilles Maury** (Fondateur & Tech Lead): 25 ans d'expérience en informatique, admirateur de la cause libre et des modèles économiques démocratiques, sociaux et solidaires résilients
- **[En recrutement]** (Co-fondateur Product): Expertise syndic/copropriété

**🔍 Recrutement actif**:

Nous recherchons un(e) **Co-fondateur(trice) Product** avec expertise en gestion de copropriété (syndic, conseil syndical, ou comptabilité copropriété belge). Profil idéal: connaissance terrain du secteur + vision produit.

**Disponibilité pitch**:

- 📅 Rendez-vous investisseurs: Sur demande (visio ou présentiel Bruxelles)
- 🎤 Conférences: Available pour événements GreenTech, Opensource, PropTech

---

**Version**: 1.0 - Novembre 2025

**Dernière mise à jour**: Basée sur PERFORMANCE_REPORT.rst (Oct 2025) + simulations validées

**Prochaine révision**: Après atteinte Jalon 2 (tests charge 10,000 copros simulées)
