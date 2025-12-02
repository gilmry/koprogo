=========================================
application/use_cases/ticket_use_cases.rs
=========================================

:Fichier: ``backend/src/application/use_cases/ticket_use_cases.rs``
:Type: RUST
:Lignes de Code: 287
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **ticket**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``TicketUseCases``
- ``TicketStatistics``

Fonctions
---------

- ``new()``
- ``create_ticket()``
- ``get_ticket()``
- ``list_tickets_by_building()``
- ``list_tickets_by_organization()``
- ``list_my_tickets()``
- ``list_assigned_tickets()``
- ``list_tickets_by_status()``
- ``assign_ticket()``
- ``start_work()``
- ``resolve_ticket()``
- ``close_ticket()``
- ``cancel_ticket()``
- ``reopen_ticket()``
- ``delete_ticket()``

*... et 2 autres fonctions*

Code Source
===========

Voir: ``backend/src/application/use_cases/ticket_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

