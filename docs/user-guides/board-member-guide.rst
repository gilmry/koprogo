==================================================
Guide Utilisateur : Conseil Syndical
==================================================

:Version: 1.0.0
:Date: 10 novembre 2025
:Public: Membres du conseil syndical
:Voir aussi: :doc:`syndic-guide` | :doc:`owner-guide` | :doc:`accountant-guide`

📋 Vue d'ensemble
=================

Ce guide explique comment utiliser **KoproGo** en tant que **membre du conseil syndical**. Vous y trouverez :

- ✅ Rôle et responsabilités du conseil syndical
- ✅ Validation des dépenses importantes
- ✅ Suivi budgétaire et financier
- ✅ Préparation et organisation des AG
- ✅ Contrôle des prestataires et contrats
- ✅ Communication avec les copropriétaires

🏛️ Le Conseil Syndical : Cadre Légal
======================================

Obligation légale (Belgique)
-----------------------------

En Belgique, le **conseil syndical** est **obligatoire pour les copropriétés de plus de 20 lots** (Art. 577-8/4 du Code civil).

**Rôle** :

- Assister le syndic dans sa mission
- Contrôler la gestion de l'immeuble
- Valider les dépenses importantes
- Préparer les assemblées générales
- Représenter les copropriétaires

**Composition** :

- 3 à 5 membres élus en AG
- Mandat de 3 ans renouvelable
- Membres issus des copropriétaires

Responsabilités dans KoproGo
-----------------------------

Dans KoproGo, le conseil syndical a accès à :

- ✅ Tableau de bord financier
- ✅ Validation des dépenses > seuil défini en AG
- ✅ Suivi des contrats et prestataires
- ✅ Préparation des documents d'AG
- ✅ Statistiques et rapports
- ✅ Communication avec copropriétaires

🚀 Premiers pas
================

Connexion et rôle
-----------------

1. **Connexion** : ``POST /api/v1/auth/login``

   .. code-block:: json

      {
        "email": "votre.email@example.com",
        "password": "votre_mot_de_passe"
      }

2. **Vérification du rôle** : ``GET /api/v1/auth/me``

   .. code-block:: json

      {
        "id": "user-uuid",
        "email": "votre.email@example.com",
        "roles": [
          {
            "role_type": "BoardMember",
            "organization_id": "residence-les-erables-uuid",
            "organization_name": "Résidence Les Érables"
          },
          {
            "role_type": "Owner",
            "organization_id": "residence-les-erables-uuid"
          }
        ],
        "active_role": "BoardMember"
      }

**Note** : Un membre du conseil est généralement aussi copropriétaire. Vous pouvez basculer entre les rôles via ``POST /api/v1/auth/switch-role``.

Accès au tableau de bord
-------------------------

**Endpoint** : ``GET /api/v1/board/dashboard``

.. code-block:: json

   {
     "building_name": "Résidence Les Érables",
     "total_units": 24,
     "total_owners": 22,
     "financial_summary": {
       "current_balance": 42000.00,
       "annual_budget": 120000.00,
       "spent_to_date": 87500.00,
       "budget_remaining": 32500.00
     },
     "pending_approvals": 3,
     "overdue_payments": 8,
     "next_meeting": {
       "date": "2025-12-15T18:00:00Z",
       "type": "GeneralAssembly"
     }
   }

💰 Validation des dépenses
============================

Workflow d'approbation
----------------------

KoproGo implémente un workflow de validation à 3 niveaux :

.. code-block:: text

   Draft (Syndic) → PendingApproval (Conseil) → Approved
                                              ↘ Rejected

**Seuils** (définissables en AG) :

- < 500€ : Approbation automatique syndic
- 500€ - 5,000€ : Approbation conseil syndical
- > 5,000€ : Approbation AG

Consulter les dépenses en attente
----------------------------------

**Endpoint** : ``GET /api/v1/expenses?status=PendingApproval``

.. code-block:: json

   [
     {
       "id": "expense-uuid-1",
       "description": "Réparation ascenseur - Remplacement câble",
       "amount": 2500.00,
       "expense_date": "2025-11-10",
       "category": "Maintenance",
       "status": "PendingApproval",
       "submitted_by": "syndic@example.com",
       "submitted_date": "2025-11-08T10:00:00Z",
       "supporting_documents": [
         {
           "title": "Devis Ascenseurs SPRL",
           "file_path": "/uploads/devis-ascenseur-2025-11.pdf"
         },
         {
           "title": "Rapport technique",
           "file_path": "/uploads/rapport-tech-ascenseur.pdf"
         }
       ]
     }
   ]

