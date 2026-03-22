=======================================================================
Issue #205: feat: Charge Distribution System (Répartition des Charges)
=======================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,phase:vps track:software,priority:critical finance,legal-compliance
:Assignees: Unassigned
:Created: 2026-02-18
:Updated: 2026-02-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/205>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## ✅ DÉJÀ IMPLÉMENTÉ — Issue créée rétroactivement (audit 2026-02-18)
   
   Répartition des charges entre copropriétaires selon les tantièmes/millièmes (Article 577-2 Code Civil belge).
   
   ### Backend
   - **Repository** : `charge_distribution_repository_impl.rs` (271 LOC)
   - **Use Cases** : `charge_distribution_use_cases.rs`
   - **Handlers** : `charge_distribution_handlers.rs` (92 LOC)
   - **DTOs** : `charge_distribution_dto.rs`
   
   ### Endpoints REST (4)
   - `POST /charge-distributions/calculate` — Calculer répartition
   - `GET /charge-distributions/:id` — Détail répartition
   - `GET /charge-distributions/owners/:id` — Répartitions d'un propriétaire
   - `GET /charge-distributions/owners/:id/total-due` — Total dû par propriétaire
   
   ### Migration
   - Table `charge_distributions` dans `20251104000001_enrich_expenses_invoice_workflow.sql`
   - Champs : `expense_id`, `unit_id`, `owner_id`, `quota_percentage`, `amount_due`
   
   ### Frontend
   - API : `charge-distributions.ts`
   - Intégré aux pages expenses et building detail
   
   *Issue créée par audit de synchronisation code↔issues — feature déjà implémentée*

.. raw:: html

   </div>

