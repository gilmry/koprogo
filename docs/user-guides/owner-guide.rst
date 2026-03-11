============================================
Guide Utilisateur : Copropriétaire
============================================

:Version: 1.0.0
:Date: 10 novembre 2025
:Public: Copropriétaires
:Voir aussi: :doc:`syndic-guide` | :doc:`accountant-guide` | :doc:`board-member-guide`

📋 Vue d'ensemble
=================

Ce guide explique comment utiliser **KoproGo** en tant que **copropriétaire**. Vous y trouverez :

- ✅ Accès à vos informations personnelles
- ✅ Consultation de vos lots et quotes-parts
- ✅ Suivi des charges et paiements
- ✅ Accès aux documents de copropriété
- ✅ Consultation des assemblées générales
- ✅ Droits GDPR (accès, rectification, effacement)

🚀 Premiers pas
================

Connexion à votre espace
-------------------------

1. **Connexion** : ``POST /api/v1/auth/login``

   .. code-block:: json

      {
        "email": "votre.email@example.com",
        "password": "votre_mot_de_passe"
      }

2. **Vérification de votre profil** : ``GET /api/v1/auth/me``

   .. code-block:: json

      {
        "id": "owner-uuid",
        "email": "votre.email@example.com",
        "name": "Jean Dupont",
        "roles": [
          {
            "role_type": "Owner",
            "organization_id": "residence-les-erables-uuid"
          }
        ],
        "active_role": "Owner"
      }

Mot de passe oublié
-------------------

Contactez votre syndic qui peut réinitialiser votre mot de passe via l'interface d'administration.

🏠 Mes lots et quotes-parts
============================

Consulter mes lots
------------------

**Endpoint** : ``GET /api/v1/owners/{owner_id}/units``

**Réponse** :

.. code-block:: json

   [
     {
       "unit_id": "unit-uuid-1",
       "unit_number": "2B",
       "building_name": "Résidence Les Érables",
       "floor": 2,
       "area": 85.5,
       "ownership_percentage": 1.0,
       "is_primary_contact": true,
       "start_date": "2020-06-15T00:00:00Z",
       "end_date": null
     },
     {
       "unit_id": "unit-uuid-2",
       "unit_number": "4C",
       "building_name": "Résidence Les Érables",
       "floor": 4,
       "area": 120.0,
       "ownership_percentage": 0.50,
       "is_primary_contact": false,
       "start_date": "2022-03-01T00:00:00Z",
       "end_date": null
     }
   ]

**Interprétation** :

- **Lot 2B** : Vous êtes propriétaire unique (100%) et contact principal
- **Lot 4C** : Vous détenez 50% (copropriété avec une autre personne)

Historique de propriété
------------------------

**Endpoint** : ``GET /api/v1/owners/{owner_id}/units/history``

Affiche tous vos lots, y compris ceux que vous avez vendus (``end_date`` renseignée).

.. code-block:: json

   [
     {
       "unit_number": "1A",
       "building_name": "Résidence Les Érables",
       "ownership_percentage": 1.0,
       "start_date": "2018-01-10T00:00:00Z",
       "end_date": "2020-05-30T00:00:00Z"
     }
   ]

Copropriétaires du même lot
----------------------------

Si vous partagez un lot, consultez les autres copropriétaires :

**Endpoint** : ``GET /api/v1/units/{unit_id}/owners``

.. code-block:: json

   [
     {
       "owner_id": "vous-uuid",
       "owner_name": "Jean Dupont",
       "ownership_percentage": 0.50,
       "is_primary_contact": false
     },
     {
       "owner_id": "autre-uuid",
       "owner_name": "Marie Martin",
       "ownership_percentage": 0.50,
       "is_primary_contact": true
     }
   ]

Le **contact principal** reçoit les communications officielles du syndic.

💰 Mes charges et paiements
============================

Consulter mes charges
---------------------

**Endpoint** : ``GET /api/v1/expenses?building_id={uuid}&status=Approved``

.. code-block:: json

   [
     {
       "id": "expense-uuid-1",
       "description": "Charges T4 2025 - Lot 2B",
       "amount": 450.00,
       "expense_date": "2025-10-01",
       "category": "QuarterlyCharges",
       "status": "Approved",
       "due_date": "2025-10-31",
       "paid": false
     },
     {
       "id": "expense-uuid-2",
       "description": "Travaux ascenseur - Quote-part",
       "amount": 187.50,
       "expense_date": "2025-11-05",
       "category": "Maintenance",
       "status": "Approved",
       "due_date": "2025-11-30",
       "paid": true
     }
   ]

