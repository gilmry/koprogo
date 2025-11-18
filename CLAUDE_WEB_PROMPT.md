# 🤖 Claude Web Implementation Prompt

**Generated**: 2025-11-18  
**Project**: KoproGo - Belgian Property Management SaaS  
**Repository**: https://github.com/gilmry/koprogo

---

## 📋 Context

You are working on **KoproGo**, a production-ready SaaS property management platform for Belgian copropriétés (condominiums). The codebase follows **Hexagonal Architecture (Ports & Adapters)** with **Domain-Driven Design (DDD)**.

**Tech Stack**:
- Backend: Rust + Actix-web + SQLx + PostgreSQL 15
- Frontend: Astro + Svelte + Tailwind CSS
- Architecture: Domain → Application (Ports + Use Cases) → Infrastructure (Adapters)

**Current Status**: Gap analysis completed. Several features are partially implemented (domain entities exist, REST API missing).

---

## 🎯 Your Mission

Implement the **missing REST API layer** for features that already have domain entities and database migrations.

### Priority 1: Work Reports & Technical Inspections (Issue #134)

**Domain entities ALREADY exist** ✅:
- `backend/src/domain/entities/work_report.rs` (110 lines)
- `backend/src/domain/entities/technical_inspection.rs` (102 lines)

**Migrations ALREADY applied** ✅:
- `backend/migrations/20251203000000_create_work_reports.sql`
- `backend/migrations/20251203000001_create_technical_inspections.sql`

**What's MISSING** ❌:
- Repository traits (Application layer)
- PostgreSQL repository implementations (Infrastructure layer)
- Use Cases (Application layer)
- DTOs (Application layer)
- REST handlers (Infrastructure layer)
- Route configuration

---

## 📐 Implementation Guide

### Step 1: Create Repository Traits

**File**: `backend/src/application/ports/work_report_repository.rs`

```rust
use crate::domain::entities::{WorkReport, WorkType};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

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

**File**: `backend/src/application/ports/technical_inspection_repository.rs`

```rust
use crate::domain::entities::{TechnicalInspection, InspectionType, InspectionStatus};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

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

### Step 2: PostgreSQL Repository Implementations

**File**: `backend/src/infrastructure/database/repositories/work_report_repository_impl.rs`

```rust
use crate::application::ports::WorkReportRepository;
use crate::domain::entities::{WorkReport, WorkType, WarrantyType};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PostgresWorkReportRepository {
    pool: PgPool,
}

impl PostgresWorkReportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkReportRepository for PostgresWorkReportRepository {
    async fn create(&self, report: &WorkReport) -> Result<WorkReport, String> {
        let query = r#"
            INSERT INTO work_reports (
                id, building_id, contractor_id, quote_id, work_type, description,
                work_date, hours_worked, materials_cost_cents, labor_cost_cents,
                total_cost_cents, photo_urls, is_validated, warranty_type,
                warranty_expiration, notes, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
        "#;
        
        // Implementation here...
        // Follow pattern from existing repositories (e.g., quote_repository_impl.rs)
        
        Ok(report.clone())
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkReport>, String> {
        // SQL SELECT WHERE id = $1
        todo!()
    }
    
    // ... implement remaining 8 methods
}
```

**File**: `backend/src/infrastructure/database/repositories/technical_inspection_repository_impl.rs`

Similar structure for TechnicalInspection.

### Step 3: Use Cases

**File**: `backend/src/application/use_cases/work_report_use_cases.rs`

```rust
use crate::application::dto::{CreateWorkReportDto, WorkReportResponseDto};
use crate::application::ports::WorkReportRepository;
use crate::domain::entities::WorkReport;
use std::sync::Arc;
use uuid::Uuid;

pub struct WorkReportUseCases {
    work_report_repository: Arc<dyn WorkReportRepository>,
}

impl WorkReportUseCases {
    pub fn new(work_report_repository: Arc<dyn WorkReportRepository>) -> Self {
        Self {
            work_report_repository,
        }
    }

    pub async fn create_work_report(
        &self,
        dto: CreateWorkReportDto,
        organization_id: Uuid,
    ) -> Result<WorkReportResponseDto, String> {
        // 1. Create domain entity from DTO
        let work_report = WorkReport::new(
            dto.building_id,
            dto.contractor_id,
            dto.quote_id,
            dto.work_type,
            dto.description,
            dto.work_date,
            dto.hours_worked,
            dto.materials_cost_cents,
            dto.labor_cost_cents,
            dto.photo_urls,
            dto.notes,
        )?;

        // 2. Save via repository
        let saved = self.work_report_repository.create(&work_report).await?;

        // 3. Convert to DTO and return
        Ok(WorkReportResponseDto::from(saved))
    }

    // ... implement remaining 9 methods
}
```

