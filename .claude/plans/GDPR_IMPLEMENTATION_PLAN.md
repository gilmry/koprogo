# Plan d'implémentation GDPR (Issue #42) - TDD Strict

**Date**: 2025-10-30
**Issue**: #42 - feat: Implement GDPR data export & deletion (Right to be forgotten)
**Status**: En cours
**Méthodologie**: TDD Red-Green-Refactor avec commits atomiques thématiques

## Méthodologie

**TDD Red-Green-Refactor** avec commits atomiques thématiques et mise à jour du changelog à chaque étape.

### Cycle TDD pour chaque fonctionnalité
1. 🔴 **RED**: Écrire le test qui échoue
2. 🟢 **GREEN**: Écrire le code minimal pour passer le test
3. 🔵 **REFACTOR**: Améliorer le code sans casser les tests
4. ✅ **COMMIT**: Commit thématique + changelog
5. 🔍 **QUALITY**: `make lint`, `make format`, `make test`

---

## Phase 1: Migration BDD + Domain Entities

### 1.1 Migration GDPR (RED → GREEN)
- 🔴 Créer test d'intégration vérifiant colonnes manquantes
- 🟢 Créer `backend/migrations/20251030XXXXXX_add_gdpr_anonymization_fields.sql`
- 🟢 Exécuter `make migrate`
- 🔍 Vérifier avec requête SQL manuelle
- ✅ **Commit**: `chore(db): add GDPR anonymization fields to users and owners`
- 📝 **Changelog**: Ajout section `[Unreleased] - Database`

### 1.2 Domain Entity GdprExport (RED → GREEN → REFACTOR)
- 🔴 Créer tests unitaires `gdpr_export.rs` (construction, validation)
- 🟢 Implémenter structs `GdprExport`, `UserData`, `OwnerData`
- 🔵 Refactor: Builder pattern si nécessaire
- 🔍 `make test` (unit tests passent)
- ✅ **Commit**: `feat(domain): add GDPR export domain entities`
- 📝 **Changelog**: `[Unreleased] - Features`

---

## Phase 2: Application Layer - Export de données

### 2.1 Port GdprRepository (RED → GREEN)
- 🔴 Créer tests mock vérifiant contrat du trait
- 🟢 Créer `backend/src/application/ports/gdpr_repository.rs`
  - Trait avec méthodes: `aggregate_user_data()`, `find_user_exports()`
- 🔍 `make lint`
- ✅ **Commit**: `feat(ports): define GdprRepository trait for data aggregation`
- 📝 **Changelog**: `[Unreleased] - Architecture`

### 2.2 DTO GDPR Export (RED → GREEN)
- 🔴 Tests de sérialisation/désérialisation JSON
- 🟢 Créer `backend/src/application/dto/gdpr_dto.rs`
  - `GdprExportResponseDto`, validation
- 🔍 `make test`
- ✅ **Commit**: `feat(dto): add GDPR export response DTOs`
- 📝 **Changelog**: `[Unreleased] - Features`

### 2.3 Use Case Export (RED → GREEN → REFACTOR)
- 🔴 Tests unitaires avec mock repository:
  - Test: export_user_data réussit avec données complètes
  - Test: export_user_data échoue si utilisateur inexistant
  - Test: export_user_data inclut toutes les entités liées
- 🟢 Créer `backend/src/application/use_cases/gdpr_use_cases.rs`
  - Implémenter `export_user_data(user_id)`
- 🔵 Refactor: Extraction méthodes privées si nécessaire
- 🔍 `make test` (100% coverage domain + application)
- ✅ **Commit**: `feat(use-case): implement GDPR data export use case`
- 📝 **Changelog**: `[Unreleased] - Features`

---

## Phase 3: Infrastructure - Repository Implementation Export

### 3.1 Repository Implementation (RED → GREEN → REFACTOR)
- 🔴 Tests d'intégration avec testcontainers PostgreSQL:
  - Test: aggregate_user_data retourne données complètes
  - Test: agrégation multi-tables (users, owners, units, expenses, documents)
  - Test: gestion utilisateur sans données liées
