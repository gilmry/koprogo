# Critical Issues Analysis

**Branch**: `claude/review-remaining-issues-018z8PJuUPF4CXEuhBN9zV3y`
**Date**: 2025-11-17
**Status**: Integration branch merged successfully (270 → 0 compilation errors ✅)

---

## Summary

**Total Critical Issues**: 14
**Closed** ✅: 4 (29%)
**Open** ❌: 10 (71%)

**Estimated Effort for Remaining**: ~20-30 days (full-time)

---

## ✅ CLOSED Critical Issues (Already Implemented)

### Software Features (4 issues)

1. **#73: Invoice Workflow with VAT** ✅
   - Complete invoice encoding system with approval workflow
   - VAT management (6%, 12%, 21%)
   - Multi-line items with automatic calculations
   - Status: Draft → PendingApproval → Approved/Rejected
   - **Location**: `backend/src/domain/entities/expense.rs`, `invoice_line_item.rs`

2. **#77: Financial Reports Generation** ✅
   - Balance sheet (Bilan)
   - Income statement (Compte de résultats)
   - Based on Belgian PCMN chart
   - **Location**: `backend/src/application/use_cases/financial_report_use_cases.rs`

3. **#79: Belgian Accounting Chart (PCMN)** ✅
   - Complete implementation of Plan Comptable Minimum Normalisé
   - ~90 accounts pre-seeded (8 classes)
   - Hierarchical structure (classes, sub-classes, groups, accounts)
   - **Location**: `backend/src/domain/entities/account.rs`
   - **Documentation**: `docs/BELGIAN_ACCOUNTING_PCMN.rst`

4. **#83: Payment Recovery Workflow** ✅
   - Automated payment reminder system
   - 4 escalation levels: Gentle → Formal → FinalNotice → LegalAction
   - Automatic late payment penalty calculation (8% annual rate)
   - **Location**: `backend/src/domain/entities/payment_reminder.rs`
   - **Documentation**: `docs/PAYMENT_RECOVERY_WORKFLOW.rst`

---

## ❌ OPEN Critical Issues (Need Implementation)

### Priority 1: Legal Compliance & Production Blockers (4 issues)

#### #42: GDPR Data Export & Deletion ⚖️ LEGAL REQUIRED
- **Effort**: 2-3 days
- **Priority**: HIGHEST - Legal requirement for EU deployment
- **Status**: 0% implemented

**Requirements**:
- ✅ Audit logging foundation exists (`infrastructure/audit.rs`)
- ❌ Missing: Right to access (data export API)
- ❌ Missing: Right to erasure (right to be forgotten)
- ❌ Missing: Data portability (JSON export)

**Implementation**:
- New endpoint: `GET /api/v1/gdpr/export` (JSON export of all user data)
- New endpoint: `DELETE /api/v1/gdpr/erase` (soft delete + anonymization)
- Database migration: Add `is_anonymized`, `deletion_requested_at`, `deleted_at` fields
- Anonymization strategy: Keep financial records (7 years Belgium), anonymize linkage
- Cron job: Purge old data after retention period

**Files to Create**:
- `backend/src/application/use_cases/gdpr_use_cases.rs`
- `backend/src/infrastructure/web/handlers/gdpr_handlers.rs`
- `backend/src/application/dto/gdpr_dto.rs`
- `backend/migrations/XXX_add_gdpr_fields.sql`
- `frontend/src/pages/settings/privacy.astro`

---

#### #82: Board of Directors (Conseil de Copropriété) 🚫 BLOCKS PRODUCTION
- **Effort**: 12-15 hours
- **Priority**: CRITICAL - Blocks production for buildings >20 units (majority of Belgian market)
- **Legal**: Article 577-8/4 Belgian Civil Code - MANDATORY for >20 units

