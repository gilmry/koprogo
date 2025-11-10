===================================
Guide Utilisateur : Syndic
===================================

:Version: 1.0.0
:Date: 10 novembre 2025
:Public: Syndics de copropriété
:Voir aussi: :doc:`owner-guide` | :doc:`accountant-guide` | :doc:`board-member-guide`

📋 Vue d'ensemble
=================

Ce guide explique comment utiliser **KoproGo** en tant que **syndic de copropriété**. Vous y trouverez :

- ✅ Configuration initiale de votre immeuble
- ✅ Gestion des copropriétaires et des lots
- ✅ Création et suivi des dépenses
- ✅ Workflow d'approbation des factures
- ✅ Relances de paiement automatisées
- ✅ Comptabilité PCMN belge
- ✅ Organisation des assemblées générales

🚀 Premiers pas
================

Connexion et rôle actif
-----------------------

1. **Connexion** : ``POST /api/v1/auth/login``

   .. code-block:: json

      {
        "email": "syndic@example.com",
        "password": "votre_mot_de_passe"
      }

2. **Sélection du rôle** : Si vous gérez plusieurs immeubles, sélectionnez votre rôle actif

   .. code-block:: bash

      POST /api/v1/auth/switch-role
      {
        "role_id": "uuid-du-role-syndic"
      }

3. **Vérification** : ``GET /api/v1/auth/me`` retourne vos informations avec le rôle actif

Configuration d'un nouvel immeuble
----------------------------------

**Endpoint** : ``POST /api/v1/buildings``

.. code-block:: json

   {
     "name": "Résidence Les Érables",
     "address": "123 Avenue de Tervuren, 1040 Bruxelles",
     "total_units": 24,
     "construction_year": 1985
   }

**Réponse** :

.. code-block:: json

   {
     "id": "550e8400-e29b-41d4-a716-446655440000",
     "name": "Résidence Les Érables",
     "address": "123 Avenue de Tervuren, 1040 Bruxelles",
     "total_units": 24,
     "construction_year": 1985,
     "created_at": "2025-11-10T10:00:00Z",
     "updated_at": "2025-11-10T10:00:00Z"
   }

🏠 Gestion des lots et copropriétaires
========================================

Créer un lot
------------

**Endpoint** : ``POST /api/v1/units``

.. code-block:: json

   {
     "building_id": "550e8400-e29b-41d4-a716-446655440000",
     "unit_number": "2B",
     "floor": 2,
     "area": 85.5,
     "type": "Apartment"
   }

Ajouter un copropriétaire au lot
---------------------------------

**Endpoint** : ``POST /api/v1/units/{unit_id}/owners``

.. code-block:: json

   {
     "owner_id": "owner-uuid",
     "ownership_percentage": 0.50,
     "is_primary_contact": true
   }

**Validation automatique** :

- ✅ Quote-part entre 0% et 100%
- ✅ Somme des quotes-parts ≤ 100%
- ✅ Un seul contact principal par lot

Cas multi-propriétaires
------------------------

**Exemple** : Appartement détenu par 2 copropriétaires (50/50)

.. code-block:: bash

   # Premier copropriétaire (contact principal)
   POST /api/v1/units/{unit_id}/owners
   {
     "owner_id": "owner-1-uuid",
     "ownership_percentage": 0.50,
     "is_primary_contact": true
   }

   # Deuxième copropriétaire
   POST /api/v1/units/{unit_id}/owners
   {
     "owner_id": "owner-2-uuid",
     "ownership_percentage": 0.50,
     "is_primary_contact": false
   }

**Vérification** : ``GET /api/v1/units/{unit_id}/owners/total-percentage`` retourne ``1.0`` (100%)

Historique et transferts
-------------------------

**Consulter l'historique** :

.. code-block:: bash

   GET /api/v1/units/{unit_id}/owners/history

**Transférer une quote-part** :

.. code-block:: json

   POST /api/v1/units/{unit_id}/owners/transfer
   {
     "from_owner_id": "ancien-proprietaire-uuid",
     "to_owner_id": "nouveau-proprietaire-uuid"
   }

