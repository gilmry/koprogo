=======================================
application/use_cases/poll_use_cases.rs
=======================================

:Fichier: ``backend/src/application/use_cases/poll_use_cases.rs``
:Type: RUST
:Lignes de Code: 899
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **poll**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``PollUseCases``

Fonctions
---------

- ``new()``
- ``create_poll()``
- ``update_poll()``
- ``get_poll()``
- ``list_polls_paginated()``
- ``find_active_polls()``
- ``publish_poll()``
- ``close_poll()``
- ``cancel_poll()``
- ``delete_poll()``
- ``cast_vote()``
- ``get_poll_results()``
- ``get_building_statistics()``
- ``auto_close_expired_polls()``

Code Source
===========

Voir: ``backend/src/application/use_cases/poll_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

