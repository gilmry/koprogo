====================================================
R&D: API Publique v2 — OpenAPI, SDK Multi-langages
====================================================

Issues: #111, #232
Status: Design Phase
Phase: Jalon 5 (Mobile & API Publique)

.. contents::
   :depth: 2

Overview
========

KoproGo will publish a **public API v2** for third-party integrations:

1. **PropTech integrations** (accounting software, document management, CRM)
2. **Notary platforms** (états datés requests, property transfer data)
3. **Insurance companies** (building risk assessment, claims data)
4. **Energy providers** (campaign participation, consumption reporting)
5. **Government agencies** (statistics, regulatory compliance)

This R&D documents design principles, endpoint priorities, and implementation roadmap.

Current API (v1) — Internal Only
=================================

The current **511 endpoints** at ``/api/v1/`` are designed for **internal use** (SPA frontend):

**Characteristics**:

- **Authentication**: JWT session tokens (short-lived, user-scoped)
- **Rate limiting**: None (assumes trusted client)
- **Versioning**: None (breaking changes require frontend redeploy)
- **Documentation**: In CLAUDE.md (not machine-readable)
- **SDKs**: None (SPA makes direct HTTP calls)
- **Changelog**: Git history only (no formal versioning)

**Why unsuitable for third-party use**:

- JWT tokens tied to user sessions (can't be revoked per-app)
- No API key concept (can't grant least-privilege access)
- No rate limiting (prevents abuse/DoS)
- Breaking changes crash integrations (no deprecation period)
- No formal SLAs (no uptime guarantees)

Why API v2?
===========

**Market demand**:

- **Accounting software** (Sage, Zoho Books): Want to import KoproGo expenses
- **Notaries** (Belfius Notaire): Want automated état daté generation
- **Insurance** (AXA Belgium): Want building specs for risk scoring
- **Energy brokers** (Gazelec, Engie): Want consumption data for campaigns
- **PropTech startups** (Padsplit, Coloc): Want to integrate co-living management

**Strategic value**:

- Increases platform stickiness (hard to leave if integrated with other tools)
- Creates network effects (more integrations = more users)
- Opens revenue stream (API premium tier = EUR 100-1K/year)
- Reduces support burden (integrations automate manual workflows)

API v2 Design Principles
========================

1. **OpenAPI 3.1 specification**
   - Machine-readable (auto-generates SDKs, interactive docs)
   - Industry standard (Swagger, ReDoc support)
   - Version controlled (git history of schema changes)

2. **API Keys (long-lived, scoped)**
   - Format: ``kpg_live_xxxxx...`` (prefix indicates environment)
   - Stored hashed in database (never sent plaintext after creation)
   - Can be revoked without affecting user sessions
   - Scoped to specific resources (``buildings:read``, ``expenses:write``)

3. **Rate limiting**
   - Free tier: 100 requests/min (600K/month)
   - Professional tier: 1,000 requests/min (60M/month)
   - Enterprise: Custom limits (negotiated)
   - Headers: ``X-RateLimit-Remaining``, ``X-RateLimit-Reset``

4. **Webhooks (push events)**
   - Instead of polling, KoproGo pushes events to third-party
   - Events: ``expense.created``, ``meeting.held``, ``resolution.closed``
   - Retry logic (exponential backoff, max 48 hours)
   - Signature verification (HMAC-SHA256)

5. **Versioning (semantic versioning)**
   - Current API: ``v2.0.0`` at release
   - Breaking changes: ``v2.1.0`` → ``v3.0.0`` (12-month deprecation)
   - Non-breaking additions: ``v2.0.0`` → ``v2.1.0`` (transparent to clients)
   - Sunset policy: Old versions deprecated after 18 months

6. **Pagination (cursor-based for large datasets)**
   - Offset/limit can be inefficient for large tables (millions of rows)
   - Cursor-based: Resume from last record (deterministic, efficient)
   - Example: ``GET /v2/expenses?limit=100&after=cursor_xyz``

7. **Webhooks with retries**
   - Automatic retries for failed deliveries (48-hour window)
   - Exponential backoff: 1s → 5s → 30s → 5min → 30min
   - Dead-letter queue for repeated failures

Priority Endpoints for v2
=========================

**Priority 1: Notary & Legal (highest demand)**

Notaries need automated access to:

.. code-block:: text

   GET  /v2/buildings/{id}
   GET  /v2/buildings/{id}/units
   GET  /v2/units/{id}/owners
   GET  /v2/etats-dates/{reference}
   POST /v2/etats-dates
   GET  /v2/etats-dates/{id}/pdf

Use case: Notary app queries building for sale, auto-generates état daté.

**Priority 2: Accounting/Financial (second wave)**

Accountants need:

.. code-block:: text

   GET  /v2/organizations/{id}/expenses
   GET  /v2/organizations/{id}/payments
   GET  /v2/organizations/{id}/accounts
   GET  /v2/reports/balance-sheet
   GET  /v2/reports/income-statement
   POST /v2/expenses/{id}/export-accounting

Use case: Monthly export of charges for accounting software (Sage).

**Priority 3: Energy & Sustainability**

Energy campaigns need:

.. code-block:: text

   GET  /v2/energy-campaigns/{id}
   GET  /v2/energy-campaigns/{id}/stats
   POST /v2/energy-bills/upload
   GET  /v2/energy-bills/{id}/decrypt

Use case: Energy broker gets anonymized consumption stats for campaign.

**Priority 4: Community/Integration (future)**

Community features need:

.. code-block:: text

   GET  /v2/buildings/{id}/sel-exchanges
   GET  /v2/buildings/{id}/announcements
   GET  /v2/users/{id}/gamification-stats
   POST /v2/webhooks/subscribe

Use case: Community app embeds KoproGo announcements and SEL listings.

API Key System Design
====================

**Database schema**:

.. code-block:: sql

   CREATE TABLE api_keys (
       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       organization_id UUID NOT NULL REFERENCES organizations(id),
       key_hash        VARCHAR(255) NOT NULL UNIQUE,        -- SHA256(key)
       key_prefix      VARCHAR(20) NOT NULL UNIQUE,         -- kpg_live_XXX
       name            VARCHAR(255) NOT NULL,               -- "Sage Export"
       description     TEXT,
       scopes          VARCHAR[] NOT NULL,                  -- ['expenses:read', 'buildings:read']
       rate_limit      INTEGER DEFAULT 100,                 -- req/min
       is_active       BOOLEAN DEFAULT TRUE,
       last_used_at    TIMESTAMPTZ,
       created_by      UUID REFERENCES users(id),           -- Who created
       created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       expires_at      TIMESTAMPTZ,                         -- Optional expiry
       INDEX idx_key_hash ON key_hash,
       INDEX idx_organization_active ON (organization_id, is_active)
   );

   CREATE TABLE api_key_usage (
       id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       api_key_id      UUID NOT NULL REFERENCES api_keys(id),
       endpoint        VARCHAR(255),                         -- GET /v2/expenses
       status_code     INTEGER,
       response_time_ms INTEGER,
       timestamp       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
       INDEX idx_key_timestamp ON (api_key_id, timestamp)
   );

**Key creation flow**:

.. code-block:: rust

   // backend/src/infrastructure/web/handlers/api_key_handlers.rs
   #[post("/v2/api-keys")]
   pub async fn create_api_key(
       user: AuthenticatedUser,
       req: web::Json<CreateApiKeyRequest>,
       repo: web::Data<ApiKeyRepository>,
   ) -> impl Responder {
       // Step 1: Validate scopes (user can only grant permissions they have)
       for scope in &req.scopes {
           if !user_can_grant_scope(&user, scope) {
               return error("Insufficient permission to grant scope: {}", scope);
           }
       }

       // Step 2: Generate random 32-byte key
       let key = generate_secure_random_key();
       let key_hash = sha256(&key);
       let key_prefix = format!("kpg_live_{}", &key[0..8]); // First 8 chars

       // Step 3: Store hashed key (never store plaintext)
       let api_key = ApiKey {
           key_hash,
           key_prefix,
           name: req.name.clone(),
           scopes: req.scopes.clone(),
           rate_limit: req.rate_limit.unwrap_or(100),
           organization_id: user.org_id,
           created_by: user.id,
       };
       repo.create(&api_key).await?;

       // Step 4: Return plaintext key (only once!)
       response(json!({
           "key": format!("{}{}", &key_prefix, &key),
           "message": "Store this key safely. It won't be shown again."
       }))
   }

**API key validation middleware**:

.. code-block:: rust

   // backend/src/infrastructure/auth/api_key_middleware.rs
   pub struct ApiKeyAuth {
       key_hash: String,
       organization_id: Uuid,
       scopes: Vec<String>,
   }

   pub async fn validate_api_key(
       req: &HttpRequest,
       repo: &ApiKeyRepository,
   ) -> Result<ApiKeyAuth> {
       // Extract from header: Authorization: ApiKey kpg_live_xxxxx
       let header = req.headers().get("Authorization")?;
       let auth_value = header.to_str()?;
       let (auth_type, key) = auth_value.split_once(' ')?;

       if auth_type != "ApiKey" {
           return Err("Invalid auth type".into());
       }

       let key_hash = sha256(key);
       let api_key = repo.find_by_hash(&key_hash).await?;

       if !api_key.is_active {
           return Err("API key is inactive".into());
       }

       if let Some(expires_at) = api_key.expires_at {
           if expires_at < Utc::now() {
               return Err("API key expired".into());
           }
       }

       Ok(ApiKeyAuth {
           key_hash,
           organization_id: api_key.organization_id,
           scopes: api_key.scopes,
       })
   }

**Endpoint with scope check**:

.. code-block:: rust

   #[get("/v2/expenses")]
   pub async fn list_expenses(
       auth: ApiKeyAuth,                         // API key auth (not JWT)
       req: web::Query<ListExpensesQuery>,
       repo: web::Data<ExpenseRepository>,
   ) -> impl Responder {
       // Check scope
       if !auth.scopes.contains(&"expenses:read".to_string()) {
           return error("Scope 'expenses:read' required", 403);
       }

       // Fetch expenses for authenticated organization
       let expenses = repo.find_by_organization(&auth.organization_id).await?;
       response(expenses)
   }

Rate Limiting Implementation
============================

Using token bucket algorithm:

.. code-block:: rust

   // backend/src/infrastructure/rate_limit/rate_limiter.rs
   pub struct RateLimiter {
       storage: web::Data<redis::Client>,  // Redis for fast lookups
   }

   impl RateLimiter {
       pub async fn check_and_decrement(
           &self,
           api_key_id: &Uuid,
           limit: u32,
       ) -> Result<RateLimitStatus> {
           let key = format!("ratelimit:{}:{}", api_key_id, current_minute());
           let current = self.storage.incr(&key).await.unwrap_or(0);

           if current > limit as i64 {
               Ok(RateLimitStatus {
                   allowed: false,
                   remaining: 0,
                   reset_at: next_minute(),
               })
           } else {
               Ok(RateLimitStatus {
                   allowed: true,
                   remaining: (limit as i64 - current) as u32,
                   reset_at: next_minute(),
               })
           }
       }
   }

   // Middleware
   pub async fn rate_limit_middleware(
       req: HttpRequest,
       auth: ApiKeyAuth,
       limiter: web::Data<RateLimiter>,
   ) -> Result<(), Error> {
       let status = limiter.check_and_decrement(&auth.key_id, auth.rate_limit).await?;

       // Set response headers
       req.extensions_mut().insert(status);

       if !status.allowed {
           return Err(ErrorTooManyRequests("Rate limit exceeded").into());
       }
       Ok(())
   }

OpenAPI Specification
====================

**schema.yaml** (OpenAPI 3.1):

.. code-block:: yaml

   openapi: 3.1.0
   info:
     title: KoproGo API v2
     description: Property management platform for Belgian copropriétés
     version: 2.0.0
     contact:
       name: KoproGo Support
       email: support@koprogo.be

   servers:
     - url: https://api.koprogo.be/v2
       description: Production
     - url: https://sandbox.koprogo.be/v2
       description: Sandbox (testing)

   components:
     securitySchemes:
       ApiKey:
         type: apiKey
         in: header
         name: Authorization
         description: "Format: ApiKey kpg_live_xxxxx"

   paths:
     /buildings/{id}:
       get:
         summary: Get building details
         operationId: getBuildingPublic
         security:
           - ApiKey: []
         parameters:
           - name: id
             in: path
             required: true
             schema:
               type: string
               format: uuid
         responses:
           '200':
             description: Building details
             content:
               application/json:
                 schema:
                   $ref: '#/components/schemas/Building'
           '404':
             description: Building not found
           '429':
             description: Rate limit exceeded

   /expenses:
     get:
       summary: List expenses
       operationId: listExpenses
       security:
         - ApiKey: []
       parameters:
         - name: building_id
           in: query
           schema:
             type: string
             format: uuid
         - name: limit
           in: query
           schema:
             type: integer
             default: 50
         - name: after
           in: query
           description: Cursor token for pagination
           schema:
             type: string
       responses:
         '200':
           description: List of expenses
           content:
             application/json:
               schema:
                 type: object
                 properties:
                   data:
                     type: array
                     items:
                       $ref: '#/components/schemas/Expense'
                   next_cursor:
                     type: string

SDK Generation
==============

Using **OpenAPI Generator** (https://openapi-generator.tech/):

**TypeScript/JavaScript SDK**:

.. code-block:: bash

   openapi-generator generate \
     -i schema.yaml \
     -g typescript-axios \
     -o sdks/typescript

   # Generates: sdks/typescript/
   #   ├── src/models/Building.ts
   #   ├── src/models/Expense.ts
   #   ├── src/apis/BuildingsApi.ts
   #   ├── src/apis/ExpensesApi.ts
   #   └── package.json (auto-published to npm)

**Usage**:

.. code-block:: typescript

   import { BuildingsApi, Configuration } from '@koprogo/api-v2';

   const config = new Configuration({
       apiKey: 'kpg_live_xxxxx',
   });
   const api = new BuildingsApi(config);

   const building = await api.getBuildingPublic({ id: 'uuid-123' });
   console.log(building.data);

**Python SDK**:

.. code-block:: bash

   openapi-generator generate \
     -i schema.yaml \
     -g python \
     -o sdks/python

**Usage**:

.. code-block:: python

   from koprogo_api import BuildingsApi, Configuration

   config = Configuration(api_key='kpg_live_xxxxx')
   api = BuildingsApi(config)

   building = api.get_building_public(id='uuid-123')
   print(building.name)

**Rust SDK**:

.. code-block:: bash

   openapi-generator generate \
     -i schema.yaml \
     -g rust \
     -o sdks/rust

Webhooks Design
===============

**Webhook payload** (POST to partner endpoint):

.. code-block:: json

   POST https://partner.example.com/koprogo/webhooks
   X-Signature: sha256=xxxxx
   Content-Type: application/json

   {
     "id": "evt_1234567890",
     "timestamp": "2026-03-23T10:30:00Z",
     "event": "expense.created",
     "organization_id": "org-uuid",
     "data": {
       "expense_id": "exp-uuid",
       "building_id": "bld-uuid",
       "title": "Elevator Maintenance",
       "amount_cents": 50000,
       "status": "Draft"
     }
   }

**Signature verification** (partner-side):

.. code-block:: python

   import hmac
   import hashlib
   import json

   def verify_webhook(request, secret):
       signature = request.headers.get('X-Signature', '')
       payload = request.data  # Raw request body

       # Reconstruct signature
       expected = 'sha256=' + hmac.new(
           secret.encode(),
           payload,
           hashlib.sha256
       ).hexdigest()

       return hmac.compare_digest(signature, expected)

**Retry logic**:

.. code-block:: rust

   // backend/src/jobs/webhook_delivery.rs
   pub async fn deliver_webhook(
       webhook_id: Uuid,
       payload: serde_json::Value,
       url: &str,
       secret: &str,
   ) -> Result<()> {
       let signature = generate_signature(&secret, &payload);

       for attempt in 1..=5 {
           match http_client.post(url)
               .header("X-Signature", signature.clone())
               .json(&payload)
               .send()
               .await {
               Ok(resp) if resp.status() == 200 => return Ok(()),
               _ => {
                   let backoff = exponential_backoff(attempt); // 1s, 5s, 30s, 5min, 30min
                   tokio::time::sleep(backoff).await;
               }
           }
       }

       // Failed after 5 retries, move to dead-letter queue
       move_to_dead_letter_queue(webhook_id).await?;
       Ok(())
   }

Implementation Timeline
=======================

**Phase 1: Design & Setup (2 weeks)**

- Finalize OpenAPI specification
- Design API key system (database, middleware)
- Set up documentation site (ReDoc or Swagger UI)

**Phase 2: Core Endpoints (3 weeks)**

- Implement API key authentication
- Build Priority 1 endpoints (notary, building, units, etats-dates)
- Rate limiting middleware
- SDK generation setup

**Phase 3: Financial & Accounting (2 weeks)**

- Priority 2 endpoints (expenses, payments, accounts, reports)
- Export formats (CSV, JSON)
- Webhook infrastructure

**Phase 4: Energy & Community (2 weeks)**

- Priority 3 endpoints (energy campaigns, stats)
- Priority 4 endpoints (SEL, announcements)
- Webhooks for real-time events

**Phase 5: Testing & Documentation (2 weeks)**

- API key security testing
- Webhook delivery testing (with partner simulation)
- SDK integration tests (TypeScript, Python, Rust)
- Developer documentation

**Phase 6: Beta Program (4 weeks)**

- Invite 3-5 PropTech partners
- Real-world testing
- Feedback integration
- Bug fixes

**Total**: ~15 weeks (3.5 months)

Cost Estimation
===============

- **Development**: ~60 developer-days (~3 months)
- **Hosting** (sandbox + production): No additional (same infrastructure)
- **Documentation site**: EUR 50/month (ReDoc CDN or self-hosted)
- **SDK maintenance**: Automated (OpenAPI Generator)

Revenue Potential
=================

**Free tier**: 100 req/min (target: hobbyist developers)

**Professional tier**: EUR 100/month → EUR 1.2K/year
- 1,000 req/min
- Priority support
- SLA 99.5% uptime

**Enterprise tier**: EUR 5-10K/year
- Custom rate limits
- Dedicated support
- Custom webhooks
- Direct contact with engineering

**Expected adoption**: 5-10% of organizations (50-100 API integrations at 1000 users)
**Estimated revenue**: EUR 50-100K/year (Year 1)

Security Considerations
=======================

1. **API Key storage**: Hash with Argon2 (not SHA256 which is fast/reversible)
2. **Key rotation**: Encourage annual rotation (UI reminder)
3. **Scope enforcement**: Never grant scopes user doesn't have
4. **Audit trail**: Log all API key creation/deletion
5. **IP allowlisting** (optional): Premium feature for enterprises
6. **Webhook signature**: HMAC-SHA256 prevents spoofing
7. **API versioning**: Never break existing clients (12-month notice)

Monitoring & Alerting
======================

**Prometheus metrics**:

.. code-block:: text

   koprogo_api_requests_total{endpoint="/v2/expenses", status="200"}
   koprogo_api_request_duration_seconds{endpoint="/v2/expenses"}
   koprogo_api_key_usage{api_key_id="..."}
   koprogo_webhook_delivery_attempts{status="success"}
   koprogo_webhook_delivery_attempts{status="failed"}

**Alerting rules**:

- API latency P95 > 1s (alert: performance degradation)
- Error rate > 5% (alert: availability issue)
- Webhook failures > 10% (alert: partner integration issue)
- Missing API keys (alert: compromised keys?)

Backward Compatibility
======================

API v1 (internal) continues unchanged:

- v1 routes at ``/api/v1/`` (SPA frontend still uses)
- v2 routes at ``/api/v2/`` (third-party integrations)
- No migration required (both coexist)
- v1 sunset planned for Year 3 (18-month notice when announced)

Related Issues
==============

- **#111**: Public API design
- **#232**: SDK generation and deployment
- **#200**: API versioning strategy
- **#201**: Webhook infrastructure
- **#202**: API key management

References
==========

- `OpenAPI 3.1 Specification <https://spec.openapis.org/oas/v3.1.0>`_
- `OpenAPI Generator <https://openapi-generator.tech/>`_
- `JSON API Standard <https://jsonapi.org/>`_
- `REST API Best Practices <https://restfulapi.net/>`_
- `Stripe API Design (reference) <https://stripe.com/docs/api>`_
- `GitHub API v3 (reference) <https://docs.github.com/en/rest/>`_
