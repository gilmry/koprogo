====================================
Ticket Management System (Issue #85)
====================================

Overview
========

KoproGo provides a complete maintenance ticket system for property managers
(syndics) and co-owners. Tickets track maintenance requests from creation
through resolution, with automatic due dates based on priority.

Belgian Context
===============

Belgian copropriété law (Article 577-8 §4 Code Civil) requires the syndic
to ensure proper maintenance of common areas. This system provides:

- **Traceability**: Full audit trail of maintenance requests
- **SLA compliance**: Automatic due dates by priority level
- **Contractor management**: Assignment and progress tracking

Workflow
========

::

    Open → Assigned → InProgress → Resolved → Closed
      ↓                              ↓
    Cancelled                      Reopened → Open

States
------

- **Open**: Ticket created by owner or syndic
- **Assigned**: Contractor designated
- **InProgress**: Work started
- **Resolved**: Work completed, pending validation
- **Closed**: Validated and archived
- **Cancelled**: Ticket no longer relevant

Priorities & SLA
=================

+----------+----------+-------------------------+
| Priority | Due Time | Use Case                |
+==========+==========+=========================+
| Critical | 1 hour   | Gas leak, flooding      |
+----------+----------+-------------------------+
| Urgent   | 4 hours  | Elevator stuck, no heat |
+----------+----------+-------------------------+
| High     | 24 hours | Broken lock, water leak |
+----------+----------+-------------------------+
| Medium   | 3 days   | Light replacement       |
+----------+----------+-------------------------+
| Low      | 7 days   | Cosmetic repair         |
+----------+----------+-------------------------+

Categories
==========

- Plumbing, Electrical, Heating, Cleaning, Security, General, Emergency

API Endpoints
=============

- ``POST /tickets`` - Create ticket
- ``GET /tickets/:id`` - Get ticket
- ``GET /buildings/:id/tickets`` - List building tickets
- ``GET /tickets/my`` - List requester's tickets
- ``GET /tickets/assigned`` - List contractor's tickets
- ``GET /tickets/status/:status`` - Filter by status
- ``PUT /tickets/:id/assign`` - Assign to contractor
- ``PUT /tickets/:id/start`` - Start work
- ``PUT /tickets/:id/resolve`` - Mark resolved
- ``PUT /tickets/:id/close`` - Close ticket
- ``PUT /tickets/:id/cancel`` - Cancel
- ``PUT /tickets/:id/reopen`` - Reopen
- ``GET /tickets/statistics`` - Statistics
- ``GET /tickets/overdue`` - Overdue tickets
- ``DELETE /tickets/:id`` - Delete

Architecture
============

- **Domain**: ``backend/src/domain/entities/ticket.rs`` (310 lines)
- **Use Cases**: ``backend/src/application/use_cases/ticket_use_cases.rs`` (18 methods)
- **Repository**: ``backend/src/infrastructure/database/repositories/ticket_repository_impl.rs``
- **Handlers**: ``backend/src/infrastructure/web/handlers/ticket_handlers.rs`` (17 endpoints)
- **Migration**: ``backend/migrations/20251117000000_create_tickets.sql``
- **BDD Tests**: ``backend/tests/features/tickets.feature`` (17 scenarios)

Audit Events
============

``TicketCreated``, ``TicketAssigned``, ``TicketStatusChanged``,
``TicketResolved``, ``TicketClosed``, ``TicketCancelled``,
``TicketReopened``, ``TicketDeleted``
