=====================================================
Tickets & Maintenance Requests (Issue #85)
=====================================================

:Date: Mars 2026
:Version: 1.0.0
:Issue GitHub: #85
:Statut: Production-ready (Backend complet)

Vue d'ensemble
==============

Systeme de gestion des demandes de maintenance et interventions avec workflow complet, gestion des priorites avec SLA automatiques, et assignation de prestataires.

Workflow
--------

.. code-block:: text

   Open --> Assigned --> InProgress --> Resolved --> Closed
     |                                    |
     '--> Cancelled                       '--> Reopened --> Open

Priorites et SLA
-----------------

+----------+-------------+--------------------+
| Priorite | Delai SLA   | Due date auto      |
+==========+=============+====================+
| Critical | 1 heure     | Oui                |
| Urgent   | 4 heures    | Oui                |
| High     | 24 heures   | Oui                |
| Medium   | 3 jours     | Oui                |
| Low      | 7 jours     | Oui                |
+----------+-------------+--------------------+

Categories
----------

- Plumbing, Electrical, Heating, Cleaning, Security, General, Emergency

Architecture
============

.. code-block:: text

   Domain Layer
     '- Ticket entity (title, description, priority, status, category,
        due_date, assigned_contractor_id)

   Application Layer
     |- TicketRepository trait (18 methods)
     '- TicketUseCases (18 methods)

   Infrastructure Layer
     |- PostgresTicketRepository
     |- ticket_handlers (17 endpoints)
     '- Migration: 20251117000000_create_tickets.sql

Endpoints API (17)
==================

- ``POST /tickets`` - Creer un ticket
- ``GET /tickets/:id`` - Obtenir un ticket
- ``GET /buildings/:id/tickets`` - Tickets d'un immeuble
- ``GET /tickets/my`` - Mes tickets (demandeur)
- ``GET /tickets/assigned`` - Tickets assignes (prestataire)
- ``GET /tickets/status/:status`` - Par statut
- ``DELETE /tickets/:id`` - Supprimer
- ``PUT /tickets/:id/assign`` - Assigner a un prestataire
- ``PUT /tickets/:id/start`` - Demarrer le travail
- ``PUT /tickets/:id/resolve`` - Marquer resolu
- ``PUT /tickets/:id/close`` - Fermer
- ``PUT /tickets/:id/cancel`` - Annuler
- ``PUT /tickets/:id/reopen`` - Rouvrir
- ``GET /tickets/statistics`` - Statistiques
- ``GET /tickets/overdue`` - Tickets en retard

Fichiers sources
================

- Domain: ``backend/src/domain/entities/ticket.rs`` (310 lignes)
- Use Cases: ``backend/src/application/use_cases/ticket_use_cases.rs``
- Repository: ``backend/src/infrastructure/database/repositories/ticket_repository_impl.rs``
- Handlers: ``backend/src/infrastructure/web/handlers/ticket_handlers.rs``
- Migration: ``backend/migrations/20251117000000_create_tickets.sql``

Audit Events
============

TicketCreated, TicketAssigned, TicketStatusChanged, TicketResolved, TicketClosed, TicketCancelled, TicketReopened, TicketDeleted
