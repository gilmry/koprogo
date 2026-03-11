=============================================
application/use_cases/resolution_use_cases.rs
=============================================

:Fichier: ``backend/src/application/use_cases/resolution_use_cases.rs``
:Type: RUST
:Lignes de Code: 598
:Couche: Application (Use Cases)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Use Cases pour **resolution**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``ResolutionUseCases``
- ``VoteStatistics``

Fonctions
---------

- ``new()``
- ``create_resolution()``
- ``get_resolution()``
- ``get_meeting_resolutions()``
- ``get_resolutions_by_status()``
- ``update_resolution()``
- ``delete_resolution()``
- ``cast_vote()``
- ``change_vote()``
- ``get_resolution_votes()``
- ``get_owner_votes()``
- ``close_voting()``
- ``get_meeting_vote_summary()``
- ``has_unit_voted()``
- ``get_vote_statistics()``

Code Source
===========

Voir: ``backend/src/application/use_cases/resolution_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

