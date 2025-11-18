==========================
domain/entities/meeting.rs
==========================

:Fichier: ``backend/src/domain/entities/meeting.rs``
:Type: RUST
:Lignes de Code: 266
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **assemblée générale**. Gestion agenda, procès-verbaux et résolutions votées.

API Publique
============

Structures
----------

- ``Meeting``

Énumérations
------------

- ``MeetingType``
- ``MeetingStatus``

Fonctions
---------

- ``new()``
- ``add_agenda_item()``
- ``complete()``
- ``cancel()``
- ``reschedule()``
- ``is_upcoming()``

Code Source
===========

Voir: ``backend/src/domain/entities/meeting.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