KoproGo clôture automatiquement l'ancienne relation (``end_date`` renseignée) et en crée une nouvelle avec la même quote-part.

💰 Gestion des dépenses et factures
=====================================

Workflow d'approbation
-----------------------

KoproGo utilise un workflow structuré pour les dépenses :

.. code-block:: text

   Draft → PendingApproval → Approved
                          ↘ Rejected

États disponibles :

- **Draft** : Brouillon, éditable
- **PendingApproval** : Soumise pour approbation (conseil syndical)
- **Approved** : Approuvée et payable
- **Rejected** : Refusée avec motif

Créer une dépense
-----------------

**Endpoint** : ``POST /api/v1/expenses``

.. code-block:: json

   {
     "building_id": "550e8400-e29b-41d4-a716-446655440000",
     "description": "Réparation ascenseur - Maintenance annuelle",
     "amount": 1250.00,
     "expense_date": "2025-11-10",
     "category": "Maintenance",
     "status": "Draft"
   }

Ajouter des lignes de facturation (TVA belge)
----------------------------------------------

KoproGo supporte les taux de TVA belges : **6%**, **12%**, **21%**

**Endpoint** : ``POST /api/v1/expenses/{expense_id}/line-items``

.. code-block:: json

   {
     "description": "Main d'œuvre technicien",
     "quantity": 4.0,
     "unit_price": 75.00,
     "vat_rate": 0.21
   }

**Calculs automatiques** :

- Montant HT : 4 × 75.00 = 300.00€
- TVA (21%) : 300.00 × 0.21 = 63.00€
- Montant TTC : 363.00€

Soumettre pour approbation
---------------------------

.. code-block:: bash

   PUT /api/v1/expenses/{expense_id}/submit-for-approval

La dépense passe de ``Draft`` à ``PendingApproval`` et devient **non modifiable**.

Approuver/rejeter (rôle conseil syndical)
------------------------------------------

**Approuver** :

.. code-block:: bash

   PUT /api/v1/expenses/{expense_id}/approve

**Rejeter** :

.. code-block:: json

   PUT /api/v1/expenses/{expense_id}/reject
   {
     "rejection_reason": "Budget insuffisant pour ce trimestre"
   }

Marquer comme payée
--------------------

.. code-block:: bash

   PUT /api/v1/expenses/{expense_id}/mark-paid

🔔 Relances de paiement automatisées
======================================

KoproGo gère automatiquement les relances pour les impayés avec 4 niveaux d'escalade :

Niveaux d'escalade
------------------

.. list-table::
   :header-rows: 1
   :widths: 20 20 30 30

   * - Niveau
     - Délai
     - Ton
     - Action
   * - **Gentle**
     - J+15
     - Rappel courtois
     - Email automatique
   * - **Formal**
     - J+30
     - Mise en demeure
     - Lettre recommandée
   * - **FinalNotice**
     - J+45
     - Dernier avertissement
     - Pénalités activées
   * - **LegalAction**
     - J+60
     - Procédure judiciaire
     - Transmission avocat

Créer une relance
-----------------

**Endpoint** : ``POST /api/v1/payment-reminders``

.. code-block:: json

   {
     "expense_id": "expense-uuid",
     "owner_id": "owner-uuid",
     "reminder_level": "Gentle",
     "message": "Rappel aimable concernant les charges du T4 2025"
   }

Calculer les pénalités de retard
---------------------------------

KoproGo applique le **taux légal belge de 8% annuel** :

.. code-block:: bash

   GET /api/v1/payment-reminders/{reminder_id}/calculate-penalties

**Exemple** :

- Montant dû : 1,000€
- Retard : 60 jours
- Pénalités : 1,000 × 0.08 × (60/365) = **13.15€**

Escalade automatique
--------------------

.. code-block:: bash

   PUT /api/v1/payment-reminders/{reminder_id}/escalate

KoproGo incrémente automatiquement le niveau de relance et calcule les nouvelles pénalités.

