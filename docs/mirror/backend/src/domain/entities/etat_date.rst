============================
domain/entities/etat_date.rs
============================

:Fichier: ``backend/src/domain/entities/etat_date.rs``
:Type: RUST
:Lignes de Code: 620
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **etat date**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``EtatDate``

Énumérations
------------

- ``EtatDateStatus``
- ``EtatDateLanguage``

Fonctions
---------

- ``new()``
- ``mark_in_progress()``
- ``mark_generated()``
- ``mark_delivered()``
- ``is_expired()``
- ``is_overdue()``
- ``days_since_request()``
- ``update_financial_data()``
- ``update_additional_data()``

Code Source
===========

Voir: ``backend/src/domain/entities/etat_date.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

