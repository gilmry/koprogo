=============================================================================================
Issue #552: POST /work-reports + /technical-inspections renvoient 400 depuis /building-detail
=============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: bug,javascript track:software,rust
:Assignees: Unassigned
:Created: 2026-05-20
:Updated: 2026-05-20
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/552>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Observé en live console (browser) sur page `/building-detail?id=7fc28890-...` :
   
   ```
   XHRPOST /api/v1/work-reports         [HTTP/1.1 400 Bad Request 7ms]
   XHRPOST /api/v1/technical-inspections [HTTP/1.1 400 Bad Request 6ms]
   ```
   
   Et confirmé côté backend log :
   ```
   2026-05-20T17:29:38  "POST /api/v1/work-reports HTTP/1.1" 400 42 "http://localhost/building-detail?id=..."
   2026-05-20T17:30:05  "POST /api/v1/technical-inspections HTTP/1.1" 400 42 "http://localhost/building-detail?id=..."
   ```
   
   Body de réponse = 42 bytes (typique `{"error":"..."}` court). Les GET correspondants (`GET /buildings/{id}/work-reports` et `GET /buildings/{id}/technical-inspections`) **passent en 200**, donc auth + DB sont OK.
   
   ## Symptômes
   
   L'utilisateur sur `/building-detail` tente de créer un work-report ou une technical-inspection via le formulaire UI. Le POST échoue avec 400 (validation backend rejette le payload) avant même d'atteindre la couche métier.
   
   ## Cause probable (à confirmer)
   
   Les 2 handlers utilisent `web::Json<DTO>` stricte côté Actix :
   - [backend/src/infrastructure/web/handlers/work_report_handlers.rs:17](backend/src/infrastructure/web/handlers/work_report_handlers.rs#L17) → `request: web::Json<CreateWorkReportDto>`
   - Handler équivalent pour technical-inspections (à vérifier).
   
   Côté FE :
   - [frontend/src/lib/api/work-reports.ts:105-107](frontend/src/lib/api/work-reports.ts#L105-L107) → `api.post("/work-reports", data)` avec `CreateWorkReportDto`
   - [frontend/src/lib/api/inspections.ts:119-121](frontend/src/lib/api/inspections.ts#L119-L121) → `api.post("/technical-inspections", data)` avec `CreateInspectionDto`
   
   **Hypothèse principale** : drift de schéma FE↔BE.
   - Champ requis manquant côté FE (e.g. `building_id`, `organization_id`, `contractor_id`)
   - OU type/format incorrect (date, decimal, enum)
   - OU FE envoie un champ snake_case alors que BE attend camelCase (ou inverse)
   - OU enum value FE ≠ BE (e.g. WorkType / WarrantyType)
   
   Le contract-types check CI (anti-drift end-to-end) **devrait** rattraper ça — peut-être qu'un type a été régénéré côté FE sans que le formulaire ait été mis à jour, ou que les DTOs ne sont pas exposés dans `openapi.json`.
   
   ## Recette (proposée)
   
   1. **Capturer le payload exact** envoyé par le FE : DevTools → Network → POST work-reports → Request Body (JSON)
   2. **Capturer le body 400** côté backend : DevTools → Network → POST → Response Body (le message d'erreur)
   3. **Diff** entre payload envoyé et schéma `CreateWorkReportDto` Rust (champs requis, types)
   4. Idem pour `CreateInspectionDto` / `technical-inspections`
   5. Selon trouvailles : fix FE (manque champ / type) OU fix BE (validation trop stricte / DTO out of sync)
   6. Ajouter Playwright test reproducteur dans `frontend/tests/e2e/` (pattern : créer building → tenter create work-report via API+UI → 201 attendu)
   
   ## Critères d'acceptation
   
   - [ ] Les 2 POST renvoient 201 (ou 200) quand le formulaire UI est soumis avec données valides
   - [ ] Test Playwright `@happy` : create work-report + technical-inspection passent
   - [ ] Si le drift FE↔BE est trouvé : ADR ou note dans `docs/` pour le bloquer en CI via le contract-types check
   
   ## Hors-scope
   
   - Pas lié à #550 (auth refresh — déjà fixé). Les calls portent bien un `Authorization` header (sinon ce serait 401, pas 400).
   - Pas lié à #549 (gate CI go-live — sécu/IaC indépendants).
   
   ## Liens
   
   - Page concernée : [BuildingDetail.svelte](frontend/src/components/BuildingDetail.svelte)
   - Composants enfants : `WorkReportList.svelte`, vraisemblablement un équivalent inspections.
   - Contract drift gate : workflow CI `Contract Types Check (end-to-end anti-drift)`

.. raw:: html

   </div>

