# Issue #016 - Plan Comptable Normalisé Belge

**Priorité**: 🔴 CRITIQUE
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `backend`, `critical`, `finance`, `legal-compliance`

---

## 📋 Description

Implémenter un plan comptable normalisé conforme aux exigences comptables belges pour les copropriétés. Le système actuel utilise des catégories d'expenses basiques (`maintenance`, `repairs`, `insurance`, etc.) qui ne sont pas conformes au plan comptable légal belge.

**Contexte légal** : En Belgique, les copropriétés doivent tenir une comptabilité selon un plan comptable normalisé (arrêté royal du 12 juillet 2012). Sans cela, les comptes présentés en AG peuvent être contestés.

---

## 🎯 Objectifs

- [ ] Créer un plan comptable conforme à la législation belge
- [ ] Migrer les catégories d'expenses existantes vers le nouveau plan
- [ ] Implémenter la structure de comptes (classe 4, 5, 6, 7)
- [ ] Ajouter la ventilation par nature et destination
- [ ] Générer les états financiers conformes (bilan, compte de résultat)
- [ ] Documenter le mapping entre ancien et nouveau système

---

## 📐 Spécifications Techniques

### Plan Comptable Belge pour Copropriétés

Le plan comptable belge pour copropriétés comprend 4 classes principales :

#### Classe 4 : Créances et Dettes
- **40xx** : Fournisseurs (dettes envers prestataires)
- **41xx** : Copropriétaires (créances charges)
- **44xx** : TVA

#### Classe 5 : Trésorerie
- **50xx** : Compte courant
- **51xx** : Compte épargne (fonds de réserve)
- **52xx** : Placements à terme

#### Classe 6 : Charges
- **60xx** : Charges courantes
  - 6000 : Assurance
  - 6010 : Entretien et réparations
  - 6020 : Eau, gaz, électricité
  - 6030 : Chauffage
  - 6040 : Nettoyage
  - 6050 : Ascenseur
  - 6060 : Jardin et espaces verts
  - 6070 : Honoraires syndic
  - 6080 : Frais administratifs
  - 6090 : Divers
- **61xx** : Charges extraordinaires
  - 6100 : Gros travaux (ravalement, toiture)
  - 6110 : Travaux d'amélioration
  - 6120 : Diagnostics obligatoires
  - 6130 : Travaux de sécurité

#### Classe 7 : Produits
- **70xx** : Appels de fonds ordinaires
- **71xx** : Appels de fonds extraordinaires
- **72xx** : Intérêts et produits financiers
- **73xx** : Produits divers (locations parties communes)

---

## 🔧 Détails d'Implémentation

### 1. Domain Layer - Nouveau Enum AccountCode

