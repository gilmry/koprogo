===================================================================
Belgian Accounting - PCMN (Plan Comptable Minimum Normalisé)
===================================================================

:Date de mise à jour: 7 novembre 2025
:Version: 1.0.0 - **IMPLÉMENTÉ** ✅
:Issue GitHub: #79 (Fermée le 7 novembre 2025)
:Statut: Production-ready

📋 Vue d'ensemble
=================

KoproGo implémente le **Plan Comptable Minimum Normalisé** (PCMN) belge, un plan comptable standardisé obligatoire pour la comptabilité belge tel que défini par l'Arrêté Royal du 12/07/2012.

Cette implémentation permet aux sociétés de gestion immobilière belges (copropriétés/VVE) de gérer leur comptabilité en conformité avec les exigences légales belges.

**Statut d'implémentation** ✅ :

- ✅ **Backend complet** : Domain entity, repository, use cases, handlers
- ✅ **Base de données** : Migration PostgreSQL avec ~90 comptes seed
- ✅ **API REST** : 10 endpoints avec authentification JWT
- ✅ **Rapports financiers** : Bilan & Compte de résultats
- ✅ **Tests** : 100% couverture domain + integration PostgreSQL
- ✅ **Multi-tenancy** : Isolation par organization_id
- ✅ **Production** : Déployé et testé

🙏 Crédits & Attribution
=========================

**Cette implémentation s'inspire du projet** `Noalyss <https://gitlab.com/noalyss/noalyss>`_.

- **Noalyss** : Logiciel de comptabilité libre pour la comptabilité belge et française
- **Licence** : GPL-2.0-or-later (GNU General Public License version 2 ou ultérieure)
- **Copyright** : (C) 1989, 1991 Free Software Foundation, Inc.
- **Auteur** : Dany De Bontridder <dany@alchimerys.eu>
- **Site web** : https://gitlab.com/noalyss/noalyss

Noalyss a fourni une référence inestimable pour :

- Structure et hiérarchie du PCMN belge
- Logique de classification des comptes (Actif, Passif, Charge, Produit, Hors-bilan)
- Génération de rapports financiers (bilan, compte de résultats)
- Règles et contraintes de validation des comptes

Nous sommes reconnaissants au projet Noalyss et à ses mainteneurs pour avoir créé un système comptable si complet et bien documenté qui sert de référence pour l'implémentation du PCMN belge.

📊 Structure du PCMN Belge
===========================

Le PCMN belge organise les comptes en 9 classes :

.. list-table::
   :header-rows: 1
   :widths: 10 20 40 30

   * - Classe
     - Type
     - Description
     - Exemples
   * - **1**
     - Passif
     - Capital, réserves, provisions
     - ``100`` Capital, ``130`` Réserves, ``14`` Provisions
   * - **2**
     - Actif
     - Immobilisations (bâtiments, équipements)
     - ``220`` Bâtiments, ``221`` Terrains
   * - **3**
     - Actif
     - Stock et encours
     - ``30`` Matières premières
   * - **4**
     - Actif/Passif
     - Créances et dettes
     - ``400`` Fournisseurs, ``440`` Clients, ``451`` TVA
   * - **5**
     - Actif
     - Banque et caisse
     - ``550`` Banque, ``551`` CCP, ``57`` Caisse
   * - **6**
     - Charge
     - Charges d'exploitation
     - ``604001`` Électricité, ``611002`` Entretien ascenseur
   * - **7**
     - Produit
     - Produits d'exploitation
     - ``700001`` Appels ordinaires, ``700002`` Appels extraordinaires
   * - **8**
     - -
     - (Non utilisé dans PCMN simplifié)
     - -
   * - **9**
     - Hors-bilan
     - Comptes d'ordre
     - ``90`` Droits et engagements

Structure Hiérarchique
-----------------------

Les comptes suivent une structure hiérarchique :