Statistiques de recouvrement
-----------------------------

.. code-block:: bash

   GET /api/v1/payment-reminders/stats

**Réponse** :

.. code-block:: json

   {
     "total_reminders": 45,
     "by_level": {
       "Gentle": 20,
       "Formal": 15,
       "FinalNotice": 8,
       "LegalAction": 2
     },
     "total_penalties": 3456.78,
     "recovery_rate": 0.87
   }

📊 Comptabilité PCMN belge
============================

KoproGo implémente le **Plan Comptable Minimum Normalisé** (AR 12/07/2012).

Comptes pré-configurés
-----------------------

~90 comptes sont pré-seed és dans 8 classes :

.. list-table::
   :header-rows: 1
   :widths: 10 30 60

   * - Classe
     - Type
     - Exemples
   * - **1**
     - Passif
     - 100 Capital, 130 Réserves, 14 Provisions
   * - **2**
     - Actif
     - 220 Bâtiments, 221 Terrains
   * - **3**
     - Actif
     - 30 Matières premières
   * - **4**
     - Actif/Passif
     - 400 Fournisseurs, 440 Clients, 451 TVA
   * - **5**
     - Actif
     - 550 Banque, 551 CCP, 57 Caisse
   * - **6**
     - Charges
     - 610 Charges immobilières, 61 Services
   * - **7**
     - Produits
     - 700 Charges locatives, 74 Produits divers
   * - **8**
     - Hors-bilan
     - 80 Droits et engagements

Consulter le plan comptable
----------------------------

.. code-block:: bash

   GET /api/v1/accounts

**Recherche par code** :

.. code-block:: bash

   GET /api/v1/accounts/code/451000

Créer un compte personnalisé
-----------------------------

.. code-block:: json

   POST /api/v1/accounts
   {
     "code": "612500",
     "name": "Frais de gestion syndic",
     "account_type": "Expense",
     "parent_code": "61"
   }

Générer des rapports financiers
--------------------------------

**Bilan comptable** :

.. code-block:: bash

   GET /api/v1/reports/balance-sheet?year=2025

**Compte de résultats** :

.. code-block:: bash

   GET /api/v1/reports/income-statement?year=2025&quarter=4

**Réponse** :

.. code-block:: json

   {
     "period": "2025-Q4",
     "income": {
       "rental_charges": 45000.00,
       "other_income": 2500.00,
       "total": 47500.00
     },
     "expenses": {
       "maintenance": 12000.00,
       "utilities": 18000.00,
       "management_fees": 3500.00,
       "insurance": 4200.00,
       "total": 37700.00
     },
     "net_result": 9800.00
   }

📅 Gestion des assemblées générales
=====================================

Créer une assemblée
-------------------

**Endpoint** : ``POST /api/v1/meetings``

.. code-block:: json

   {
     "building_id": "550e8400-e29b-41d4-a716-446655440000",
     "meeting_type": "GeneralAssembly",
     "scheduled_date": "2025-12-15T18:00:00Z",
     "location": "Salle communautaire - Rez-de-chaussée",
     "agenda": "1. Approbation des comptes 2025\n2. Budget prévisionnel 2026\n3. Travaux de toiture\n4. Questions diverses"
   }

Joindre des documents
---------------------

.. code-block:: json

   POST /api/v1/documents
   {
     "title": "Convocation AG - 15 décembre 2025",
     "document_type": "MeetingInvitation",
     "file_path": "/uploads/convocation-ag-2025-12.pdf",
     "meeting_id": "meeting-uuid"
   }

Enregistrer le procès-verbal
-----------------------------

.. code-block:: json

   PUT /api/v1/meetings/{meeting_id}
   {
     "minutes": "## Procès-verbal AG du 15/12/2025\n\n**Présents** : 18/24 copropriétaires\n**Quorum** : Atteint (75%)\n\n### 1. Approbation des comptes\nComptes 2025 approuvés à l'unanimité...",
     "status": "Completed"
   }

