=====================================================
Multi-Channel Notification System (Issue #86)
=====================================================

:Date: Mars 2026
:Version: 1.0.0
:Issue GitHub: #86
:Statut: Production-ready (Backend complet)

Vue d'ensemble
==============

Systeme de notifications multi-canal avec 22 types de notifications, preferences granulaires par utilisateur, et suivi de lecture.

Canaux de notification
----------------------

.. list-table::
   :header-rows: 1

   * - Canal
     - Usage
   * - Email
     - Canal principal, toutes notifications
   * - SMS
     - Notifications urgentes uniquement
   * - Push
     - Notifications mobiles
   * - InApp
     - Dashboard web (auto-sent a la creation)

Types de notifications (22)
---------------------------

MeetingReminder, PaymentDue, DocumentShared, TicketUpdate, SystemAlert, ExpenseApproved, ExpenseRejected, MaintenanceScheduled, VoteReminder, NewResolution, BoardDecision, BudgetApproved, ConvocationSent, QuoteReceived, ContractorAssigned, PaymentReceived, OwnershipTransfer, InsuranceExpiry, LegalNotice, EmergencyAlert, CommunityEvent, GeneralAnnouncement.

Architecture
============

.. code-block:: text

   Domain Layer
     |- Notification entity (title, message, type, channel, is_read)
     '- NotificationPreference entity (par type et canal)

   Application Layer
     |- NotificationRepository trait
     |- NotificationPreferenceRepository trait
     '- NotificationUseCases (13 methods)

   Infrastructure Layer
     |- PostgresNotificationRepository
     |- PostgresNotificationPreferenceRepository
     |- notification_handlers (11 endpoints)
     '- Migration: 20251117000001_create_notifications.sql

Comportement special
--------------------

Les notifications InApp sont automatiquement marquees comme "Sent" a la creation (pas de canal de livraison externe requis).

Endpoints API (11)
==================

Notifications
-------------

- ``POST /notifications`` - Creer une notification
- ``GET /notifications/:id`` - Obtenir une notification
- ``GET /notifications/my`` - Mes notifications
- ``GET /notifications/unread`` - Non lues
- ``PUT /notifications/:id/read`` - Marquer comme lue
- ``PUT /notifications/read-all`` - Tout marquer comme lu
- ``DELETE /notifications/:id`` - Supprimer
- ``GET /notifications/stats`` - Statistiques

Preferences
-----------

- ``GET /notification-preferences/:user_id`` - Preferences utilisateur
- ``GET /notification-preferences/:user_id/:type`` - Preference specifique
- ``PUT /notification-preferences/:user_id/:type`` - Modifier preference

Fichiers sources
================

- Domain: ``backend/src/domain/entities/notification.rs``, ``notification_preference.rs``
- Use Cases: ``backend/src/application/use_cases/notification_use_cases.rs``
- Repositories: ``backend/src/infrastructure/database/repositories/notification_repository_impl.rs``, ``notification_preference_repository_impl.rs``
- Handlers: ``backend/src/infrastructure/web/handlers/notification_handlers.rs``
- Migration: ``backend/migrations/20251117000001_create_notifications.sql``