Calculer ma quote-part
----------------------

Pour les dépenses communes, votre quote-part est calculée selon votre pourcentage de propriété :

**Exemple** :

- Réparation toiture : **12,000€**
- Votre quote-part lot 2B (100%) : **500€** (12,000 × 0.042, si 24 lots)
- Votre quote-part lot 4C (50%) : **125€** (12,000 × 0.042 × 0.50)
- **Total à votre charge** : **625€**

Consulter mes paiements
------------------------

**Endpoint** : ``GET /api/v1/expenses?owner_id={owner_id}&paid=true``

Affiche l'historique de vos paiements effectués.

Relances de paiement
---------------------

Si vous avez des impayés, vous recevrez des relances progressives :

.. list-table::
   :header-rows: 1
   :widths: 20 20 60

   * - Niveau
     - Délai
     - Description
   * - **Gentle**
     - J+15
     - Rappel courtois par email
   * - **Formal**
     - J+30
     - Mise en demeure formelle (lettre recommandée)
   * - **FinalNotice**
     - J+45
     - Dernier avertissement + pénalités de retard (8% annuel)
   * - **LegalAction**
     - J+60
     - Procédure judiciaire engagée

**Consulter vos relances** :

.. code-block:: bash

   GET /api/v1/owners/{owner_id}/payment-reminders

**Éviter les pénalités** : Payez avant la date d'échéance ou contactez votre syndic pour un échéancier.

Détail des lignes de facturation
---------------------------------

**Endpoint** : ``GET /api/v1/expenses/{expense_id}``

.. code-block:: json

   {
     "id": "expense-uuid",
     "description": "Charges T4 2025 - Lot 2B",
     "amount": 450.00,
     "line_items": [
       {
         "description": "Eau",
         "quantity": 1.0,
         "unit_price": 80.00,
         "vat_rate": 0.06,
         "total_excl_vat": 80.00,
         "vat_amount": 4.80,
         "total_incl_vat": 84.80
       },
       {
         "description": "Chauffage",
         "quantity": 1.0,
         "unit_price": 150.00,
         "vat_rate": 0.21,
         "total_excl_vat": 150.00,
         "vat_amount": 31.50,
         "total_incl_vat": 181.50
       },
       {
         "description": "Entretien communs",
         "quantity": 1.0,
         "unit_price": 120.00,
         "vat_rate": 0.21,
         "total_excl_vat": 120.00,
         "vat_amount": 25.20,
         "total_incl_vat": 145.20
       }
     ],
     "total_excl_vat": 350.00,
     "total_vat": 61.50,
     "total_incl_vat": 411.50
   }

📄 Documents de copropriété
=============================

Consulter les documents
-----------------------

**Endpoint** : ``GET /api/v1/documents?building_id={uuid}``

.. code-block:: json

   [
     {
       "id": "doc-uuid-1",
       "title": "Convocation AG - 15 décembre 2025",
       "document_type": "MeetingInvitation",
       "file_path": "/uploads/convocation-ag-2025-12.pdf",
       "created_at": "2025-11-01T10:00:00Z"
     },
     {
       "id": "doc-uuid-2",
       "title": "Procès-verbal AG - 15 juin 2025",
       "document_type": "MeetingMinutes",
       "file_path": "/uploads/pv-ag-2025-06.pdf",
       "created_at": "2025-06-20T14:30:00Z"
     },
     {
       "id": "doc-uuid-3",
       "title": "Contrat assurance immeuble 2025",
       "document_type": "InsurancePolicy",
       "file_path": "/uploads/assurance-2025.pdf",
       "created_at": "2025-01-15T09:00:00Z"
     }
   ]

Télécharger un document
------------------------

.. code-block:: bash

   GET /api/v1/documents/{document_id}/download

Types de documents disponibles
-------------------------------

- **MeetingInvitation** : Convocations aux assemblées générales
- **MeetingMinutes** : Procès-verbaux d'AG
- **Invoice** : Factures et décomptes de charges
- **Contract** : Contrats (assurance, maintenance, etc.)
- **MaintenanceReport** : Rapports de maintenance
- **FinancialReport** : Rapports financiers annuels
- **InsurancePolicy** : Polices d'assurance
- **Other** : Autres documents

📅 Assemblées générales
=========================

Consulter les prochaines AG
----------------------------

**Endpoint** : ``GET /api/v1/meetings?building_id={uuid}&status=Scheduled``