**Requirements**:
- New role: `BoardMember` with special permissions
- Entity `BoardMember` (user_id, building_id, position, mandate_start/end)
- Entity `BoardDecision` (subject, decision_text, deadline, status)
- Election workflow (vote in AG)
- Dashboard: Track syndic compliance with AG decisions
- Tracking delays: Quotes (30d), Works (60d), Minutes (30d)
- SQL trigger: Enforce incompatibility (syndic cannot be board member)

**Permissions**:
- Consult all copropriété documents
- Request accounts from syndic
- Convene AG if syndic is deficient
- Verify execution of AG decisions

**Impact**: Without this, cannot deploy for 70%+ of target market

---

#### #80: État Daté Generation 📄 LEGAL REQUIRED
- **Effort**: 6-8 hours
- **Priority**: HIGH - Required for ALL property sales (Article 577-2 Belgian Civil Code)
- **Impact**: BLOCKS ALL PROPERTY SALES without this document

**Requirements**:
- Entity `EtatDate` (building_id, unit_id, reference_date, data JSONB, status)
- PDF generation with 16 mandatory legal sections
- Workflow: Request → Generation (max 15 days) → Delivery
- Endpoints: `POST /units/:id/etat-date`, `GET /etat-dates/:id/pdf`

**16 Legal Sections**:
1. Building and unit identification
2. Share of ordinary/extraordinary charges
3. Owner's financial situation
4. Provision amounts
5. Credit/debit balance
6. Voted but unpaid works
7. Ongoing litigation
8. Building insurance
9. Copropriété regulations
10. Recent AG minutes
11. Budget forecast
12. Reserve fund
13. Copropriété debts/credits
14. Work progress
15. Guarantees and mortgages
16. Miscellaneous observations

**Dependencies**: Requires #79 (PCMN) for financial section ✅ DONE

---

#### #81: Annual Budget System 💰 LEGAL REQUIRED
- **Effort**: 8-10 hours
- **Priority**: HIGH - Legal obligation to vote budget in AG before fiscal year
- **Status**: 0% implemented

**Requirements**:
- Entity `Budget` (fiscal_year, ordinary_budget, extraordinary_budget, status)
- Automatic monthly provision calculation
- Variance analysis (budget vs actual) monthly
- AG vote required before fiscal year start
- Dashboard: Alerts for budget overruns (>10%)

**Budget Structure**:
```rust
pub struct Budget {
    id: Uuid,
    building_id: Uuid,
    fiscal_year: i32,
    ordinary_budget: Decimal,      // Regular charges
    extraordinary_budget: Decimal,  // Works
    status: BudgetStatus,          // Draft, Voted, Active
    approved_at: Option<DateTime>,
    approved_by_meeting_id: Option<Uuid>,
}
```

**Dependencies**: #79 (PCMN for categorization) ✅ DONE

---

### Priority 2: Core Features (3 issues)

#### #75: Meeting Management API 🗳️ CORE FEATURE
- **Effort**: 6-8 hours
- **Priority**: HIGH - Core copropriété functionality
- **Status**: Domain entity exists, no API exposed

**Requirements**:
- 10 REST API endpoints
- Use cases layer (MeetingUseCases)
- DTOs (meeting_dto.rs)
- HTTP handlers (meeting_handlers.rs)
- E2E + BDD tests

**Endpoints**:
- `POST /api/v1/meetings` - Create assembly (Syndic+)
- `GET /api/v1/meetings` - List assemblies (Owner+)
- `GET /api/v1/meetings/:id` - Assembly details (Owner+)
- `PUT /api/v1/meetings/:id` - Update assembly (Syndic+)
- `DELETE /api/v1/meetings/:id` - Delete assembly (SuperAdmin)
- `GET /api/v1/buildings/:id/meetings` - Building's assemblies
- `PUT /api/v1/meetings/:id/publish` - Publish agenda (Syndic)
- `POST /api/v1/meetings/:id/minutes` - Add minutes (Syndic)
- `GET /api/v1/meetings/:id/minutes` - Get minutes (Owner+)
- `PUT /api/v1/meetings/:id/close` - Close assembly (Syndic)

