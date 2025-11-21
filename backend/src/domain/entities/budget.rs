use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Statut du budget annuel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
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
    pub ordinary_budget: f64,

    /// Budget charges extraordinaires (€) - Travaux et dépenses exceptionnelles
    pub extraordinary_budget: f64,

    /// Budget total (€) = ordinaire + extraordinaire
    pub total_budget: f64,

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
    pub monthly_provision_amount: f64,

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
        ordinary_budget: f64,
        extraordinary_budget: f64,
    ) -> Result<Self, String> {
        // Validations
        if fiscal_year < 2000 || fiscal_year > 2100 {
            return Err("Fiscal year must be between 2000 and 2100".to_string());
        }

        if ordinary_budget < 0.0 {
            return Err("Ordinary budget cannot be negative".to_string());
        }

        if extraordinary_budget < 0.0 {
            return Err("Extraordinary budget cannot be negative".to_string());
        }

        let total_budget = ordinary_budget + extraordinary_budget;

        if total_budget == 0.0 {
            return Err("Total budget cannot be zero".to_string());
        }

        // Calcul provisions mensuelles
        let monthly_provision_amount = total_budget / 12.0;

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
        ordinary_budget: f64,
        extraordinary_budget: f64,
    ) -> Result<(), String> {
        if self.status != BudgetStatus::Draft {
            return Err("Can only update amounts in Draft status".to_string());
        }

        if ordinary_budget < 0.0 {
            return Err("Ordinary budget cannot be negative".to_string());
        }

        if extraordinary_budget < 0.0 {
            return Err("Extraordinary budget cannot be negative".to_string());
        }

        let total_budget = ordinary_budget + extraordinary_budget;

        if total_budget == 0.0 {
            return Err("Total budget cannot be zero".to_string());
        }

        self.ordinary_budget = ordinary_budget;
        self.extraordinary_budget = extraordinary_budget;
        self.total_budget = total_budget;
        self.monthly_provision_amount = total_budget / 12.0;
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

    #[test]
    fn test_create_budget_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0);

        assert!(budget.is_ok());
        let b = budget.unwrap();
        assert_eq!(b.fiscal_year, 2025);
        assert_eq!(b.ordinary_budget, 50000.0);
        assert_eq!(b.extraordinary_budget, 25000.0);
        assert_eq!(b.total_budget, 75000.0);
        assert_eq!(b.monthly_provision_amount, 6250.0); // 75000 / 12
        assert_eq!(b.status, BudgetStatus::Draft);
    }

    #[test]
    fn test_create_budget_invalid_year() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Budget::new(org_id, building_id, 1999, 50000.0, 25000.0);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 2000 and 2100"));
    }

    #[test]
    fn test_create_budget_negative_amounts() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result1 = Budget::new(org_id, building_id, 2025, -1000.0, 25000.0);
        assert!(result1.is_err());

        let result2 = Budget::new(org_id, building_id, 2025, 50000.0, -1000.0);
        assert!(result2.is_err());
    }

    #[test]
    fn test_create_budget_zero_total() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = Budget::new(org_id, building_id, 2025, 0.0, 0.0);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Total budget cannot be zero");
    }

    #[test]
    fn test_submit_for_approval() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();

        assert!(budget.submit_for_approval().is_ok());
        assert_eq!(budget.status, BudgetStatus::Submitted);
        assert!(budget.submitted_date.is_some());
    }

    #[test]
    fn test_approve_budget() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();
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

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();
        budget.submit_for_approval().unwrap();

        assert!(budget.reject().is_ok());
        assert_eq!(budget.status, BudgetStatus::Rejected);
    }

    #[test]
    fn test_archive_budget() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();
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

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();

        assert!(budget.update_amounts(60000.0, 30000.0).is_ok());
        assert_eq!(budget.ordinary_budget, 60000.0);
        assert_eq!(budget.extraordinary_budget, 30000.0);
        assert_eq!(budget.total_budget, 90000.0);
        assert_eq!(budget.monthly_provision_amount, 7500.0);
    }

    #[test]
    fn test_update_amounts_submitted_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();
        budget.submit_for_approval().unwrap();

        let result = budget.update_amounts(60000.0, 30000.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("only update amounts in Draft"));
    }

    #[test]
    fn test_workflow_draft_to_approved() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let meeting_id = Uuid::new_v4();

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();

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

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();

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

        let mut budget = Budget::new(org_id, building_id, 2025, 50000.0, 25000.0).unwrap();

        budget.update_notes("Budget prévisionnel incluant réfection toiture".to_string());
        assert_eq!(
            budget.notes,
            Some("Budget prévisionnel incluant réfection toiture".to_string())
        );
    }
}
