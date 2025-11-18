========================================
domain/entities/convocation_recipient.rs
========================================

:Fichier: ``backend/src/domain/entities/convocation_recipient.rs``
:Type: RUST
:Lignes de Code: 404
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **convocation recipient**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``ConvocationRecipient``

Énumérations
------------

- ``AttendanceStatus``

Fonctions
---------

- ``to_db_string()``
- ``from_db_string()``
- ``new()``
- ``mark_email_sent()``
- ``mark_email_failed()``
- ``mark_email_opened()``
- ``mark_reminder_sent()``
- ``mark_reminder_opened()``
- ``update_attendance_status()``
- ``set_proxy()``
- ``remove_proxy()``
- ``has_opened_email()``
- ``has_opened_reminder()``
- ``needs_reminder()``
- ``has_confirmed_attendance()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/domain/entities/convocation_recipient.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

