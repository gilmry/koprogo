============================================================================
Issue #571: [Story 3.5] Délégation temporaire UserRoleAssignment.valid_until
============================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust security,maury track-h-conformite,slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/571>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.5 — Délégation temporaire `UserRoleAssignment.valid_until`
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.5-role-delegation-temp` · Refs: #556
   
   ## Goal
   
   Extension table `user_role_assignments` avec `valid_until` (NULLABLE = permanent) + `delegated_from_user_id` (NULLABLE). Use-case `delegate_role` + audit.
   
   ## Contexte Maury
   
   - **FR/INV** : FR8 ; INV-8
   - **Effort** : S
   - **Deps** : Story 3.1
   - **ADR refs** : —
   - **Cluster coord** : —
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Syndic délègue role syndic à Owner Pierre pour 7 jours → Pierre voit menus syndic dans cette fenêtre → rôle expiré auto
   - **@edge** : Délégation juste à `valid_until` → action OK ; +1ms → 403
   - **@security** : Owner Pierre ne peut pas re-déléguer à un tiers (la délégation est non-transitive)
   - **@negative** : Délégation avec `valid_until < now()` → 422
   
   ## data-testid
   
   `role-delegate-submit`, `role-delegate-target-input`, `role-delegate-until-input`
   
   ## Files
   
   - `backend/migrations/20260605_030000_extend_user_role_assignments.sql` + DOWN
   - `backend/src/domain/entities/user_role_assignment.rs` (refacto)
   - `backend/src/application/use_cases/role_delegation_use_cases.rs` (NEW)
   - `backend/tests/features/role_delegation.feature`
   
   ## Definition of Done
   
   - [ ] Migration valid_until + delegated_from_user_id (NULLABLE) + DOWN
   - [ ] Use-case delegate_role avec validation valid_until > now() + non-transitivité
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.5
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

