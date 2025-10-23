# Issue #003 - Génération de Rapports Financiers

**Priorité**: 🔴 CRITIQUE
**Estimation**: 10-12 heures
**Labels**: `enhancement`, `backend`, `frontend`, `critical`, `finance`

---

## 📋 Description

Développer le système complet de génération de rapports financiers pour la gestion de copropriété. La page UI existe (`frontend/src/pages/reports.astro`) mais aucun backend n'est implémenté. Le service de domaine `ExpenseCalculator` est déjà présent et doit être intégré.

**Contexte métier** : Les rapports financiers sont essentiels pour la transparence envers les copropriétaires et les obligations légales (assemblées générales, audit comptable).

---

## 🎯 Objectifs

- [ ] Créer les endpoints pour génération de rapports
- [ ] Implémenter les calculs financiers (appels de fonds, budget, impayés)
- [ ] Générer des exports PDF et Excel
- [ ] Créer des graphiques de données financières
- [ ] Intégrer le frontend avec composants interactifs
- [ ] Permettre la planification de rapports automatiques

---

## 📐 Spécifications Techniques

### Types de Rapports à Implémenter

| Rapport | Description | Fréquence | Destinataire |
|---------|-------------|-----------|--------------|
| **Appel de fonds** | Calcul charges par copropriétaire selon quote-part | Trimestriel | Owners |
| **Budget prévisionnel** | Estimation annuelle des charges | Annuel | AG |
| **Compte de résultat** | Charges réelles vs budget | Annuel | AG |
| **État des impayés** | Liste copropriétaires en retard de paiement | Mensuel | Syndic |
| **Tableau de répartition** | Quote-part par lot | À la demande | Owners |
| **Export comptable FEC** | Fichier des Écritures Comptables (format légal) | Annuel | Comptable |

---

### Architecture

```
Domain (✅ EXISTANT)
  └─ services/expense_calculator.rs

Application (❌ À CRÉER)
  ├─ use_cases/report_use_cases.rs
  ├─ dto/report_dto.rs
  └─ services/report_generator_service.rs (trait)

Infrastructure (❌ À CRÉER)
  ├─ web/handlers/report_handlers.rs
  ├─ reporting/pdf_generator.rs
  ├─ reporting/excel_generator.rs
  └─ reporting/chart_generator.rs

Frontend (⚠️ À COMPLÉTER)
  ├─ src/pages/reports.astro (existe)
  └─ src/components/ReportDashboard.svelte (à créer)
```

### Endpoints à implémenter

| Méthode | Endpoint | Description | Format |
|---------|----------|-------------|--------|
| `GET` | `/api/v1/reports/call-for-funds/:building_id` | Appel de fonds | JSON/PDF |
| `GET` | `/api/v1/reports/budget/:building_id/:year` | Budget prévisionnel | JSON/PDF |
| `GET` | `/api/v1/reports/profit-loss/:building_id/:year` | Compte résultat | JSON/PDF |
| `GET` | `/api/v1/reports/overdue/:building_id` | État impayés | JSON/Excel |
| `GET` | `/api/v1/reports/quota-distribution/:building_id` | Répartition quotes-parts | JSON/PDF |
| `GET` | `/api/v1/reports/fec/:building_id/:year` | Export FEC | TXT |
| `POST` | `/api/v1/reports/schedule` | Planifier rapport auto | JSON |

---

## 📝 User Stories

### US1 - Génération appel de fonds (Syndic)
```gherkin
En tant que syndic
Je veux générer un appel de fonds trimestriel
Afin d'envoyer les montants dus aux copropriétaires

Scénario: Génération Q1 2025
  Étant donné un immeuble avec 10 lots
  Et des dépenses de 15 000€ pour Q1 2025
  Quand je demande GET /reports/call-for-funds/{building_id}?period=Q1-2025
  Alors je reçois un rapport avec :
    - Total des charges : 15 000€
    - Montant par lot calculé selon quote-part
    - Format PDF téléchargeable
```

### US2 - Consultation état impayés (Syndic)
```gherkin
En tant que syndic
Je veux voir les copropriétaires en retard de paiement
Afin de faire des relances

Scénario: Liste impayés > 30 jours
  Étant donné 3 copropriétaires en retard
  Quand je demande GET /reports/overdue/{building_id}
  Alors je vois pour chaque copropriétaire :
    - Nom
    - Montant dû
    - Nombre de jours de retard
    - Numéro de lot
```

