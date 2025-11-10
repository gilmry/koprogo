==========================================
Guide Utilisateur : Comptable
==========================================

:Version: 1.0.0
:Date: 10 novembre 2025
:Public: Comptables et experts-comptables
:Voir aussi: :doc:`syndic-guide` | :doc:`owner-guide` | :doc:`board-member-guide`

📋 Vue d'ensemble
=================

Ce guide explique comment utiliser **KoproGo** en tant que **comptable** d'une ou plusieurs copropriétés. Vous y trouverez :

- ✅ Plan Comptable Minimum Normalisé (PCMN) belge conforme AR 12/07/2012
- ✅ Gestion comptable multi-organisations
- ✅ Écritures comptables et imputations
- ✅ Génération de rapports financiers (bilan, compte de résultats)
- ✅ TVA belge (6%, 12%, 21%)
- ✅ Exports comptables pour logiciels tiers
- ✅ Clôture d'exercice et reports à nouveau

🚀 Premiers pas
================

Connexion et rôle multi-organisations
--------------------------------------

1. **Connexion** : ``POST /api/v1/auth/login``

   .. code-block:: json

      {
        "email": "comptable@cabinet-example.com",
        "password": "votre_mot_de_passe"
      }

2. **Vérification des rôles** : ``GET /api/v1/auth/me``

   .. code-block:: json

      {
        "id": "user-uuid",
        "email": "comptable@cabinet-example.com",
        "roles": [
          {
            "role_id": "role-1-uuid",
            "role_type": "Accountant",
            "organization_id": "residence-les-erables-uuid",
            "organization_name": "Résidence Les Érables"
          },
          {
            "role_id": "role-2-uuid",
            "role_type": "Accountant",
            "organization_id": "residence-jardins-uuid",
            "organization_name": "Résidence Les Jardins"
          }
        ],
        "active_role": "role-1-uuid"
      }

3. **Changer d'organisation** : ``POST /api/v1/auth/switch-role``

   .. code-block:: json

      {
        "role_id": "role-2-uuid"
      }

Le rôle actif détermine quelle copropriété vous gérez. Toutes les opérations comptables sont isolées par ``organization_id``.

📊 Plan Comptable PCMN Belge
==============================

Vue d'ensemble du PCMN
----------------------

KoproGo implémente le **Plan Comptable Minimum Normalisé** conformément à l'Arrêté Royal du 12/07/2012.

**90 comptes pré-seed és** organisés en 9 classes :

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
     - 100 Capital, 130 Réserves, 14 Provisions
   * - **2**
     - Actif
     - Immobilisations
     - 220 Bâtiments, 221 Terrains, 24 Mobilier
   * - **3**
     - Actif
     - Stock et encours
     - 30 Matières premières, 32 Marchandises
   * - **4**
     - Actif/Passif
     - Créances et dettes
     - 400 Fournisseurs, 440 Clients, 451 TVA
   * - **5**
     - Actif
     - Banque et caisse
     - 550 Banque, 551 CCP, 57 Caisse
   * - **6**
     - Charges
     - Charges d'exploitation
     - 610 Charges immobilières, 61 Services
   * - **7**
     - Produits
     - Produits d'exploitation
     - 700 Charges locatives, 74 Produits divers
   * - **8**
     - Hors-bilan
     - Droits et engagements
     - 80 Garanties, 81 Engagements
   * - **0**
     - Spécial
     - Comptes généraux (non utilisé pour copropriétés)
     - -

Consulter le plan comptable
----------------------------

**Lister tous les comptes** :

.. code-block:: bash

   GET /api/v1/accounts?organization_id={uuid}

**Rechercher par code** :

.. code-block:: bash

   GET /api/v1/accounts/code/451000

**Réponse** :

.. code-block:: json

   {
     "id": "account-uuid",
     "code": "451000",
     "name": "TVA à récupérer",
     "account_type": "Asset",
     "parent_code": "45",
     "organization_id": "org-uuid",
     "is_active": true,
     "created_at": "2025-01-01T00:00:00Z"
   }

Créer un compte personnalisé
-----------------------------

**Endpoint** : ``POST /api/v1/accounts``

.. code-block:: json

   {
     "code": "612500",
     "name": "Frais de gestion syndic",
     "account_type": "Expense",
     "parent_code": "61",
     "organization_id": "org-uuid"
   }

**Validation** :

- ✅ Code unique par organisation
- ✅ Type de compte valide (Asset, Liability, Equity, Income, Expense, OffBalance)
- ✅ Parent code doit exister (si spécifié)
- ✅ Hiérarchie cohérente (ex: compte 612500 sous parent 61)