- 🟢 Créer `backend/src/infrastructure/database/repositories/gdpr_repository_impl.rs`
  - Implémenter requêtes SQL avec JOINs
- 🔵 Refactor: Optimiser requêtes, extraire SQL constants
- 🔍 `cargo test --test integration` passe
- ✅ **Commit**: `feat(infra): implement PostgreSQL GDPR repository for data aggregation`
- 📝 **Changelog**: `[Unreleased] - Features`

---

## Phase 4: Infrastructure - Web Handler Export

### 4.1 Handler Export Endpoint (RED → GREEN → REFACTOR)
- 🔴 Tests E2E avec Actix test:
  - Test: GET /api/v1/gdpr/export retourne 200 + JSON complet
  - Test: Retourne 401 sans token JWT
  - Test: Utilisateur ne peut exporter que ses propres données
  - Test: SuperAdmin peut exporter n'importe quel utilisateur
- 🟢 Créer `backend/src/infrastructure/web/handlers/gdpr_handlers.rs`
  - Handler `export_user_data_handler()`
- 🟢 Modifier `backend/src/infrastructure/web/routes.rs`
- 🟢 Modifier `backend/src/infrastructure/web/app_state.rs`
- 🔵 Refactor: Extraction validation, error handling
- 🔍 `cargo test --test e2e` passe
- ✅ **Commit**: `feat(api): add GET /api/v1/gdpr/export endpoint`
- 📝 **Changelog**: `[Unreleased] - API`

### 4.2 Audit Logging Export (RED → GREEN)
- 🔴 Test: Vérifier événement `GdprDataExported` loggé
- 🟢 Modifier `backend/src/infrastructure/audit.rs`
  - Ajouter `GdprDataExported`, `GdprExportFailed`
- 🟢 Intégrer logging dans handler
- 🔍 `make test`
- ✅ **Commit**: `feat(audit): add GDPR export audit events`
- 📝 **Changelog**: `[Unreleased] - Security`

---

## Phase 5: Application Layer - Suppression de données

### 5.1 Port Anonymization (RED → GREEN)
- 🔴 Tests mock pour méthodes d'anonymisation
- 🟢 Étendre `gdpr_repository.rs`
  - Ajouter: `anonymize_user()`, `anonymize_owner()`, `check_legal_holds()`
- 🔍 `make lint`
- ✅ **Commit**: `feat(ports): add anonymization methods to GdprRepository`
- 📝 **Changelog**: `[Unreleased] - Architecture`

### 5.2 DTO GDPR Erase (RED → GREEN)
- 🔴 Tests validation DTO
- 🟢 Ajouter `GdprEraseRequestDto`, `GdprEraseResponseDto` dans `gdpr_dto.rs`
- 🔍 `make test`
- ✅ **Commit**: `feat(dto): add GDPR erase request/response DTOs`
- 📝 **Changelog**: `[Unreleased] - Features`

### 5.3 Use Case Erase (RED → GREEN → REFACTOR)
- 🔴 Tests unitaires avec mocks:
  - Test: erase_user_data anonymise user et owners
  - Test: échoue si utilisateur déjà anonymisé
  - Test: vérifie permissions (user can only erase self)
  - Test: préserve intégrité référentielle
- 🟢 Implémenter `erase_user_data(user_id, requesting_user_id)` dans `gdpr_use_cases.rs`
- 🔵 Refactor: Transaction handling, rollback on error
- 🔍 `make test`
- ✅ **Commit**: `feat(use-case): implement GDPR data erasure with anonymization`
- 📝 **Changelog**: `[Unreleased] - Features`

---

## Phase 6: Infrastructure - Repository Implementation Erase

### 6.1 Repository Anonymization (RED → GREEN → REFACTOR)
- 🔴 Tests d'intégration:
  - Test: anonymize_user remplace PII par [ANONYMIZED-{uuid}]
  - Test: anonymize_owner idem
  - Test: transaction atomique (rollback si échec partiel)
  - Test: préserve audit logs
