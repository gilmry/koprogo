=========================
domain/entities/ticket.rs
=========================

:Fichier: ``backend/src/domain/entities/ticket.rs``
:Type: RUST
:Lignes de Code: 442
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **demande d'intervention** (maintenance, dépannage). Workflow de gestion avec priorités et deadlines automatiques selon criticité.

API Publique
============

Structures
----------

- ``Ticket``

Énumérations
------------

- ``TicketCategory``
- ``TicketPriority``
- ``TicketStatus``

Fonctions
---------

- ``new()``
- ``assign()``
- ``start_work()``
- ``resolve()``
- ``close()``
- ``cancel()``
- ``reopen()``
- ``is_overdue()``

Code Source
===========

Voir: ``backend/src/domain/entities/ticket.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