### Step 4: DTOs

**File**: `backend/src/application/dto/work_report_dto.rs`

```rust
use crate::domain::entities::{WorkReport, WorkType, WarrantyType};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl From<WorkReport> for WorkReportResponseDto {
    fn from(report: WorkReport) -> Self {
        // Convert domain entity to DTO
        todo!()
    }
}
```

### Step 5: REST Handlers

**File**: `backend/src/infrastructure/web/handlers/work_report_handlers.rs`

```rust
use crate::application::dto::{CreateWorkReportDto, WorkReportResponseDto};
use crate::application::use_cases::WorkReportUseCases;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::middleware::AuthenticatedUser;
use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

#[post("/work-reports")]
pub async fn create_work_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<CreateWorkReportDto>,
) -> HttpResponse {
    let use_cases = WorkReportUseCases::new(state.work_report_repository.clone());
    
    match use_cases.create_work_report(dto.into_inner(), user.organization_id).await {
        Ok(report) => HttpResponse::Created().json(report),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        })),
    }
}

#[get("/work-reports/{id}")]
pub async fn get_work_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> HttpResponse {
    // Implementation...
    todo!()
}

// ... implement remaining 8 endpoints
```

### Step 6: Wire Routes

**File**: `backend/src/infrastructure/web/routes.rs`

Add after line 366:

