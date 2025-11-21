================================
domain/entities/journal_entry.rs
================================

:Fichier: ``backend/src/domain/entities/journal_entry.rs``
:Type: RUST
:Lignes de Code: 453
:Couche: Domain (Métier)
:Tests: ✅ Oui

À quoi sert ce fichier ?
========================

Entité de domaine **journal entry**. Contient la logique métier pure avec validation des invariants métier dans le constructeur.

API Publique
============

Structures
----------

- ``JournalEntry``
- ``JournalEntryLine``

Fonctions
---------

- ``new()``
- ``total_debits()``
- ``total_credits()``
- ``is_balanced()``
- ``new_debit()``
- ``new_credit()``
- ``amount()``
- ``is_debit()``
- ``is_credit()``

Code Source
===========

Voir: ``backend/src/domain/entities/journal_entry.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

