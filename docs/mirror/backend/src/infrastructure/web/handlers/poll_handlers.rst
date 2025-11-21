============================================
infrastructure/web/handlers/poll_handlers.rs
============================================

:Fichier: ``backend/src/infrastructure/web/handlers/poll_handlers.rs``
:Type: RUST
:Lignes de Code: 444
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **poll**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``ListPollsQuery``
- ``PollStatisticsResponse``

Fonctions
---------

- ``create_poll()``
- ``get_poll()``
- ``update_poll()``
- ``list_polls()``
- ``find_active_polls()``
- ``publish_poll()``
- ``close_poll()``
- ``cancel_poll()``
- ``delete_poll()``
- ``cast_poll_vote()``
- ``get_poll_results()``
- ``get_poll_building_statistics()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/poll_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