```rust
// Work Reports (Contractor Backoffice - Issue #134)
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
// Technical Inspections (Contractor Backoffice - Issue #134)
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

### Step 7: Update AppState

**File**: `backend/src/infrastructure/web/app_state.rs`

Add to struct:

```rust
pub work_report_repository: Arc<dyn WorkReportRepository>,
pub technical_inspection_repository: Arc<dyn TechnicalInspectionRepository>,
```

Update initialization in `main.rs`.

### Step 8: Update mod.rs Files

**Update 5 mod.rs files**:
1. `backend/src/application/ports/mod.rs`
2. `backend/src/infrastructure/database/repositories/mod.rs`
3. `backend/src/application/use_cases/mod.rs`
4. `backend/src/application/dto/mod.rs`
5. `backend/src/infrastructure/web/handlers/mod.rs`

---

## 🎯 Priority 2: PDF Generation Extension (Issue #47)

**Context**: Meeting minutes PDF export exists. Need to extend for contracts + financial reports.

### Tasks

1. **Create PDF Template Engine**:
   - Use existing PDF generation code as reference
   - Add templates for: Employment contracts, Service contracts, Financial year-end reports

2. **Endpoints to Add**:
   - `GET /api/v1/contracts/:id/pdf` - Generate contract PDF
   - `GET /api/v1/budgets/:id/financial-report/pdf` - Generate financial report PDF

---

## 🎯 Priority 3: Rate Limiting (Issue #78 - remaining)

**Context**: 2FA already implemented. Need anti-brute-force protection.

### Tasks

1. **Add actix-governor middleware**:
   ```toml
   [dependencies]
   actix-governor = "0.5"
   ```

2. **Configure rate limiting**:
   - Login endpoint: 5 attempts / 15 minutes
   - API endpoints: 100 requests / minute
   - Document upload: 10 uploads / hour

3. **Update routes.rs** with governor middleware.

---

## ✅ Acceptance Criteria

For each feature:
- [ ] Repository trait defined (Application layer)
- [ ] PostgreSQL repository implementation (Infrastructure layer)
- [ ] Use Cases implemented (Application layer)
- [ ] DTOs created (Application layer)
- [ ] REST handlers implemented (Infrastructure layer)
- [ ] Routes wired in routes.rs
- [ ] AppState updated
- [ ] All mod.rs files updated
- [ ] Compiles without errors: `cargo build`
- [ ] Tests pass: `cargo test`
- [ ] API documented in OpenAPI/Swagger

---

## 📚 Reference Files (Study These)

**For Repository Pattern**:
- `backend/src/application/ports/quote_repository.rs`
- `backend/src/infrastructure/database/repositories/quote_repository_impl.rs`

**For Use Cases**:
- `backend/src/application/use_cases/quote_use_cases.rs`

**For DTOs**:
- `backend/src/application/dto/quote_dto.rs`

**For REST Handlers**:
- `backend/src/infrastructure/web/handlers/quote_handlers.rs`

**For Domain Entities (ALREADY EXIST)**:
- `backend/src/domain/entities/work_report.rs`
- `backend/src/domain/entities/technical_inspection.rs`

---

## 🚀 Development Workflow

1. **Start PostgreSQL**:
   ```bash
   make docker-up
   ```

2. **Run migrations**:
   ```bash
   cd backend && sqlx migrate run
   ```

3. **Develop with auto-reload**:
   ```bash
   make dev  # Backend only
   # OR
   make dev-all  # Backend + Frontend + PostgreSQL
   ```

4. **Test**:
   ```bash
   cargo test --lib  # Unit tests
   cargo test --test integration  # Integration tests
   cargo test --test e2e  # E2E tests
   ```

5. **Format & Lint**:
   ```bash
   make format
   make lint
   ```

---

## 📋 Deliverables Checklist

### Priority 1: Work Reports & Technical Inspections (Issue #134)
- [ ] `work_report_repository.rs` (trait)
- [ ] `work_report_repository_impl.rs` (PostgreSQL)
- [ ] `technical_inspection_repository.rs` (trait)
- [ ] `technical_inspection_repository_impl.rs` (PostgreSQL)
- [ ] `work_report_use_cases.rs` (10 methods)
- [ ] `technical_inspection_use_cases.rs` (10 methods)
- [ ] `work_report_dto.rs` (2 DTOs)
- [ ] `technical_inspection_dto.rs` (2 DTOs)
- [ ] `work_report_handlers.rs` (10 endpoints)
- [ ] `technical_inspection_handlers.rs` (10 endpoints)
- [ ] Routes wired in `routes.rs`
- [ ] AppState updated in `app_state.rs` + `main.rs`
- [ ] 5 mod.rs files updated

**Total**: ~1,500-2,000 lines of code, 20 REST endpoints

### Priority 2: PDF Extension (Issue #47)
- [ ] Contract PDF template
- [ ] Financial report PDF template
- [ ] 2 new PDF endpoints

### Priority 3: Rate Limiting (Issue #78)
- [ ] actix-governor dependency added
- [ ] Rate limiting middleware configured
- [ ] Applied to login + API endpoints

---

## 🎯 Success Metrics

- ✅ All endpoints return 200 OK for valid requests
- ✅ All endpoints return appropriate 4xx/5xx for errors
- ✅ `cargo build --release` succeeds
- ✅ `cargo test` passes 100%
- ✅ `cargo clippy -- -D warnings` passes
- ✅ API documentation generated (Swagger UI accessible)

---

## 💡 Tips for Claude Web

1. **Start with Repository Pattern**: Study `quote_repository_impl.rs` - it's the most similar to what you need to build.

2. **Follow Hexagonal Architecture Strictly**:
   - Domain = Pure business logic (ALREADY DONE)
   - Application = Ports (traits) + Use Cases + DTOs
   - Infrastructure = Adapters (PostgreSQL, REST API)

3. **Use Existing Patterns**: Don't reinvent the wheel. Copy-paste from similar implementations (quotes, payments, tickets).

4. **SQL Queries**: Use SQLx with compile-time verification. Test queries with:
   ```bash
   cargo sqlx prepare
   ```

5. **Error Handling**: Always return `Result<T, String>` and provide user-friendly error messages.

6. **GDPR Compliance**: All database writes should trigger audit logs (see existing handlers).

7. **Multi-tenancy**: Always filter by `organization_id` in repository queries.

---

## 📞 Questions?

If stuck, check:
- `backend/README.md` - Architecture overview
- `docs/ROADMAP_PAR_CAPACITES.rst` - Feature roadmap
- `CLAUDE.md` - Project instructions
- `GAP_ANALYSIS.md` - Gap analysis report (this prompted your work)

---

**Good luck! 🚀**

**Estimated Time**: 2-3 days for Priority 1 + 2 + 3