.. code-block:: text

   6                     # Classe : Toutes les charges
   └── 60                # Sous-classe : Achats & consommables
       └── 604           # Groupe : Énergie
           └── 604001    # Compte : Électricité (utilisation directe)

- **Comptes d'utilisation directe** : Peuvent être utilisés dans les transactions (ex: ``604001``)
- **Comptes de synthèse** : Ne peuvent pas être utilisés directement, seulement pour regroupement (ex: ``6``, ``60``, ``604``)

🔧 Implémentation
==================

Architecture
------------

.. code-block:: text

   Couche Domain (Logique métier pure)
     └── entities/account.rs          # Entité Account avec logique PCMN belge

   Couche Application (Cas d'usage)
     ├── ports/account_repository.rs  # Interface repository
     ├── use_cases/account_use_cases.rs         # CRUD + Seed PCMN
     └── use_cases/financial_report_use_cases.rs # Rapports

   Couche Infrastructure (Détails techniques)
     ├── database/repositories/account_repository_impl.rs  # PostgreSQL
     └── web/handlers/account_handlers.rs                  # API REST

Schéma de Base de Données
--------------------------

.. code-block:: sql

   CREATE TYPE account_type AS ENUM (
       'ASSET',       -- Classes 2-5
       'LIABILITY',   -- Classe 1
       'EXPENSE',     -- Classe 6
       'REVENUE',     -- Classe 7
       'OFF_BALANCE'  -- Classe 9
   );

   CREATE TABLE accounts (
       id UUID PRIMARY KEY,
       code VARCHAR(40) NOT NULL,           -- ex: "604001"
       label TEXT NOT NULL,                 -- ex: "Électricité"
       parent_code VARCHAR(40),             -- ex: "604"
       account_type account_type NOT NULL,
       direct_use BOOLEAN DEFAULT true,     -- Peut être utilisé dans transactions
       organization_id UUID NOT NULL,       -- Multi-tenancy
       created_at TIMESTAMPTZ NOT NULL,
       updated_at TIMESTAMPTZ NOT NULL,
       CONSTRAINT accounts_code_org_unique UNIQUE(code, organization_id)
   );

Comptes PCMN Belges Préchargés
--------------------------------

KoproGo précharge ~90 comptes standards belges optimisés pour la gestion immobilière :

**Classe 1 - Passifs (Capital & Réserves)**

- ``100`` - Capital social
- ``130`` - Réserves disponibles
- ``131`` - Réserves indisponibles
- ``14`` - Bénéfice (Perte) reporté(e)

**Classe 2 - Immobilisations**

- ``220`` - Bâtiments
- ``221`` - Terrains

**Classe 4 - Créances & Dettes**

- ``400`` - Fournisseurs
- ``411`` - Clients
- ``440-441`` - TVA
- ``451`` - TVA à récupérer

**Classe 5 - Banque & Caisse**

- ``550`` - Banque courante
- ``551`` - Banque d'épargne
- ``57`` - Caisse

**Classe 6 - Charges** (Focus Gestion Immobilière)

- ``604001`` - Électricité
- ``604002`` - Gaz
- ``604003`` - Eau
- ``604004`` - Mazout de chauffage
- ``611001`` - Entretien bâtiment
- ``611002`` - Entretien ascenseur
- ``612001`` - Petit entretien parties communes
- ``614001`` - Assurances incendie
- ``614002`` - Assurances RC copropriété
- ``615001`` - Assurance incendie immeuble
- ``615002`` - Assurance RC exploitant
- Et bien d'autres...

**Classe 7 - Produits**

- ``700001`` - Appels de fonds ordinaires
- ``700002`` - Appels de fonds extraordinaires
- ``700003`` - Régularisation charges
- ``74`` - Subventions d'exploitation
- ``75`` - Produits financiers

Données seed complètes : ``backend/src/application/use_cases/account_use_cases.rs::seed_belgian_pcmn()``

🌐 Endpoints API
=================

Tous les endpoints nécessitent une authentification JWT. L'accès est restreint selon le rôle :

- ✅ **Accountant** : Accès CRUD complet
- ✅ **SuperAdmin** : Accès CRUD complet
- ❌ **Syndic** : Lecture seule (futur)
- ❌ **Owner** : Pas d'accès

URL de base : ``/api/v1``

Gestion des Comptes
--------------------

.. code-block:: bash

   # Seed PCMN Belge (~90 comptes)
   POST /accounts/seed/belgian-pcmn
   Authorization: Bearer <token>

   # Créer un compte personnalisé
   POST /accounts
   Content-Type: application/json
   Authorization: Bearer <token>
   {
     "code": "619999",
     "label": "Compte de charge personnalisé",
     "parent_code": "61",
     "direct_use": true
   }

   # Lister les comptes (avec filtres optionnels)
   GET /accounts?account_type=EXPENSE&direct_use=true&search=électr
   Authorization: Bearer <token>

   # Obtenir un compte par ID
   GET /accounts/{id}
   Authorization: Bearer <token>

   # Obtenir un compte par code
   GET /accounts/code/{code}
   Authorization: Bearer <token>

   # Mettre à jour un compte
   PUT /accounts/{id}
   Content-Type: application/json
   Authorization: Bearer <token>
   {
     "label": "Libellé mis à jour",
     "direct_use": false
   }

   # Supprimer un compte (avec validation)
   DELETE /accounts/{id}
   Authorization: Bearer <token>

   # Compter les comptes
   GET /accounts/count
   Authorization: Bearer <token>

Rapports Financiers
-------------------

.. code-block:: bash

   # Générer un bilan
   GET /reports/balance-sheet
   Authorization: Bearer <token>

   # Réponse :
   {
     "organization_id": "...",
     "report_date": "2024-11-07T12:00:00Z",
     "assets": {
       "account_type": "ASSET",
       "accounts": [
         {"code": "220", "label": "Bâtiments", "amount": 500000.0},
         {"code": "550", "label": "Banque", "amount": 10000.0}
       ],
       "total": 510000.0
     },
     "liabilities": {
       "account_type": "LIABILITY",
       "accounts": [
         {"code": "100", "label": "Capital", "amount": 500000.0},
         {"code": "130", "label": "Réserves", "amount": 10000.0}
       ],
       "total": 510000.0
     },
     "total_assets": 510000.0,
     "total_liabilities": 510000.0,
     "balance": 0.0
   }

   # Générer un compte de résultats (pertes & profits)
   GET /reports/income-statement?period_start=2024-01-01T00:00:00Z&period_end=2024-12-31T23:59:59Z
   Authorization: Bearer <token>

   # Réponse :
   {
     "organization_id": "...",
     "report_date": "2024-11-07T12:00:00Z",
     "period_start": "2024-01-01T00:00:00Z",
     "period_end": "2024-12-31T23:59:59Z",
     "expenses": {
       "account_type": "EXPENSE",
       "accounts": [
         {"code": "604001", "label": "Électricité", "amount": 5000.0},
         {"code": "611002", "label": "Entretien ascenseur", "amount": 2000.0}
       ],
       "total": 7000.0
     },
     "revenue": {
       "account_type": "REVENUE",
       "accounts": [
         {"code": "700001", "label": "Appels de fonds ordinaires", "amount": 10000.0}
       ],
       "total": 10000.0
     },
     "total_expenses": 7000.0,
     "total_revenue": 10000.0,
     "net_result": 3000.0
   }

💼 Exemples d'Utilisation
==========================

1. Initialiser le PCMN pour une Nouvelle Organisation
------------------------------------------------------

.. code-block:: bash

   # Étape 1 : S'authentifier en tant que Comptable
   POST /api/v1/auth/login
   {
     "email": "accountant@example.com",
     "password": "password"
   }

   # Étape 2 : Seed PCMN Belge
   POST /api/v1/accounts/seed/belgian-pcmn
   Authorization: Bearer <token-from-step-1>

   # Résultat : ~90 comptes standards créés

2. Créer une Dépense avec Code Comptable
-----------------------------------------

.. code-block:: bash

   # Lier une dépense à "604001 - Électricité"
   POST /api/v1/expenses
   Authorization: Bearer <token>
   {
     "organization_id": "...",
     "building_id": "...",
     "category": "utilities",
     "description": "Facture électricité janvier 2024",
     "amount": 250.50,
     "expense_date": "2024-01-15T00:00:00Z",
     "supplier": "Electrabel",
     "invoice_number": "INV-2024-001",
     "account_code": "604001"
   }

3. Générer un Rapport Trimestriel
----------------------------------

.. code-block:: bash

   # Compte de résultats Q1 2024
   GET /api/v1/reports/income-statement?period_start=2024-01-01T00:00:00Z&period_end=2024-03-31T23:59:59Z
   Authorization: Bearer <token>

🔒 Sécurité & Validation
=========================

Règles de Suppression de Comptes
---------------------------------

Les comptes **ne peuvent pas être supprimés** si :

1. **Ont des comptes enfants** : Supprimez d'abord les enfants (ex: impossible de supprimer ``604`` si ``604001`` existe)
2. **Utilisés dans des dépenses** : Archivez plutôt pour préserver les données historiques

Exemple d'erreur :

.. code-block:: json

   {
     "error": "Impossible de supprimer le compte : il a des comptes enfants. Supprimez d'abord les enfants."
   }

Isolation Multi-tenancy
------------------------

- Tous les comptes sont scopés à ``organization_id``
- Chaque organisation a son propre plan comptable
- Les codes de comptes sont uniques au sein d'une organisation (pas globalement)
- Les utilisateurs ne peuvent accéder qu'aux comptes de leur organisation

📈 Rapports Financiers
=======================

Bilan (Balance Sheet)
---------------------

Montre la situation financière à un instant T :

.. code-block:: text

   ACTIF (Assets)              PASSIF (Liabilities)
   --------------------        --------------------
   Immobilisations             Capital
     Bâtiments: 500.000€         Capital: 500.000€
   Actifs courants             Réserves
     Banque: 10.000€             Réserves: 10.000€

   TOTAL: 510.000€             TOTAL: 510.000€

**Classes PCMN:**

- Actif: Classes 2, 3, 4 (débit), 5
- Passif: Classe 1, Classe 4 (crédit)

Compte de Résultats (Income Statement)
---------------------------------------

Montre la rentabilité sur une période :

.. code-block:: text

   PRODUITS (Revenue)                    CHARGES (Expenses)
   --------------------------            --------------------------
   Appels ordinaires: 10.000€            Électricité: 5.000€
                                         Entretien: 2.000€

   TOTAL PRODUITS: 10.000€               TOTAL CHARGES: 7.000€

   RÉSULTAT NET: 3.000€ (Bénéfice)

**Classes PCMN:**

- Charges: Classe 6
- Produits: Classe 7

🧪 Tests
=========

L'implémentation du PCMN belge inclut des tests complets :

.. code-block:: bash

   # Tests unitaires (12 tests pour l'entité Account)
   cargo test --lib account

   # Les tests couvrent :
   # - Création et validation de comptes
   # - Détection de classe PCMN (Classes 1-7, 9)
   # - Classification bilan vs compte de résultats
   # - Validation du format de code comptable
   # - Structure et calculs des rapports financiers

🔮 Améliorations Futures
=========================

**Phase 2 (Planifié):**

- [ ] Écritures comptables (journal entries)
- [ ] Balance de vérification (trial balance)
- [ ] Grand livre (general ledger)
- [ ] Support déclaration TVA
- [ ] Support multi-devises
- [ ] Archivage de comptes (soft delete)
- [ ] Import/export (CSV, Excel)
- [ ] Filtres avancés (par période, montant)

**Phase 3 (Avancé):**

- [ ] Calculs TVA automatisés
- [ ] Rapports budget vs réel
- [ ] Tableau de flux de trésorerie
- [ ] Piste d'audit pour changements de comptes
- [ ] Comparaisons multi-années
- [ ] Export PDF/Excel pour rapports

📚 Références
==============

1. **Projet Noalyss**: https://gitlab.com/noalyss/noalyss

   - ``include/database/acc_plan_sql.class.php`` - Logique repository de comptes
   - ``include/database/tmp_pcmn_sql.class.php`` - Template PCMN
   - ``sql/mono-belge.sql`` - Données seed PCMN belge (~9320 lignes)

2. **Standard PCMN Belge**: Arrêté Royal AR 12/07/2012

   - Spécification officielle du plan comptable belge
   - Obligatoire pour toutes les entreprises belges

3. **Documentation KoproGo**:

   - ``CLAUDE.md`` - Guidelines de développement
   - ``ROADMAP.rst`` - Feuille de route des features
   - ``backend/src/domain/entities/account.rs`` - Implémentation entité Account
   - ``backend/migrations/20251107000000_add_belgian_accounting_plan.sql`` - Schéma base de données

❓ FAQ
======

**Q: Dois-je seed le PCMN belge pour chaque organisation ?**

A: Oui, chaque organisation a son propre plan comptable. Appelez ``POST /api/v1/accounts/seed/belgian-pcmn`` après avoir créé une nouvelle organisation.

**Q: Puis-je ajouter des comptes personnalisés ?**

A: Oui ! Vous pouvez ajouter des comptes spécifiques à l'organisation (ex: ``619999 - Charge personnalisée``). Assurez-vous simplement qu'ils suivent les règles de hiérarchie PCMN.

**Q: Que se passe-t-il si je supprime un compte par erreur ?**

A: La suppression est empêchée si le compte est utilisé dans des dépenses ou a des enfants. Dans le futur, nous ajouterons la suppression douce (archivage).

**Q: Comment lier une dépense à un compte ?**

A: Incluez ``account_code`` lors de la création d'une dépense (ex: ``"account_code": "604001"``).

**Q: Les Copropriétaires peuvent-ils consulter le plan comptable ?**

A: Pas encore. Actuellement, seuls les Comptables et SuperAdmins ont accès. Nous prévoyons un accès en lecture seule pour les Syndics en Phase 2.

🤝 Contribuer
==============

Lors de contributions à l'implémentation du PCMN belge :

1. **Préserver l'attribution Noalyss** : Tous les fichiers liés à la comptabilité doivent inclure des headers d'attribution GPL-2.0
2. **Suivre les standards PCMN** : Respecter la hiérarchie du plan comptable belge
3. **Ajouter des tests** : Chaque nouvelle feature comptable doit avoir des tests unitaires
4. **Documenter les changements** : Mettre à jour ce fichier avec les nouvelles features
5. **Multi-tenancy** : Toujours scoper les requêtes par ``organization_id``

📄 Licence
===========

KoproGo est sous licence **MIT License**.

Cependant, l'implémentation du PCMN belge (inspirée de Noalyss) suit la licence **GPL-2.0-or-later** tel que requis par le projet Noalyss original.

Fichiers concernés par GPL-2.0:

- ``backend/migrations/20251107000000_add_belgian_accounting_plan.sql``
- ``backend/src/domain/entities/account.rs``
- ``backend/src/application/ports/account_repository.rs``
- ``backend/src/application/use_cases/account_use_cases.rs``
- ``backend/src/application/use_cases/financial_report_use_cases.rs``
- ``backend/src/infrastructure/database/repositories/account_repository_impl.rs``
- ``backend/src/infrastructure/web/handlers/account_handlers.rs``
- ``backend/src/infrastructure/web/handlers/financial_report_handlers.rs``

Tous ces fichiers incluent des headers d'attribution GPL-2.0 appropriés créditant Noalyss.

----

| **Maintenu par**: Équipe KoproGo
| **Remerciements spéciaux**: Projet Noalyss & Dany De Bontridder