**Fichier** : `backend/src/domain/entities/account_code.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "account_code", rename_all = "snake_case")]
pub enum AccountCode {
    // Classe 4: Créances et Dettes
    Suppliers,           // 40xx
    Owners,              // 41xx
    Vat,                 // 44xx

    // Classe 5: Trésorerie
    CurrentAccount,      // 50xx
    ReserveFund,         // 51xx
    Investments,         // 52xx

    // Classe 6: Charges courantes
    Insurance,           // 6000
    Maintenance,         // 6010
    Utilities,           // 6020
    Heating,             // 6030
    Cleaning,            // 6040
    Elevator,            // 6050
    Gardening,           // 6060
    SyndicFees,          // 6070
    Administrative,      // 6080
    MiscellaneousExpenses, // 6090

    // Classe 6: Charges extraordinaires
    MajorWorks,          // 6100
    Improvements,        // 6110
    Diagnostics,         // 6120
    SafetyWorks,         // 6130

    // Classe 7: Produits
    OrdinaryCalls,       // 70xx
    ExtraordinaryCalls,  // 71xx
    FinancialIncome,     // 72xx
    OtherIncome,         // 73xx
}

impl AccountCode {
    /// Retourne le code comptable numérique
    pub fn code(&self) -> &str {
        match self {
            AccountCode::Suppliers => "40",
            AccountCode::Owners => "41",
            AccountCode::Vat => "44",
            AccountCode::CurrentAccount => "50",
            AccountCode::ReserveFund => "51",
            AccountCode::Investments => "52",
            AccountCode::Insurance => "6000",
            AccountCode::Maintenance => "6010",
            AccountCode::Utilities => "6020",
            AccountCode::Heating => "6030",
            AccountCode::Cleaning => "6040",
            AccountCode::Elevator => "6050",
            AccountCode::Gardening => "6060",
            AccountCode::SyndicFees => "6070",
            AccountCode::Administrative => "6080",
            AccountCode::MiscellaneousExpenses => "6090",
            AccountCode::MajorWorks => "6100",
            AccountCode::Improvements => "6110",
            AccountCode::Diagnostics => "6120",
            AccountCode::SafetyWorks => "6130",
            AccountCode::OrdinaryCalls => "70",
            AccountCode::ExtraordinaryCalls => "71",
            AccountCode::FinancialIncome => "72",
            AccountCode::OtherIncome => "73",
        }
    }

    /// Retourne le libellé du compte
    pub fn label(&self) -> &str {
        match self {
            AccountCode::Insurance => "Assurance",
            AccountCode::Maintenance => "Entretien et réparations",
            AccountCode::Utilities => "Eau, gaz, électricité",
            AccountCode::Heating => "Chauffage",
            AccountCode::Cleaning => "Nettoyage",
            AccountCode::Elevator => "Ascenseur",
            AccountCode::Gardening => "Jardin et espaces verts",
            AccountCode::SyndicFees => "Honoraires syndic",
            AccountCode::Administrative => "Frais administratifs",
            AccountCode::MiscellaneousExpenses => "Divers",
            AccountCode::MajorWorks => "Gros travaux",
            AccountCode::Improvements => "Travaux d'amélioration",
            AccountCode::Diagnostics => "Diagnostics obligatoires",
            AccountCode::SafetyWorks => "Travaux de sécurité",
            AccountCode::OrdinaryCalls => "Appels de fonds ordinaires",
            AccountCode::ExtraordinaryCalls => "Appels de fonds extraordinaires",
            AccountCode::FinancialIncome => "Intérêts et produits financiers",
            AccountCode::OtherIncome => "Produits divers",
            _ => "Autre",
        }
    }

    /// Retourne la classe comptable (4, 5, 6, 7)
    pub fn class(&self) -> u8 {
        match self {
            AccountCode::Suppliers | AccountCode::Owners | AccountCode::Vat => 4,
            AccountCode::CurrentAccount | AccountCode::ReserveFund | AccountCode::Investments => 5,
            AccountCode::Insurance | AccountCode::Maintenance | AccountCode::Utilities
            | AccountCode::Heating | AccountCode::Cleaning | AccountCode::Elevator
            | AccountCode::Gardening | AccountCode::SyndicFees | AccountCode::Administrative
            | AccountCode::MiscellaneousExpenses | AccountCode::MajorWorks
            | AccountCode::Improvements | AccountCode::Diagnostics | AccountCode::SafetyWorks => 6,
            AccountCode::OrdinaryCalls | AccountCode::ExtraordinaryCalls
            | AccountCode::FinancialIncome | AccountCode::OtherIncome => 7,
        }
    }

    /// Indique si c'est une charge ordinaire ou extraordinaire
    pub fn is_ordinary(&self) -> bool {
        matches!(
            self,
            AccountCode::Insurance
                | AccountCode::Maintenance
                | AccountCode::Utilities
                | AccountCode::Heating
                | AccountCode::Cleaning
                | AccountCode::Elevator
                | AccountCode::Gardening
                | AccountCode::SyndicFees
                | AccountCode::Administrative
                | AccountCode::MiscellaneousExpenses
        )
    }

    pub fn is_extraordinary(&self) -> bool {
        matches!(
            self,
            AccountCode::MajorWorks
                | AccountCode::Improvements
                | AccountCode::Diagnostics
                | AccountCode::SafetyWorks
        )
    }
}
```

---

### 2. Migration Database

**Fichier** : `backend/migrations/20251101000000_add_belgian_account_code.sql`