📄 Gestion documentaire
=========================

Ajouter un document
-------------------

.. code-block:: json

   POST /api/v1/documents
   {
     "title": "Contrat ascenseur 2025-2028",
     "document_type": "Contract",
     "file_path": "/uploads/contrat-ascenseur-2025.pdf",
     "building_id": "building-uuid"
   }

Types de documents supportés
-----------------------------

- ``Invoice`` : Factures
- ``Contract`` : Contrats
- ``MeetingInvitation`` : Convocations AG
- ``MeetingMinutes`` : Procès-verbaux
- ``InsurancePolicy`` : Polices d'assurance
- ``MaintenanceReport`` : Rapports de maintenance
- ``FinancialReport`` : Rapports financiers
- ``Other`` : Autres documents

Lier un document à une dépense
-------------------------------

.. code-block:: json

   PUT /api/v1/documents/{document_id}
   {
     "expense_id": "expense-uuid"
   }

Rechercher des documents
-------------------------

.. code-block:: bash

   GET /api/v1/documents?building_id={uuid}&document_type=Invoice&limit=20&offset=0

🔐 Sécurité et GDPR
====================

KoproGo est conforme GDPR avec les fonctionnalités suivantes :

Droit d'accès (Art. 15)
------------------------

.. code-block:: bash

   GET /api/v1/gdpr/owners/{owner_id}/data-export

**Retourne** : Toutes les données personnelles au format JSON.

Droit à l'effacement (Art. 17)
-------------------------------

.. code-block:: bash

   DELETE /api/v1/gdpr/owners/{owner_id}/delete-data

**Validation** : Impossible si le copropriétaire a des dettes ou litiges en cours.

Droit à la portabilité (Art. 20)
---------------------------------

.. code-block:: bash

   GET /api/v1/gdpr/owners/{owner_id}/portable-data

**Format** : JSON structuré pour import dans un autre système.

Audit logging
-------------

Toutes les actions sensibles sont tracées :

.. code-block:: bash

   GET /api/v1/audit-logs?entity_type=Owner&entity_id={uuid}

📊 Tableau de bord et statistiques
====================================

Statistiques globales
---------------------

.. code-block:: bash

   GET /api/v1/stats/dashboard

**Réponse** :

.. code-block:: json

   {
     "total_buildings": 3,
     "total_units": 72,
     "total_owners": 65,
     "pending_expenses": 12,
     "total_pending_amount": 45678.90,
     "overdue_payments": 8,
     "overdue_amount": 12340.00,
     "next_meetings": [
       {
         "id": "meeting-uuid",
         "building_name": "Résidence Les Érables",
         "scheduled_date": "2025-12-15T18:00:00Z"
       }
     ]
   }

Statistiques par immeuble
--------------------------

.. code-block:: bash

   GET /api/v1/buildings/{building_id}/stats

🛠️ Dépannage
==============

Problèmes courants
------------------

**Quote-part totale > 100%**

.. code-block:: text

   Erreur : "Total ownership percentage would exceed 100%"
   Solution : Vérifier la somme des quotes-parts actives avec
   GET /api/v1/units/{unit_id}/owners/total-percentage

**Modification d'une dépense approuvée**

.. code-block:: text

   Erreur : "Cannot modify expense in status Approved"
   Solution : Les dépenses approuvées sont immutables. Créer une nouvelle dépense
   ou rejeter puis modifier.

**Contact principal multiple**

.. code-block:: text

   Erreur : "Only one owner can be primary contact"
   Solution : KoproGo désactive automatiquement l'ancien contact principal.
   Vérifier avec GET /api/v1/units/{unit_id}/owners

📞 Support et documentation
=============================

- **Documentation complète** : https://docs.koprogo.com
- **API Reference** : https://api.koprogo.com/docs (Swagger UI)
- **Issues GitHub** : https://github.com/gilmry/koprogo/issues
- **Support email** : support@koprogo.com

---

**Version** : 1.0.0 | **Dernière mise à jour** : 10 novembre 2025
