# Belgian Accounting - PCMN (Plan Comptable Minimum Normalisé)

## 📋 Overview

KoproGo implements the Belgian PCMN (Plan Comptable Minimum Normalisé), a standardized chart of accounts required for Belgian accounting as defined by the Royal Decree AR 12/07/2012.

This implementation allows Belgian property management companies (copropriétés/VVE) to manage their accounting in compliance with Belgian legal requirements.

## 🙏 Credits & Attribution

**This implementation is inspired by the [Noalyss](https://gitlab.com/noalyss/noalyss) project.**

- **Noalyss**: Free accounting software for Belgian and French accounting
- **License**: GPL-2.0-or-later (GNU General Public License version 2 or later)
- **Copyright**: (C) 1989, 1991 Free Software Foundation, Inc.
- **Author**: Dany De Bontridder <dany@alchimerys.eu>
- **Website**: https://gitlab.com/noalyss/noalyss

Noalyss provided invaluable reference for:
- Belgian PCMN account structure and hierarchy
- Account classification logic (Asset, Liability, Expense, Revenue, Off-Balance)
- Financial report generation (balance sheet, income statement)
- Account validation rules and constraints

We are grateful to the Noalyss project and its maintainers for creating such a comprehensive and well-documented accounting system that serves as a reference for Belgian PCMN implementation.

## 📊 Belgian PCMN Structure

The Belgian PCMN organizes accounts into 9 classes:

| Class | Type | Description | Examples |
|-------|------|-------------|----------|
| **1** | Liability | Capital, reserves, provisions | `100` Capital, `130` Reserves, `14` Provisions |
| **2** | Asset | Fixed assets (buildings, equipment) | `220` Buildings, `221` Land |
| **3** | Asset | Inventory and work in progress | `30` Raw materials |
| **4** | Asset/Liability | Receivables and payables | `400` Suppliers, `440` Clients, `451` TVA |
| **5** | Asset | Bank and cash | `550` Bank, `551` Post office, `57` Cash |
| **6** | Expense | Operating expenses | `604001` Electricity, `611002` Elevator maintenance |
| **7** | Revenue | Operating revenue | `700001` Regular fees, `700002` Extraordinary fees |
| **8** | - | (Not used in simplified PCMN) | - |
| **9** | Off-Balance | Memorandum accounts | `90` Rights and commitments |

### Hierarchical Structure

Accounts follow a hierarchical structure:

```
6                     # Class: All expenses
└── 60                # Sub-class: Purchases & consumables
    └── 604           # Group: Energy
        └── 604001    # Account: Electricity (direct use)
```

- **Direct use accounts**: Can be used in transactions (e.g., `604001`)
- **Summary accounts**: Cannot be used directly, only for grouping (e.g., `6`, `60`, `604`)

## 🔧 Implementation

### Architecture

```
Domain Layer (Core Business Logic)
  └── entities/account.rs          # Account entity with Belgian PCMN logic

Application Layer (Use Cases)
  ├── ports/account_repository.rs  # Repository interface
  ├── use_cases/account_use_cases.rs         # CRUD + PCMN seed
  └── use_cases/financial_report_use_cases.rs # Reports

Infrastructure Layer (Technical Details)
  ├── database/repositories/account_repository_impl.rs  # PostgreSQL
  └── web/handlers/account_handlers.rs                  # REST API
```

### Database Schema

```sql
CREATE TYPE account_type AS ENUM (
    'ASSET',       -- Classes 2-5
    'LIABILITY',   -- Class 1
    'EXPENSE',     -- Class 6
    'REVENUE',     -- Class 7
    'OFF_BALANCE'  -- Class 9
);

CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    code VARCHAR(40) NOT NULL,           -- e.g., "604001"
    label TEXT NOT NULL,                 -- e.g., "Électricité"
    parent_code VARCHAR(40),             -- e.g., "604"
    account_type account_type NOT NULL,
    direct_use BOOLEAN DEFAULT true,     -- Can be used in transactions
    organization_id UUID NOT NULL,       -- Multi-tenancy
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT accounts_code_org_unique UNIQUE(code, organization_id)
);
```

### Seeded Belgian PCMN Accounts

KoproGo pre-seeds ~90 standard Belgian accounts optimized for property management:

**Class 1 - Liabilities (Capital & Reserves)**
- `100` - Capital social
- `130` - Réserves disponibles
- `131` - Réserves indisponibles
- `14` - Bénéfice (Perte) reporté(e)

**Class 2 - Fixed Assets**
- `220` - Bâtiments
- `221` - Terrains

**Class 4 - Receivables & Payables**
- `400` - Fournisseurs
- `411` - Clients
- `440-441` - TVA
- `451` - TVA à récupérer

**Class 5 - Bank & Cash**
- `550` - Banque courante
- `551` - Banque d'épargne
- `57` - Caisse

**Class 6 - Expenses** (Property Management Focus)
- `604001` - Électricité
- `604002` - Gaz
- `604003` - Eau
- `604004` - Mazout de chauffage
- `611001` - Entretien bâtiment
- `611002` - Entretien ascenseur
- `612001` - Petit entretien parties communes
- `614001` - Assurances incendie
- `614002` - Assurances RC copropriété
- `615001` - Assurance incendie immeuble
- `615002` - Assurance RC exploitant
- And many more...

**Class 7 - Revenue**
- `700001` - Appels de fonds ordinaires
- `700002` - Appels de fonds extraordinaires
- `700003` - Régularisation charges
- `74` - Subventions d'exploitation
- `75` - Produits financiers

Full seed data: `backend/src/application/use_cases/account_use_cases.rs::seed_belgian_pcmn()`

## 🌐 API Endpoints

All endpoints require JWT authentication. Access is restricted based on role:
- ✅ **Accountant**: Full CRUD access
- ✅ **SuperAdmin**: Full CRUD access
- ❌ **Syndic**: Read-only (future)
- ❌ **Owner**: No access

Base URL: `/api/v1`

### Account Management

```bash
# Seed Belgian PCMN (~90 accounts)
POST /accounts/seed/belgian-pcmn
Authorization: Bearer <token>

# Create custom account
POST /accounts
Content-Type: application/json
Authorization: Bearer <token>
{
  "code": "619999",
  "label": "Custom expense account",
  "parent_code": "61",
  "direct_use": true
}

# List accounts (with optional filters)
GET /accounts?account_type=EXPENSE&direct_use=true&search=électr
Authorization: Bearer <token>

# Get account by ID
GET /accounts/{id}
Authorization: Bearer <token>

# Get account by code
GET /accounts/code/{code}
Authorization: Bearer <token>

# Update account
PUT /accounts/{id}
Content-Type: application/json
Authorization: Bearer <token>
{
  "label": "Updated label",
  "direct_use": false
}

# Delete account (with validation)
DELETE /accounts/{id}
Authorization: Bearer <token>

# Count accounts
GET /accounts/count
Authorization: Bearer <token>
```

### Financial Reports

```bash
# Generate balance sheet
GET /reports/balance-sheet
Authorization: Bearer <token>

# Response:
{
  "organization_id": "...",
  "report_date": "2024-11-07T12:00:00Z",
  "assets": {
    "account_type": "ASSET",
    "accounts": [
      {"code": "220", "label": "Bâtiments", "amount": 500000.0},
      {"code": "550", "label": "Banque", "amount": 10000.0}
    ],
    "total": 510000.0
  },
  "liabilities": {
    "account_type": "LIABILITY",
    "accounts": [
      {"code": "100", "label": "Capital", "amount": 500000.0},
      {"code": "130", "label": "Réserves", "amount": 10000.0}
    ],
    "total": 510000.0
  },
  "total_assets": 510000.0,
  "total_liabilities": 510000.0,
  "balance": 0.0
}

# Generate income statement (profit & loss)
GET /reports/income-statement?period_start=2024-01-01T00:00:00Z&period_end=2024-12-31T23:59:59Z
Authorization: Bearer <token>

# Response:
{
  "organization_id": "...",
  "report_date": "2024-11-07T12:00:00Z",
  "period_start": "2024-01-01T00:00:00Z",
  "period_end": "2024-12-31T23:59:59Z",
  "expenses": {
    "account_type": "EXPENSE",
    "accounts": [
      {"code": "604001", "label": "Électricité", "amount": 5000.0},
      {"code": "611002", "label": "Entretien ascenseur", "amount": 2000.0}
    ],
    "total": 7000.0
  },
  "revenue": {
    "account_type": "REVENUE",
    "accounts": [
      {"code": "700001", "label": "Appels de fonds ordinaires", "amount": 10000.0}
    ],
    "total": 10000.0
  },
  "total_expenses": 7000.0,
  "total_revenue": 10000.0,
  "net_result": 3000.0
}
```

## 💼 Usage Examples

### 1. Initialize PCMN for New Organization

```bash
# Step 1: Authenticate as Accountant
POST /api/v1/auth/login
{
  "email": "accountant@example.com",
  "password": "password"
}

# Step 2: Seed Belgian PCMN
POST /api/v1/accounts/seed/belgian-pcmn
Authorization: Bearer <token-from-step-1>

# Result: ~90 standard accounts created
```

### 2. Create Expense with Account Code

```bash
# Link expense to "604001 - Électricité"
POST /api/v1/expenses
Authorization: Bearer <token>
{
  "organization_id": "...",
  "building_id": "...",
  "category": "utilities",
  "description": "Facture électricité janvier 2024",
  "amount": 250.50,
  "expense_date": "2024-01-15T00:00:00Z",
  "supplier": "Electrabel",
  "invoice_number": "INV-2024-001",
  "account_code": "604001"
}
```

### 3. Generate Quarterly Report

```bash
# Q1 2024 income statement
GET /api/v1/reports/income-statement?period_start=2024-01-01T00:00:00Z&period_end=2024-03-31T23:59:59Z
Authorization: Bearer <token>
```

## 🔒 Security & Validation

### Account Deletion Rules

Accounts **cannot be deleted** if:
1. **Has child accounts**: Delete children first (e.g., cannot delete `604` if `604001` exists)
2. **Used in expenses**: Archive instead to preserve historical data

Example error:
```json
{
  "error": "Cannot delete account: it has child accounts. Delete children first."
}
```

### Multi-tenancy Isolation

- All accounts are scoped to `organization_id`
- Each organization has its own chart of accounts
- Account codes are unique within an organization (not globally)
- Users can only access accounts in their organization

## 📈 Financial Reports

### Balance Sheet (Bilan)

Shows financial position at a specific point in time:

```
ASSETS (Actif)              LIABILITIES (Passif)
--------------------        --------------------
Fixed Assets                Capital
  Buildings: 500,000€         Capital: 500,000€
Current Assets              Reserves
  Bank: 10,000€               Reserves: 10,000€

TOTAL: 510,000€             TOTAL: 510,000€
```

**PCMN Classes:**
- Assets: Classes 2, 3, 4 (debit), 5
- Liabilities: Class 1, Class 4 (credit)

### Income Statement (Compte de résultats)

Shows profitability over a time period:

```
REVENUE (Produits)                    EXPENSES (Charges)
--------------------------            --------------------------
Regular fees: 10,000€                 Electricity: 5,000€
                                      Maintenance: 2,000€

TOTAL REVENUE: 10,000€                TOTAL EXPENSES: 7,000€

NET RESULT: 3,000€ (Profit)
```

**PCMN Classes:**
- Expenses: Class 6
- Revenue: Class 7

## 🧪 Testing

The Belgian PCMN implementation includes comprehensive tests:

```bash
# Unit tests (12 tests for Account entity)
cargo test --lib account

# Tests cover:
# - Account creation and validation
# - PCMN class detection (Classes 1-7, 9)
# - Balance sheet vs income statement classification
# - Account code format validation
# - Financial report structure and calculations
```

## 🔮 Future Enhancements

**Phase 2 (Planned):**
- [ ] Journal entries (écritures)
- [ ] Trial balance (balance de vérification)
- [ ] General ledger (grand livre)
- [ ] VAT declaration support
- [ ] Multi-currency support
- [ ] Account archiving (soft delete)
- [ ] Import/export (CSV, Excel)
- [ ] Advanced filters (by date range, amount)

**Phase 3 (Advanced):**
- [ ] Automated VAT calculations
- [ ] Budget vs actual reports
- [ ] Cash flow statement
- [ ] Audit trail for account changes
- [ ] Multi-year comparisons
- [ ] PDF/Excel export for reports

## 📚 References

1. **Noalyss Project**: https://gitlab.com/noalyss/noalyss
   - `include/database/acc_plan_sql.class.php` - Account repository logic
   - `include/database/tmp_pcmn_sql.class.php` - PCMN template
   - `sql/mono-belge.sql` - Belgian PCMN seed data (~9320 lines)

2. **Belgian PCMN Standard**: Royal Decree AR 12/07/2012
   - Official Belgian chart of accounts specification
   - Required for all Belgian companies

3. **KoproGo Documentation**:
   - `CLAUDE.md` - Development guidelines
   - `ROADMAP.md` - Feature roadmap
   - `backend/src/domain/entities/account.rs` - Account entity implementation
   - `backend/migrations/20251107000000_add_belgian_accounting_plan.sql` - Database schema

## ❓ FAQ

**Q: Do I need to seed the Belgian PCMN for every organization?**
A: Yes, each organization has its own chart of accounts. Call `POST /api/v1/accounts/seed/belgian-pcmn` after creating a new organization.

**Q: Can I add custom accounts?**
A: Yes! You can add organization-specific accounts (e.g., `619999 - Custom expense`). Just ensure they follow PCMN hierarchy rules.

**Q: What happens if I delete an account by mistake?**
A: Deletion is prevented if the account is used in expenses or has children. In the future, we'll add soft deletion (archiving).

**Q: How do I link an expense to an account?**
A: Include `account_code` when creating an expense (e.g., `"account_code": "604001"`).

**Q: Can Owners view the chart of accounts?**
A: Not yet. Currently, only Accountants and SuperAdmins have access. We're planning read-only access for Syndics in Phase 2.

## 🤝 Contributing

When contributing to the Belgian PCMN implementation:

1. **Preserve Noalyss attribution**: All accounting-related files must include GPL-2.0 attribution headers
2. **Follow PCMN standards**: Respect the Belgian chart of accounts hierarchy
3. **Add tests**: Every new accounting feature must have unit tests
4. **Document changes**: Update this file with new features
5. **Multi-tenancy**: Always scope queries by `organization_id`

## 📄 License

KoproGo is licensed under the **MIT License**.

However, the Belgian PCMN implementation (inspired by Noalyss) follows the **GPL-2.0-or-later** license as required by the original Noalyss project.

Files affected by GPL-2.0:
- `backend/migrations/20251107000000_add_belgian_accounting_plan.sql`
- `backend/src/domain/entities/account.rs`
- `backend/src/application/ports/account_repository.rs`
- `backend/src/application/use_cases/account_use_cases.rs`
- `backend/src/application/use_cases/financial_report_use_cases.rs`
- `backend/src/infrastructure/database/repositories/account_repository_impl.rs`
- `backend/src/infrastructure/web/handlers/account_handlers.rs`
- `backend/src/infrastructure/web/handlers/financial_report_handlers.rs`

All these files include proper GPL-2.0 attribution headers crediting Noalyss.

---

**Version**: 1.0.0 (November 2024)
**Last Updated**: 2024-11-07
**Maintained by**: KoproGo Team
**Special Thanks**: Noalyss Project & Dany De Bontridder
