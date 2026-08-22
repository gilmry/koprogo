=============================================================================
Issue #589: [Story 5.5] Comptable (encodeur ET émetteur) 403 sur /community/*
=============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust security,community maury,track-h-conformite slice-5
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/589>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 5.5 — Comptable (encodeur ET émetteur) 403 sur `/community/*`
   
   > Maury Phase 6 Exécution · Slice 5 · Story `story/5.5-accountant-community-forbidden` · Refs: #556
   
   ## Goal
   
   Vérification middleware `community.read_access` exclut explicitement accountant.encodeur ET accountant.emetteur. CdC participe en tant qu'owner (cf. FR30).
   
   ## Contexte Maury
   
   - **FR/INV** : FR28, FR30 ; INV-6
   - **Effort** : S
   - **Deps** : Story 5.3
   - **ADR refs** : —
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Owner Marie accède `/community/sel` → OK ; CdC Catherine idem (rôle owner sous-jacent)
   - **@edge** : Comptable Paul (encodeur) cumule role owner → accède via role owner OK
   - **@security** : Comptable Pierre (émetteur) pur sans role owner → 403 INV-6
   - **@negative** : Accès direct URL bypass UI → 403 middleware
   
   ## data-testid
   
   `community-no-access-message` (visible si 403)
   
   ## Files
   
   - `backend/src/infrastructure/web/middleware/community_access_guard.rs` (NEW)
   - `backend/tests/features/community_accountant_forbidden.feature`
   
   ## Definition of Done
   
   - [ ] Middleware community_access_guard refus accountant pur
   - [ ] Exception : accountant + role owner → accès via owner OK
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §7 Story 5.5
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

