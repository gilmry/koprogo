# Issue #018 - Budget Prévisionnel Annuel

**Priorité**: 🔴 CRITIQUE
**Estimation**: 8-10 heures
**Labels**: `enhancement`, `backend`, `frontend`, `critical`, `finance`, `legal-compliance`

---

## 📋 Description

Implémenter un système complet de **budget prévisionnel annuel** pour les copropriétés, conformément aux obligations légales belges. Le budget prévisionnel doit être voté en assemblée générale et sert de base aux appels de fonds trimestriels ou mensuels.

**Contexte légal** : En Belgique, le syndic doit présenter chaque année en AG un budget prévisionnel détaillant les charges courantes et extraordinaires prévisibles pour l'année à venir. Ce budget doit être voté par les copropriétaires.

**Impact métier** : Sans budget prévisionnel, impossible de calculer correctement les provisions/appels de fonds. C'est la base de toute la gestion financière d'une copropriété.

---

## 🎯 Objectifs

- [ ] Créer l'entité domain `Budget`
- [ ] Implémenter la création/modification de budgets annuels
- [ ] Calculer automatiquement les provisions basées sur le budget
- [ ] Comparer budget prévisionnel vs dépenses réelles
- [ ] Générer les rapports d'écarts (variance analysis)
- [ ] Exposer API pour gestion budgets
- [ ] Interface frontend pour création/édition budgets
- [ ] Vote AG sur budget (lien avec Issue #046 Voting)

---

## 📐 Spécifications Techniques

### Structure d'un Budget Prévisionnel

Un budget annuel comprend :

#### 1. Charges Courantes (Budget Ordinaire)
- Assurance immeuble
- Entretien et petites réparations
- Eau, gaz, électricité parties communes
- Chauffage (si collectif)
- Nettoyage parties communes
- Ascenseur (maintenance, contrôles)
- Jardin et espaces verts
- Honoraires syndic
- Frais administratifs (comptabilité, courrier, etc.)

#### 2. Charges Extraordinaires (Budget Travaux)
- Gros travaux votés (ravalement, toiture, etc.)
- Travaux d'amélioration
- Diagnostics obligatoires (amiante, plomb, performance énergétique)
- Travaux de mise en conformité

#### 3. Provisions/Réserves
- Fonds de roulement
- Fonds de réserve (travaux futurs)

---

## 🔧 Détails d'Implémentation

### 1. Domain Layer - Entity Budget

**Fichier** : `backend/src/domain/entities/budget.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub fiscal_year: i32,
    pub status: BudgetStatus,
    pub voted_at: Option<DateTime<Utc>>,
    pub meeting_id: Option<Uuid>, // AG où le budget a été voté

    // Budget ordinaire (charges courantes)
    pub ordinary_budget: BudgetSection,

    // Budget extraordinaire (travaux)
    pub extraordinary_budget: BudgetSection,

    // Provisions
    pub working_capital_target: f64,
    pub reserve_fund_target: f64,

    // Métadonnées
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "budget_status", rename_all = "snake_case")]
pub enum BudgetStatus {
    Draft,       // En préparation
    Proposed,    // Proposé en AG
    Approved,    // Voté et approuvé
    Rejected,    // Rejeté en AG
    Active,      // Budget en cours d'exécution
    Closed,      // Exercice terminé
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSection {
    pub total: f64,
    pub line_items: Vec<BudgetLineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLineItem {
    pub id: Uuid,
    pub account_code: String, // Lien avec Issue #016 Plan Comptable
    pub label: String,
    pub budgeted_amount: f64,
    pub actual_amount: f64,   // Dépenses réelles (mise à jour en cours d'année)
    pub variance: f64,         // Écart (budgeted - actual)
    pub variance_percentage: f64,
    pub notes: Option<String>,
}

impl Budget {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        fiscal_year: i32,
        created_by: Uuid,
    ) -> Result<Self, String> {
        if fiscal_year < 2020 || fiscal_year > 2100 {
            return Err("Invalid fiscal year".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            fiscal_year,
            status: BudgetStatus::Draft,
            voted_at: None,
            meeting_id: None,
            ordinary_budget: BudgetSection {
                total: 0.0,
                line_items: Vec::new(),
            },
            extraordinary_budget: BudgetSection {
                total: 0.0,
                line_items: Vec::new(),
            },
            working_capital_target: 0.0,
            reserve_fund_target: 0.0,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn add_line_item(
        &mut self,
        is_extraordinary: bool,
        account_code: String,
        label: String,
        budgeted_amount: f64,
    ) -> Result<(), String> {
        if budgeted_amount < 0.0 {
            return Err("Budgeted amount cannot be negative".to_string());
        }

        let line_item = BudgetLineItem {
            id: Uuid::new_v4(),
            account_code,
            label,
            budgeted_amount,
            actual_amount: 0.0,
            variance: budgeted_amount,
            variance_percentage: 0.0,
            notes: None,
        };

        if is_extraordinary {
            self.extraordinary_budget.line_items.push(line_item);
            self.recalculate_total(true);
        } else {
            self.ordinary_budget.line_items.push(line_item);
            self.recalculate_total(false);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_actual_amounts(&mut self, expenses: Vec<(String, f64)>) {
        // expenses: Vec<(account_code, amount)>
        for (account_code, amount) in expenses {
            // Chercher dans ordinary_budget
            for item in &mut self.ordinary_budget.line_items {
                if item.account_code == account_code {
                    item.actual_amount = amount;
                    item.variance = item.budgeted_amount - amount;
                    item.variance_percentage = if item.budgeted_amount > 0.0 {
                        (item.variance / item.budgeted_amount) * 100.0
                    } else {
                        0.0
                    };
                }
            }

            // Chercher dans extraordinary_budget
            for item in &mut self.extraordinary_budget.line_items {
                if item.account_code == account_code {
                    item.actual_amount = amount;
                    item.variance = item.budgeted_amount - amount;
                    item.variance_percentage = if item.budgeted_amount > 0.0 {
                        (item.variance / item.budgeted_amount) * 100.0
                    } else {
                        0.0
                    };
                }
            }
        }
        self.updated_at = Utc::now();
    }

    fn recalculate_total(&mut self, is_extraordinary: bool) {
        if is_extraordinary {
            self.extraordinary_budget.total = self
                .extraordinary_budget
                .line_items
                .iter()
                .map(|item| item.budgeted_amount)
                .sum();
        } else {
            self.ordinary_budget.total = self
                .ordinary_budget
                .line_items
                .iter()
                .map(|item| item.budgeted_amount)
                .sum();
        }
    }

    pub fn total_budget(&self) -> f64 {
        self.ordinary_budget.total + self.extraordinary_budget.total
    }

    pub fn approve(&mut self, meeting_id: Uuid) {
        self.status = BudgetStatus::Approved;
        self.voted_at = Some(Utc::now());
        self.meeting_id = Some(meeting_id);
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        if self.status != BudgetStatus::Approved {
            return;
        }
        self.status = BudgetStatus::Active;
        self.updated_at = Utc::now();
    }

    pub fn close(&mut self) {
        self.status = BudgetStatus::Closed;
        self.updated_at = Utc::now();
    }

    pub fn calculate_quarterly_provision(&self) -> f64 {
        // Provision trimestrielle = budget ordinaire / 4
        self.ordinary_budget.total / 4.0
    }

    pub fn calculate_monthly_provision(&self) -> f64 {
        // Provision mensuelle = budget ordinaire / 12
        self.ordinary_budget.total / 12.0
    }
}
```

---

### 2. Migration Database

**Fichier** : `backend/migrations/20251101000002_create_budgets.sql`

```sql
CREATE TYPE budget_status AS ENUM (
    'draft',
    'proposed',
    'approved',
    'rejected',
    'active',
    'closed'
);

CREATE TABLE budgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE CASCADE,
    fiscal_year INTEGER NOT NULL,
    status budget_status NOT NULL DEFAULT 'draft',
    voted_at TIMESTAMPTZ,
    meeting_id UUID REFERENCES meetings(id),

    -- Budget data (JSONB pour flexibilité)
    ordinary_budget JSONB NOT NULL DEFAULT '{"total": 0, "line_items": []}',
    extraordinary_budget JSONB NOT NULL DEFAULT '{"total": 0, "line_items": []}',

    working_capital_target DECIMAL(12, 2) NOT NULL DEFAULT 0,
    reserve_fund_target DECIMAL(12, 2) NOT NULL DEFAULT 0,

    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Contrainte unicité : un seul budget actif par building/année
    UNIQUE(building_id, fiscal_year)
);

-- Indexes
CREATE INDEX idx_budgets_building ON budgets(building_id);
CREATE INDEX idx_budgets_fiscal_year ON budgets(fiscal_year);
CREATE INDEX idx_budgets_status ON budgets(status);
CREATE INDEX idx_budgets_building_year ON budgets(building_id, fiscal_year);

-- Vue pour faciliter les requêtes
CREATE VIEW active_budgets AS
SELECT
    b.*,
    bg.name AS building_name,
    EXTRACT(YEAR FROM NOW()) AS current_year
FROM budgets b
JOIN buildings bg ON b.building_id = bg.id
WHERE b.status = 'active';
```

---

### 3. Application Layer - Use Cases

**Fichier** : `backend/src/application/use_cases/budget_use_cases.rs`

```rust
use crate::domain::entities::budget::*;
use crate::application::ports::budget_repository::BudgetRepository;
use crate::application::ports::expense_repository::ExpenseRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct BudgetUseCases {
    budget_repo: Arc<dyn BudgetRepository>,
    expense_repo: Arc<dyn ExpenseRepository>,
}

impl BudgetUseCases {
    pub fn new(
        budget_repo: Arc<dyn BudgetRepository>,
        expense_repo: Arc<dyn ExpenseRepository>,
    ) -> Self {
        Self {
            budget_repo,
            expense_repo,
        }
    }

    pub async fn create_budget(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        fiscal_year: i32,
        created_by: Uuid,
    ) -> Result<Budget, String> {
        // Vérifier qu'il n'existe pas déjà un budget pour cette année
        if let Some(_existing) = self
            .budget_repo
            .find_by_building_and_year(building_id, fiscal_year)
            .await?
        {
            return Err(format!(
                "Budget already exists for year {}",
                fiscal_year
            ));
        }

        let budget = Budget::new(organization_id, building_id, fiscal_year, created_by)?;
        self.budget_repo.create(&budget).await
    }

    pub async fn add_budget_line(
        &self,
        budget_id: Uuid,
        is_extraordinary: bool,
        account_code: String,
        label: String,
        budgeted_amount: f64,
    ) -> Result<Budget, String> {
        let mut budget = self
            .budget_repo
            .find_by_id(budget_id)
            .await?
            .ok_or("Budget not found")?;

        if budget.status != BudgetStatus::Draft && budget.status != BudgetStatus::Proposed {
            return Err("Cannot modify approved budget".to_string());
        }

        budget.add_line_item(is_extraordinary, account_code, label, budgeted_amount)?;
        self.budget_repo.update(&budget).await
    }

    pub async fn approve_budget(
        &self,
        budget_id: Uuid,
        meeting_id: Uuid,
    ) -> Result<Budget, String> {
        let mut budget = self
            .budget_repo
            .find_by_id(budget_id)
            .await?
            .ok_or("Budget not found")?;

        budget.approve(meeting_id);
        self.budget_repo.update(&budget).await
    }

    pub async fn activate_budget(&self, budget_id: Uuid) -> Result<Budget, String> {
        let mut budget = self
            .budget_repo
            .find_by_id(budget_id)
            .await?
            .ok_or("Budget not found")?;

        budget.activate();
        self.budget_repo.update(&budget).await
    }

    pub async fn update_actual_amounts(
        &self,
        budget_id: Uuid,
    ) -> Result<Budget, String> {
        let mut budget = self
            .budget_repo
            .find_by_id(budget_id)
            .await?
            .ok_or("Budget not found")?;

        // Récupérer toutes les expenses de l'année fiscale
        let expenses = self
            .expense_repo
            .find_by_building_and_year(budget.building_id, budget.fiscal_year)
            .await?;

        // Grouper par account_code et sommer
        let mut expenses_by_code: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for expense in expenses {
            let code = expense.account_code.unwrap_or_default(); // TODO: account_code from Issue #016
            *expenses_by_code.entry(code).or_insert(0.0) += expense.amount;
        }

        let expenses_vec: Vec<(String, f64)> = expenses_by_code.into_iter().collect();
        budget.update_actual_amounts(expenses_vec);

        self.budget_repo.update(&budget).await
    }

    pub async fn get_variance_report(
        &self,
        budget_id: Uuid,
    ) -> Result<VarianceReport, String> {
        let budget = self
            .budget_repo
            .find_by_id(budget_id)
            .await?
            .ok_or("Budget not found")?;

        // TODO: Calculer rapport d'écarts détaillé
        Ok(VarianceReport {
            budget_id: budget.id,
            fiscal_year: budget.fiscal_year,
            total_budgeted: budget.total_budget(),
            total_actual: 0.0, // TODO: sum actual_amounts
            total_variance: 0.0,
            variance_percentage: 0.0,
            line_items: vec![],
        })
    }

    pub async fn copy_from_previous_year(
        &self,
        building_id: Uuid,
        from_year: i32,
        to_year: i32,
        created_by: Uuid,
    ) -> Result<Budget, String> {
        // Récupérer budget année précédente
        let previous_budget = self
            .budget_repo
            .find_by_building_and_year(building_id, from_year)
            .await?
            .ok_or("Previous year budget not found")?;

        // Créer nouveau budget avec les mêmes lignes (ajustées +2% inflation par ex.)
        let mut new_budget = Budget::new(
            previous_budget.organization_id,
            building_id,
            to_year,
            created_by,
        )?;

        for item in &previous_budget.ordinary_budget.line_items {
            new_budget.add_line_item(
                false,
                item.account_code.clone(),
                item.label.clone(),
                item.budgeted_amount * 1.02, // +2% inflation
            )?;
        }

        for item in &previous_budget.extraordinary_budget.line_items {
            new_budget.add_line_item(
                true,
                item.account_code.clone(),
                item.label.clone(),
                item.budgeted_amount,
            )?;
        }

        self.budget_repo.create(&new_budget).await
    }
}

#[derive(Debug, serde::Serialize)]
pub struct VarianceReport {
    pub budget_id: Uuid,
    pub fiscal_year: i32,
    pub total_budgeted: f64,
    pub total_actual: f64,
    pub total_variance: f64,
    pub variance_percentage: f64,
    pub line_items: Vec<VarianceLineItem>,
}

#[derive(Debug, serde::Serialize)]
pub struct VarianceLineItem {
    pub account_code: String,
    pub label: String,
    pub budgeted: f64,
    pub actual: f64,
    pub variance: f64,
    pub variance_percentage: f64,
}
```

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Un syndic peut créer un budget pour une année fiscale
- [ ] Le budget peut être modifié en mode Draft/Proposed
- [ ] Le budget peut être approuvé en AG (lien avec Issue #046)
- [ ] Les provisions trimestrielles/mensuelles sont calculées automatiquement
- [ ] Les dépenses réelles sont comparées au budget
- [ ] Un rapport d'écarts (variance) est généré automatiquement
- [ ] Impossible de créer 2 budgets pour la même année

### Techniques
- [ ] Migration SQL s'exécute sans erreur
- [ ] Tests unitaires pour Budget entity
- [ ] Tests E2E pour use cases
- [ ] Frontend permet création/édition budgets
- [ ] Performance : calcul variance < 500ms pour 100 lignes

---

## 🧪 Plan de Tests

### Tests Unitaires

```rust
#[test]
fn test_budget_creation() {
    let budget = Budget::new(Uuid::new_v4(), Uuid::new_v4(), 2025, Uuid::new_v4()).unwrap();
    assert_eq!(budget.status, BudgetStatus::Draft);
    assert_eq!(budget.fiscal_year, 2025);
}

#[test]
fn test_add_line_item() {
    let mut budget = Budget::new(Uuid::new_v4(), Uuid::new_v4(), 2025, Uuid::new_v4()).unwrap();
    budget.add_line_item(false, "6000".to_string(), "Assurance".to_string(), 1200.0).unwrap();
    assert_eq!(budget.ordinary_budget.line_items.len(), 1);
    assert_eq!(budget.ordinary_budget.total, 1200.0);
}

#[test]
fn test_calculate_quarterly_provision() {
    let mut budget = Budget::new(Uuid::new_v4(), Uuid::new_v4(), 2025, Uuid::new_v4()).unwrap();
    budget.add_line_item(false, "6000".to_string(), "Assurance".to_string(), 12000.0).unwrap();
    assert_eq!(budget.calculate_quarterly_provision(), 3000.0);
}
```

---

## 🔗 Dépendances

### Bloquantes
- ✅ Building entity exists
- ✅ Expense entity exists

### Recommandées
- Issue #016 : Plan Comptable Belge (account_code structuré)
- Issue #046 : Voting System (vote AG budget)
- Issue #047 : PDF Generation (export budget PDF)

---

## 📚 Ressources

### Références
- **Gestion copropriété Belgique** : Arrêté royal sur budgets prévisionnels
- **Comptabilité** : Plan comptable normalisé copropriétés

---

## 🚀 Checklist de Développement

- [ ] 1. Créer `domain/entities/budget.rs`
- [ ] 2. Créer migration SQL
- [ ] 3. Créer `BudgetRepository` trait + impl
- [ ] 4. Créer `BudgetUseCases`
- [ ] 5. Créer handlers HTTP
- [ ] 6. Ajouter routes dans `routes.rs`
- [ ] 7. Tests unitaires (15+ tests)
- [ ] 8. Tests E2E (8+ tests)
- [ ] 9. Frontend: page création budget
- [ ] 10. Frontend: dashboard budget vs réel
- [ ] 11. Documentation
- [ ] 12. Commit : `feat: implement annual budgeting system`

---

**Créé le** : 2025-11-01
**Assigné à** : À définir
**Milestone** : v1.0 - MVP Complet
**Impact** : CRITIQUE - Base de toute gestion financière copropriété
