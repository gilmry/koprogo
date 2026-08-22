================================================================================================
Issue #590: [Story 5.6] Activation/désactivation modules audité + archivage data (jamais delete)
================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust security,maury track-h-conformite,slice-5
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/590>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 5.6 — Activation/désactivation modules audité + archivage data
   
   > Maury Phase 6 Exécution · Slice 5 · Story `story/5.6-modules-archived-audited` · Refs: #556
   
   ## Goal
   
   Use-cases enable/disable avec audit immuable + archivage data (`archived_at` set, jamais DELETE). Re-activation = restauration. Vote AG requis pour Accounting/Governance.
   
   ## Contexte Maury
   
   - **FR/INV** : FR41, FR42 ; INV-26, INV-27
   - **Effort** : M
   - **Deps** : Story 5.1
   - **ADR refs** : ADR-0015
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Admin disable community ACP X → archived_at set → re-enable → data intacte + audit cycle
   - **@edge** : Désactivation module avec dépendance active (AG planifiée si Governance désactivé) → 422 + message clair
   - **@security** : Admin tente disable accounting sans vote AG ≥ 50% → 403 INV-26 ; tentative DELETE row → impossible (use-cases ne l'exposent pas)
   - **@negative** : Re-enable d'un module jamais activé → 404 `ModuleNeverEnabled`
   
   ## data-testid
   
   `module-audit-log-{{event_id}}`, `module-archived-banner`
   
   ## Files
   
   - `backend/src/application/use_cases/module_registry_use_cases.rs` (extension : audit + dependency check)
   - `backend/tests/features/module_lifecycle.feature`
   
   ## Definition of Done
   
   - [ ] Use-case disable archived_at set + audit_event + dependency_check
   - [ ] Use-case enable check vote_ag pour modules sensibles
   - [ ] Re-enable restaure data intacte
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §7 Story 5.6
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

