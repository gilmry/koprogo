=============================================
Contractor Quotes System (Issue #91)
=============================================

Overview
========

Management of contractor quotes for building maintenance and renovation
works, with strict compliance to Belgian copropriété law requirements.

Belgian Legal Requirements
==========================

Belgian copropriété law mandates:

- **3 quotes minimum** for construction works exceeding **5,000€**
- **General assembly vote** required for major works
- **10-year structural warranty** ("décennale") for structural works
- **2-year warranty** for apparent defects

The system enforces these rules through automatic scoring and comparison.

Automatic Scoring Algorithm
============================

When comparing quotes, the system applies weighted scoring:

+------------+--------+----------------------------------+
| Criterion  | Weight | Logic                            |
+============+========+==================================+
| Price      | 40%    | Lower price = higher score       |
+------------+--------+----------------------------------+
| Delay      | 30%    | Shorter duration = higher score  |
+------------+--------+----------------------------------+
| Warranty   | 20%    | Longer warranty = higher score   |
+------------+--------+----------------------------------+
| Reputation | 10%    | Higher rating (0-100) = higher   |
+------------+--------+----------------------------------+

Quote Workflow
==============

::

    Requested → Received → UnderReview → Accepted
                                       → Rejected
                                       → Expired
                               ↓
                           Withdrawn

- **Requested**: Syndic sends quote request to contractor
- **Received**: Contractor submits pricing
- **UnderReview**: Syndic evaluates the quote
- **Accepted/Rejected**: Decision with audit trail
- **Expired**: Past validity_date
- **Withdrawn**: Contractor withdraws

Belgian VAT Rates
=================

- **6%** reduced rate (renovations of buildings >10 years old)
- **21%** standard rate (new construction)

Decision Audit Trail
====================

Each accept/reject records:

- ``decision_at``: Timestamp
- ``decision_by``: User UUID
- ``decision_notes``: Justification text

API Endpoints (15)
==================

- ``POST /quotes`` - Create quote request
- ``GET /quotes/:id`` - Get details
- ``GET /buildings/:id/quotes`` - List building quotes
- ``POST /quotes/:id/submit`` - Contractor submits pricing
- ``POST /quotes/:id/review`` - Start review
- ``POST /quotes/:id/accept`` - Accept (audit trail)
- ``POST /quotes/:id/reject`` - Reject (audit trail)
- ``POST /quotes/:id/withdraw`` - Contractor withdraws
- ``POST /quotes/compare`` - Compare multiple quotes (scoring)
- ``PUT /quotes/:id/contractor-rating`` - Update rating

Architecture
============

- **Domain**: ``quote.rs`` (661 lines, 7 state transitions)
- **Use Cases**: ``quote_use_cases.rs`` (20 methods including compare)
- **Migration**: ``20251120150000_create_quotes.sql``
- **BDD Tests**: ``quotes.feature`` (13 scenarios)
- **Total**: ~2,161 lines of code
