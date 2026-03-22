=========================================================
Issue #201: feat: Call for Funds System (Appels de Fonds)
=========================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,phase:vps track:software,priority:critical finance,legal-compliance
:Assignees: Unassigned
:Created: 2026-02-18
:Updated: 2026-02-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/201>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## ✅ DÉJÀ IMPLÉMENTÉ — Issue créée rétroactivement (audit 2026-02-18)
   
   Système d'appels de fonds pour la copropriété (charges trimestrielles, fonds de réserve).
   
   ### Backend
   - **Repository** : `call_for_funds_repository_impl.rs` (259 LOC)
   - **Use Cases** : `call_for_funds_use_cases.rs`
   - **Handlers** : `call_for_funds_handlers.rs` (215 LOC)
   - **DTOs** : `call_for_funds_dto.rs`
   
   ### Endpoints REST (7)
   - `POST /call-for-funds` — Créer appel de fonds
   - `GET /call-for-funds/:id` — Détail
   - `GET /call-for-funds` — Lister tous
   - `GET /call-for-funds/overdue` — Appels en retard
   - `POST /call-for-funds/:id/send` — Envoyer aux propriétaires
   - `PUT /call-for-funds/:id/cancel` — Annuler
   - `DELETE /call-for-funds/:id` — Supprimer
   
   ### Migration
   - `20251111015338_create_call_for_funds.sql`
   
   ### Frontend
   - Page : `/call-for-funds`
   - Composants : `CallForFundsForm.svelte`, `CallForFundsList.svelte`
   - API : wrapper dans `api.ts`
   
   *Issue créée par audit de synchronisation code↔issues — feature déjà implémentée*

.. raw:: html

   </div>

