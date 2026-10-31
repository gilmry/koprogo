================================================================
Workflow Complet d'Encodage de Factures avec Approbation
================================================================

:Date de mise à jour: 7 novembre 2025
:Version: 1.0.0 - **IMPLÉMENTÉ** ✅
:Issue GitHub: #73 (Fermée le 7 novembre 2025)
:Statut: Implémenté, jamais exploité en production (v0.1.0 pre-release)

📋 Vue d'ensemble
=================

KoproGo implémente un **workflow complet de validation de factures** conforme aux bonnes pratiques de gestion immobilière belge, avec séparation des rôles et contrôles multi-niveaux.

Ce système permet aux copropriétés de gérer le cycle de vie complet des factures depuis l'encodage jusqu'au paiement, avec approbation obligatoire et traçabilité complète.

**Statut d'implémentation** ✅ :

- ✅ **États du workflow** : Draft → PendingApproval → Approved/Rejected
- ✅ **Gestion TVA** : 6%, 12%, 21% avec calculs automatiques
- ✅ **Lignes de facturation** : Support multi-lignes avec quantités
- ✅ **Validation métier** : Empêche modification après approbation
- ✅ **Backend complet** : InvoiceLineItem, ApprovalStatus enum, workflow
- ✅ **Tests** : Scénarios BDD + E2E avec workflow complet
- ✅ **API REST** : Endpoints CRUD + approbation/rejet
- ✅ **Production** : Déployé et testé

🔄 Workflow de Validation
==========================

Diagramme d'État
----------------

.. code-block:: text

   ┌─────────┐
   │  DRAFT  │  ← État initial (créé par Syndic/Comptable)
   └────┬────┘
        │ submit_for_approval()
        ↓
   ┌──────────────────┐
   │ PENDING_APPROVAL │  ← En attente d'approbation
   └────┬─────────────┘
        │
        ├──→ approve() ──→ ┌──────────┐
        │                  │ APPROVED │  ← Approuvé (prêt paiement)
        │                  └──────────┘
        │
        └──→ reject() ───→ ┌──────────┐
                           │ REJECTED │  ← Rejeté (motif obligatoire)
                           └────┬─────┘
                                │ resubmit()
                                ↓
                           ┌─────────┐
                           │  DRAFT  │  ← Retour au brouillon
                           └─────────┘

États et Transitions
--------------------

.. list-table::
   :header-rows: 1
   :widths: 20 30 30 20

   * - État
     - Description
     - Transitions autorisées
     - Modifiable ?
   * - **Draft**
     - Brouillon en cours d'encodage
     - → PendingApproval (submit)
     - ✅ Oui
   * - **PendingApproval**
     - En attente d'approbation
     - → Approved (approve)
       → Rejected (reject)
     - ❌ Non
   * - **Approved**
     - Approuvé, prêt pour paiement
     - (terminal)
     - ❌ Non
   * - **Rejected**
     - Rejeté (avec motif)
     - → Draft (resubmit)
     - ❌ Non

Règles Métier
-------------

1. **Modification**

   - ✅ Autorisée uniquement en état **Draft**
   - ❌ Interdite après soumission (PendingApproval, Approved, Rejected)
   - Erreur : ``"Cannot modify invoice: invoice is not in Draft state"``

2. **Soumission pour Approbation**

   - ✅ Uniquement depuis l'état **Draft**
   - ❌ Impossible de soumettre deux fois (``Already in PendingApproval state``)

3. **Approbation**

   - ✅ Uniquement depuis l'état **PendingApproval**
   - ❌ Impossible d'approuver un brouillon directement

4. **Rejet**

   - ✅ Uniquement depuis l'état **PendingApproval**
   - **Motif obligatoire** (ex: "Montant incorrect", "Fournisseur non autorisé")
   - Erreur si motif vide : ``"Rejection reason is required"``

5. **Resoumission**

   - ✅ Uniquement depuis l'état **Rejected**
   - Retour automatique en état **Draft** pour correction

🧾 Gestion de la TVA
=====================

Taux TVA Belges Standards
--------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 45 40

   * - Taux
     - Application
     - Exemples
   * - **0%**
     - Pas de TVA
     - Certains services non soumis
   * - **6%**
     - Taux réduit
     - Énergie (électricité, gaz), travaux rénovation énergétique
   * - **12%**
     - Taux intermédiaire
     - Certains travaux de construction
   * - **21%**
     - Taux normal
     - Services généraux, maintenance, assurances