**Blocks**: #19 (AG Convocations), #22 (Board elections)

---

#### #76: Document Upload/Download System 📎 CORE FEATURE
- **Effort**: 8-10 hours
- **Priority**: HIGH - Essential for copropriété document management
- **Status**: 0% implemented

**Requirements**:
- File storage service trait (abstraction)
- Upload/download handlers (multipart form-data)
- Document categorization (Minutes, Invoices, Contracts, etc.)
- Role-based access control
- Svelte component with drag-and-drop
- Image/PDF preview
- E2E tests

**Document Types**:
```rust
pub enum DocumentType {
    MeetingMinutes,    // AG minutes
    Invoice,           // Invoices
    Contract,          // Service contracts
    Regulation,        // Internal regulations
    Insurance,         // Insurance
    WorkReport,        // Work reports
    Other,
}
```

**Endpoints**:
- `POST /api/v1/documents/upload` (Syndic+)
- `GET /api/v1/documents/:id` (Owner+)
- `GET /api/v1/documents` (Owner+)
- `DELETE /api/v1/documents/:id` (Syndic+)
- `GET /api/v1/buildings/:id/documents` (Owner+)

**Frontend**:
```svelte
<FileUploader
  accept=".pdf,.jpg,.png"
  maxSize={10MB}
  onUpload={handleUpload}
  showPreview={true}
/>
```

**Blocks**: #80 (État Daté PDF), #20 (Maintenance log), #24 (Quotes)

---

#### #78: Security Hardening 🔒 PRODUCTION REQUIRED
- **Effort**: 10-12 hours
- **Priority**: HIGH - Required for production deployment
- **Status**: Partial (JWT exists, needs enhancement)

**Requirements**:
- Rate limiting (100 req/min public, 1000 req/min authenticated, 5 login attempts/15min)
- JWT refresh token rotation (access 15min, refresh 7 days)
- CORS strict whitelist
- Structured logging (JSON format)
- 2FA optional (TOTP)
- Audit logs for sensitive actions
- Security headers (HSTS, CSP, X-Frame-Options, etc.)

**Implementation**:
- Rate limiting with Redis or in-memory store
- Refresh token table + rotation logic
- CORS configuration per environment
- Structured logger (tracing-subscriber JSON formatter)
- TOTP library integration (optional feature flag)
- Audit log expansion (currently exists in `infrastructure/audit.rs`)

**Acceptance**:
- Rate limiting active and tested
- Refresh tokens functional
- Audit logs for: Login/Logout, Data modifications, Deletions, GDPR exports
- CORS configured for production
- Security headers validated

---

### Priority 3: Infrastructure (3 issues)

#### #39: LUKS Encryption at Rest 🔐 INFRASTRUCTURE
- **Effort**: 1-2 days
- **Priority**: MEDIUM - GDPR compliance requirement
- **Status**: 0% implemented

**Requirements**:
- LUKS full-disk encryption for VPS data volumes
- Encrypted volumes: PostgreSQL data (`/var/lib/docker/volumes/postgres_data`), uploads
- Key management strategy (TPM 2.0, Tang server, or manual unlock)
- Ansible integration for automated setup
- Auto-unlock on reboot (no manual intervention)

**Performance**:
- Expected overhead: 3-7%
- Acceptable for compliance
- Modern CPUs have AES-NI acceleration

**Testing**:
- Encryption verified with `cryptsetup status`
- Database read/write performance <10% degradation
- VPS reboot with auto-unlock
- Disaster recovery procedure tested

---

#### #40: Encrypted Backups 💾 INFRASTRUCTURE
- **Effort**: 1 day
- **Priority**: MEDIUM - GDPR compliance requirement
- **Status**: Basic backup exists, NO encryption

**Current**: Daily PostgreSQL dumps to local filesystem, no encryption, no off-site