Détail d'une dépense
--------------------

**Endpoint** : ``GET /api/v1/expenses/{expense_id}``

.. code-block:: json

   {
     "id": "expense-uuid-1",
     "description": "Réparation ascenseur - Remplacement câble",
     "amount": 2500.00,
     "expense_date": "2025-11-10",
     "category": "Maintenance",
     "status": "PendingApproval",
     "line_items": [
       {
         "description": "Câble acier 10mm - 50m",
         "quantity": 1.0,
         "unit_price": 800.00,
         "vat_rate": 0.21,
         "total_incl_vat": 968.00
       },
       {
         "description": "Main d'œuvre technicien (8h)",
         "quantity": 8.0,
         "unit_price": 85.00,
         "vat_rate": 0.21,
         "total_incl_vat": 822.80
       },
       {
         "description": "Déplacement et matériel",
         "quantity": 1.0,
         "unit_price": 150.00,
         "vat_rate": 0.21,
         "total_incl_vat": 181.50
       }
     ],
     "total_excl_vat": 2066.12,
     "total_vat": 433.88,
     "total_incl_vat": 2500.00,
     "supporting_documents": [...],
     "comparison": {
       "budget_category": "Maintenance",
       "budget_allocated": 8000.00,
       "spent_to_date": 4200.00,
       "this_expense": 2500.00,
       "remaining_after": 1300.00
     }
   }

Approuver une dépense
---------------------

**Endpoint** : ``PUT /api/v1/expenses/{expense_id}/approve``

.. code-block:: json

   {
     "approval_notes": "Dépense approuvée après vérification devis et urgence technique confirmée."
   }

**Validation** :

- ✅ Rôle ``BoardMember`` requis
- ✅ Dépense en statut ``PendingApproval``
- ✅ Montant < seuil AG

**Effet** : Statut passe à ``Approved``, syndic peut procéder au paiement.

Rejeter une dépense
-------------------

**Endpoint** : ``PUT /api/v1/expenses/{expense_id}/reject``

.. code-block:: json

   {
     "rejection_reason": "Montant trop élevé. Demande de 2 devis supplémentaires pour comparaison."
   }

**Effet** : Statut passe à ``Rejected``, syndic notifié et peut créer une nouvelle version.

Demander des informations complémentaires
------------------------------------------

.. code-block:: json

   POST /api/v1/expenses/{expense_id}/request-info
   {
     "message": "Merci de fournir un second devis comparatif et le rapport d'inspection de l'ascenseur."
   }

**Effet** : Notification au syndic, dépense reste en ``PendingApproval``.

📊 Suivi budgétaire
====================

Budget annuel
-------------

**Consulter le budget** : ``GET /api/v1/board/budget?year=2025``

.. code-block:: json

   {
     "year": 2025,
     "total_budget": 120000.00,
     "categories": [
       {
         "category": "Maintenance",
         "budget": 30000.00,
         "spent": 18500.00,
         "committed": 5000.00,
         "remaining": 6500.00,
         "usage_percentage": 0.78
       },
       {
         "category": "Utilities",
         "budget": 45000.00,
         "spent": 32000.00,
         "committed": 10000.00,
         "remaining": 3000.00,
         "usage_percentage": 0.93
       },
       {
         "category": "Insurance",
         "budget": 15000.00,
         "spent": 14200.00,
         "committed": 0.00,
         "remaining": 800.00,
         "usage_percentage": 0.95
       },
       {
         "category": "ManagementFees",
         "budget": 12000.00,
         "spent": 9000.00,
         "committed": 3000.00,
         "remaining": 0.00,
         "usage_percentage": 1.00
       },
       {
         "category": "Reserve",
         "budget": 18000.00,
         "spent": 0.00,
         "committed": 0.00,
         "remaining": 18000.00,
         "usage_percentage": 0.00
       }
     ],
     "overall": {
       "spent": 73700.00,
       "committed": 18000.00,
       "remaining": 28300.00,
       "usage_percentage": 0.76
     }
   }

Alertes budgétaires
-------------------

.. code-block:: bash

   GET /api/v1/board/budget-alerts

.. code-block:: json

   {
     "alerts": [
       {
         "severity": "WARNING",
         "category": "Utilities",
         "message": "Budget dépassé à 93% alors que nous sommes à 83% de l'année",
         "recommendation": "Prévoir rallonge budgétaire ou réduire consommation"
       },
       {
         "severity": "CRITICAL",
         "category": "Insurance",
         "message": "Budget épuisé à 95%, reste 2 mois",
         "recommendation": "Utiliser fonds de réserve ou appel exceptionnel"
       }
     ]
   }

