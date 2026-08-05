====================================================================================================
Issue #579: [Story 4.4] Adapter ElectronicSignatureProvider (port + 3 adapters eID/itsme/Universign)
====================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:software,rust security,legal-compliance maury,track-h-conformite slice-4
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/579>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Story 4.4 — Adapter `ElectronicSignatureProvider` (port + 3 adapters eID/itsme/Universign)
   
   > Maury Phase 6 Exécution · Slice 4 · Story `story/4.4-signature-provider-port` · Refs: #556
   
   ## Goal
   
   Port hexagonal `ElectronicSignatureProvider` + 3 adapters : eID belge (FAS), itsme, Universign. Sélection par ACP (préférence cabinet) avec fallback Universign pour non-BE.
   
   ## Contexte Maury
   
   - **FR/INV** : FR16, FR20, FR25 ; ADR-0014
   - **Effort** : L
   - **Deps** : Story 1.1
   - **ADR refs** : **ADR-0014** (signature électronique eIDAS)
   - **Cluster coord** : NEW → AppError natif
   
   ## Acceptance Criteria (4 catégories)
   
   - **@happy** : eID belge demande signature → reçoit `QualifiedSignature` + persiste audit ; itsme idem ; Universign idem (mock en dev)
   - **@edge** : Préférence cabinet = itsme → fallback Universign si user non-BE
   - **@security** : Hash document calculé avant envoi prestataire (HMAC SHA-256) ; vérification à réception
   - **@negative** : Prestataire timeout → retry exponential backoff 3× → erreur typée + audit
   
   ## data-testid — (backend pur, UI dans 4.3)
   
   ## Files
   
   - `backend/src/application/ports/electronic_signature_provider.rs`
   - `backend/src/infrastructure/external/signature_provider_eid.rs`
   - `backend/src/infrastructure/external/signature_provider_itsme.rs`
   - `backend/src/infrastructure/external/signature_provider_universign.rs`
   - `backend/tests/integration/signature_providers_test.rs` (mocks)
   
   ## Definition of Done
   
   - [ ] Port trait async (request_signature, fetch_signature)
   - [ ] 3 adapters (eID/itsme/Universign) avec abstraction commune
   - [ ] Sélection par ACP préférence + fallback non-BE
   - [ ] Hash document HMAC SHA-256 + vérification
   - [ ] Retry exponential backoff 3×
   - [ ] Tests integration mocks VERTS
   - [ ] Caractérisation FE reste VERTE
   - [ ] PR mergeable sur `feature/dev`
   
   ## Liens
   
   - Source de vérité : [`docs/maury/refonte-ux-multi-role-acp/stories.md`](docs/maury/refonte-ux-multi-role-acp/stories.md) §6 Story 4.4
   - Architecture ADR-0014 : [`docs/maury/refonte-ux-multi-role-acp/architecture.md`](docs/maury/refonte-ux-multi-role-acp/architecture.md) §4
   - Epic Maury : #556
   
   🤖 Sous-issue Phase 6 Exécution Maury — Tier 1 humain pour création/fermeture.

.. raw:: html

   </div>

