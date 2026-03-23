==========================================================
R&D: Architecture BI Dashboard — Pipeline d'agrégation
==========================================================

Issue: #224
Status: Design Phase
Phase: Jalon 3 (Features Différenciantes)
Date: 2026-03-23

.. contents::
   :depth: 3

Overview
========

KoproGo BI Dashboard provides role-specific business intelligence for three user types:

* **Syndics** (property managers): occupancy, maintenance metrics, payment collection, budget variance
* **Accountants** (financial professionals): cash flow analysis, P&L vs budget, aging receivables, PCMN reporting
* **SuperAdmins** (platform operators): SaaS metrics (MRR, churn, buildings, users), API usage patterns

Current State
=============

**Data availability** ✓:
  * PostgreSQL domain entities (Buildings, Units, UnitOwners, Expenses, Meetings, Payments, etc.)
  * IoT readings (Linky smart meters, temperature sensors - Issue #133)
  * Payment history (Stripe integration - Issue #84)
  * Technical inspections (work reports - Issue #134)
  * Meeting records (AG sessions, voting - Issue #46, #88)

**Missing**:
  * Aggregation layer (materialized views, reporting database)
  * BI frontend (charts, dashboards, drill-down)
  * Role-based metric definitions
  * Real-time data refresh pipeline

Technology Stack Decision
=========================

Evaluation Matrix
-----------------

+--------------------+----------+---------+----------+-------+------+
| Solution           | Language | Hosting | GDPR EU  | Cost  | Self |
+====================+==========+=========+==========+=======+======+
| **Apache Superset**| Python   | Docker  | ✓        | Free  | ✓    |
+--------------------+----------+---------+----------+-------+------+
| Grafana            | Go       | Docker  | ✓        | Free  | ✓    |
+--------------------+----------+---------+----------+-------+------+
| Tableau            | Java     | Cloud   | ⚠        | 70€+  | ✗    |
+--------------------+----------+---------+----------+-------+------+
| Looker             | Java     | Cloud   | ⚠        | 1000€+| ✗    |
+--------------------+----------+---------+----------+-------+------+
| Metabase           | Clojure  | Docker  | ✓        | Free  | ✓    |
+--------------------+----------+---------+----------+-------+------+
| Custom (Recharts)  | Svelte   | Built-in| ✓        | Dev   | ✓    |
+--------------------+----------+---------+----------+-------+------+

**Recommended Approach: Custom Svelte + Recharts Dashboard**

**Rationale**:

1. **No New Infrastructure**: Leverages existing Astro + Svelte frontend
2. **GDPR-Native**: All data stays within KoproGo PostgreSQL, no external SaaS
3. **Fast Development**: Recharts learning curve < 1 week
4. **User Control**: Customizable metrics per role, no black-box SaaS
5. **Cost**: Zero (vs. Superset Docker overhead)
6. **Integration**: Seamless with existing JWT auth and role-based access

**Alternative (If Need Pivot)**:

* **Apache Superset** for heavy self-hosted BI (requires Docker container + PostgreSQL read replica)
* **Grafana** for monitoring metrics (already deployed for infrastructure monitoring)

Architecture Design
===================

Data Pipeline
-------------

.. code-block:: text

    PostgreSQL (Transactional)
            ↓ (scheduled jobs)
    Materialized Views Layer
            ↓ (SELECT * FROM mv_*)
    REST API Endpoints (/api/v1/analytics/*)
            ↓ (Recharts + D3.js)
    Svelte Components (Dashboard Pages)
            ↓
    PDF Export (ReportLab or Typst)

Materialized Views Strategy
-----------------------------

Pre-computed aggregations refresh **nightly** or **on-demand**:

.. code-block:: sql

    -- Monthly expenses by category (syndic dashboard)
    CREATE MATERIALIZED VIEW mv_monthly_expenses AS
    SELECT
        building_id,
        DATE_TRUNC('month', expense_date)::date AS month,
        expense_category,
        COUNT(*) as count,
        SUM(amount_cents) as total_amount_cents,
        AVG(amount_cents) as avg_amount_cents
    FROM expenses
    WHERE organization_id = $1
    GROUP BY building_id, month, expense_category
    ORDER BY month DESC;

    -- Payment collection rate (accountant dashboard)
    CREATE MATERIALIZED VIEW mv_payment_collection AS
    SELECT
        building_id,
        owner_id,
        COALESCE(SUM(CASE WHEN status = 'Succeeded' THEN amount_cents ELSE 0 END), 0)::numeric / 100 as paid_total,
        SUM(oc.amount::numeric) / 100 as due_total,
        ROUND(100.0 * COALESCE(SUM(CASE WHEN status = 'Succeeded' THEN amount_cents ELSE 0 END), 0) /
              NULLIF(SUM(oc.amount::numeric), 0), 2) as collection_rate
    FROM payments p
    FULL OUTER JOIN owner_contributions oc ON p.owner_id = oc.owner_id
    WHERE organization_id = $1
    GROUP BY building_id, owner_id;

    -- IoT consumption trends (sustainability)
    CREATE MATERIALIZED VIEW mv_iot_consumption_trends AS
    SELECT
        building_id,
        DATE_TRUNC('day', timestamp)::date AS day,
        device_type,
        metric_type,
        AVG(value) as avg_value,
        MAX(value) as max_value,
        MIN(value) as min_value
    FROM iot_readings
    WHERE organization_id = $1
    GROUP BY building_id, day, device_type, metric_type;

REST API Layer
--------------

New endpoints under ``/api/v1/analytics/``:

.. code-block:: rust

    // Syndic Dashboard
    GET /api/v1/analytics/buildings/:id/overview
    GET /api/v1/analytics/buildings/:id/maintenance/tickets
    GET /api/v1/analytics/buildings/:id/payments/collection
    GET /api/v1/analytics/buildings/:id/expenses/trends

    // Accountant Dashboard
    GET /api/v1/analytics/organizations/:id/cash-flow
    GET /api/v1/analytics/organizations/:id/pl-vs-budget
    GET /api/v1/analytics/organizations/:id/receivables/aging
    GET /api/v1/analytics/organizations/:id/accounts/top-variance

    // SuperAdmin Dashboard
    GET /api/v1/analytics/saas/mrr
    GET /api/v1/analytics/saas/churn-rate
    GET /api/v1/analytics/saas/api-usage
    GET /api/v1/analytics/saas/buildings-by-region

Database Schema
---------------

.. code-block:: sql

    -- Support table for custom dashboards
    CREATE TABLE dashboard_widgets (
        id UUID PRIMARY KEY,
        organization_id UUID NOT NULL REFERENCES organizations(id),
        user_id UUID NOT NULL REFERENCES users(id),
        dashboard_name VARCHAR(255) NOT NULL,
        widget_type VARCHAR(50),        -- 'line_chart', 'bar_chart', 'kpi', 'table'
        metric_query VARCHAR,           -- SELECT query on materialized views
        position_x INT,
        position_y INT,
        width INT,                      -- grid width (1-12)
        height INT,                     -- grid height (1-6)
        refresh_interval_minutes INT,   -- auto-refresh setting
        created_at TIMESTAMPTZ,
        updated_at TIMESTAMPTZ,
        UNIQUE (organization_id, user_id, dashboard_name)
    );

Key Metrics by Role
===================

Syndic Dashboard
----------------

**Occupancy & Tenancy Metrics**:

* **Active Units**: COUNT(units WHERE no_vacancy_date IS NULL)
* **Occupancy Rate**: (occupied_units / total_units) × 100%
* **Turnover Rate**: transfers_last_12mo / avg_units
* **Average Holding Period**: 1 / turnover_rate

**Maintenance Performance**:

* **Open Tickets**: COUNT(tickets WHERE status IN ('Open', 'Assigned', 'InProgress'))
* **Overdue Tickets**: COUNT(tickets WHERE due_date < NOW AND status != 'Closed')
* **Avg Resolution Time**: AVG(EXTRACT(DAY FROM closed_at - created_at)) for closed tickets
* **First Response Time**: AVG(EXTRACT(HOUR FROM assigned_at - created_at))
* **MTBF** (Mean Time Between Failures): days_in_period / num_incidents

**Payment Collection**:

* **Collection Rate**: paid_amount / invoiced_amount
* **Days Sales Outstanding (DSO)**: (accounts_receivable / revenue) × 365
* **Aging Buckets**: payments by (0-30d, 31-60d, 61-90d, 90d+)
* **Payment Methods Used**: % card vs. SEPA debit vs. bank transfer

**Budget Variance**:

* **YTD Variance**: (actual_expenses - budgeted_amount) / budgeted_amount
* **Category Overruns**: Identify top 5 spending categories above budget
* **Burn Rate**: monthly_avg_spend projected to year-end
* **Forecast Accuracy**: (budget - actual) vs. historical std dev

**Sustainability**:

* **Energy Consumption**: kWh/m²/month from IoT
* **Carbon Intensity**: gCO2/m²/year from Linky
* **Renewable %**: green_energy_pct from energy campaigns

Accountant Dashboard
--------------------

**Cash Flow Statement**:

* **Opening Balance**: balance from prior month
* **Cash Inflows**: payments, deposits, interest
* **Cash Outflows**: expenses, refunds, transfers
* **Closing Balance**: cash available
* **Operating Cash Flow**: from mv_payment_collection + mv_monthly_expenses
* **Free Cash Flow**: OCF - capital expenditures

**P&L vs Budget**:

* **Revenue**: SUM(calls_for_funds, special_assessments, interest) YTD
* **COGS**: Maintenance + utilities + staff
* **Gross Margin**: (Revenue - COGS) / Revenue
* **Operating Expenses**: Insurance + admin + professional fees
* **EBITDA**: Operating profit before depreciation
* **Budget vs Actual**: variance % by expense category

**Receivables Aging**:

.. list-table::
   :header-rows: 1

   * - Bucket
     - Days Overdue
     - Count
     - Amount
     - % of Total
   * - Current
     - 0-30
     - 45
     - €12,500
     - 42%
   * - 31-60
     - 31-60
     - 12
     - €4,200
     - 14%
   * - 61-90
     - 61-90
     - 8
     - €2,100
     - 7%
   * - 90+
     - 90+
     - 5
     - €1,200
     - 4%

**PCMN Account Reconciliation**:

* **Account Balances**: Current balance by account code (PCMN hierarchy)
* **Reconciliation Status**: Matched vs. unmatched transactions
* **Variance to GL**: Discrepancies in journal entries
* **Depreciation Schedule**: Asset depreciation tracking

SuperAdmin Dashboard
--------------------

**SaaS Metrics** (KPI):

* **Monthly Recurring Revenue (MRR)**: ∑ active_organizations × subscription_fee
* **Customer Acquisition Cost (CAC)**: Marketing spend / new_customers
* **Lifetime Value (LTV)**: avg_revenue_per_organization × avg_customer_lifetime
* **Churn Rate**: (churned_orgs / start_orgs) × 100%
* **Net Revenue Retention (NRR)**: (MRR_end + expansion - contraction - churn) / MRR_start

**Usage Metrics**:

* **Buildings by Region**: MAP (Wallonie, Bruxelles, Flandre)
* **Users by Role**: COUNT by (Syndic, Accountant, Owner, Contractor)
* **API Call Volume**: requests/hour (by endpoint, by organization)
* **Database Utilization**: rows_count by entity
* **Error Rates**: % 5xx responses by endpoint

**Infrastructure Health**:

* **API Latency P99**: < 5ms target (monitored via Prometheus)
* **Database Connections**: Current vs. max pool
* **Storage Usage**: GB used vs. quota
* **Backup Status**: Last backup timestamp, success/fail

Implementation Plan
===================

Phase 1: Accounting KPIs (3-4 weeks)
------------------------------------

**Goals**: Accountant dashboard MVP

**Deliverables**:

1. Materialized Views (SQL)
   * ``mv_monthly_expenses``
   * ``mv_payment_collection``
   * ``mv_pcmn_account_balances``

2. REST API Endpoints
   * ``GET /api/v1/analytics/organizations/:id/cash-flow``
   * ``GET /api/v1/analytics/organizations/:id/pl-vs-budget``
   * ``GET /api/v1/analytics/organizations/:id/receivables/aging``

3. Svelte Components
   * ``AccountantDashboard.svelte`` (main layout)
   * ``CashFlowChart.svelte`` (line chart with Recharts)
   * ``AgingTable.svelte`` (sortable table)

4. Testing
   * Unit tests for metric calculations
   * Integration tests with PostgreSQL materialized views
   * E2E test of dashboard page

**Timeline**:

* Week 1: Design materialized views, create migration
* Week 2: REST API implementation
* Week 3: Svelte components + styling
* Week 4: Testing + refinement

Phase 2: Maintenance & Payment Metrics (3-4 weeks)
---------------------------------------------------

**Goals**: Syndic dashboard MVP

**Deliverables**:

1. Materialized Views
   * ``mv_ticket_statistics`` (resolution time, overdue count)
   * ``mv_payment_trends`` (collection rate over time)
   * ``mv_budget_variance`` (actual vs. budget by category)

2. REST API Endpoints
   * ``GET /api/v1/analytics/buildings/:id/maintenance/tickets``
   * ``GET /api/v1/analytics/buildings/:id/payments/collection``
   * ``GET /api/v1/analytics/buildings/:id/expenses/trends``

3. Svelte Components
   * ``SyndicDashboard.svelte``
   * ``TicketPerformanceChart.svelte``
   * ``PaymentCollectionGauge.svelte``

4. Testing
   * Bulk data simulation (1000 tickets, 10k payments)
   * Performance testing (materialized view refresh < 5s)

**Timeline**: 3-4 weeks after Phase 1

Phase 3: Predictive Analytics & Exports (4-6 weeks)
----------------------------------------------------

**Goals**: Advanced features for power users

**Features**:

1. Forecasting
   * Linear regression on expense trends → annual projection
   * Seasonal decomposition (summer utilities vs. winter)
   * Anomaly detection (outlier expenses)

2. Custom Dashboard Builder
   * Drag-and-drop widget layout
   * Custom metric queries (SQL UI)
   * Scheduled PDF reports (weekly/monthly)

3. Export Formats
   * PDF reports (Typst templates)
   * Excel workbooks (calamine crate)
   * CSV exports for external tools

4. Benchmarking
   * Compare vs. peer buildings (anonymized aggregates)
   * Industry standards (energy intensity, maintenance cost/unit)

**Dependencies**: Phase 1 + Phase 2 complete

Recharts Integration Example
=============================

Line Chart: Monthly Expense Trends
-----------------------------------

.. code-block:: svelte

    <script>
        import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

        export let data = [];  // from API: [{ month: '2026-01', amount: 5000 }, ...]
    </script>

    <ResponsiveContainer width="100%" height={400}>
        <LineChart data={data} margin={{ top: 5, right: 30, left: 0, bottom: 5 }}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="month" />
            <YAxis />
            <Tooltip formatter={(value) => `€${value.toLocaleString()}`} />
            <Legend />
            <Line type="monotone" dataKey="amount" stroke="#8884d8" name="Expenses" />
        </LineChart>
    </ResponsiveContainer>

Bar Chart: Aging Receivables
------------------------------

.. code-block:: svelte

    <script>
        import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

        export let data = [];  // [{ bucket: '0-30d', amount: 12500, count: 45 }, ...]
    </script>

    <ResponsiveContainer width="100%" height={400}>
        <BarChart data={data}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="bucket" />
            <YAxis yAxisId="left" label={{ value: 'Amount (€)', angle: -90, position: 'insideLeft' }} />
            <YAxis yAxisId="right" orientation="right" label={{ value: 'Count', angle: 90, position: 'insideRight' }} />
            <Tooltip />
            <Legend />
            <Bar yAxisId="left" dataKey="amount" fill="#82ca9d" name="Amount (€)" />
            <Bar yAxisId="right" dataKey="count" fill="#ffc658" name="Count" />
        </BarChart>
    </ResponsiveContainer>

KPI Card: Collection Rate
---------------------------

.. code-block:: svelte

    <script>
        export let title = '';
        export let value = 0;  // e.g., 85.5
        export let unit = '';  // e.g., '%'
        export let trend = 0;  // +5.2 (positive) or -2.1 (negative)
        export let target = 0; // e.g., 90
    </script>

    <div class="bg-white p-6 rounded-lg shadow">
        <h3 class="text-gray-500 text-sm font-semibold uppercase">{title}</h3>
        <div class="mt-2 flex items-baseline">
            <span class="text-3xl font-bold text-gray-900">{value}</span>
            <span class="ml-2 text-sm text-gray-500">{unit}</span>
        </div>
        <div class="mt-2 flex items-center text-sm" class:text-green-600={trend > 0} class:text-red-600={trend < 0}>
            {#if trend > 0}↑{/if}{#if trend < 0}↓{/if}
            {Math.abs(trend)}% vs last month
        </div>
        <div class="mt-2 text-xs text-gray-400">Target: {target}{unit}</div>
        {#if value < target}
            <div class="mt-2 bg-red-100 text-red-800 text-xs px-2 py-1 rounded">
                Below target by {target - value}{unit}
            </div>
        {/if}
    </div>

Security & GDPR Considerations
===============================

**Data Anonymization**:

* Dashboards show aggregated metrics (SUM, AVG, COUNT) only
* NO personal data (owner names, emails, phone numbers) in charts
* Syndic dashboard limited to their own building(s)
* Accountant dashboard limited to their organization
* SuperAdmin access to aggregated data only (no personal details)

**Access Control**:

* Middleware checks user role before serving analytics endpoints
* Matrix:

  +-----------+-----+-----+------+
  | Role      | Own | Org | All  |
  +===========+=====+=====+======+
  | Syndic    | ✓   | ✗   | ✗    |
  +-----------+-----+-----+------+
  | Accountant| ✗   | ✓   | ✗    |
  +-----------+-----+-----+------+
  | SuperAdmin| ✗   | ✗   | ✓    |
  +-----------+-----+-----+------+

**Materialized View Refresh**:

* Nightly batch refresh (3 AM UTC, during low traffic)
* Alternatively: on-demand refresh with 5-minute cache (Redis)
* Log all queries for audit trail (Issue #89 - monitoring)

**Data Retention**:

* Materialized views preserve historical data for trend analysis
* Archive old views annually (e.g., mv_2025_expenses)
* GDPR Art. 17 support: when owner deleted, re-aggregate metrics

Performance Considerations
===========================

**Materialized View Refresh Performance**:

.. code-block:: sql

    -- Refresh statistic (pre-computed aggregates)
    -- Expected time: < 5 seconds for typical copropriété (50 units)

    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_monthly_expenses;

    -- If slow, add indexes:
    CREATE INDEX idx_expenses_date_category
        ON expenses(organization_id, expense_date, expense_category);
    CREATE INDEX idx_payments_status_date
        ON payments(organization_id, status, created_at DESC);

**Recharts Chart Rendering**:

* Data limit: < 500 data points per chart (performance)
* For larger datasets: aggregate on server (GROUP BY month, year, etc.)
* Tooltip performance: use ``wrapperStyle={{ pointerEvents: 'none' }}`` to avoid re-renders

**API Caching Strategy**:

.. code-block:: rust

    // Cache analytics responses for 5 minutes
    #[get("/analytics/buildings/{building_id}/overview")]
    async fn get_building_overview(
        building_id: web::Path<Uuid>,
        cache: web::Data<RedisClient>,
    ) -> Result<HttpResponse> {
        let cache_key = format!("dashboard:building:{}:overview", building_id);

        // Try cache first
        if let Ok(cached) = cache.get::<String>(&cache_key) {
            return Ok(HttpResponse::Ok().json(serde_json::from_str(&cached)?));
        }

        // Compute fresh
        let data = /* query materialized views */;
        cache.set_ex(&cache_key, serde_json::to_string(&data)?, 300)?;

        Ok(HttpResponse::Ok().json(data))
    }

Risks & Mitigations
====================

+---------------------+-------------------------------+---------------------------+
| Risk                | Impact                        | Mitigation                |
+=====================+===============================+===========================+
| Materialized view   | Stale metrics, user confusion  | Nightly refresh + clear   |
| staleness           |                               | timestamp on dashboard    |
+---------------------+-------------------------------+---------------------------+
| Query performance   | Slow dashboard, timeout       | Index on aggregation      |
| degradation         |                               | columns, time series      |
|                     |                               | partitioning              |
+---------------------+-------------------------------+---------------------------+
| Over-aggregation    | Loss of granularity detail    | Support drill-down from   |
|                     |                               | summary → detailed table  |
+---------------------+-------------------------------+---------------------------+
| User errors in      | Misleading analysis, wrong    | Validation, read-only     |
| queries (Phase 3)   | decisions                     | query builder UI          |
+---------------------+-------------------------------+---------------------------+
| Recharts bundle     | Larger frontend JS            | Tree-shake unused charts, |
| size bloat          |                               | async load on demand      |
+---------------------+-------------------------------+---------------------------+

Next Steps
==========

1. **Stakeholder Approval**: Review with 3-5 early syndics (validate metric usefulness)
2. **Prototype Phase 1**: Build accountant dashboard MVP (cash flow only)
3. **Alpha Testing**: Deploy to 2-3 beta organizations
4. **Iterate**: Refine based on feedback
5. **Scale**: Roll out to all organizations

Success Criteria
================

* **Adoption**: 50%+ of syndics log in to dashboard monthly
* **Performance**: Chart rendering < 1 second
* **Accuracy**: Dashboard metrics match manual GL reconciliation
* **Retention**: 30-day retention rate > 70%