Projection fin d'année
-----------------------

.. code-block:: bash

   GET /api/v1/board/budget-projection?year=2025

.. code-block:: json

   {
     "year": 2025,
     "current_date": "2025-11-10",
     "projection_date": "2025-12-31",
     "method": "LinearExtrapolation",
     "results": {
       "total_budget": 120000.00,
       "projected_spending": 128500.00,
       "projected_overrun": 8500.00,
       "confidence": 0.75,
       "categories_at_risk": ["Utilities", "Insurance"]
     },
     "recommendations": [
       "Préparer appel de fonds exceptionnel (8,500€)",
       "Discuter rallonge budgétaire lors de la prochaine AG",
       "Étudier réduction consommation énergétique"
     ]
   }

📄 Préparation des AG
======================

Créer un projet d'ordre du jour
--------------------------------

**Endpoint** : ``POST /api/v1/board/meeting-drafts``

.. code-block:: json

   {
     "meeting_type": "GeneralAssembly",
     "proposed_date": "2025-12-15T18:00:00Z",
     "location": "Salle communautaire - Rez-de-chaussée",
     "agenda_items": [
       {
         "order": 1,
         "title": "Approbation des comptes 2025",
         "description": "Présentation bilan financier et compte de résultats",
         "attachments": ["financial-report-2025.pdf"]
       },
       {
         "order": 2,
         "title": "Budget prévisionnel 2026",
         "description": "Vote du budget et des charges trimestrielles",
         "attachments": ["budget-2026.pdf"]
       },
       {
         "order": 3,
         "title": "Travaux de toiture",
         "description": "Validation devis et calendrier travaux",
         "attachments": ["devis-toiture-1.pdf", "devis-toiture-2.pdf"]
       }
     ]
   }

Générer les documents d'AG
---------------------------

**Convocation automatique** :

.. code-block:: bash

   POST /api/v1/board/generate-invitation?meeting_id={uuid}

**Contenu généré** :

- Ordre du jour détaillé
- Rapports financiers annexés
- Formulaire de procuration
- Instructions d'accès (physique + visio si applicable)

**Envoi** : Email automatique à tous les copropriétaires (15 jours avant AG minimum, conforme loi belge).

Suivi des procurations
-----------------------

.. code-block:: bash

   GET /api/v1/board/meeting/{meeting_id}/proxies

.. code-block:: json

   {
     "meeting_id": "meeting-uuid",
     "scheduled_date": "2025-12-15T18:00:00Z",
     "total_units": 24,
     "proxies": [
       {
         "owner_name": "Marie Dubois",
         "unit_number": "3A",
         "proxy_to": "Jean Martin",
         "received_date": "2025-11-25T10:00:00Z",
         "status": "Valid"
       }
     ],
     "quorum": {
       "required": 12,
       "confirmed": 18,
       "with_proxies": 21,
       "status": "QuorumReached"
     }
   }

Votes et résolutions
--------------------

**Enregistrer les votes** :

.. code-block:: json

   POST /api/v1/board/meeting/{meeting_id}/votes
   {
     "agenda_item_id": "item-uuid",
     "resolution": "Approbation des comptes 2025",
     "votes": {
       "for": 20,
       "against": 1,
       "abstain": 3
     },
     "result": "Approved",
     "notes": "Approuvé à la majorité des 4/5"
   }

Rédiger le procès-verbal
-------------------------

**Template pré-rempli** :

.. code-block:: bash

   GET /api/v1/board/meeting/{meeting_id}/minutes-template

**Enregistrer le PV** :

.. code-block:: json

   PUT /api/v1/meetings/{meeting_id}
   {
     "minutes": "## Procès-verbal AG du 15/12/2025\n\n**Présents** : 18/24 copropriétaires\n**Procurations** : 3\n**Quorum** : Atteint (87.5%)\n\n### 1. Approbation des comptes 2025\n**Vote** : Pour 20, Contre 1, Abstention 3\n**Résultat** : Approuvé\n\n...",
     "status": "Completed"
   }

**Publication** : PV accessible à tous les copropriétaires via ``GET /api/v1/documents``.

📋 Contrôle des prestataires
==============================

Liste des contrats actifs
--------------------------

**Endpoint** : ``GET /api/v1/board/contracts``

