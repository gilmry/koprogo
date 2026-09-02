use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Statut du budget annuel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "budget_status", rename_all = "snake_case")]
pub enum BudgetStatus {
    Draft,     // Brouillon (en préparation)
    Submitted, // Soumis pour vote en AG
    Approved,  // Approuvé par l'AG (actif)
    Rejected,  // Rejeté par l'AG
    Archived,  // Archivé (exercice terminé)
}

/// Représente un budget annuel de copropriété (ordinaire + extraordinaire)
///
/// Obligation légale belge: Le budget doit être voté en AG avant le début
/// de l'exercice fiscal. Il détermine les provisions mensuelles à appeler
/// auprès des copropriétaires.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Budget {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,

    /// Année fiscale (ex: 2025)
    pub fiscal_year: i32,

    /// Budget charges ordinaires (€) - Charges courantes récurrentes
    pub ordinary_budget: Decimal,

    /// Budget charges extraordinaires (€) - Travaux et dépenses exceptionnelles
    pub extraordinary_budget: Decimal,

    /// Budget total (€) = ordinaire + extraordinaire
    pub total_budget: Decimal,

    /// Statut du budget
    pub status: BudgetStatus,

    /// Date de soumission pour vote AG
    pub submitted_date: Option<DateTime<Utc>>,

    /// Date d'approbation par l'AG
    pub approved_date: Option<DateTime<Utc>>,

    /// ID de l'AG qui a approuvé le budget
    pub approved_by_meeting_id: Option<Uuid>,

    /// Montant mensuel des provisions à appeler (€)
    /// = total_budget / 12 mois
    pub monthly_provision_amount: Decimal,

    /// Notes / Commentaires
    pub notes: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Budget {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        fiscal_year: i32,
        ordinary_budget: Decimal,
        extraordinary_budget: Decimal,
    ) -> Result<Self, String> {
        // Validations
        if fiscal_year < 2000 || fiscal_year > 2100 {
            return Err("Fiscal year must be between 2000 and 2100".to_string());
        }

        if ordinary_budget < Decimal::ZERO {
            return Err("Ordinary budget cannot be negative".to_string());
        }

        if extraordinary_budget < Decimal::ZERO {
            return Err("Extraordinary budget cannot be negative".to_string());
        }

        let total_budget = ordinary_budget + extraordinary_budget;

        if total_budget == Decimal::ZERO {
            return Err("Total budget cannot be zero".to_string());
        }

        // Calcul provisions mensuelles (division Decimal exacte — cf. ADR-0007)
        let monthly_provision_amount = total_budget / Decimal::from(12);

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            fiscal_year,
            ordinary_budget,
            extraordinary_budget,
            total_budget,
            status: BudgetStatus::Draft,
            submitted_date: None,
            approved_date: None,
            approved_by_meeting_id: None,
            monthly_provision_amount,
            notes: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Soumet le budget pour vote en AG
    pub fn submit_for_approval(&mut self) -> Result<(), String> {
        match self.status {
            BudgetStatus::Draft | BudgetStatus::Rejected => {
                self.status = BudgetStatus::Submitted;
                self.submitted_date = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot submit budget with status {:?}",
                self.status
            )),
        }
    }

    /// Approuve le budget (vote AG positif)
    pub fn approve(&mut self, meeting_id: Uuid) -> Result<(), String> {
        match self.status {
            BudgetStatus::Submitted => {
                self.status = BudgetStatus::Approved;
                self.approved_date = Some(Utc::now());
                self.approved_by_meeting_id = Some(meeting_id);
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot approve budget with status {:?}",
                self.status
            )),
        }
    }

    /// Rejette le budget (vote AG négatif)
    pub fn reject(&mut self) -> Result<(), String> {
        match self.status {
            BudgetStatus::Submitted => {
                self.status = BudgetStatus::Rejected;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot reject budget with status {:?}",
                self.status
            )),
        }
    }

    /// Archive le budget (fin d'exercice)
    pub fn archive(&mut self) -> Result<(), String> {
        match self.status {
            BudgetStatus::Approved => {
                self.status = BudgetStatus::Archived;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!(
                "Cannot archive budget with status {:?}",
                self.status
            )),
        }
    }

    /// Met à jour les montants du budget (uniquement en Draft)
    pub fn update_amounts(
        &mut self,
        ordinary_budget: Decimal,
        extraordinary_budget: Decimal,
    ) -> Result<(), String> {
        if !self.is_editable() {
            return Err("Can only update amounts in Draft or Rejected status".to_string());
        }

        if ordinary_budget < Decimal::ZERO {
            return Err("Ordinary budget cannot be negative".to_string());
        }

        if extraordinary_budget < Decimal::ZERO {
            return Err("Extraordinary budget cannot be negative".to_string());
        }

        let total_budget = ordinary_budget + extraordinary_budget;

        if total_budget == Decimal::ZERO {
            return Err("Total budget cannot be zero".to_string());
        }

        self.ordinary_budget = ordinary_budget;
        self.extraordinary_budget = extraordinary_budget;
        self.total_budget = total_budget;
        self.monthly_provision_amount = total_budget / Decimal::from(12);
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Ajoute/met à jour les notes
    pub fn update_notes(&mut self, notes: String) {
        self.notes = Some(notes);
        self.updated_at = Utc::now();
    }

    /// Vérifie si le budget est actif (approuvé et pas encore archivé)
    pub fn is_active(&self) -> bool {
        self.status == BudgetStatus::Approved
    }

    /// Vérifie si le budget peut être modifié
    pub fn is_editable(&self) -> bool {
        matches!(self.status, BudgetStatus::Draft | BudgetStatus::Rejected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // ----- Story H11 (CL4) — montants Budget en Decimal exact (4-cat) --------

    #[test]
    fn happy_monthly_provision_is_exact_decimal() {
        // 75000 / 12 = 6250 exact (Decimal, pas de dérive en virgule flottante).
        let b = Budget::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            2025,
            dec!(50000),
            dec!(25000),
        )
        .unwrap();
        assert_eq!(b.total_budget, dec!(75000));
        assert_eq!(b.monthly_provision_amount, dec!(6250));
    }

    #[test]
    fn edge_no_floating_point_drift_on_sum() {
        // En virgule flottante binaire, 0.10 + 0.20 != 0.30 (dérive IEEE 754).
        // En Decimal c'est 0.30 exact.
        let b = Budget::new(Uuid::new_v4(), Uuid::new_v4(), 2025, dec!(0.10), dec!(0.20)).unwrap();
        assert_eq!(b.total_budget, dec!(0.30));
    }

    #[test]
    fn security_large_budget_no_overflow() {
        // Decimal supporte ~7.9e28 : un budget géant ne panique pas.
        let b = Budget::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            2025,
            dec!(900000000000),
            dec!(100000000000),
        )
        .unwrap();
        assert_eq!(b.total_budget, dec!(1000000000000));
    }

    #[test]
    fn negative_budget_rejected_typed() {
        let r = Budget::new(Uuid::new_v4(), Uuid::new_v4(), 2025, dec!(-1), dec!(0));
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("negative"));
    }

    #[test]
    fn test_create_budget_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000));

        assert!(budget.is_ok());
        let b = budget.unwrap();
        assert_eq!(b.fiscal_year, 2025);
        assert_eq!(b.ordinary_budget, dec!(50000));
        assert_eq!(b.extraordinary_budget, dec!(25000));
        assert_eq!(b.total_budget, dec!(75000));
        assert_eq!(b.monthly_provision_amount, dec!(6250)); // 75000 / 12
        assert_eq!(b.status, BudgetStatus::Draft);
    }

    #[test]
    fn test_create_budget_invalid_year() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Budget::new(org_id, building_id, 1999, dec!(50000), dec!(25000));

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 2000 and 2100"));
    }

    #[test]
    fn test_create_budget_negative_amounts() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result1 = Budget::new(org_id, building_id, 2025, dec!(-1000), dec!(25000));
        assert!(result1.is_err());

        let result2 = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(-1000));
        assert!(result2.is_err());
    }

    #[test]
    fn test_create_budget_zero_total() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Budget::new(org_id, building_id, 2025, dec!(0), dec!(0));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Total budget cannot be zero");
    }

    #[test]
    fn test_submit_for_approval() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();

        assert!(budget.submit_for_approval().is_ok());
        assert_eq!(budget.status, BudgetStatus::Submitted);
        assert!(budget.submitted_date.is_some());
    }

    #[test]
    fn test_approve_budget() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();
        budget.submit_for_approval().unwrap();

        assert!(budget.approve(meeting_id).is_ok());
        assert_eq!(budget.status, BudgetStatus::Approved);
        assert!(budget.approved_date.is_some());
        assert_eq!(budget.approved_by_meeting_id, Some(meeting_id));
        assert!(budget.is_active());
    }

    #[test]
    fn test_reject_budget() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();
        budget.submit_for_approval().unwrap();

        assert!(budget.reject().is_ok());
        assert_eq!(budget.status, BudgetStatus::Rejected);
    }

    #[test]
    fn test_archive_budget() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();
        budget.submit_for_approval().unwrap();
        budget.approve(meeting_id).unwrap();

        assert!(budget.archive().is_ok());
        assert_eq!(budget.status, BudgetStatus::Archived);
        assert!(!budget.is_active());
    }

    #[test]
    fn test_update_amounts_draft() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();

        assert!(budget.update_amounts(dec!(60000), dec!(30000)).is_ok());
        assert_eq!(budget.ordinary_budget, dec!(60000));
        assert_eq!(budget.extraordinary_budget, dec!(30000));
        assert_eq!(budget.total_budget, dec!(90000));
        assert_eq!(budget.monthly_provision_amount, dec!(7500));
    }

    #[test]
    fn test_update_amounts_submitted_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();
        budget.submit_for_approval().unwrap();

        let result = budget.update_amounts(dec!(60000), dec!(30000));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("only update amounts in Draft or Rejected"));
    }

    #[test]
    fn test_workflow_draft_to_approved() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();

        // Draft → Submitted
        assert_eq!(budget.status, BudgetStatus::Draft);
        budget.submit_for_approval().unwrap();
        assert_eq!(budget.status, BudgetStatus::Submitted);

        // Submitted → Approved
        budget.approve(meeting_id).unwrap();
        assert_eq!(budget.status, BudgetStatus::Approved);
        assert!(budget.is_active());
        assert!(!budget.is_editable());

        // Approved → Archived
        budget.archive().unwrap();
        assert_eq!(budget.status, BudgetStatus::Archived);
        assert!(!budget.is_active());
    }

    #[test]
    fn test_workflow_draft_to_rejected_to_resubmit() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();

        // Draft → Submitted → Rejected
        budget.submit_for_approval().unwrap();
        budget.reject().unwrap();
        assert_eq!(budget.status, BudgetStatus::Rejected);
        assert!(budget.is_editable());

        // Rejected → can be resubmitted
        assert!(budget.submit_for_approval().is_ok());
        assert_eq!(budget.status, BudgetStatus::Submitted);
    }

    #[test]
    fn test_update_notes() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, dec!(50000), dec!(25000)).unwrap();

        budget.update_notes("Budget prévisionnel incluant réfection toiture".to_string());
        assert_eq!(
            budget.notes,
            Some("Budget prévisionnel incluant réfection toiture".to_string())
        );
    }
}