### US3 - Export comptable FEC (Comptable)
```gherkin
En tant que comptable
Je veux exporter le FEC annuel
Afin de respecter les obligations fiscales

Scénario: Export FEC 2024
  Étant donné toutes les écritures de 2024
  Quand je demande GET /reports/fec/{building_id}/2024
  Alors je reçois un fichier .txt avec format :
    JournalCode|JournalLib|EcritureNum|EcritureDate|CompteNum|...
```

---

## 🔧 Détails d'Implémentation

### 1. Report DTOs

**Fichier** : `backend/src/application/dto/report_dto.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Serialize)]
pub struct CallForFundsReport {
    pub building_id: Uuid,
    pub period: String,
    pub total_expenses: Decimal,
    pub generated_at: DateTime<Utc>,
    pub line_items: Vec<CallForFundsLineItem>,
}

#[derive(Debug, Serialize)]
pub struct CallForFundsLineItem {
    pub unit_number: String,
    pub owner_name: String,
    pub quota: Decimal,
    pub amount_due: Decimal,
    pub previous_balance: Decimal,
    pub total_due: Decimal,
}

#[derive(Debug, Serialize)]
pub struct OverdueReport {
    pub building_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub total_overdue: Decimal,
    pub overdue_count: i32,
    pub items: Vec<OverdueItem>,
}

#[derive(Debug, Serialize)]
pub struct OverdueItem {
    pub owner_id: Uuid,
    pub owner_name: String,
    pub unit_number: String,
    pub amount_overdue: Decimal,
    pub days_overdue: i32,
    pub oldest_unpaid_date: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BudgetReport {
    pub building_id: Uuid,
    pub year: i32,
    pub total_budget: Decimal,
    pub categories: Vec<BudgetCategory>,
}

#[derive(Debug, Serialize)]
pub struct BudgetCategory {
    pub category: String,
    pub budgeted_amount: Decimal,
    pub actual_amount: Decimal,
    pub variance: Decimal,
    pub variance_percentage: Decimal,
}

#[derive(Debug, Serialize)]
pub struct FECEntry {
    pub journal_code: String,
    pub journal_lib: String,
    pub ecriture_num: String,
    pub ecriture_date: String,
    pub compte_num: String,
    pub compte_lib: String,
    pub comp_aux_num: Option<String>,
    pub comp_aux_lib: Option<String>,
    pub piece_ref: Option<String>,
    pub piece_date: String,
    pub ecriture_lib: String,
    pub debit: Decimal,
    pub credit: Decimal,
    pub ecriture_let: Option<String>,
    pub date_let: Option<String>,
    pub valid_date: String,
    pub montant_devise: Option<Decimal>,
    pub idevise: Option<String>,
}
```

### 2. Report Use Cases

**Fichier** : `backend/src/application/use_cases/report_use_cases.rs`

