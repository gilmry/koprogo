==========================================================
R&D: Empreinte carbone et reporting ESG/durabilité
==========================================================

Issue: #229
Status: Design Phase
Phase: Jalon 3 (Features Différenciantes)
Date: 2026-03-23

.. contents::
   :depth: 3

Overview
========

KoproGo implements **Environmental, Social, and Governance (ESG)** reporting for Belgian properties, supporting:

* **Carbon footprint tracking** (Scope 1, 2, 3 emissions)
* **Energy performance metrics** (compliance with Belgian RE2050 requirements)
* **Sustainability reporting** (DPE/PEB integration, EU taxonomy alignment)
* **Green energy tracking** (renewable % from energy buying groups - Issue #133)
* **KoproGo platform carbon neutrality** (< 0.5g CO2/request target from CLAUDE.md)

Regulatory Context
==================

Belgian Legal Requirements
---------------------------

1. **RE2050 Law (Energy Transition)** [L'Accord Régional Énergie 2050]

   * **Wallonie**: Buildings reduce energy by 40% by 2030 (vs. 2010 baseline)
   * **Bruxelles**: -40% by 2030, -55% by 2040, -80% by 2050
   * **Flandre**: -35% by 2030 (EU RED III compliance)
   * Mandatory annual energy audits for buildings > 250 m²
   * Reporting to regional authorities (Bruxelles-Environnement, SPW Énergie, etc.)

2. **PEB/DPE (Certificat de Performance Énergétique)**

   * Required for property sales, leases, and rental advertising
   * Shows energy class: A (best) → G (worst)
   * Updated every 10 years (mandatory)
   * Value added to property listing (legal requirement for sale)
   * Linked to Unit.dpe_class (domain entity field)

3. **EU Taxonomy Regulation** [Regulation (EU) 2020/852]

   * Applies to large companies (> 500 employees OR > €25M revenue OR > €12.5M assets)
   * Real estate classifications (sustainable vs. non-sustainable)
   * Disclosure of CapEx/OpEx in sustainable activities
   * Timeline: 2024 onwards

4. **SFDR (Sustainable Finance Disclosure Regulation)**

   * Large financial institutions investing in real estate must report
   * Article 10 (Article 8 sub-fund) disclosures
   * KoproGo as SaaS may need to provide data to investor-clients (indirect)

Domain Model Extensions
=======================

Existing Entities with ESG Fields
-----------------------------------

**Unit** (existing):

.. code-block:: rust

    pub struct Unit {
        pub id: Uuid,
        pub building_id: Uuid,
        pub unit_number: String,
        pub floor: i32,
        pub area_m2: f32,
        // NEW ESG FIELDS:
        pub dpe_class: Option<String>,          // A-G from PEB
        pub dpe_score: Option<f32>,             // kWh/m²/year
        pub dpe_renewal_date: Option<DateTime>, // 10-year mandate
        pub orientation: Option<String>,        // N, NE, E, SE, S, SW, W, NW (affects solar potential)
        pub insulation_rating: Option<String>,  // Poor, Fair, Good, Excellent
        pub heating_system: Option<String>,     // Boiler, HeatPump, Electric, Geothermal, Hybrid
        pub cooling_system: Option<String>,     // None, AC, HeatPump, RadiantCooling
        pub window_type: Option<String>,        // SinglePane, Double, Triple, etc.
    }

**Building** (existing):

.. code-block:: rust

    pub struct Building {
        pub id: Uuid,
        pub organization_id: Uuid,
        pub name: String,
        pub address: String,
        pub city: String,
        pub total_units: i32,
        pub construction_year: i32,
        // NEW ESG FIELDS:
        pub total_area_m2: f32,
        pub energy_label: Option<String>,       // Building-level PEB (average of units)
        pub renewable_energy_sources: Vec<String>,  // ["Solar", "Geothermal", "HeatPump"]
        pub ev_charging_stations: i32,          // Count available
        pub green_space_m2: Option<f32>,
        pub district_heating: bool,
        pub epc_audit_date: Option<DateTime>,
        pub epc_audit_next_due: Option<DateTime>,
    }

**New Entity: EnergyConsumption**

.. code-block:: rust

    pub struct EnergyConsumption {
        pub id: Uuid,
        pub building_id: Uuid,
        pub unit_id: Option<Uuid>,              // NULL = building-wide
        pub month: YearMonth,                   // YYYY-MM
        pub electricity_kwh: f32,
        pub gas_kwh: f32,
        pub heating_kwh: f32,
        pub water_m3: Option<f32>,
        pub renewable_kwh: f32,                 // green energy from campaign
        pub source: String,                     // 'Linky', 'Ores', 'Manual', 'IoTDevice'
        pub verified: bool,                     // Audited/certified by third-party
    }

Carbon Footprint Calculation
=============================

Scope Classification (GHG Protocol)
-----------------------------------

* **Scope 1**: Direct emissions from owned equipment (boilers, generators) — < 1% typically
* **Scope 2**: Indirect from purchased electricity (Linky readings)
* **Scope 3**: Indirect from value chain (materials, transport, waste)

Carbon Factors (Belgium)
------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 20 30

   * - Energy Type
     - Carbon Factor
     - Source
   * - Electricity (Belgium grid avg)
     - 0.195 kg CO2/kWh
     - ADEME / Eurostat 2024
   * - Natural Gas
     - 0.202 kg CO2/kWh
     - IPCC AR6
   * - Renewable (Solar, Wind)
     - 0.012 kg CO2/kWh
     - ADEME lifecycle
   * - District Heating (fossil)
     - 0.120 kg CO2/kWh
     - ADEME
   * - Waste (per ton)
     - 0.5 kg CO2/ton
     - ADEME (transport to landfill)

**Calculation**:

.. code-block:: rust

    pub struct CarbonFootprint {
        pub scope_1_kg_co2: f32,    // from fuel use
        pub scope_2_kg_co2: f32,    // from electricity
        pub scope_3_kg_co2: f32,    // from materials, transport
        pub total_kg_co2: f32,
        pub per_m2: f32,            // kg CO2 / m² / year
    }

    impl CarbonFootprint {
        pub fn calculate(building: &Building, year_consumption: &EnergyConsumption) -> Self {
            let scope_2 = (year_consumption.electricity_kwh
                - year_consumption.renewable_kwh) * 0.195;  // non-renewable grid electricity

            let scope_2_renewable = year_consumption.renewable_kwh * 0.012;  // green energy

            let scope_1 = year_consumption.gas_kwh * 0.202;  // natural gas

            // Scope 3: estimate from work reports materials
            let scope_3 = estimate_scope_3(&building.id, year);  // TODO

            let total = scope_1 + scope_2 + scope_3;
            let per_m2 = total / building.total_area_m2;

            Self {
                scope_1_kg_co2: scope_1,
                scope_2_kg_co2: scope_2 + scope_2_renewable,
                scope_3_kg_co2: scope_3,
                total_kg_co2: total,
                per_m2,
            }
        }
    }

ESG Metrics Dashboard
=====================

Sustainability KPIs
-------------------

**Energy Metrics**:

* **Energy Intensity**: kWh/m²/year
  * Benchmark: Belgium avg 250 kWh/m²/year
  * Target RE2050: -40% = 150 kWh/m²/year by 2030
  * Good: < 200 kWh/m²/year
  * Excellent: < 100 kWh/m²/year

* **Carbon Intensity**: kg CO2/m²/year
  * Benchmark: Belgium avg ~50 kg CO2/m²/year
  * Target RE2050: -40% = 30 kg CO2/m²/year
  * Good: < 35 kg CO2/m²/year
  * Excellent: < 15 kg CO2/m²/year (fully renewable)

* **Renewable Energy %**: (renewable_kwh / total_kwh) × 100%
  * From IoT Linky device + energy campaigns
  * EU RED III target: 42.5% by 2030
  * Belgium target: 80% by 2050

**Water & Waste**:

* **Water Consumption**: m³/person/year (if occupancy data available)
* **Waste Diversion Rate**: (recycled_tons / total_waste_tons) × 100%

**Building Quality**:

* **PEB/DPE Score**: kWh/m²/year (official rating A-G)
* **EPC Audit Compliance**: days until next audit due
* **Green Building Certifications**: BREEAM, HQE, WIRED, etc.

**Social Equity** (if expanded):

* **Affordability Index**: (avg_rent / regional_median_rent) × 100%
* **Community Services Nearby**: count (schools, hospitals, transit)

**Governance**:

* **Green Decision Rate**: % of meeting resolutions related to sustainability
* **Sustainability Committee**: Active = 1, Inactive = 0

Implementation: Reporting Module
=================================

Annual Sustainability Report (PDF)
-----------------------------------

**Structure**:

.. code-block:: text

    1. Executive Summary (1 page)
       - Key metrics (energy intensity, carbon footprint, renewable %)
       - YoY comparison (2025 vs 2024)
       - Highlights and progress toward RE2050

    2. Energy Performance (2-3 pages)
       - Monthly consumption trends (line chart)
       - By fuel type breakdown (pie chart)
       - Per-unit heatmap (if sub-metering available)

    3. Carbon Footprint (1-2 pages)
       - Scope 1, 2, 3 breakdown
       - Historical trend (5-year backlog)
       - Comparison to similar buildings

    4. Green Energy & Sustainability (1 page)
       - Renewable % achieved
       - Energy buying group results
       - Solar production (if applicable)

    5. PEB/DPE Status (1 page)
       - Current rating (A-G)
       - Next audit due date
       - Historical ratings (show improvement)

    6. Sustainability Initiatives (1-2 pages)
       - Completed projects (insulation, HVAC, windows)
       - Planned improvements (with estimated ROI)
       - Community engagement (voting results on green measures)

    7. Financial Impact (1 page)
       - Savings from energy reduction
       - Subsidies received (Brussels, Wallonia, Flanders)
       - Payback period for investments

    8. Governance (1 page)
       - Board decisions related to sustainability
       - Owner participation in green voting

**Technology**: Typst (like Issue #222 - PDF generation)

.. code-block:: typst

    #let building_name = "Rue de Rivoli 45, Brussels"
    #let energy_intensity = "185 kWh/m²/year"
    #let carbon_intensity = "36 kg CO2/m²/year"
    #let renewable_pct = "25%"
    #let prev_intensity = "210 kWh/m²/year"

    = Sustainability Report #datetime.today().year()
    #building_name

    == Executive Summary

    Our building achieved *#energy_intensity* energy consumption in #datetime.today().year(),
    a #calc.abs(185 - 210) kWh/m² improvement vs. prior year.

    Carbon footprint: *#carbon_intensity*, with *#renewable_pct* renewable energy.

    Progress toward RE2050 target (-40% by 2030):
    - Current: 185 kWh/m² (12% below 2010 baseline)
    - Target 2030: 150 kWh/m² (28% remaining to save)

Database Schema
===============

.. code-block:: sql

    -- Energy consumption tracking (from IoT or manual)
    CREATE TABLE energy_consumption (
        id UUID PRIMARY KEY,
        building_id UUID NOT NULL REFERENCES buildings(id),
        unit_id UUID REFERENCES units(id),
        month DATE NOT NULL,
        electricity_kwh NUMERIC(10,2) NOT NULL,
        gas_kwh NUMERIC(10,2) DEFAULT 0,
        heating_kwh NUMERIC(10,2) DEFAULT 0,
        water_m3 NUMERIC(8,3),
        renewable_kwh NUMERIC(10,2) DEFAULT 0,
        source VARCHAR(50),  -- 'Linky', 'Ores', 'Manual', 'IoTDevice'
        verified BOOLEAN DEFAULT FALSE,
        created_at TIMESTAMPTZ,
        updated_at TIMESTAMPTZ,
        UNIQUE(building_id, unit_id, month)
    );

    -- Annual carbon footprint summary
    CREATE TABLE carbon_footprints (
        id UUID PRIMARY KEY,
        building_id UUID NOT NULL REFERENCES buildings(id),
        fiscal_year INT NOT NULL,
        scope_1_kg_co2 NUMERIC(12,2),     -- from gas, oil
        scope_2_kg_co2 NUMERIC(12,2),     -- from electricity
        scope_3_kg_co2 NUMERIC(12,2),     -- from materials, transport
        total_kg_co2 NUMERIC(12,2) NOT NULL,
        per_m2 NUMERIC(8,3),              -- intensity
        renewable_pct NUMERIC(5,2),
        data_quality_score INT DEFAULT 75, -- 0-100 (impact of missing data)
        verified_by UUID REFERENCES users(id),
        verified_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ,
        UNIQUE(building_id, fiscal_year)
    );

    -- Sustainability reports (generated PDF records)
    CREATE TABLE sustainability_reports (
        id UUID PRIMARY KEY,
        building_id UUID NOT NULL REFERENCES buildings(id),
        fiscal_year INT NOT NULL,
        report_type VARCHAR(50),  -- 'Annual', 'Quarterly', 'RE2050Audit'
        pdf_file_path VARCHAR(255),
        pdf_generated_at TIMESTAMPTZ,
        generated_by UUID REFERENCES users(id),
        created_at TIMESTAMPTZ,
        UNIQUE(building_id, fiscal_year, report_type)
    );

API Endpoints
=============

.. code-block:: rust

    // Energy consumption tracking
    GET /api/v1/analytics/buildings/:id/energy/monthly
    POST /api/v1/analytics/buildings/:id/energy/readings
    GET /api/v1/analytics/buildings/:id/energy/forecast

    // Carbon footprint
    GET /api/v1/analytics/buildings/:id/carbon-footprint
    GET /api/v1/analytics/buildings/:id/carbon-footprint/scope-breakdown

    // Sustainability metrics
    GET /api/v1/analytics/buildings/:id/sustainability/kpis
    GET /api/v1/analytics/buildings/:id/peb-status
    GET /api/v1/analytics/buildings/:id/renewable-energy-percentage

    // Reports
    POST /api/v1/analytics/buildings/:id/reports/sustainability
    GET /api/v1/analytics/buildings/:id/reports/sustainability/:id
    GET /api/v1/analytics/buildings/:id/reports/list

    // RE2050 compliance tracking
    GET /api/v1/analytics/buildings/:id/re2050-progress

Integration Points
===================

**IoT Linky Data** (Issue #133):
  * ``IoTReading`` table provides raw kWh data
  * Daily aggregation to ``EnergyConsumption`` (monthly view)
  * Automatic carbon calculation

**Energy Buying Campaigns** (Issue #95 - Energy):
  * Store renewable % of selected offer
  * Track actual green energy delivered (from supplier invoice)
  * Update ``renewable_kwh`` in ``EnergyConsumption``

**Work Reports** (Issue #134):
  * When contractor reports renovation (insulation, windows, HVAC):
    * Estimate energy savings (ROI calculator)
    * Link to expected kWh reduction
  * Use for Scope 3 calculation (materials embodied carbon)

**DPE/PEB Data Integration**:
  * Belgian PEB registers (Bruxelles-Environnement, SPW, VEKA)
  * API integration (if available) to auto-import PEB scores
  * Manual upload of PDF report (OCR extraction if needed)

**Building Registry**:
  * Walloon AIB (Banque de Données des Immeubles et Habitants)
  * Bruxellois CADRE
  * Use for benchmarking against similar buildings

KoproGo Platform Carbon Footprint
==================================

**Target** (from CLAUDE.md): < 0.5g CO2/request

**Calculation**:

.. code-block:: text

    CO2 per request = (Data center emissions + Network energy) / requests_per_year

    Data center: ~100g CO2/kWh (European average with renewables mix)
    Estimated KoproGo usage: 0.001 kWh per request
    = 0.1g CO2 per request

    Network: ~0.05g CO2 per 100MB transferred
    Estimated payload: 10KB per request (API JSON response)
    = 0.005g CO2 per request

    Total: 0.105g CO2/request (UNDER 0.5g target) ✓

**Monitoring**:

* Track Prometheus metrics: request_duration_seconds, response_size_bytes
* Calculate daily CO2 estimate via dashboard metric
* Green hosting provider (EU datacenter with renewable energy commitment)
* Annual audit by third-party (ISO 14064-1)

Security & GDPR
===============

**Privacy Considerations**:

* Energy consumption (per unit) may be considered personal data (GDPR)
  * Requires explicit consent (Article 6.1.a) before collection
  * Special care for combined consumption (can identify occupancy patterns)
  * Anonymize in aggregate reports (only building-level metrics)

* Carbon footprint reports: Non-personal aggregate data (safe)

**Data Retention**:

* Keep 7 years for audit trail (Belgium legal requirement)
* Archive anonymized aggregates indefinitely (for historical comparison)

Risks & Mitigations
====================

+----------------------+----------------------------+------------------------------+
| Risk                 | Impact                     | Mitigation                   |
+======================+============================+==============================+
| Incomplete IoT data  | Inaccurate carbon calc     | Flag data_quality_score < 70 |
|                      |                            | and prompt for manual entries |
+----------------------+----------------------------+------------------------------+
| Volatile energy      | Misleading trends          | 3-year rolling average for   |
| prices               |                            | financial impact             |
+----------------------+----------------------------+------------------------------+
| Gaming (data falsif) | False compliance claims    | Cryptographic signing of     |
|                      | (esp. for EU taxonomy)     | reports + auditor trails     |
+----------------------+----------------------------+------------------------------+
| Regulatory change    | Reports become obsolete    | Modular carbon factor lookup |
| (carbon factors)     |                            | (update centrally, not hard  |
|                      |                            | coded)                       |
+----------------------+----------------------------+------------------------------+

Implementation Roadmap
======================

**Q2 2026 (Phase 1: Foundation)**
  * Add ESG fields to Unit, Building domain entities
  * Create EnergyConsumption and CarbonFootprint tables
  * Build carbon calculation use case (Scope 2 from Linky)
  * API endpoint for fetching energy data

**Q3 2026 (Phase 2: Reporting)**
  * Build Sustainability Report generator (Typst)
  * Add analytics dashboard for syndics (energy trends)
  * Integrate with PEB/DPE data source
  * Export to PDF + Excel formats

**Q4 2026 (Phase 3: RE2050 & Compliance)**
  * RE2050 progress calculator
  * Automated compliance reporting for regional authorities
  * Integration with energy buying groups (renewable % tracking)
  * Alert system for audit due dates

**Q1 2027 (Phase 4: Advanced)**
  * Scope 3 estimation (from work reports)
  * Benchmarking against similar buildings
  * Green financing finder (subsidies database)
  * EU Taxonomy classification automation
  * SFDR disclosure export

Benchmarks & Industry Standards
===============================

**Belgian Building Averages**:

* Residential: 250-350 kWh/m²/year
* Office: 200-300 kWh/m²/year
* Retail: 300-400 kWh/m²/year
* Industrial: 150-250 kWh/m²/year

**RE2050 Targets by Region**:

.. list-table::
   :header-rows: 1

   * - Region
     - 2020 Baseline
     - 2030 Target
     - 2040 Target
     - 2050 Target
   * - Wallonie
     - 100%
     - 60% (-40%)
     - N/A
     - 20% (-80%)
   * - Bruxelles
     - 100%
     - 60% (-40%)
     - 45% (-55%)
     - 20% (-80%)
   * - Flandre
     - 100%
     - 65% (-35%)
     - N/A
     - Neutrality

Success Criteria
================

* **Data Coverage**: > 80% of buildings with monthly energy readings by Q3 2026
* **Report Quality**: Annual reports pass auditor verification
* **User Engagement**: 50%+ of syndics access sustainability dashboard
* **Compliance**: 100% of RE2050-tracked buildings submit annual reports
* **Accuracy**: Carbon calc within ±5% of third-party audit (ISO 14064-1)
