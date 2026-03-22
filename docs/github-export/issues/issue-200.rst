============================================================
Issue #200: feat: Double-Entry Bookkeeping (Journal Entries)
============================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,phase:vps track:software,priority:critical finance,legal-compliance
:Assignees: Unassigned
:Created: 2026-02-18
:Updated: 2026-02-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/200>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## ✅ DÉJÀ IMPLÉMENTÉ — Issue créée rétroactivement (audit 2026-02-18)
   
   Système de comptabilité en partie double conforme au PCMN belge.
   
   ### Backend
   - **Domain** : `journal_entry.rs` (entity)
   - **Repository** : `journal_entry_repository_impl.rs` (764 LOC)
   - **Use Cases** : `journal_entry_use_cases.rs`
   - **Handlers** : `journal_entry_handlers.rs` (454 LOC)
   - **DTOs** : `journal_entry_dto.rs` (dont `JournalEntryLineDto`)
   
   ### Endpoints REST
   - `POST /journal-entries` — Créer écriture comptable
   - `GET /journal-entries` — Lister écritures
   - `GET /journal-entries/:id` — Détail écriture
   - `DELETE /journal-entries/:id` — Supprimer écriture
   
   ### Migrations
   - `20251110140000_create_journal_entries_tables.sql` (journal_entries + journal_entry_lines)
   - `20251110140001_fix_journal_balance_trigger.sql` (validation débit=crédit)
   - `20251110220000_add_building_and_journal_type.sql`
   
   ### Validation métier
   - Trigger PostgreSQL `validate_journal_entry_balance()` : total débits = total crédits
   - FK composite vers `accounts(organization_id, code)` (PCMN)
   - Inspiré de Noalyss (GPL-2.0+)
   
   ### Frontend
   - Page : `/journal-entries`
   - Composant : `JournalEntryForm.svelte`
   
   *Issue créée par audit de synchronisation code↔issues — feature déjà implémentée*

.. raw:: html

   </div>