```rust
use crate::application::ports::*;
use crate::application::dto::report_dto::*;
use crate::domain::services::expense_calculator::ExpenseCalculator;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Datelike, Utc};
use rust_decimal::Decimal;

pub struct ReportUseCases {
    building_repo: Arc<dyn BuildingRepository>,
    unit_repo: Arc<dyn UnitRepository>,
    expense_repo: Arc<dyn ExpenseRepository>,
    owner_repo: Arc<dyn OwnerRepository>,
    expense_calculator: Arc<ExpenseCalculator>,
}

impl ReportUseCases {
    pub fn new(
        building_repo: Arc<dyn BuildingRepository>,
        unit_repo: Arc<dyn UnitRepository>,
        expense_repo: Arc<dyn ExpenseRepository>,
        owner_repo: Arc<dyn OwnerRepository>,
        expense_calculator: Arc<ExpenseCalculator>,
    ) -> Self {
        Self {
            building_repo,
            unit_repo,
            expense_repo,
            owner_repo,
            expense_calculator,
        }
    }

    pub async fn generate_call_for_funds(
        &self,
        building_id: Uuid,
        period: Option<String>,
    ) -> Result<CallForFundsReport, String> {
        // 1. Récupérer toutes les unités du building
        let units = self.unit_repo.find_by_building(building_id).await?;

        // 2. Récupérer dépenses de la période
        let expenses = self.expense_repo.find_by_building(building_id).await?;

        // 3. Calculer total dépenses
        let total_expenses: Decimal = expenses
            .iter()
            .map(|e| e.amount)
            .sum();

        // 4. Pour chaque unité, calculer montant dû
        let mut line_items = Vec::new();
        for unit in units {
            let quota = unit.quota.unwrap_or(Decimal::ZERO);
            let amount_due = self.expense_calculator.calculate_unit_share(
                &expenses,
                quota,
            );

            // Récupérer owner
            let owner = if let Some(owner_id) = unit.owner_id {
                self.owner_repo.find_by_id(owner_id).await.ok()
            } else {
                None
            };

            line_items.push(CallForFundsLineItem {
                unit_number: unit.unit_number.clone(),
                owner_name: owner.map(|o| o.full_name()).unwrap_or_else(|| "N/A".to_string()),
                quota,
                amount_due,
                previous_balance: Decimal::ZERO, // TODO: implémenter tracking paiements
                total_due: amount_due,
            });
        }

        Ok(CallForFundsReport {
            building_id,
            period: period.unwrap_or_else(|| format!("Q{}-{}", (Utc::now().month() - 1) / 3 + 1, Utc::now().year())),
            total_expenses,
            generated_at: Utc::now(),
            line_items,
        })
    }

    pub async fn generate_overdue_report(
        &self,
        building_id: Uuid,
    ) -> Result<OverdueReport, String> {
        // 1. Récupérer toutes les dépenses impayées
        let all_expenses = self.expense_repo.find_by_building(building_id).await?;
        let overdue_expenses: Vec<_> = all_expenses
            .into_iter()
            .filter(|e| {
                e.status == crate::domain::entities::expense::ExpenseStatus::Overdue
            })
            .collect();

        // 2. Grouper par owner
        let units = self.unit_repo.find_by_building(building_id).await?;
        let mut items = Vec::new();
        let mut total_overdue = Decimal::ZERO;

        for unit in units {
            if let Some(owner_id) = unit.owner_id {
                // Calculer montant impayé pour ce owner
                let owner_expenses: Vec<_> = overdue_expenses
                    .iter()
                    .filter(|e| {
                        // Logique pour associer expense à owner via unit
                        // Simplifié ici
                        true
                    })
                    .collect();

                if !owner_expenses.is_empty() {
                    let amount_overdue: Decimal = owner_expenses
                        .iter()
                        .map(|e| {
                            self.expense_calculator.calculate_unit_share(
                                &[(*e).clone()],
                                unit.quota.unwrap_or(Decimal::ZERO),
                            )
                        })
                        .sum();

                    let owner = self.owner_repo.find_by_id(owner_id).await?;
                    let oldest_date = owner_expenses
                        .iter()
                        .map(|e| e.due_date)
                        .min()
                        .unwrap_or(Utc::now());

                    let days_overdue = (Utc::now() - oldest_date).num_days() as i32;

                    items.push(OverdueItem {
                        owner_id,
                        owner_name: owner.full_name(),
                        unit_number: unit.unit_number.clone(),
                        amount_overdue,
                        days_overdue,
                        oldest_unpaid_date: oldest_date,
                    });

                    total_overdue += amount_overdue;
                }
            }
        }

        Ok(OverdueReport {
            building_id,
            generated_at: Utc::now(),
            total_overdue,
            overdue_count: items.len() as i32,
            items,
        })
    }

    pub async fn generate_budget_report(
        &self,
        building_id: Uuid,
        year: i32,
    ) -> Result<BudgetReport, String> {
        // 1. Récupérer dépenses de l'année
        let expenses = self.expense_repo.find_by_building(building_id).await?;
        let year_expenses: Vec<_> = expenses
            .into_iter()
            .filter(|e| e.due_date.year() == year)
            .collect();

        // 2. Grouper par catégorie
        use std::collections::HashMap;
        let mut category_map: HashMap<String, Decimal> = HashMap::new();

        for expense in &year_expenses {
            let category = format!("{:?}", expense.category);
            *category_map.entry(category).or_insert(Decimal::ZERO) += expense.amount;
        }

        // 3. Calculer variance (nécessite table budget prévisionnel - TODO)
        let categories: Vec<BudgetCategory> = category_map
            .into_iter()
            .map(|(category, actual_amount)| {
                let budgeted_amount = Decimal::ZERO; // TODO: récupérer depuis table budgets
                let variance = actual_amount - budgeted_amount;
                let variance_percentage = if budgeted_amount > Decimal::ZERO {
                    (variance / budgeted_amount) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };

                BudgetCategory {
                    category,
                    budgeted_amount,
                    actual_amount,
                    variance,
                    variance_percentage,
                }
            })
            .collect();

        let total_budget: Decimal = categories.iter().map(|c| c.actual_amount).sum();

        Ok(BudgetReport {
            building_id,
            year,
            total_budget,
            categories,
        })
    }

    pub async fn generate_fec_export(
        &self,
        building_id: Uuid,
        year: i32,
    ) -> Result<String, String> {
        // Format FEC (Fichier des Écritures Comptables)
        // Voir spécification DGFiP
        let expenses = self.expense_repo.find_by_building(building_id).await?;
        let year_expenses: Vec<_> = expenses
            .into_iter()
            .filter(|e| e.due_date.year() == year)
            .collect();

        let mut fec_lines = vec![
            "JournalCode|JournalLib|EcritureNum|EcritureDate|CompteNum|CompteLib|CompAuxNum|CompAuxLib|PieceRef|PieceDate|EcritureLib|Debit|Credit|EcritureLet|DateLet|ValidDate|Montantdevise|Idevise".to_string()
        ];

        for (idx, expense) in year_expenses.iter().enumerate() {
            let line = format!(
                "AC|Achats|{}|{}|607000|Achats de marchandises|||INV-{}|{}|{}|{}|0.00|||{}|{}|EUR",
                idx + 1,
                expense.due_date.format("%Y%m%d"),
                expense.id,
                expense.due_date.format("%Y%m%d"),
                expense.description,
                expense.amount,
                expense.due_date.format("%Y%m%d"),
                expense.amount,
            );
            fec_lines.push(line);
        }

        Ok(fec_lines.join("\n"))
    }
}
```