- 🟢 Implémenter dans `gdpr_repository_impl.rs`
- 🟢 Modifier `user_repository_impl.rs` et `owner_repository_impl.rs`
- 🔵 Refactor: SQL en constantes, helper functions
- 🔍 `cargo test --test integration`
- ✅ **Commit**: `feat(infra): implement GDPR anonymization in repositories`
- 📝 **Changelog**: `[Unreleased] - Features`

---

## Phase 7: Infrastructure - Web Handler Erase

### 7.1 Handler Erase Endpoint (RED → GREEN → REFACTOR)
- 🔴 Tests E2E:
  - Test: DELETE /api/v1/gdpr/erase retourne 200
  - Test: Données anonymisées en BDD après appel
  - Test: 401 sans auth
  - Test: User ne peut supprimer que son compte
  - Test: SuperAdmin peut supprimer n'importe qui
- 🟢 Ajouter `erase_user_data_handler()` dans `gdpr_handlers.rs`
- 🟢 Ajouter route dans `routes.rs`
- 🔵 Refactor: Confirmation password re-auth (optionnel)
- 🔍 `cargo test --test e2e`
- ✅ **Commit**: `feat(api): add DELETE /api/v1/gdpr/erase endpoint`
- 📝 **Changelog**: `[Unreleased] - API`

### 7.2 Audit Logging Erase (RED → GREEN)
- 🔴 Test: Événement `GdprDataErased` loggé
- 🟢 Ajouter `GdprDataErased`, `GdprErasureFailed` dans `audit.rs`
- 🟢 Logger dans handler
- 🔍 `make test`
- ✅ **Commit**: `feat(audit): add GDPR erasure audit events`
- 📝 **Changelog**: `[Unreleased] - Security`

---

## Phase 8: Infrastructure - Admin Endpoints

### 8.1 Admin Export List (RED → GREEN)
- 🔴 Tests E2E:
  - Test: GET /api/v1/admin/gdpr/exports retourne liste
  - Test: 403 si non-SuperAdmin
- 🟢 Handler `list_gdpr_exports_handler()` dans `gdpr_handlers.rs`
- 🟢 Route admin
- 🔍 `cargo test --test e2e`
- ✅ **Commit**: `feat(api): add admin GDPR exports list endpoint`
- 📝 **Changelog**: `[Unreleased] - API`

### 8.2 Admin Manual Erase (RED → GREEN)
- 🔴 Tests E2E:
  - Test: POST /admin/gdpr/erase/:user_id réussit (SuperAdmin)
  - Test: 403 si non-SuperAdmin
- 🟢 Handler `admin_erase_user_handler()`
- 🟢 Route admin
- 🔍 `cargo test --test e2e`
- ✅ **Commit**: `feat(api): add admin manual GDPR erasure endpoint`
- 📝 **Changelog**: `[Unreleased] - API`

---

## Phase 9: Tests BDD (Cucumber/Gherkin)

### 9.1 Feature GDPR (RED → GREEN)
- 🔴 Créer `backend/tests/features/gdpr.feature`
  - Scénarios: Export réussi, Erase réussi, Admin operations, Unauthorized access
- 🟢 Implémenter steps dans `backend/tests/bdd.rs`
- 🔍 `cargo test --test bdd` passe
- ✅ **Commit**: `test(bdd): add GDPR Cucumber scenarios`
- 📝 **Changelog**: `[Unreleased] - Tests`

---

## Phase 10: Frontend - Privacy Page Utilisateur

### 10.1 Privacy Settings Component (RED → GREEN → REFACTOR)
- 🔴 Tests composant Svelte (si framework test configuré)
- 🟢 Créer `frontend/src/components/PrivacySettings.svelte`
  - Bouton export, bouton suppression, affichage statut
- 🟢 Créer `frontend/src/pages/privacy.astro`
- 🔵 Refactor: Extraction sous-composants
- 🔍 Test manuel E2E
- ✅ **Commit**: `feat(frontend): add user privacy settings page`
- 📝 **Changelog**: `[Unreleased] - Frontend`

### 10.2 GDPR Export Modal (RED → GREEN)
- 🔴 Tests interaction modal
- 🟢 Créer `frontend/src/components/GdprExportModal.svelte`
  - API call, download JSON, loading states