Calcul Automatique
------------------

Le système calcule automatiquement :

.. code-block:: rust

   // Formules
   TVA Amount = Base Amount HT × (TVA Rate / 100)
   Total TTC = Base Amount HT + TVA Amount

   // Exemple : Facture 1000€ HT à 21% TVA
   Base Amount HT: 1000.00€
   TVA (21%):      210.00€   (1000 × 0.21)
   Total TTC:      1210.00€  (1000 + 210)

Recalcul TVA
------------

.. code-block:: bash

   # Recalculer la TVA après modification montant
   let invoice = expense.recalculate_vat()?;

   # Le système met à jour automatiquement :
   # - vat_amount
   # - amount (TTC)

💰 Lignes de Facturation (Multi-lignes)
========================================

Structure
---------

Une facture peut contenir plusieurs lignes avec :

- **Description** : Libellé de la ligne (obligatoire, trimé)
- **Quantité** : Nombre d'unités (> 0)
- **Prix Unitaire HT** : Prix hors TVA par unité (≥ 0)
- **Taux TVA** : 0%, 6%, 12%, ou 21%
- **Montants calculés** : Sous-total HT, TVA, Total TTC

Exemple
-------

.. code-block:: json

   [
     {
       "description": "Entretien ascenseur mensuel",
       "quantity": 1,
       "unit_price": 150.00,
       "vat_rate": 21.0
     },
     {
       "description": "Électricité parties communes (kWh)",
       "quantity": 450,
       "unit_price": 0.28,
       "vat_rate": 6.0
     },
     {
       "description": "Assurance RC copropriété",
       "quantity": 1,
       "unit_price": 800.00,
       "vat_rate": 21.0
     }
   ]

Calculs Multi-lignes
--------------------

.. code-block:: rust

   // Pour chaque ligne :
   Subtotal HT = Quantity × Unit Price
   TVA Line = Subtotal HT × (VAT Rate / 100)
   Total TTC Line = Subtotal HT + TVA Line

   // Totaux facture :
   Total Invoice HT = Σ (Subtotal HT)
   Total Invoice TVA = Σ (TVA Line)
   Total Invoice TTC = Total Invoice HT + Total Invoice TVA

Exemple Complet
---------------

.. code-block:: text

   FACTURE INV-2024-001
   Fournisseur: Maintenance SA
   Date: 15/01/2024

   Ligne 1: Entretien ascenseur mensuel
     Quantité: 1 × 150.00€ = 150.00€ HT
     TVA 21%:                  31.50€
     Total ligne:             181.50€ TTC

   Ligne 2: Électricité PC (450 kWh)
     Quantité: 450 × 0.28€ = 126.00€ HT
     TVA 6%:                    7.56€
     Total ligne:             133.56€ TTC

   Ligne 3: Assurance RC
     Quantité: 1 × 800.00€ = 800.00€ HT
     TVA 21%:                 168.00€
     Total ligne:             968.00€ TTC

   ─────────────────────────────────────
   TOTAL HT:                1,076.00€
   TOTAL TVA:                 207.06€
   TOTAL TTC:               1,283.06€

🌐 API Endpoints
================

Base URL : ``/api/v1/expenses``

Créer une Facture (Draft)
--------------------------

.. code-block:: bash

   POST /api/v1/expenses
   Authorization: Bearer <token>
   Content-Type: application/json

   {
     "organization_id": "uuid",
     "building_id": "uuid",
     "category": "maintenance",
     "description": "Facture entretien mensuel",
     "amount": 1000.00,
     "expense_date": "2024-01-15T00:00:00Z",
     "supplier": "Maintenance SA",
     "invoice_number": "INV-2024-001",
     "account_code": "611001",
     "vat_rate": 21.0,
     "approval_status": "Draft"
   }

   # Réponse: 201 Created
   {
     "id": "uuid",
     "approval_status": "Draft",
     "amount": 1210.00,
     "vat_amount": 210.00,
     "can_be_modified": true,
     ...
   }

Soumettre pour Approbation
---------------------------

.. code-block:: bash

   POST /api/v1/expenses/{id}/submit-for-approval
   Authorization: Bearer <token>

   # Réponse: 200 OK
   {
     "id": "uuid",
     "approval_status": "PendingApproval",
     "can_be_modified": false,
     ...
   }

Approuver une Facture
---------------------