```sql
-- Créer le type ENUM pour les codes comptables belges
CREATE TYPE account_code AS ENUM (
    'suppliers',
    'owners',
    'vat',
    'current_account',
    'reserve_fund',
    'investments',
    'insurance',
    'maintenance',
    'utilities',
    'heating',
    'cleaning',
    'elevator',
    'gardening',
    'syndic_fees',
    'administrative',
    'miscellaneous_expenses',
    'major_works',
    'improvements',
    'diagnostics',
    'safety_works',
    'ordinary_calls',
    'extraordinary_calls',
    'financial_income',
    'other_income'
);

-- Ajouter la colonne account_code à la table expenses
ALTER TABLE expenses ADD COLUMN account_code account_code;

-- Migrer les données existantes (mapping best-effort)
UPDATE expenses SET account_code = CASE
    WHEN category = 'insurance' THEN 'insurance'::account_code
    WHEN category = 'maintenance' THEN 'maintenance'::account_code
    WHEN category = 'repairs' THEN 'maintenance'::account_code
    WHEN category = 'utilities' THEN 'utilities'::account_code
    WHEN category = 'cleaning' THEN 'cleaning'::account_code
    WHEN category = 'administration' THEN 'administrative'::account_code
    WHEN category = 'works' THEN 'major_works'::account_code
    ELSE 'miscellaneous_expenses'::account_code
END;

-- Rendre obligatoire après migration
ALTER TABLE expenses ALTER COLUMN account_code SET NOT NULL;

-- Index pour requêtes par code comptable
CREATE INDEX idx_expenses_account_code ON expenses(account_code);
CREATE INDEX idx_expenses_building_account_code ON expenses(building_id, account_code);

-- Vue pour états financiers
CREATE VIEW financial_statements AS
SELECT
    e.building_id,
    e.organization_id,
    EXTRACT(YEAR FROM e.expense_date) AS fiscal_year,
    e.account_code,
    SUM(e.amount) AS total_amount,
    COUNT(*) AS transaction_count
FROM expenses e
GROUP BY e.building_id, e.organization_id, fiscal_year, e.account_code;
```

---

### 3. Application Layer - Financial Reporting

**Fichier** : `backend/src/application/use_cases/financial_reporting_use_cases.rs`

```rust
use crate::domain::entities::account_code::AccountCode;
use crate::application::ports::expense_repository::ExpenseRepository;
use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BalanceSheet {
    pub building_id: Uuid,
    pub fiscal_year: i32,
    pub assets: AssetsSection,
    pub liabilities: LiabilitiesSection,
}

#[derive(Debug, Serialize)]
pub struct AssetsSection {
    pub current_account: f64,
    pub reserve_fund: f64,
    pub investments: f64,
    pub total: f64,
}

#[derive(Debug, Serialize)]
pub struct LiabilitiesSection {
    pub suppliers: f64,
    pub owners: f64,
    pub vat: f64,
    pub total: f64,
}

#[derive(Debug, Serialize)]
pub struct IncomeStatement {
    pub building_id: Uuid,
    pub fiscal_year: i32,
    pub revenue: RevenueSection,
    pub expenses: ExpensesSection,
    pub net_result: f64,
}

#[derive(Debug, Serialize)]
pub struct RevenueSection {
    pub ordinary_calls: f64,
    pub extraordinary_calls: f64,
    pub financial_income: f64,
    pub other_income: f64,
    pub total: f64,
}

#[derive(Debug, Serialize)]
pub struct ExpensesSection {
    pub ordinary_expenses: Vec<ExpenseLine>,
    pub extraordinary_expenses: Vec<ExpenseLine>,
    pub total_ordinary: f64,
    pub total_extraordinary: f64,
    pub total: f64,
}

#[derive(Debug, Serialize)]
pub struct ExpenseLine {
    pub account_code: AccountCode,
    pub label: String,
    pub amount: f64,
}

pub struct FinancialReportingUseCases {
    expense_repo: Arc<dyn ExpenseRepository>,
}

impl FinancialReportingUseCases {
    pub fn new(expense_repo: Arc<dyn ExpenseRepository>) -> Self {
        Self { expense_repo }
    }

    pub async fn generate_balance_sheet(
        &self,
        building_id: Uuid,
        fiscal_year: i32,
    ) -> Result<BalanceSheet, String> {
        // Calculer les actifs et passifs
        // TODO: Implémenter en utilisant les données de classe 4 et 5
        todo!("Implémenter génération bilan")
    }

    pub async fn generate_income_statement(
        &self,
        building_id: Uuid,
        fiscal_year: i32,
    ) -> Result<IncomeStatement, String> {
        // Calculer revenus (classe 7) et charges (classe 6)
        // TODO: Implémenter en utilisant account_code
        todo!("Implémenter génération compte de résultat")
    }

    pub async fn export_to_csv(
        &self,
        building_id: Uuid,
        fiscal_year: i32,
    ) -> Result<String, String> {
        // Export CSV format comptable belge
        todo!("Implémenter export CSV")
    }
}
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Toutes les catégories d'expenses peuvent être mappées vers un AccountCode
- [ ] Les états financiers (bilan + compte de résultat) sont générés correctement
- [ ] La ventilation ordinaire/extraordinaire est automatique
- [ ] Export CSV conforme au format belge
- [ ] Migration des données existantes réussie sans perte

### Techniques
- [ ] Migration SQL s'exécute sans erreur
- [ ] Tous les tests unitaires passent
- [ ] Tests E2E pour génération états financiers
- [ ] Performance : génération rapport < 1s pour 1000 expenses

### Documentation
- [ ] Guide mapping ancien/nouveau plan comptable
- [ ] Documentation utilisateur pour comptables
- [ ] Exemples de rapports générés

---

## 🧪 Plan de Tests

### Tests Unitaires

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_code_class() {
        assert_eq!(AccountCode::Insurance.class(), 6);
        assert_eq!(AccountCode::OrdinaryCalls.class(), 7);
        assert_eq!(AccountCode::ReserveFund.class(), 5);
    }

    #[test]
    fn test_ordinary_vs_extraordinary() {
        assert!(AccountCode::Maintenance.is_ordinary());
        assert!(!AccountCode::Maintenance.is_extraordinary());
        assert!(AccountCode::MajorWorks.is_extraordinary());
        assert!(!AccountCode::MajorWorks.is_ordinary());
    }

    #[test]
    fn test_account_code_labels() {
        assert_eq!(AccountCode::Insurance.label(), "Assurance");
        assert_eq!(AccountCode::Elevator.label(), "Ascenseur");
    }
}
```

