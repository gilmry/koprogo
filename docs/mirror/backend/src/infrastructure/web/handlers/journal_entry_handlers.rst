=====================================================
infrastructure/web/handlers/journal_entry_handlers.rs
=====================================================

:Fichier: ``backend/src/infrastructure/web/handlers/journal_entry_handlers.rs``
:Type: RUST
:Lignes de Code: 455
:Couche: Infrastructure (Adaptateurs)
:Tests: ❌ Non

À quoi sert ce fichier ?
========================

Handlers HTTP (Actix-web) pour **journal entry**. Gestion des requêtes REST API avec authentification, validation et gestion d'erreurs.

API Publique
============

Structures
----------

- ``CreateJournalEntryRequest``
- ``JournalEntryLineRequest``
- ``JournalEntryResponse``
- ``JournalEntryLineResponse``
- ``JournalEntryWithLinesResponse``
- ``ListJournalEntriesQuery``

Fonctions
---------

- ``create_journal_entry()``
- ``list_journal_entries()``
- ``get_journal_entry()``
- ``delete_journal_entry()``

Code Source
===========

Voir: ``backend/src/infrastructure/web/handlers/journal_entry_handlers.rs``

Documentation Connexe
=====================

.. seealso::

   - :doc:`/CLAUDE`
   - :doc:`/ARCHITECTURE`

