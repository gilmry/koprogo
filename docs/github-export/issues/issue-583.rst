=======================================================================================
Issue #583: [Story 4.8] [cluster-coord] CommissaireAuxComptes + VerificationCertificate
=======================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust finance,legal-compliance governance,maury track-h-conformite,cluster-coord slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/583>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.8 — `CommissaireAuxComptes` + `VerificationCertificate` `[cluster-coord]`
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.8-commissaire-verification-cert` · Refs: #556 · Coord cluster #433 + #555
   
   ## Goal
   
   Entité `Commissaire` (Art. 3.88 CC) + use-case `sign_certificate(financial_period)` → `VerificationCertificate` signée eIDAS. Workflow PRE-clôture comptes annuels.
   
   ## Contexte Maury
   
   - **FR/INV** : FR20, FR25 ; INV-11, brief C10
   - **Effort** : M
   - **Deps** : Story 4.4
   - **ADR refs** : ADR-0014
   - **Cluster coord** : **`[cluster-coord]` #433 simultané** (montants PCMN clôture annuelle Decimal) ; **#555 simultané** sur accounting
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : Commissaire signe certificat période 2026 → VerificationCertificate persisté + comptes annuels passent en status Verified
   - **@edge** : Commissaire mandate_until expiré → 403
   - **@security** : Syndic ne peut pas signer à la place du Commissaire ; édit écriture après signature → 403
   - **@negative** : Tentative clôture comptes annuels sans VerificationCertificate → 422
   
   ## data-testid
   
   `commissaire-sign-cert-submit`, `verification-cert-status`, `annual-accounts-close-submit`
   
   ## Files
   
   - `backend/src/domain/entities/commissaire.rs` (NEW ou extension)
   - `backend/src/domain/entities/verification_certificate.rs` (NEW)
   - `backend/migrations/20260610_040000_create_verification_certificates.sql` + DOWN
   - `backend/src/application/use_cases/verification_certificate_use_cases.rs`
   - `backend/tests/features/commissaire_certificate.feature`
   
   ## Definition of Done
   
   - [ ] Entité Commissaire + VerificationCertificate
   - [ ] Use-case sign_certificate (Commissaire seulement, mandate actif)
   - [ ] Workflow clôture comptes annuels refuse sans cert
   - [ ] PR `[cluster-coord]` : #433 + #555 simultanés sur accounting touchés
   - [ ] BDD 4-cat VERT
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.8
   - Cluster Decimal : #433 · Cluster Result : #555 · Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