.. code-block:: bash

   POST /api/v1/expenses/{id}/approve
   Authorization: Bearer <token>

   # Réponse: 200 OK
   {
     "id": "uuid",
     "approval_status": "Approved",
     "can_be_modified": false,
     ...
   }

Rejeter une Facture
-------------------

.. code-block:: bash

   POST /api/v1/expenses/{id}/reject
   Authorization: Bearer <token>
   Content-Type: application/json

   {
     "rejection_reason": "Montant incorrect - vérifier facture originale"
   }

   # Réponse: 200 OK
   {
     "id": "uuid",
     "approval_status": "Rejected",
     "rejection_reason": "Montant incorrect...",
     "can_be_modified": false,
     ...
   }

Resoumett re une Facture Rejetée
----------------------------------

.. code-block:: bash

   POST /api/v1/expenses/{id}/resubmit
   Authorization: Bearer <token>

   # Réponse: 200 OK
   {
     "id": "uuid",
     "approval_status": "Draft",
     "rejection_reason": null,
     "can_be_modified": true,
     ...
   }

🔒 Permissions & Sécurité
==========================

Matrice de Permissions
----------------------

.. list-table::
   :header-rows: 1
   :widths: 20 20 20 20 20

   * - Rôle
     - Créer
     - Soumettre
     - Approuver
     - Rejeter
   * - **SuperAdmin**
     - ✅
     - ✅
     - ✅
     - ✅
   * - **Syndic**
     - ✅
     - ✅
     - ❌
     - ❌
   * - **Accountant**
     - ✅
     - ✅
     - ✅
     - ✅
   * - **Owner**
     - ❌
     - ❌
     - ❌
     - ❌

Séparation des Rôles (Recommandé)
----------------------------------

**Best Practice** : Séparer encodage et approbation

- **Syndic** encode les factures (créer, soumettre)
- **Comptable** approuve/rejette les factures
- **SuperAdmin** peut tout faire (contrôle qualité)

Cela assure un **contrôle à 4 yeux** et évite les conflits d'intérêt.

🧪 Tests
========

Le workflow de factures est couvert par des tests complets :

Tests Unitaires (Domain)
-------------------------

.. code-block:: bash

   cargo test --lib expense

   # Tests incluent :
   # - États et transitions du workflow
   # - Validation modification (can_be_modified)
   # - Calculs TVA (6%, 12%, 21%)
   # - Rejet avec/sans motif
   # - Cycle complet Draft → Approved
   # - Cycle complet Draft → Rejected → Draft

Tests BDD (Gherkin)
-------------------

.. code-block:: gherkin

   Feature: Workflow de Validation de Factures

     Scenario: Soumission et approbation d'une facture
       Given une facture en état "Draft"
       When je soumets la facture pour approbation
       Then l'état devient "PendingApproval"
       When un comptable approuve la facture
       Then l'état devient "Approved"
       And la facture ne peut plus être modifiée

     Scenario: Rejet et resoumission d'une facture
       Given une facture en état "PendingApproval"
       When un comptable rejette avec motif "Montant incorrect"
       Then l'état devient "Rejected"
       When l'encodeur resoumette la facture
       Then l'état redevient "Draft"
       And la facture peut être modifiée

Tests E2E (API)
---------------

.. code-block:: bash

   cargo test --test e2e invoice_workflow

   # Tests incluent :
   # - POST /expenses (création Draft)
   # - POST /expenses/{id}/submit-for-approval
   # - POST /expenses/{id}/approve
   # - POST /expenses/{id}/reject (avec motif)
   # - POST /expenses/{id}/resubmit
   # - PUT /expenses/{id} (autorisé en Draft, interdit après)

💼 Cas d'Usage Complets
========================

Cas 1 : Workflow Standard (Approbation)
----------------------------------------

.. code-block:: bash

   # Étape 1 : Syndic encode une facture (état Draft)
   POST /api/v1/expenses
   {
     "description": "Facture entretien ascenseur",
     "amount": 1000.00,
     "vat_rate": 21.0,
     "approval_status": "Draft"
   }
   # → État : Draft, can_be_modified: true

   # Étape 2 : Syndic soumet pour approbation
   POST /api/v1/expenses/{id}/submit-for-approval
   # → État : PendingApproval, can_be_modified: false

   # Étape 3 : Comptable approuve
   POST /api/v1/expenses/{id}/approve
   # → État : Approved, can_be_modified: false

   # Étape 4 : Paiement (marquer comme payé)
   PUT /api/v1/expenses/{id}/mark-paid
   # → paid: true, paid_date: "2024-01-20T10:30:00Z"