- 🔍 Test manuel
- ✅ **Commit**: `feat(frontend): add GDPR data export modal`
- 📝 **Changelog**: `[Unreleased] - Frontend`

### 10.3 GDPR Erase Modal (RED → GREEN)
- 🔴 Tests confirmation double-check
- 🟢 Créer `frontend/src/components/GdprEraseModal.svelte`
  - Confirmation, API call, redirect
- 🔍 Test manuel
- ✅ **Commit**: `feat(frontend): add GDPR account deletion modal`
- 📝 **Changelog**: `[Unreleased] - Frontend`

---

## Phase 11: Frontend - Admin Dashboard

### 11.1 Admin GDPR Dashboard (RED → GREEN → REFACTOR)
- 🔴 Tests composant admin
- 🟢 Créer `frontend/src/components/admin/GdprDashboard.svelte`
  - Liste exports, stats, boutons actions
- 🟢 Créer `frontend/src/pages/admin/gdpr.astro`
- 🔵 Refactor: Pagination, filtres
- 🔍 Test manuel E2E
- ✅ **Commit**: `feat(frontend): add admin GDPR management dashboard`
- 📝 **Changelog**: `[Unreleased] - Frontend`

---

## Phase 12: Tests End-to-End Frontend (Playwright)

### 12.1 E2E User Journey (RED → GREEN)
- 🔴 Créer `frontend/tests/e2e/gdpr-user.spec.ts`
  - Scénario: Login → Privacy page → Export data → Delete account
- 🟢 Implémenter tests avec data-testid
- 🔍 `npm run test:e2e` passe
- ✅ **Commit**: `test(e2e): add user GDPR workflow Playwright tests`
- 📝 **Changelog**: `[Unreleased] - Tests`

### 12.2 E2E Admin Journey (RED → GREEN)
- 🔴 Créer `frontend/tests/e2e/gdpr-admin.spec.ts`
  - Scénario: Login SuperAdmin → GDPR dashboard → Manual erase
- 🟢 Implémenter tests
- 🔍 `npm run test:e2e`
- ✅ **Commit**: `test(e2e): add admin GDPR management Playwright tests`
- 📝 **Changelog**: `[Unreleased] - Tests`

---

## Phase 13: Documentation

### 13.1 Documentation Technique (GREEN)
- 🟢 Créer `docs/GDPR_COMPLIANCE.md`
  - Procédures, conformité légale, retention policies
- 🟢 Modifier `CLAUDE.md`
  - Section GDPR implementation, endpoints, exemples
- ✅ **Commit**: `docs: add GDPR compliance documentation`
- 📝 **Changelog**: `[Unreleased] - Documentation`

### 13.2 Update Roadmap (GREEN)
- 🟢 Modifier `docs/ROADMAP.md`
  - Marquer issue #42 complétée
- ✅ **Commit**: `docs(roadmap): mark GDPR implementation as completed`
- 📝 **Changelog**: `[Unreleased] - Documentation`

---

## Phase 14: Quality Gates & Final Review

### 14.1 Quality Checks
```bash
make format      # Formatting auto
make lint        # Clippy warnings = 0
make test        # Tous tests passent (unit + integration + e2e + bdd)
make coverage    # Coverage > 80%
```
- ✅ **Commit**: `chore: run quality checks and fix linting`
- 📝 **Changelog**: `[Unreleased] - Chore`

### 14.2 Changelog Consolidation
- 🟢 Finaliser `docs/CHANGELOG.md`
  - Regrouper tous les ajouts sous version `[Unreleased]`
  - Catégories: Features, API, Security, Frontend, Tests, Documentation
- ✅ **Commit**: `docs(changelog): consolidate GDPR implementation entries`

### 14.3 Final Integration Test
- 🔍 Test manuel complet du workflow utilisateur et admin
- 🔍 Vérifier audit logs en BDD
- 🔍 Valider JSON export conforme GDPR
- ✅ **Commit**: `test: validate complete GDPR workflow integration`

---

## Commits Thématiques Attendus (~25-30 commits)

**Pattern**: `<type>(<scope>): <description>`

