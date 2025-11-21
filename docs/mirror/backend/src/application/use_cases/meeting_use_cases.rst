==========================================
application/use_cases/meeting_use_cases.rs
==========================================

:Fichier: ``backend/src/application/use_cases/meeting_use_cases.rs``
:Type: RUST
:Lignes de Code: 172
:Couche: Application (Use Cases)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Use Cases pour **meeting**. Orchestration de la logique applicative en utilisant les ports et les entités de domaine.

API Publique
============

Structures
----------

- ``MeetingUseCases``

Fonctions
---------

- ``new()``
- ``create_meeting()``
- ``get_meeting()``
- ``list_meetings_by_building()``
- ``list_meetings_paginated()``
- ``update_meeting()``
- ``add_agenda_item()``
- ``complete_meeting()``
- ``cancel_meeting()``
- ``reschedule_meeting()``
- ``delete_meeting()``

Code Source
===========

Voir: ``backend/src/application/use_cases/meeting_use_cases.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

