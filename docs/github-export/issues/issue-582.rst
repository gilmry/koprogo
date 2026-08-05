============================================================
Issue #582: [Story 4.7] CdC membre élu + action create_alert
============================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust legal-compliance,governance maury,track-h-conformite slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/582>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.7 — `CdC` membre élu + action `create_alert`
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.7-cdc-elected-alert` · Refs: #556
   
   ## Goal
   
   Entité `CdC` (conseil de copropriété — Art. 3.87 §1 CC) avec élection en AG + action `create_alert(text, severity, target=AG_next)`.
   
   ## Contexte Maury
   
   - **FR/INV** : FR19 ; INV-12, brief C13
   - **Effort** : S
   - **Deps** : Story 4.5
   - **ADR refs** : —
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Élection CdC en AG → 3 membres élus → CdC peut create_alert visible AG suivante
   - **@edge** : CdC membre démissionne → mandate_until = now() → perd droits
   - **@security** : Owner non-élu CdC tente create_alert → 403 ; après mandate_until → 403
   - **@negative** : Élection sans quorum AG → 422
   
   ## data-testid
   
   `cdc-elect-submit`, `cdc-member-list`, `cdc-alert-create-submit`
   
   ## Files
   
   - `backend/src/domain/entities/cdc.rs` (extension ou NEW selon existant)
   - `backend/src/application/use_cases/cdc_use_cases.rs`
   - `backend/tests/features/cdc_alert.feature`
   
   ## Definition of Done
   
   - [ ] Entité CdC avec membres élus + mandate_until
   - [ ] Use-case create_alert (CdC autorisé seulement)
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.7
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