**Requirements**:
- GPG encryption layer for PostgreSQL dumps
- S3 off-site storage (OVH Object Storage recommended)
- Key management procedures
- Restore testing automation (monthly cron)

**Retention Policy**:
- Daily: Keep 7 days
- Weekly: Keep 4 weeks
- Monthly: Keep 12 months
- Yearly: Keep indefinitely

**Implementation**:
```bash
# Backup script
docker exec postgres pg_dump -U koprogo koprogo_db | \
  gzip -9 | \
  gpg --encrypt --recipient backup@koprogo.local \
  > /backups/koprogo_$(date).sql.gz.gpg

# Upload to S3
s3cmd put /backups/*.gpg s3://koprogo-backups/ --server-side-encryption
```

**Dependencies**: #39 (encryption at rest) - Should be done together

---

#### #41: Monitoring Stack 📊 INFRASTRUCTURE
- **Effort**: 3-5 days
- **Priority**: MEDIUM - Production observability
- **Status**: Basic scripts exist (30%), no centralized monitoring

**Current**: Bash scripts + cron jobs (RAM, CPU, disk, PostgreSQL slow queries)

**Requirements**:
- Prometheus (metrics collection & storage)
- Grafana (dashboards & visualization)
- Loki (log aggregation)
- Alertmanager (alert routing & notifications)
- Node Exporter (system metrics)
- PostgreSQL Exporter (database metrics)
- cAdvisor (container metrics)

**Dashboards**:
1. KoproGo overview (system)
2. PostgreSQL metrics
3. Docker containers
4. Traefik HTTP traffic

**Alerts**:
- High CPU (>85% for 5min)
- High Memory (>85% for 5min)
- Low disk space (<20%)
- PostgreSQL down
- PostgreSQL slow queries (P99 > 5ms)
- Container down
- Backup failed (>24h)

**Resource Requirements**:
- ~500MB RAM
- ~15GB disk (30d metrics, 7d logs)
- Acceptable on 2GB VPS (25% overhead)

**Backend Integration**:
- Add `/metrics` endpoint (Prometheus format)
- Add `prometheus` crate to Cargo.toml

---

## Recommended Implementation Order

### Phase 1: Legal Compliance & Production Blockers (2-3 weeks)
1. **#42 GDPR** (2-3 days) - Highest legal priority
2. **#78 Security Hardening** (10-12h) - Production security
3. **#82 Board of Directors** (12-15h) - Unblocks >20 unit buildings
4. **#80 État Daté** (6-8h) - Required for property sales
5. **#81 Budget System** (8-10h) - Legal requirement

### Phase 2: Core Features (1.5-2 weeks)
6. **#75 Meeting Management** (6-8h) - Core functionality
7. **#76 Document Management** (8-10h) - Core functionality

### Phase 3: Infrastructure (1 week)
8. **#39 LUKS Encryption** (1-2 days) - Can be done in parallel
9. **#40 Encrypted Backups** (1 day) - Depends on #39
10. **#41 Monitoring Stack** (3-5 days) - Can be done in parallel

**Total Estimated Time**: 4-6 weeks (full-time) or 2-3 months (part-time)

---

## Quick Wins (Can be done immediately)

1. **#75 Meeting Management API** (6-8h) - Domain entity exists, just need API layer
2. **#78 Security Hardening** (10-12h) - JWT already exists, needs enhancement
3. **#81 Budget System** (8-10h) - PCMN already implemented, just add budget logic

---

## Next Steps

**Immediate Action**:
- Start with **#42 GDPR** (highest legal priority)
- OR **#75 Meeting Management** (quickest win, unlocks other features)

**Question for Product Owner**:
- What is the target deployment timeline?
- Are we targeting buildings >20 units immediately? (determines #82 priority)
- Are property sales a near-term use case? (determines #80 priority)

---

**Analysis completed on**: `claude/review-remaining-issues-018z8PJuUPF4CXEuhBN9zV3y` branch
