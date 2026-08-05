=======================================================================================================================
Issue #567: [Story 3.1] [cluster-coord] Sous-rôles métier (accountant.encodeur/emetteur + community.moderator + autres)
=======================================================================================================================

:State: **CLOSED**
:Milestone: No milestone
:Labels: track:software,rust security,maury track-h-conformite,cluster-coord slice-3
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-07-26
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/567>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 3.1 — Sous-rôles métier (accountant.encodeur/emetteur + community.moderator + autres) `[cluster-coord]`
   
   > Maury Phase 6 Exécution · Slice 3 · Story `story/3.1-subroles-metier` · Refs: #556 · Coord cluster #433 + #555
   
   ## Goal
   
   Extension `UserRoleAssignment.role` enum : `accountant.encodeur`, `accountant.emetteur`, `community.moderator`, `lawyer`, `notary`, `amo`, `architect`, `bet`, `warden`. Refacto permission checks distinguant Encodeur vs Émetteur (FR21).
   
   ## Contexte Maury
   
   - **FR/INV** : FR5, FR21 ; INV-4, INV-10
   - **Effort** : M
   - **Deps** : Story 1.1
   - **ADR refs** : —
   - **Cluster coord** : **`[cluster-coord]` #433 simultané** sur expense/call_for_funds (Decimal monétaire) ; **#555 simultané** si use-cases touchés ont des `Result<_, String>` legacy
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Accountant.encodeur peut créer Invoice ; Accountant.emetteur peut créer Expense + CallForFunds
   - **@edge** : User cumule encodeur ET émetteur → tous droits réunis (union)
   - **@security** : Accountant.encodeur tente POST `/expenses` → 403 INV-10 ; encodeur tente POST `/call-for-funds` → 403
   - **@negative** : Assignment avec role inconnu → 422 ; clean-up role string trim+lowercase
   
   ## data-testid — (RBAC backend, pas UI direct)
   
   ## Files
   
   - `backend/migrations/20260615_010000_split_accountant_roles.sql` (seed + enum Rust)
   - `backend/src/domain/value_objects/role.rs` (extension enum)
   - `backend/src/application/use_cases/invoice_use_cases.rs` (NEW — Encodeur)
   - `backend/src/application/use_cases/expense_use_cases.rs` (refacto permission + Decimal + AppError)
   - `backend/src/application/use_cases/call_for_funds_use_cases.rs` (refacto idem)
   - `backend/tests/features/accountant_subroles.feature`
   
   ## Definition of Done
   
   - [ ] Role enum étendu (9 nouveaux variants)
   - [ ] Invoice use-case NEW (Encodeur)
   - [ ] Expense + CallForFunds refacto permission + **migration Decimal simultanée** + **AppError simultané**
   - [ ] BDD 4-cat VERT
   - [ ] PR `[cluster-coord]` étiquetée
   - [ ] Caractérisation FE (story 0.1) reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §5 Story 3.1
   - Cluster Decimal : #433 · Cluster Result : #555 · Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