.. code-block:: json

   [
     {
       "id": "meeting-uuid",
       "meeting_type": "GeneralAssembly",
       "scheduled_date": "2025-12-15T18:00:00Z",
       "location": "Salle communautaire - Rez-de-chaussée",
       "agenda": "1. Approbation des comptes 2025\n2. Budget prévisionnel 2026\n3. Travaux de toiture\n4. Questions diverses",
       "status": "Scheduled"
     }
   ]

Consulter les procès-verbaux
-----------------------------

**Endpoint** : ``GET /api/v1/meetings?building_id={uuid}&status=Completed``

.. code-block:: json

   [
     {
       "id": "meeting-uuid-past",
       "meeting_type": "GeneralAssembly",
       "scheduled_date": "2025-06-15T18:00:00Z",
       "status": "Completed",
       "minutes": "## Procès-verbal AG du 15/06/2025\n\n**Présents** : 20/24 copropriétaires\n**Quorum** : Atteint (83%)\n\n### 1. Approbation des comptes\nComptes 2024 approuvés à l'unanimité..."
     }
   ]

Soumettre des questions
------------------------

Pour ajouter un point à l'ordre du jour ou poser une question, contactez votre syndic par email ou téléphone. Les questions diverses peuvent être abordées en fin d'assemblée.

Pouvoir de représentation
--------------------------

Si vous ne pouvez pas assister à une AG, vous pouvez donner pouvoir à un autre copropriétaire. Contactez votre syndic pour obtenir le formulaire de procuration.

👤 Mes informations personnelles
==================================

Consulter mon profil
---------------------

**Endpoint** : ``GET /api/v1/owners/{owner_id}``

.. code-block:: json

   {
     "id": "owner-uuid",
     "name": "Jean Dupont",
     "email": "jean.dupont@example.com",
     "phone": "+32 2 123 45 67",
     "address": "123 Avenue de Tervuren, 1040 Bruxelles",
     "created_at": "2020-06-15T10:00:00Z",
     "updated_at": "2025-11-10T14:30:00Z"
   }

Modifier mes coordonnées
-------------------------

Pour modifier votre email, téléphone ou adresse, contactez votre syndic qui effectuera la mise à jour.

**Sécurité** : Seul le syndic peut modifier vos informations pour éviter les usurpations d'identité.

🔐 Droits GDPR
===============

KoproGo respecte le **Règlement Général sur la Protection des Données** (RGPD/GDPR).

Droit d'accès (Art. 15)
------------------------

**Télécharger toutes vos données** :

.. code-block:: bash

   GET /api/v1/gdpr/owners/{owner_id}/data-export

**Réponse** : Fichier JSON contenant :

- Vos informations personnelles
- Vos lots et quotes-parts
- Historique de propriété
- Charges et paiements
- Documents associés
- Participation aux AG

Droit de rectification (Art. 16)
---------------------------------

Pour corriger des informations erronées, contactez votre syndic avec les justificatifs nécessaires.

Droit à l'effacement (Art. 17)
-------------------------------

**Demander la suppression de vos données** :

.. code-block:: bash

   DELETE /api/v1/gdpr/owners/{owner_id}/delete-data

**Conditions** :

- ✅ Aucune dette en cours
- ✅ Aucun litige en cours
- ✅ Aucun lot détenu actuellement

Si vous êtes toujours copropriétaire, vos données ne peuvent pas être supprimées (obligation légale de conservation comptable).

Droit à la portabilité (Art. 20)
---------------------------------

**Récupérer vos données dans un format structuré** :

.. code-block:: bash

   GET /api/v1/gdpr/owners/{owner_id}/portable-data

**Format** : JSON structuré compatible pour import dans un autre système.

Droit d'opposition (Art. 21)
-----------------------------

Vous pouvez vous opposer au traitement de vos données à des fins de marketing. Contactez votre syndic pour exercer ce droit.

**Note** : Le traitement des données nécessaires à la gestion de la copropriété (charges, AG, etc.) ne peut pas faire l'objet d'opposition (obligation légale).

Audit des accès
----------------

**Consulter qui a accédé à vos données** :

.. code-block:: bash

   GET /api/v1/audit-logs?entity_type=Owner&entity_id={owner_id}

.. code-block:: json

   [
     {
       "timestamp": "2025-11-10T10:30:00Z",
       "user": "syndic@example.com",
       "action": "READ",
       "entity_type": "Owner",
       "entity_id": "owner-uuid",
       "ip_address": "192.168.1.100"
     }
   ]

📊 Mes statistiques
====================

