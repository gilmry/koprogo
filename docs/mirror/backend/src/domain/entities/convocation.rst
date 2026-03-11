==============================
domain/entities/convocation.rs
==============================

:Fichier: ``backend/src/domain/entities/convocation.rs``
:Type: RUST
:Lignes de Code: 525
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine représentant une **convocation d'assemblée générale**. Validation automatique des délais légaux belges (15j ordinaire, 8j extraordinaire).

API Publique
============

Structures
----------

- ``Convocation``

Énumérations
------------

- ``ConvocationType``
- ``ConvocationStatus``

Fonctions
---------

- ``minimum_notice_days()``
- ``to_db_string()``
- ``from_db_string()``
- ``to_db_string()``
- ``from_db_string()``
- ``new()``
- ``schedule()``
- ``mark_sent()``
- ``cancel()``
- ``mark_reminder_sent()``
- ``update_tracking_counts()``
- ``respects_legal_deadline()``
- ``days_until_meeting()``
- ``should_send_reminder()``
- ``opening_rate()``

*... et 1 autres fonctions*

Code Source
===========

Voir: ``backend/src/domain/entities/convocation.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