### 3. PDF Generator

**Fichier** : `backend/src/infrastructure/reporting/pdf_generator.rs`

```rust
use printpdf::*;
use crate::application::dto::report_dto::CallForFundsReport;

pub struct PdfGenerator;

impl PdfGenerator {
    pub fn generate_call_for_funds_pdf(report: &CallForFundsReport) -> Result<Vec<u8>, String> {
        // 1. Créer document PDF
        let (doc, page1, layer1) = PdfDocument::new(
            "Appel de Fonds",
            Mm(210.0),
            Mm(297.0),
            "Layer 1",
        );

        // 2. Ajouter contenu (titre, tableau, totaux)
        // Utiliser printpdf pour dessiner texte et formes

        // 3. Sauvegarder en Vec<u8>
        let mut buffer = Vec::new();
        doc.save(&mut buffer)
            .map_err(|e| format!("PDF generation error: {}", e))?;

        Ok(buffer)
    }
}
```

**Note** : Pour une solution plus rapide, considérer l'utilisation de templates HTML + headless Chrome (via `headless_chrome` crate).

---

## ✅ Critères d'Acceptation

### Fonctionnels
- [ ] Appel de fonds avec calcul exact par quote-part
- [ ] État impayés avec nombre de jours de retard
- [ ] Budget avec variance budgétée vs réalisée
- [ ] Export FEC conforme format DGFiP
- [ ] Génération PDF lisible et formaté
- [ ] Export Excel avec formules

### Performance
- [ ] Génération rapport < 2s pour building de 100 lots
- [ ] Export FEC < 5s pour 1000 écritures

### Tests
- [ ] Tests unitaires calculs financiers (ExpenseCalculator)
- [ ] Tests E2E pour chaque type de rapport
- [ ] Tests validation format FEC

---

## 🧪 Plan de Tests

```rust
#[tokio::test]
async fn test_generate_call_for_funds() {
    // Créer building avec 5 units
    // Créer 10 expenses (total 10000€)
    // Générer rapport
    // Vérifier total_expenses = 10000
    // Vérifier montants par unit selon quota
}

#[tokio::test]
async fn test_overdue_report() {
    // Créer 2 expenses overdue
    // Générer rapport impayés
    // Vérifier 2 items
    // Vérifier calcul jours de retard
}

#[tokio::test]
async fn test_fec_export_format() {
    // Générer FEC
    // Vérifier format pipe-separated
    // Vérifier header correct
    // Vérifier dates format YYYYMMDD
}
```

---

## 🔗 Dépendances Cargo

```toml
# PDF generation
printpdf = "0.7"
# Alternative: headless_chrome = "1.0"

# Excel generation
rust_xlsxwriter = "0.68"

# Decimal arithmetic
rust_decimal = "1.33"
```

---

## 🚀 Checklist

- [ ] 1. Créer report_dto.rs
- [ ] 2. Créer report_use_cases.rs
- [ ] 3. Intégrer ExpenseCalculator existant
- [ ] 4. Créer pdf_generator.rs
- [ ] 5. Créer excel_generator.rs
- [ ] 6. Créer report_handlers.rs
- [ ] 7. Ajouter routes
- [ ] 8. Tests unitaires
- [ ] 9. Tests E2E
- [ ] 10. Composant frontend ReportDashboard.svelte
- [ ] 11. Documentation
- [ ] 12. Commit : `feat: implement financial reporting system`

---

**Créé le** : 2025-10-23
**Milestone** : v1.0 - MVP Complet
