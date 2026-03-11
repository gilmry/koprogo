========================================================================
Issue #134: feat: Complete Work Reports & Technical Inspections REST API
========================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,phase:vps track:software,priority:high
:Assignees: Unassigned
:Created: 2025-11-18
:Updated: 2025-11-18
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/134>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## 🔧 Complete REST API for Contractor Backoffice (Issue #52 Split)
   
   **Priority**: 🔴 HIGH | **Phase**: VPS (Jalon 3) | **Track**: Software  
   **Parent Issue**: #52 (Contractor backoffice)
   
   ---
   
   ## 📋 Context
   
   Domain entities and migrations **ALREADY IMPLEMENTED** ✅:
   - ✅ Migration: `20251203000000_create_work_reports.sql`
   - ✅ Migration: `20251203000001_create_technical_inspections.sql`
   - ✅ Domain entity: `backend/src/domain/entities/work_report.rs` (110 lines)
   - ✅ Domain entity: `backend/src/domain/entities/technical_inspection.rs` (102 lines)
   
   **Missing**: REST API handlers + routes configuration ❌
   
   ---
   
   ## 🎯 Objective
   
   Expose Work Reports and Technical Inspections via REST API for contractor backoffice.
   
   ---
   
   ## 📐 Tasks
   
   ### Backend (Rust)
   
   #### 1. Repository Traits + Implementations
   
   **Create**: `backend/src/application/ports/work_report_repository.rs`
   ```rust
   #[async_trait]
   pub trait WorkReportRepository: Send + Sync {
       async fn create(&self, report: &WorkReport) -> Result<WorkReport, String>;
       async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkReport>, String>;
       async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<WorkReport>, String>;
       async fn find_by_contractor(&self, contractor_id: Uuid) -> Result<Vec<WorkReport>, String>;
       async fn find_by_quote(&self, quote_id: Uuid) -> Result<Vec<WorkReport>, String>;
       async fn update(&self, report: &WorkReport) -> Result<WorkReport, String>;
       async fn delete(&self, id: Uuid) -> Result<(), String>;
       async fn list_pending_validation(&self, organization_id: Uuid) -> Result<Vec<WorkReport>, String>;
       async fn mark_as_validated(&self, id: Uuid, validated_by: Uuid) -> Result<(), String>;
       async fn count_by_contractor(&self, contractor_id: Uuid) -> Result<i64, String>;
   }
   ```
   
   **Create**: `backend/src/infrastructure/database/repositories/work_report_repository_impl.rs`
   - PostgreSQL implementation (10 methods)
   - SQL queries with proper error handling
   
   **Create**: `backend/src/application/ports/technical_inspection_repository.rs`
   ```rust
   #[async_trait]
   pub trait TechnicalInspectionRepository: Send + Sync {
       async fn create(&self, inspection: &TechnicalInspection) -> Result<TechnicalInspection, String>;
       async fn find_by_id(&self, id: Uuid) -> Result<Option<TechnicalInspection>, String>;
       async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<TechnicalInspection>, String>;
       async fn find_by_inspector(&self, inspector_id: Uuid) -> Result<Vec<TechnicalInspection>, String>;
       async fn find_by_type(&self, building_id: Uuid, inspection_type: InspectionType) -> Result<Vec<TechnicalInspection>, String>;
       async fn find_by_status(&self, building_id: Uuid, status: InspectionStatus) -> Result<Vec<TechnicalInspection>, String>;
       async fn update(&self, inspection: &TechnicalInspection) -> Result<TechnicalInspection, String>;
       async fn delete(&self, id: Uuid) -> Result<(), String>;
       async fn find_overdue(&self, organization_id: Uuid) -> Result<Vec<TechnicalInspection>, String>;
       async fn mark_as_completed(&self, id: Uuid) -> Result<(), String>;
   }
   ```
   
   **Create**: `backend/src/infrastructure/database/repositories/technical_inspection_repository_impl.rs`
   
   #### 2. Use Cases
   
   **Create**: `backend/src/application/use_cases/work_report_use_cases.rs`
   - 10 methods matching repository + business logic
   
   **Create**: `backend/src/application/use_cases/technical_inspection_use_cases.rs`
   - 10 methods matching repository + business logic
   
   #### 3. DTOs
   
   **Create**: `backend/src/application/dto/work_report_dto.rs`
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub struct CreateWorkReportDto {
       pub building_id: Uuid,
       pub contractor_id: Uuid,
       pub quote_id: Option<Uuid>,
       pub work_type: WorkType,
       pub description: String,
       pub work_date: NaiveDate,
       pub hours_worked: f64,
       pub materials_cost_cents: Option<i64>,
       pub labor_cost_cents: Option<i64>,
       pub photo_urls: Vec<String>,
       pub notes: Option<String>,
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct WorkReportResponseDto {
       pub id: Uuid,
       pub building_id: Uuid,
       pub contractor_id: Uuid,
       pub quote_id: Option<Uuid>,
       pub work_type: WorkType,
       pub description: String,
       pub work_date: NaiveDate,
       pub hours_worked: f64,
       pub materials_cost_cents: Option<i64>,
       pub labor_cost_cents: Option<i64>,
       pub total_cost_cents: i64,
       pub photo_urls: Vec<String>,
       pub is_validated: bool,
       pub validated_by: Option<Uuid>,
       pub validated_at: Option<DateTime<Utc>>,
       pub warranty_type: Option<WarrantyType>,
       pub warranty_expiration: Option<NaiveDate>,
       pub notes: Option<String>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   ```
   
   **Create**: `backend/src/application/dto/technical_inspection_dto.rs`
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub struct CreateTechnicalInspectionDto {
       pub building_id: Uuid,
       pub inspector_id: Uuid,
       pub inspection_type: InspectionType,
       pub scheduled_date: NaiveDate,
       pub description: Option<String>,
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct TechnicalInspectionResponseDto {
       pub id: Uuid,
       pub building_id: Uuid,
       pub inspector_id: Uuid,
       pub inspection_type: InspectionType,
       pub status: InspectionStatus,
       pub scheduled_date: NaiveDate,
       pub completed_date: Option<NaiveDate>,
       pub findings: Option<String>,
       pub recommendations: Option<String>,
       pub compliance_status: bool,
       pub report_url: Option<String>,
       pub next_inspection_due: Option<NaiveDate>,
       pub description: Option<String>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   ```
   
   #### 4. REST Handlers
   
   **Create**: `backend/src/infrastructure/web/handlers/work_report_handlers.rs` (~400 lines)
   
   Endpoints (10):
   - `POST /api/v1/work-reports` - Create work report
   - `GET /api/v1/work-reports/:id` - Get work report
   - `GET /api/v1/buildings/:id/work-reports` - List building work reports
   - `GET /api/v1/contractors/:id/work-reports` - List contractor work reports  
   - `GET /api/v1/quotes/:id/work-reports` - List quote work reports
   - `PUT /api/v1/work-reports/:id` - Update work report
   - `DELETE /api/v1/work-reports/:id` - Delete work report
   - `GET /api/v1/work-reports/pending-validation` - List pending validation
   - `PUT /api/v1/work-reports/:id/validate` - Validate work report (syndic)
   - `GET /api/v1/contractors/:id/work-reports/stats` - Contractor stats
   
   **Create**: `backend/src/infrastructure/web/handlers/technical_inspection_handlers.rs` (~400 lines)
   
   Endpoints (10):
   - `POST /api/v1/inspections` - Create inspection
   - `GET /api/v1/inspections/:id` - Get inspection
   - `GET /api/v1/buildings/:id/inspections` - List building inspections
   - `GET /api/v1/buildings/:id/inspections/type/:type` - List by type
   - `GET /api/v1/buildings/:id/inspections/status/:status` - List by status
   - `PUT /api/v1/inspections/:id` - Update inspection
   - `DELETE /api/v1/inspections/:id` - Delete inspection
   - `GET /api/v1/inspections/overdue` - List overdue inspections
   - `PUT /api/v1/inspections/:id/complete` - Mark completed
   - `GET /api/v1/buildings/:id/inspections/stats` - Inspection stats
   
   #### 5. Wire Routes
   
   **Update**: `backend/src/infrastructure/web/routes.rs`
   
   Add after line 366 (Board Decisions):
   ```rust
   // Work Reports (Contractor Backoffice - Issue #52)
   .service(create_work_report)
   .service(get_work_report)
   .service(list_building_work_reports)
   .service(list_contractor_work_reports)
   .service(list_quote_work_reports)
   .service(update_work_report)
   .service(delete_work_report)
   .service(list_pending_validation_work_reports)
   .service(validate_work_report)
   .service(get_contractor_work_stats)
   // Technical Inspections (Contractor Backoffice - Issue #52)
   .service(create_inspection)
   .service(get_inspection)
   .service(list_building_inspections)
   .service(list_inspections_by_type)
   .service(list_inspections_by_status)
   .service(update_inspection)
   .service(delete_inspection)
   .service(list_overdue_inspections)
   .service(complete_inspection)
   .service(get_building_inspection_stats)
   ```
   
   #### 6. Update AppState
   
   **Update**: `backend/src/infrastructure/web/app_state.rs`
   
   Add repositories:
   ```rust
   pub work_report_repository: Arc<dyn WorkReportRepository>,
   pub technical_inspection_repository: Arc<dyn TechnicalInspectionRepository>,
   ```
   
   #### 7. Update mod.rs Files
   
   **Update**: `backend/src/application/ports/mod.rs`
   ```rust
   pub mod work_report_repository;
   pub mod technical_inspection_repository;
   pub use work_report_repository::WorkReportRepository;
   pub use technical_inspection_repository::TechnicalInspectionRepository;
   ```
   
   **Update**: `backend/src/infrastructure/database/repositories/mod.rs`
   ```rust
   pub mod work_report_repository_impl;
   pub mod technical_inspection_repository_impl;
   pub use work_report_repository_impl::PostgresWorkReportRepository;
   pub use technical_inspection_repository_impl::PostgresTechnicalInspectionRepository;
   ```
   
   **Update**: `backend/src/application/use_cases/mod.rs`
   ```rust
   pub mod work_report_use_cases;
   pub mod technical_inspection_use_cases;
   pub use work_report_use_cases::WorkReportUseCases;
   pub use technical_inspection_use_cases::TechnicalInspectionUseCases;
   ```
   
   **Update**: `backend/src/application/dto/mod.rs`
   ```rust
   pub mod work_report_dto;
   pub mod technical_inspection_dto;
   pub use work_report_dto::*;
   pub use technical_inspection_dto::*;
   ```
   
   **Update**: `backend/src/infrastructure/web/handlers/mod.rs`
   ```rust
   pub mod work_report_handlers;
   pub mod technical_inspection_handlers;
   pub use work_report_handlers::*;
   pub use technical_inspection_handlers::*;
   ```
   
   ---
   
   ## ✅ Acceptance Criteria
   
   ### Backend
   - [ ] WorkReportRepository trait + PostgreSQL impl (10 methods)
   - [ ] TechnicalInspectionRepository trait + PostgreSQL impl (10 methods)
   - [ ] WorkReportUseCases (10 methods)
   - [ ] TechnicalInspectionUseCases (10 methods)
   - [ ] 2 DTOs per entity (Create + Response)
   - [ ] 20 REST endpoints total (10 per entity)
   - [ ] Routes wired in routes.rs
   - [ ] AppState updated with repositories
   - [ ] All mod.rs files updated
   
   ### Testing
   - [ ] Unit tests for domain entities (already exist)
   - [ ] Integration tests for repositories
   - [ ] E2E tests for REST endpoints
   
   ### Documentation
   - [ ] Update CLAUDE.md with new endpoints
   - [ ] API documentation (OpenAPI/Swagger)
   
   ---
   
   ## 📦 Effort Estimé
   
   **8-12 heures** (1-2 jours dev)
   - Day 1 AM: Repositories + Use Cases (4h)
   - Day 1 PM: DTOs + Handlers Work Reports (4h)
   - Day 2 AM: Handlers Technical Inspections (3h)
   - Day 2 PM: Routes + Tests + Documentation (3h)
   
   ---
   
   ## 🔗 Dependencies
   
   - ✅ Domain entities exist
   - ✅ Migrations applied
   - ❌ REST API layer missing (this issue)
   
   ---
   
   ## 📚 Related Issues
   
   - #52: Parent issue (Contractor backoffice)
   - #91: Contractor Quotes Module (already implemented)
   - #85: Ticketing System (already implemented)
   
   ---
   
   ## 🎯 Labels
   
   `enhancement`, `phase:vps`, `track:software`, `priority:high`

.. raw:: html

   </div>