Désactiver un compte
--------------------

.. code-block:: json

   PUT /api/v1/accounts/{account_id}
   {
     "is_active": false
   }

Les comptes désactivés ne peuvent plus être utilisés pour de nouvelles écritures mais restent visibles dans l'historique.

Seed initial du PCMN
--------------------

Pour une nouvelle organisation :

.. code-block:: bash

   POST /api/v1/accounts/seed/belgian-pcmn?organization_id={uuid}

Crée automatiquement les ~90 comptes du PCMN belge standard.

💰 Gestion comptable
=====================

Écritures comptables
--------------------

**Structure d'une écriture** :

.. code-block:: json

   {
     "date": "2025-11-10",
     "description": "Paiement facture électricité novembre",
     "lines": [
       {
         "account_code": "610100",
         "description": "Électricité communs",
         "debit": 500.00,
         "credit": 0.00
       },
       {
         "account_code": "451000",
         "description": "TVA récupérable (21%)",
         "debit": 105.00,
         "credit": 0.00
       },
       {
         "account_code": "550000",
         "description": "Paiement banque",
         "debit": 0.00,
         "credit": 605.00
       }
     ]
   }

**Principe de la partie double** :

.. code-block:: text

   Débit total = Crédit total
   500.00 + 105.00 = 605.00 ✅

Imputation automatique des dépenses
------------------------------------

Lors de la création d'une dépense avec lignes de facturation, KoproGo génère automatiquement les imputations comptables :

**Exemple** : Facture d'entretien 1,000€ HT + 21% TVA

.. code-block:: json

   POST /api/v1/expenses
   {
     "building_id": "building-uuid",
     "description": "Entretien espaces verts - Novembre 2025",
     "expense_date": "2025-11-10",
     "category": "Maintenance",
     "line_items": [
       {
         "description": "Tonte et taille",
         "quantity": 10.0,
         "unit_price": 100.00,
         "vat_rate": 0.21
       }
     ]
   }

**Imputations générées** :

.. code-block:: text

   610300  Entretien espaces verts       1,000.00 €  (Débit)
   451000  TVA à récupérer                 210.00 €  (Débit)
   440000  Fournisseurs                            €  1,210.00 (Crédit)

TVA belge
---------

**Taux supportés** :

.. list-table::
   :header-rows: 1
   :widths: 20 40 40

   * - Taux
     - Application
     - Compte PCMN
   * - **6%**
     - Produits de première nécessité, travaux rénovation énergétique
     - 451000 (TVA récupérable)
   * - **12%**
     - Restauration, certains travaux immobiliers
     - 451000 (TVA récupérable)
   * - **21%**
     - Taux standard (services, maintenance, etc.)
     - 451000 (TVA récupérable)

**Déclaration TVA** :

.. code-block:: bash

   GET /api/v1/reports/vat-declaration?organization_id={uuid}&quarter=4&year=2025

.. code-block:: json

   {
     "period": "2025-Q4",
     "vat_collected": 2500.00,
     "vat_deductible": 3200.00,
     "vat_balance": -700.00,
     "status": "ToReclaim",
     "details": {
       "vat_6_percent": {
         "base": 5000.00,
         "vat": 300.00
       },
       "vat_12_percent": {
         "base": 0.00,
         "vat": 0.00
       },
       "vat_21_percent": {
         "base": 13809.52,
         "vat": 2900.00
       }
     }
   }

Charges locatives et appels de fonds
-------------------------------------

**Comptabilisation appel de fonds trimestriel** :

.. code-block:: json

   POST /api/v1/accounting/journal-entries
   {
     "date": "2025-10-01",
     "description": "Appel de fonds T4 2025",
     "lines": [
       {
         "account_code": "440000",
         "description": "Créance copropriétaires",
         "debit": 10800.00,
         "credit": 0.00
       },
       {
         "account_code": "700000",
         "description": "Charges locatives",
         "debit": 0.00,
         "credit": 10800.00
       }
     ]
   }

**Comptabilisation encaissement** :

.. code-block:: json

   {
     "date": "2025-10-25",
     "description": "Paiement charges T4 - Copro Dupont",
     "lines": [
       {
         "account_code": "550000",
         "description": "Banque",
         "debit": 450.00,
         "credit": 0.00
       },
       {
         "account_code": "440000",
         "description": "Créance copropriétaires",
         "debit": 0.00,
         "credit": 450.00
       }
     ]
   }

Grand livre
-----------

**Consulter le grand livre d'un compte** :