.. code-block:: json

   [
     {
       "id": "contract-uuid-1",
       "provider": "Ascenseurs SPRL",
       "service": "Maintenance ascenseur",
       "start_date": "2023-01-01",
       "end_date": "2026-12-31",
       "annual_cost": 3600.00,
       "payment_frequency": "Quarterly",
       "next_renewal": "2026-10-01",
       "status": "Active",
       "performance_score": 4.2
     },
     {
       "id": "contract-uuid-2",
       "provider": "CleanPro Services",
       "service": "Nettoyage communs",
       "start_date": "2024-06-01",
       "end_date": "2025-05-31",
       "annual_cost": 7200.00,
       "payment_frequency": "Monthly",
       "next_renewal": "2025-04-01",
       "status": "Active",
       "performance_score": 3.8
     }
   ]

Évaluation des prestataires
----------------------------

.. code-block:: json

   POST /api/v1/board/contracts/{contract_id}/evaluation
   {
     "period": "2025-Q3",
     "criteria": {
       "quality": 4,
       "timeliness": 5,
       "communication": 3,
       "value_for_money": 4
     },
     "overall_score": 4.0,
     "notes": "Service de qualité mais délais de réponse parfois longs"
   }

Alertes renouvellement
-----------------------

.. code-block:: bash

   GET /api/v1/board/contract-renewals

.. code-block:: json

   {
     "upcoming_renewals": [
       {
         "contract_id": "contract-uuid-2",
         "provider": "CleanPro Services",
         "service": "Nettoyage communs",
         "end_date": "2025-05-31",
         "days_until_renewal": 202,
         "recommendation": "Lancer appel d'offres 3 mois avant (février 2025)",
         "performance_score": 3.8
       }
     ]
   }

Comparaison devis
-----------------

.. code-block:: json

   POST /api/v1/board/quote-comparison
   {
     "service": "Maintenance ascenseur",
     "quotes": [
       {
         "provider": "Ascenseurs SPRL",
         "annual_cost": 3600.00,
         "includes": ["Visites trimestrielles", "Dépannage 24/7", "Pièces"],
         "rating": 4.5
       },
       {
         "provider": "LiftCare SA",
         "annual_cost": 3200.00,
         "includes": ["Visites trimestrielles", "Dépannage bureau", "Pièces"],
         "rating": 4.0
       },
       {
         "provider": "TechLift Belgium",
         "annual_cost": 4100.00,
         "includes": ["Visites mensuelles", "Dépannage 24/7", "Pièces", "Modernisation"],
         "rating": 4.8
       }
     ]
   }

**Analyse automatique** :

.. code-block:: json

   {
     "recommendation": "LiftCare SA",
     "reason": "Meilleur rapport qualité/prix (-11% vs actuel) avec services essentiels",
     "savings": 400.00,
     "risk_assessment": "LOW"
   }

📢 Communication copropriétaires
==================================

Envoyer un message collectif
-----------------------------

.. code-block:: json

   POST /api/v1/board/announcements
   {
     "subject": "Travaux de peinture cage d'escalier",
     "message": "Chers copropriétaires,\n\nLes travaux de peinture de la cage d'escalier débuteront le lundi 20 novembre et dureront 5 jours ouvrables.\n\nMerci de votre compréhension.\n\nLe Conseil Syndical",
     "priority": "Normal",
     "recipients": "AllOwners"
   }

**Options recipients** :

- ``AllOwners`` : Tous les copropriétaires
- ``BoardMembers`` : Membres du conseil uniquement
- ``Specific`` : Liste d'IDs spécifiques

Consulter les messages
-----------------------

.. code-block:: bash

   GET /api/v1/board/announcements?limit=10

Réponses et questions
---------------------

.. code-block:: bash

   GET /api/v1/board/messages/inbox

.. code-block:: json

   [
     {
       "id": "message-uuid",
       "from": "owner-uuid",
       "from_name": "Jean Dupont",
       "subject": "Question sur charges T4",
       "message": "Bonjour, je constate une augmentation de 15% sur mes charges. Pouvez-vous m'expliquer ?",
       "date": "2025-11-09T14:30:00Z",
       "status": "Unread"
     }
   ]

**Répondre** :

.. code-block:: json

   POST /api/v1/board/messages/{message_id}/reply
   {
     "message": "Bonjour Monsieur Dupont,\n\nL'augmentation est due à la hausse des tarifs énergétiques (+12%) et à des travaux exceptionnels de maintenance.\n\nCordialement,\nLe Conseil Syndical"
   }

📊 Rapports et statistiques
=============================

Rapport trimestriel
-------------------

.. code-block:: bash

   GET /api/v1/board/quarterly-report?year=2025&quarter=4

