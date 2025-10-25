# Database Constraints Reference

This file documents all database constraints to prevent seed data errors.

## ENUM Types

### unit_type
**Values**: `apartment`, `parking`, `cellar`, `commercial`, `other`
```sql
CREATE TYPE unit_type AS ENUM ('apartment', 'parking', 'cellar', 'commercial', 'other');
```

### expense_category
**Values**: `maintenance`, `repairs`, `insurance`, `utilities`, `cleaning`, `administration`, `works`, `other`
```sql
CREATE TYPE expense_category AS ENUM ('maintenance', 'repairs', 'insurance', 'utilities', 'cleaning', 'administration', 'works', 'other');
```

### document_type
**Values**: `meeting_minutes`, `financial_statement`, `invoice`, `contract`, `regulation`, `works_quote`, `other`
```sql
CREATE TYPE document_type AS ENUM ('meeting_minutes', 'financial_statement', 'invoice', 'contract', 'regulation', 'works_quote', 'other');
```

## CHECK Constraints

### organizations.subscription_plan
**Values**: `free`, `starter`, `professional`, `enterprise`
```sql
subscription_plan VARCHAR(20) NOT NULL CHECK (subscription_plan IN ('free', 'starter', 'professional', 'enterprise'))
```

### users.role
**Values**: `superadmin`, `syndic`, `accountant`, `owner`
```sql
role VARCHAR(20) NOT NULL CHECK (role IN ('superadmin', 'syndic', 'accountant', 'owner'))
```

## NOT NULL Constraints

### owners table
**Required fields**:
- `first_name` VARCHAR(100) NOT NULL
- `last_name` VARCHAR(100) NOT NULL
- `email` VARCHAR(255) NOT NULL UNIQUE
- `address` TEXT NOT NULL
- `city` VARCHAR(100) NOT NULL
- `postal_code` VARCHAR(20) NOT NULL
- `country` VARCHAR(100) NOT NULL

### buildings table
**Required fields**:
- `name` VARCHAR(255) NOT NULL
- `address` TEXT NOT NULL
- `city` VARCHAR(100) NOT NULL
- `postal_code` VARCHAR(20) NOT NULL
- `country` VARCHAR(100) NOT NULL
- `total_units` INTEGER NOT NULL CHECK (total_units > 0)

### units table
**Required fields**:
- `unit_number` VARCHAR(50) NOT NULL
- `unit_type` unit_type NOT NULL
- `floor` INTEGER NOT NULL
- `surface_area` DECIMAL(10,2) NOT NULL

## Usage in Seed Data

When creating seed data, always use values from this reference file to avoid constraint violations.

Example:
```rust
// ✅ CORRECT
let unit_types = vec!["apartment", "parking", "cellar", "commercial", "other"];
let subscription_plans = vec!["free", "starter", "professional", "enterprise"];
let user_roles = vec!["superadmin", "syndic", "accountant", "owner"];

// ❌ WRONG
let unit_types = vec!["studio", "duplex", "penthouse"]; // These don't exist!
let subscription_plans = vec!["basic", "premium"]; // These don't exist!
let user_roles = vec!["admin", "user"]; // These don't exist!
```