.. code-block:: bash

   GET /api/v1/accounting/ledger?account_code=550000&start_date=2025-01-01&end_date=2025-12-31

.. code-block:: json

   {
     "account_code": "550000",
     "account_name": "Banque",
     "period": "2025-01-01 to 2025-12-31",
     "opening_balance": 15000.00,
     "entries": [
       {
         "date": "2025-01-15",
         "description": "Encaissement charges T1",
         "debit": 10800.00,
         "credit": 0.00,
         "balance": 25800.00
       },
       {
         "date": "2025-01-20",
         "description": "Paiement facture électricité",
         "debit": 0.00,
         "credit": 605.00,
         "balance": 25195.00
       }
     ],
     "total_debit": 125000.00,
     "total_credit": 98000.00,
     "closing_balance": 42000.00
   }

Balance comptable
-----------------

**Balance générale** :

.. code-block:: bash

   GET /api/v1/accounting/trial-balance?organization_id={uuid}&date=2025-11-10

.. code-block:: json

   {
     "date": "2025-11-10",
     "accounts": [
       {
         "account_code": "100000",
         "account_name": "Capital",
         "debit": 0.00,
         "credit": 50000.00,
         "balance": -50000.00
       },
       {
         "account_code": "220000",
         "account_name": "Bâtiments",
         "debit": 500000.00,
         "credit": 0.00,
         "balance": 500000.00
       },
       {
         "account_code": "550000",
         "account_name": "Banque",
         "debit": 125000.00,
         "credit": 83000.00,
         "balance": 42000.00
       }
     ],
     "total_debit": 625000.00,
     "total_credit": 625000.00,
     "balanced": true
   }

📈 Rapports financiers
=======================

Bilan comptable
---------------

**Endpoint** : ``GET /api/v1/reports/balance-sheet?organization_id={uuid}&year=2025``

.. code-block:: json

   {
     "period": "2025",
     "assets": {
       "fixed_assets": {
         "buildings": 500000.00,
         "equipment": 25000.00,
         "total": 525000.00
       },
       "current_assets": {
         "receivables": 15000.00,
         "bank": 42000.00,
         "cash": 1500.00,
         "total": 58500.00
       },
       "total_assets": 583500.00
     },
     "liabilities": {
       "equity": {
         "capital": 50000.00,
         "reserves": 120000.00,
         "retained_earnings": 15000.00,
         "total": 185000.00
       },
       "long_term_liabilities": {
         "provisions": 10000.00,
         "total": 10000.00
       },
       "current_liabilities": {
         "suppliers": 8500.00,
         "total": 8500.00
       },
       "total_liabilities": 203500.00
     },
     "balance_check": {
       "assets": 583500.00,
       "liabilities": 203500.00,
       "difference": 380000.00,
       "balanced": true
     }
   }

**Export PDF** :

.. code-block:: bash

   GET /api/v1/reports/balance-sheet.pdf?organization_id={uuid}&year=2025

Compte de résultats
--------------------

**Endpoint** : ``GET /api/v1/reports/income-statement?organization_id={uuid}&year=2025``

.. code-block:: json

   {
     "period": "2025",
     "income": {
       "rental_charges": 43200.00,
       "other_income": 2500.00,
       "total_income": 45700.00
     },
     "expenses": {
       "property_expenses": 12000.00,
       "utilities": 18000.00,
       "maintenance": 8500.00,
       "management_fees": 3600.00,
       "insurance": 4200.00,
       "other_expenses": 1200.00,
       "total_expenses": 47500.00
     },
     "operating_result": -1800.00,
     "financial_result": 150.00,
     "net_result": -1650.00
   }

**Analyse trimestrielle** :

.. code-block:: bash

   GET /api/v1/reports/income-statement?organization_id={uuid}&year=2025&quarter=4

Rapport d'activité
------------------

.. code-block:: bash

   GET /api/v1/reports/activity-report?organization_id={uuid}&year=2025

Inclut :