.. code-block:: json

   {
     "period": "2025-Q4",
     "financial": {
       "income": 11250.00,
       "expenses": 9800.00,
       "net_result": 1450.00
     },
     "budget": {
       "planned": 30000.00,
       "spent": 9800.00,
       "variance": -0.02
     },
     "payments": {
       "collected": 10800.00,
       "outstanding": 450.00,
       "collection_rate": 0.96
     },
     "incidents": {
       "maintenance_requests": 12,
       "resolved": 10,
       "pending": 2
     }
   }

Statistiques d'occupation
--------------------------

.. code-block:: bash

   GET /api/v1/board/occupancy-stats

.. code-block:: json

   {
     "total_units": 24,
     "occupied": 22,
     "vacant": 2,
     "owner_occupied": 18,
     "rented": 4,
     "occupancy_rate": 0.92
   }

Indicateurs de performance
---------------------------

.. code-block:: bash

   GET /api/v1/board/kpis

.. code-block:: json

   {
     "financial_health": {
       "cash_coverage_months": 2.8,
       "reserve_fund_ratio": 0.15,
       "debt_ratio": 0.05,
       "rating": "Good"
     },
     "operational": {
       "maintenance_response_time_avg": 2.5,
       "owner_satisfaction": 4.2,
       "contract_renewal_rate": 0.85
     },
     "compliance": {
       "insurance_up_to_date": true,
       "safety_inspections_current": true,
       "financial_reports_filed": true
     }
   }

🔐 Sécurité et audit
=====================

Accès aux logs
--------------

.. code-block:: bash

   GET /api/v1/audit-logs?organization_id={uuid}&role=BoardMember

Consultation des actions du syndic, modifications budgétaires, paiements.

Validation 2-factor
-------------------

Pour les actions sensibles (approbation > 5,000€), activation possible de la validation 2-factor.

.. code-block:: bash

   POST /api/v1/board/enable-2fa

Permissions
-----------

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Action
     - BoardMember
     - Syndic
   * - Validation dépenses < 5,000€
     - ✅
     - ❌
   * - Validation dépenses > 5,000€
     - ❌ (AG requise)
     - ❌
   * - Consultation finances
     - ✅
     - ✅
   * - Création contrats
     - ❌
     - ✅
   * - Évaluation prestataires
     - ✅
     - ✅
   * - Préparation AG
     - ✅
     - ✅
   * - Modification comptes bancaires
     - ❌
     - ✅

🛠️ Outils pratiques
=====================

Générateur de rapports
-----------------------

.. code-block:: bash

   POST /api/v1/board/generate-report?type=annual&year=2025

**Types** :

- ``annual`` : Rapport annuel complet
- ``quarterly`` : Rapport trimestriel
- ``financial`` : Rapport financier détaillé
- ``contracts`` : État des contrats
- ``performance`` : Indicateurs de performance

Export PDF pour AG
-------------------

.. code-block:: bash

   GET /api/v1/board/meeting/{meeting_id}/export-package

**Contenu** :

- Ordre du jour
- Rapports financiers
- Devis travaux
- Contrats à renouveler
- Formulaires procuration

Calculateur de quotes-parts
----------------------------

.. code-block:: bash

   GET /api/v1/board/calculate-shares?expense_amount=12000

.. code-block:: json

   {
     "total_expense": 12000.00,
     "distribution": [
       {
         "unit_number": "1A",
         "owner_name": "Jean Dupont",
         "ownership_percentage": 0.042,
         "share": 504.00
       },
       {
         "unit_number": "2B",
         "owner_name": "Marie Martin",
         "ownership_percentage": 0.055,
         "share": 660.00
       }
     ]
   }

📚 Ressources
==============

Guides complémentaires
-----------------------

- **Guide du syndic** : :doc:`syndic-guide`
- **Guide du copropriétaire** : :doc:`owner-guide`
- **Guide du comptable** : :doc:`accountant-guide`
- **GDPR Compliance** : :doc:`../GDPR_COMPLIANCE_CHECKLIST`

Références légales (Belgique)
------------------------------

- **Code civil belge** : Art. 577-3 à 577-14 (copropriété)
- **Conseil syndical** : Art. 577-8/4 (obligation > 20 lots)
- **AG** : Art. 577-6 (quorum, majorités)
- **Comptabilité** : AR 12/07/2012 (PCMN)

Support
-------

- **Email** : support@koprogo.com
- **Documentation** : https://docs.koprogo.com
- **Issues GitHub** : https://github.com/gilmry/koprogo/issues

---

**Version** : 1.0.0 | **Dernière mise à jour** : 10 novembre 2025