**Types**: `feat`, `test`, `docs`, `chore`, `refactor`, `fix`
**Scopes**: `db`, `domain`, `ports`, `dto`, `use-case`, `infra`, `api`, `audit`, `frontend`, `e2e`, `bdd`

---

## Changelog Structure

```markdown
## [Unreleased]

### Features
- GDPR data export API endpoint
- GDPR data erasure with anonymization
- User privacy settings page
- Admin GDPR management dashboard

### API
- GET /api/v1/gdpr/export
- DELETE /api/v1/gdpr/erase
- GET /api/v1/admin/gdpr/exports
- POST /api/v1/admin/gdpr/erase/:user_id

### Security
- GDPR audit logging for all data operations
- Anonymization preserving referential integrity

### Tests
- Unit tests for GDPR use cases (100% coverage)
- Integration tests with PostgreSQL testcontainers
- E2E API tests for GDPR endpoints
- BDD Cucumber scenarios for GDPR compliance
- Playwright E2E tests for user/admin workflows

### Documentation
- GDPR compliance procedures
- Data retention policies
- API documentation updates
```

---

## Estimation

**8-12 heures** en TDD strict avec quality gates à chaque étape

---

## Fichiers à créer/modifier

### Backend (15 fichiers)
**Nouveaux**:
- `backend/migrations/20251030XXXXXX_add_gdpr_anonymization_fields.sql`
- `backend/src/domain/entities/gdpr_export.rs`
- `backend/src/application/ports/gdpr_repository.rs`
- `backend/src/application/dto/gdpr_dto.rs`
- `backend/src/application/use_cases/gdpr_use_cases.rs`
- `backend/src/infrastructure/database/repositories/gdpr_repository_impl.rs`
- `backend/src/infrastructure/web/handlers/gdpr_handlers.rs`

**Modifiés**:
- `backend/src/domain/entities/mod.rs`
- `backend/src/application/ports/mod.rs`
- `backend/src/application/dto/mod.rs`
- `backend/src/application/use_cases/mod.rs`
- `backend/src/infrastructure/audit.rs`
- `backend/src/infrastructure/database/repositories/mod.rs`
- `backend/src/infrastructure/web/app_state.rs`
- `backend/src/infrastructure/web/routes.rs`
- `backend/src/infrastructure/web/handlers/mod.rs`

### Frontend (7 fichiers nouveaux)
- `frontend/src/pages/privacy.astro`
- `frontend/src/pages/admin/gdpr.astro`
- `frontend/src/components/PrivacySettings.svelte`
- `frontend/src/components/GdprExportModal.svelte`
- `frontend/src/components/GdprEraseModal.svelte`
- `frontend/src/components/admin/GdprDashboard.svelte`

### Tests (4 fichiers nouveaux)
- `backend/tests/integration_gdpr.rs`
- `backend/tests/features/gdpr.feature`
- `frontend/tests/e2e/gdpr-user.spec.ts`
- `frontend/tests/e2e/gdpr-admin.spec.ts`

### Documentation (3 fichiers)
- `docs/GDPR_COMPLIANCE.md` (nouveau)
- `CLAUDE.md` (modifié)
- `docs/ROADMAP.md` (modifié)
- `docs/CHANGELOG.md` (modifié)

---

## Notes de conformité GDPR

### Article 15 - Right to Access
✓ User peut demander copie de toutes ses données
✓ Format machine-readable (JSON)
✓ Délai: 30 jours max (synchrone pour MVP)

### Article 17 - Right to Erasure
✓ User peut demander suppression
✓ Anonymisation (pas suppression totale)
✓ Exceptions: Obligations légales, audit logs
✓ Conservation audit logs (7 ans Belgique)

### Article 30 - Records of Processing
✓ Audit logs enregistrent toutes opérations GDPR
✓ Include: user_id, timestamp, operation type, success/failure

### Article 20 - Right to Data Portability
✓ Export format structuré JSON
✓ Transférable à autre service

---

## Références

- Issue GitHub: #42
- GDPR official: https://gdpr-info.eu/
- Belgian data retention: https://www.autoriteprotectiondonnees.be/
- Architecture: Hexagonal (Ports & Adapters) + DDD