### Tests E2E

```rust
#[actix_rt::test]
async fn test_generate_income_statement() {
    // Créer building
    // Créer expenses avec différents account_code
    // Générer compte de résultat
    // Vérifier totaux par classe
}

#[actix_rt::test]
async fn test_migration_old_to_new_categories() {
    // Créer expense avec ancienne catégorie 'maintenance'
    // Vérifier mapping vers account_code 'maintenance' (6010)
    // Vérifier que le rapport inclut correctement l'expense
}
```

---

## 🔗 Dépendances

### Bloquantes
- ✅ Expense entity existe
- ✅ ExpenseRepository implémenté

### Optionnelles
- Issue #003 : Génération rapports financiers (PDF)
- Issue #047 : PDF Generation Extended (templates états financiers)

---

## 📚 Ressources

### Références Légales
- **Arrêté royal 12 juillet 2012** : Comptabilité des copropriétés en Belgique
- **Plan comptable minimum normalisé** : https://economie.fgov.be/

### Documentation Technique
- SQLx Enums : https://docs.rs/sqlx/latest/sqlx/
- Serde custom serialization : https://serde.rs/

---

## 🚀 Checklist de Développement

- [ ] 1. Créer `domain/entities/account_code.rs`
- [ ] 2. Créer migration `20251101000000_add_belgian_account_code.sql`
- [ ] 3. Tester migration sur base de données de test
- [ ] 4. Modifier `Expense` entity pour ajouter `account_code`
- [ ] 5. Créer `financial_reporting_use_cases.rs`
- [ ] 6. Implémenter génération bilan
- [ ] 7. Implémenter génération compte de résultat
- [ ] 8. Créer tests unitaires (15+ tests)
- [ ] 9. Créer tests E2E (5+ tests)
- [ ] 10. Documenter mapping catégories
- [ ] 11. Mettre à jour frontend pour afficher nouveaux codes
- [ ] 12. Commit avec message : `feat: implement Belgian accounting plan compliance`

---

## 📊 Métriques de Succès

- **Conformité** : 100% codes comptables conformes à l'AR 12/07/2012
- **Migration** : 0% perte de données lors de migration
- **Performance** : Génération rapport < 1s pour 1000 expenses
- **Qualité** : 0 warning Clippy

---

**Créé le** : 2025-11-01
**Assigné à** : À définir
**Milestone** : v1.0 - MVP Complet - Conformité Légale Belge
**Bloque** : Production deployment (non-conformité comptable)
