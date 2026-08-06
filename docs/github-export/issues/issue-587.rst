==================================================================================================
Issue #587: [Story 5.3] Syndic = community.moderator (RBAC Community SEL/Poll/Notice/SharedObject)
==================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust security,community maury,track-h-conformite slice-5
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/587>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 5.3 — Syndic = `community.moderator` (RBAC Community SEL/Poll/Notice/SharedObject)
   
   > Maury Phase 6 Exécution · Slice 5 · Story `story/5.3-syndic-moderator` · Refs: #556
   
   ## Goal
   
   Adapter use-cases Community pour refuser participation perso syndic (create/vote/comment/échange SEL) MAIS autoriser modération (edit/supprime). Pattern `Moderator` rôle.
   
   ## Contexte Maury
   
   - **FR/INV** : FR26 ; INV-4
   - **Effort** : M
   - **Deps** : Story 3.1, Story 5.1
   - **ADR refs** : —
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic moderator édite SEL litigieux → OK ; Owner participe → OK
   - **@edge** : Syndic cumule role owner (a un lot dans l'ACP) → peut participer ès qualités owner
   - **@security** : Syndic pur (sans role owner) tente create_sel_offer → 403 INV-4 ; vote poll → 403 ; comment notice → 403
   - **@negative** : Modération sans motif texte → 422 (audit requis)
   
   ## data-testid
   
   `sel-create-submit` (caché syndic non-owner), `sel-moderate-edit`, `sel-moderate-delete`
   
   ## Files
   
   - `backend/src/application/use_cases/sel_use_cases.rs` (refacto permission)
   - `backend/src/application/use_cases/poll_use_cases.rs` (idem)
   - `backend/src/application/use_cases/notice_use_cases.rs` (idem)
   - `backend/src/application/use_cases/shared_object_use_cases.rs` (idem)
   - `backend/tests/features/community_syndic_moderator.feature`
   
   ## Definition of Done
   
   - [ ] 4 use-cases Community refacto (sel/poll/notice/shared_object)
   - [ ] Syndic = Moderator (edit/delete) sans participation perso
   - [ ] Exception : syndic+role owner peut participer
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §7 Story 5.3
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