Cas 2 : Workflow avec Rejet
----------------------------

.. code-block:: bash

   # Étape 1 : Syndic encode une facture avec erreur
   POST /api/v1/expenses
   {
     "amount": 10000.00,  # Erreur : devrait être 1000.00
     "vat_rate": 21.0
   }
   # → État : Draft

   # Étape 2 : Soumission pour approbation
   POST /api/v1/expenses/{id}/submit-for-approval
   # → État : PendingApproval

   # Étape 3 : Comptable détecte l'erreur et rejette
   POST /api/v1/expenses/{id}/reject
   {
     "rejection_reason": "Montant incorrect : 10000€ au lieu de 1000€"
   }
   # → État : Rejected
   # → rejection_reason: "Montant incorrect..."

   # Étape 4 : Syndic consulte le motif de rejet
   GET /api/v1/expenses/{id}
   # → Voir le rejection_reason

   # Étape 5 : Syndic resoumette pour correction
   POST /api/v1/expenses/{id}/resubmit
   # → État : Draft, rejection_reason: null

   # Étape 6 : Syndic corrige le montant
   PUT /api/v1/expenses/{id}
   {
     "amount": 1000.00
   }
   # → amount: 1210.00 (avec TVA)

   # Étape 7 : Nouvelle soumission
   POST /api/v1/expenses/{id}/submit-for-approval
   # → État : PendingApproval

   # Étape 8 : Approbation finale
   POST /api/v1/expenses/{id}/approve
   # → État : Approved

📊 Intégration PCMN
===================

Lien avec Plan Comptable
-------------------------

Chaque facture peut être liée à un compte PCMN :

.. code-block:: json

   {
     "account_code": "611002",  // Entretien ascenseur
     "amount": 1210.00,
     "vat_rate": 21.0
   }

Les comptes PCMN standards pour factures :

- ``604001`` - Électricité (TVA 6%)
- ``604002`` - Gaz (TVA 6%)
- ``604003`` - Eau
- ``611001`` - Entretien bâtiment (TVA 21%)
- ``611002`` - Entretien ascenseur (TVA 21%)
- ``614001`` - Assurances incendie (TVA 21%)
- ``615002`` - Assurance RC (TVA 21%)

Voir :doc:`BELGIAN_ACCOUNTING_PCMN` pour le plan comptable complet.

🔮 Évolutions Futures
======================

**Phase 2 (Planifié) :**

- [ ] Workflow d'approbation multi-niveaux (2+ approbateurs)
- [ ] Pièces jointes (PDF factures scannées)
- [ ] Notifications email (facture en attente, approuvée, rejetée)
- [ ] Historique des changements d'état (audit trail)
- [ ] Filtres avancés (par état, par fournisseur, par période)
- [ ] Export Excel des factures approuvées
- [ ] Génération PDF de synthèse mensuelle

**Phase 3 (Avancé) :**

- [ ] OCR automatique (scan facture → pré-remplissage)
- [ ] Validation automatique (règles métier, seuils)
- [ ] Intégration paiement bancaire (SEPA, Domiciliation)
- [ ] Tableau de bord temps réel (factures en attente, montants)
- [ ] Rappels automatiques (délais de paiement)

📚 Références
=============

**Code Source :**

- ``backend/src/domain/entities/expense.rs`` - Entité Expense avec workflow
- ``backend/src/domain/entities/invoice_line_item.rs`` - Lignes de facturation
- ``backend/src/application/use_cases/expense_use_cases.rs`` - Cas d'usage
- ``backend/src/infrastructure/web/handlers/expense_handlers.rs`` - API REST
- ``backend/migrations/20240228000000_create_expenses.sql`` - Schéma BDD

**Tests :**

- ``backend/src/domain/entities/expense.rs`` - Tests unitaires (20+ tests)
- ``backend/tests/features/expenses.feature`` - Tests BDD Gherkin
- ``backend/tests/e2e.rs`` - Tests E2E workflow complet

**Documentation :**

- :doc:`BELGIAN_ACCOUNTING_PCMN` - Plan comptable belge
- :doc:`PAYMENT_RECOVERY_WORKFLOW` - Workflow recouvrement
- :doc:`ROADMAP` - Feuille de route du projet

----

| **Version** : 1.0.0 (Novembre 2024)
| **Dernière mise à jour** : 7 novembre 2025
| **Maintenu par** : Équipe KoproGo