- Résumé financier (chiffre d'affaires, charges, résultat)
- Évolution trésorerie
- Impayés et taux de recouvrement
- Charges ventilées par catégorie
- Comparaison N vs N-1

📤 Exports comptables
======================

Export Noalyss
--------------

**Format compatible Noalyss** (logiciel de comptabilité GPL) :

.. code-block:: bash

   GET /api/v1/exports/noalyss?organization_id={uuid}&start_date=2025-01-01&end_date=2025-12-31

**Réponse** : Fichier CSV avec écritures comptables au format Noalyss.

Export FEC (Fichier des Écritures Comptables)
----------------------------------------------

**Format standard pour administrations fiscales** :

.. code-block:: bash

   GET /api/v1/exports/fec?organization_id={uuid}&year=2025

**Format** : Fichier texte tabulé conforme à la norme FEC.

Export Excel
------------

.. code-block:: bash

   GET /api/v1/exports/excel?organization_id={uuid}&report_type=balance-sheet&year=2025

**Formats supportés** :

- ``balance-sheet`` : Bilan comptable
- ``income-statement`` : Compte de résultats
- ``trial-balance`` : Balance générale
- ``ledger`` : Grand livre
- ``vat-declaration`` : Déclaration TVA

Export JSON
-----------

.. code-block:: bash

   GET /api/v1/exports/json?organization_id={uuid}&year=2025

Toutes les données comptables de l'année au format JSON structuré.

🔄 Clôture d'exercice
======================

Préparation de la clôture
--------------------------

**Vérifications préalables** :

1. ✅ Balance équilibrée (débit = crédit)
2. ✅ Toutes les factures imputées
3. ✅ Rapprochements bancaires effectués
4. ✅ TVA déclarée et soldée
5. ✅ Provisions évaluées

**Checklist clôture** :

.. code-block:: bash

   GET /api/v1/accounting/year-end-checklist?organization_id={uuid}&year=2025

.. code-block:: json

   {
     "year": 2025,
     "checks": [
       {
         "item": "Balance équilibrée",
         "status": "OK",
         "details": "Débit = Crédit (625,000€)"
       },
       {
         "item": "Factures en attente",
         "status": "WARNING",
         "details": "3 factures non imputées"
       },
       {
         "item": "Rapprochement bancaire",
         "status": "OK",
         "details": "Dernier rapprochement : 31/12/2025"
       },
       {
         "item": "Déclaration TVA Q4",
         "status": "PENDING",
         "details": "À soumettre avant 20/01/2026"
       }
     ],
     "ready_for_closure": false
   }

Exécuter la clôture
-------------------

.. code-block:: bash

   POST /api/v1/accounting/year-end-close?organization_id={uuid}&year=2025

**Opérations effectuées** :

1. **Calcul du résultat** : Solde comptes 6 et 7 → compte 129 (Résultat de l'exercice)
2. **Solde des comptes de gestion** : Comptes 6 et 7 remis à zéro
3. **Reports à nouveau** : Comptes de bilan conservés
4. **Verrouillage** : Écritures N non modifiables
5. **Ouverture N+1** : Balances d'ouverture créées

.. code-block:: json

   {
     "year": 2025,
     "closure_date": "2025-12-31T23:59:59Z",
     "net_result": -1650.00,
     "status": "Closed",
     "opening_balance_2026": {
       "total_assets": 583500.00,
       "total_liabilities": 203500.00,
       "equity": 183350.00
     }
   }

Réouverture d'exercice
-----------------------

En cas d'erreur, réouverture possible avec rôle ``SuperAdmin`` :

.. code-block:: bash

   POST /api/v1/accounting/year-end-reopen?organization_id={uuid}&year=2025

**Attention** : Opération sensible, génère un audit log.

🔍 Contrôles et audits
=======================

Audit trail
-----------

Toutes les opérations comptables sont tracées :

.. code-block:: bash

   GET /api/v1/audit-logs?entity_type=JournalEntry&organization_id={uuid}

.. code-block:: json

   [
     {
       "timestamp": "2025-11-10T10:30:00Z",
       "user": "comptable@example.com",
       "action": "CREATE",
       "entity_type": "JournalEntry",
       "entity_id": "entry-uuid",
       "details": {
         "description": "Paiement facture électricité",
         "amount": 605.00
       },
       "ip_address": "192.168.1.100"
     }
   ]

Rapprochement bancaire
-----------------------

.. code-block:: bash

   GET /api/v1/accounting/bank-reconciliation?account_code=550000&date=2025-11-10

.. code-block:: json

   {
     "account": "550000 - Banque",
     "date": "2025-11-10",
     "book_balance": 42000.00,
     "bank_statement_balance": 41800.00,
     "outstanding_items": [
       {
         "date": "2025-11-08",
         "description": "Chèque #1234 en circulation",
         "amount": -200.00
       }
     ],
     "reconciled_balance": 41800.00,
     "status": "Reconciled"
   }

Détection d'anomalies
----------------------

.. code-block:: bash

   GET /api/v1/accounting/anomalies?organization_id={uuid}

.. code-block:: json

   {
     "anomalies": [
       {
         "type": "UnbalancedEntry",
         "severity": "HIGH",
         "entry_id": "entry-uuid",
         "description": "Écriture déséquilibrée : Débit 500€ ≠ Crédit 505€"
       },
       {
         "type": "MissingVAT",
         "severity": "MEDIUM",
         "expense_id": "expense-uuid",
         "description": "Facture sans TVA imputée"
       },
       {
         "type": "FutureDate",
         "severity": "LOW",
         "entry_id": "entry-uuid",
         "description": "Écriture datée du futur (2025-12-25)"
       }
     ]
   }

📊 Tableau de bord comptable
==============================

Vue d'ensemble financière
--------------------------

.. code-block:: bash

   GET /api/v1/accounting/dashboard?organization_id={uuid}

.. code-block:: json

   {
     "current_balance": 42000.00,
     "receivables": 15000.00,
     "payables": 8500.00,
     "cash_flow": {
       "last_30_days": {
         "inflows": 12500.00,
         "outflows": 9800.00,
         "net": 2700.00
       }
     },
     "overdue_invoices": {
       "count": 8,
       "total_amount": 3600.00
     },
     "alerts": [
       {
         "type": "LowBalance",
         "message": "Trésorerie < 2 mois de charges",
         "severity": "WARNING"
       }
     ]
   }

Indicateurs de performance
---------------------------

**KPIs comptables** :

.. code-block:: bash

   GET /api/v1/accounting/kpis?organization_id={uuid}&year=2025

.. code-block:: json

   {
     "year": 2025,
     "kpis": {
       "collection_rate": 0.92,
       "average_payment_delay": 15.5,
       "operating_margin": -0.04,
       "cash_coverage_months": 2.8,
       "budget_variance": {
         "income": 0.03,
         "expenses": -0.07
       }
     }
   }

🛠️ Outils pour comptables
===========================

Calculatrice TVA
----------------

.. code-block:: bash

   GET /api/v1/tools/vat-calculator?amount=1000&vat_rate=0.21&mode=excl

.. code-block:: json

   {
     "amount_excl_vat": 1000.00,
     "vat_amount": 210.00,
     "amount_incl_vat": 1210.00,
     "vat_rate": 0.21
   }

**Modes** :

- ``excl`` : Montant HT → Calcul TTC
- ``incl`` : Montant TTC → Calcul HT

Générateur de lettrage
-----------------------

.. code-block:: bash

   POST /api/v1/tools/auto-match-entries?account_code=440000&organization_id={uuid}

Lettrage automatique des créances/dettes (matching invoices ↔ payments).

🔐 Sécurité et conformité
===========================

Multi-tenant strict
-------------------

- ✅ Isolation totale par ``organization_id``
- ✅ Impossible d'accéder aux données d'une autre organisation
- ✅ Validation JWT avec rôle actif

Permissions comptables
----------------------

.. list-table::
   :header-rows: 1
   :widths: 40 20 20 20

   * - Action
     - Accountant
     - Syndic
     - Owner
   * - Lecture plan comptable
     - ✅
     - ✅
     - ❌
   * - Création comptes
     - ✅
     - ❌
     - ❌
   * - Écritures comptables
     - ✅
     - ❌
     - ❌
   * - Clôture d'exercice
     - ✅
     - ❌
     - ❌
   * - Lecture rapports
     - ✅
     - ✅
     - ❌

Conservation des données
------------------------

**Durée légale (Belgique)** : **7 ans minimum**

KoproGo conserve automatiquement :

- Toutes les écritures comptables
- Pièces justificatives (factures, contrats)
- Audit logs
- Déclarations TVA

Backup automatique chiffré (GPG + S3).

📚 Ressources et support
=========================

Documentation technique
-----------------------

- **Plan comptable PCMN** : :doc:`../BELGIAN_ACCOUNTING_PCMN`
- **Guide du syndic** : :doc:`syndic-guide`
- **API Reference** : https://api.koprogo.com/docs

Crédits
-------

L'implémentation du PCMN belge s'inspire du projet **Noalyss** :

- **Projet** : https://gitlab.com/noalyss/noalyss
- **Licence** : GPL-2.0-or-later
- **Auteur** : Dany De Bontridder <dany@alchimerys.eu>

Merci au projet Noalyss pour sa référence inestimable.

Support
-------

- **Email** : support@koprogo.com
- **Documentation** : https://docs.koprogo.com
- **Issues GitHub** : https://github.com/gilmry/koprogo/issues

---

**Version** : 1.0.0 | **Dernière mise à jour** : 10 novembre 2025
