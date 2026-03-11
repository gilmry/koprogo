==============================================
infrastructure/web/handlers/ticket_handlers.rs
==============================================

:Fichier: ``backend/src/infrastructure/web/handlers/ticket_handlers.rs``
:Type: RUST
:Lignes de Code: 521
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **ticket**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``OverdueQuery``

Fonctions
---------

- ``create_ticket()``
- ``get_ticket()``
- ``list_building_tickets()``
- ``list_organization_tickets()``
- ``list_my_tickets()``
- ``list_assigned_tickets()``
- ``list_tickets_by_status()``
- ``delete_ticket()``
- ``assign_ticket()``
- ``start_work()``
- ``resolve_ticket()``
- ``close_ticket()``
- ``cancel_ticket()``
- ``reopen_ticket()``
- ``get_ticket_statistics()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/ticket_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