Résumé financier
----------------

**Endpoint** : ``GET /api/v1/owners/{owner_id}/financial-summary``

.. code-block:: json

   {
     "total_paid": 5400.00,
     "total_pending": 450.00,
     "total_overdue": 0.00,
     "average_quarterly_charges": 450.00,
     "year": 2025
   }

Historique annuel
-----------------

.. code-block:: bash

   GET /api/v1/owners/{owner_id}/annual-charges?year=2025

.. code-block:: json

   {
     "year": 2025,
     "quarters": [
       {
         "quarter": "Q1",
         "amount": 450.00,
         "paid": true,
         "paid_date": "2025-03-25T00:00:00Z"
       },
       {
         "quarter": "Q2",
         "amount": 450.00,
         "paid": true,
         "paid_date": "2025-06-28T00:00:00Z"
       },
       {
         "quarter": "Q3",
         "amount": 450.00,
         "paid": true,
         "paid_date": "2025-09-30T00:00:00Z"
       },
       {
         "quarter": "Q4",
         "amount": 450.00,
         "paid": false,
         "due_date": "2025-12-31T00:00:00Z"
       }
     ],
     "total": 1800.00,
     "total_paid": 1350.00,
     "total_pending": 450.00
   }

📱 Interface web (frontend)
=============================

KoproGo propose une interface web conviviale accessible via navigateur :

**URL** : https://app.koprogo.com

Fonctionnalités disponibles
----------------------------

- 📊 **Tableau de bord** : Vue d'ensemble de vos lots, charges et documents
- 💰 **Mes paiements** : Historique et charges en attente
- 📄 **Documents** : Accès à tous les documents de copropriété
- 📅 **Assemblées** : Convocations et procès-verbaux
- 👤 **Mon profil** : Informations personnelles et préférences
- 🔔 **Notifications** : Alertes pour nouvelles charges, AG, relances

Multi-langue
------------

L'interface est disponible en :

- 🇫🇷 Français
- 🇳🇱 Néerlandais
- 🇬🇧 Anglais

Changez la langue dans **Profil → Préférences → Langue**.

🛠️ Dépannage
==============

Je ne peux pas me connecter
----------------------------

**Vérifications** :

1. Email correct (vérifiez les fautes de frappe)
2. Mot de passe correct (sensible à la casse)
3. Compte activé par le syndic

**Solution** : Contactez votre syndic pour réinitialiser votre mot de passe.

Je ne vois pas mes charges
---------------------------

**Causes possibles** :

- Aucune charge approuvée pour la période sélectionnée
- Filtre actif sur les charges payées uniquement

**Solution** : Vérifiez les filtres ou contactez le syndic.

Ma quote-part semble incorrecte
--------------------------------

**Vérification** :

.. code-block:: bash

   GET /api/v1/units/{unit_id}/owners

Si vous détenez 50% d'un lot, votre quote-part des charges communes sera calculée sur cette base.

**Solution** : Si l'erreur persiste, contactez le syndic avec les justificatifs.

Je n'ai pas reçu la convocation à l'AG
---------------------------------------

**Vérifications** :

1. Email correct dans votre profil
2. Vérifiez vos spams/courriers indésirables
3. Consultez l'AG dans l'interface web

**Solution** : Contactez le syndic pour renvoyer la convocation.

📞 Support et contact
======================

Contact syndic
--------------

Votre syndic est votre interlocuteur principal pour :

- Modifications de coordonnées
- Questions sur les charges
- Problèmes de paiement
- Demandes de documents
- Ajout de points à l'ordre du jour des AG

Support technique KoproGo
--------------------------

Pour les problèmes techniques (connexion, bugs, suggestions) :

- **Email** : support@koprogo.com
- **Documentation** : https://docs.koprogo.com
- **FAQ** : https://koprogo.com/faq

Délégué à la protection des données (DPO)
------------------------------------------

Pour toute question GDPR :

- **Email** : dpo@koprogo.com
- **Délai de réponse** : 30 jours maximum

📚 Ressources utiles
=====================

- **Guide du syndic** : :doc:`syndic-guide`
- **Guide du comptable** : :doc:`accountant-guide`
- **Guide du conseil syndical** : :doc:`board-member-guide`
- **Documentation GDPR** : :doc:`../GDPR_COMPLIANCE_CHECKLIST`
- **Plan comptable PCMN** : :doc:`../BELGIAN_ACCOUNTING_PCMN`

---

**Version** : 1.0.0 | **Dernière mise à jour** : 10 novembre 2025
