use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Catégorie de charges
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpenseCategory {
    Maintenance,    // Entretien
    Repairs,        // Réparations
    Insurance,      // Assurance
    Utilities,      // Charges courantes (eau, électricité)
    Cleaning,       // Nettoyage
    Administration, // Administration
    Works,          // Travaux
    Other,
}

/// Statut de paiement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Paid,
    Overdue,
    Cancelled,
}

/// Statut d'approbation pour le workflow de validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalStatus {
    Draft,           // Brouillon - en cours d'édition
    PendingApproval, // Soumis pour validation
    Approved,        // Approuvé par le syndic
    Rejected,        // Rejeté
}

/// Représente une charge de copropriété / facture
///
/// Conforme au PCMN belge (Plan Comptable Minimum Normalisé).
/// Chaque charge peut être liée à un compte comptable via account_code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Expense {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub category: ExpenseCategory,
    pub description: String,

    // Montants et TVA
    pub amount: f64,                  // Montant TTC (backward compatibility)
    pub amount_excl_vat: Option<f64>, // Montant HT
    pub vat_rate: Option<f64>,        // Taux TVA (ex: 21.0 pour 21%)
    pub vat_amount: Option<f64>,      // Montant TVA
    pub amount_incl_vat: Option<f64>, // Montant TTC (explicite)

    // Dates multiples
    pub expense_date: DateTime<Utc>, // Date originale (backward compatibility)
    pub invoice_date: Option<DateTime<Utc>>, // Date de la facture
    pub due_date: Option<DateTime<Utc>>, // Date d'échéance
    pub paid_date: Option<DateTime<Utc>>, // Date de paiement effectif

    // Workflow de validation
    pub approval_status: ApprovalStatus,
    pub submitted_at: Option<DateTime<Utc>>, // Date de soumission pour validation
    pub approved_by: Option<Uuid>,           // User ID qui a approuvé/rejeté
    pub approved_at: Option<DateTime<Utc>>,  // Date d'approbation/rejet
    pub rejection_reason: Option<String>,    // Raison du rejet

    // Statut et métadonnées
    pub payment_status: PaymentStatus,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
    /// Code du compte comptable PCMN (e.g., "604001" for electricity, "611002" for elevator maintenance)
    /// References: accounts.code column in the database
    pub account_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Expense {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        category: ExpenseCategory,
        description: String,
        amount: f64,
        expense_date: DateTime<Utc>,
        supplier: Option<String>,
        invoice_number: Option<String>,
        account_code: Option<String>,
    ) -> Result<Self, String> {
        if description.is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        if amount <= 0.0 {
            return Err("Amount must be greater than 0".to_string());
        }

        // Validate account_code format if provided (Belgian PCMN codes)
        if let Some(ref code) = account_code {
            if code.is_empty() {
                return Err("Account code cannot be empty if provided".to_string());
            }
            // Belgian PCMN codes are typically 1-10 characters (e.g., "6", "60", "604001")
            if code.len() > 40 {
                return Err("Account code cannot exceed 40 characters".to_string());
            }
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            category,
            description,
            amount,
            amount_excl_vat: None,
            vat_rate: None,
            vat_amount: None,
            amount_incl_vat: Some(amount), // Pour compatibilité, amount = TTC
            expense_date,
            invoice_date: None,
            due_date: None,
            paid_date: None,
            approval_status: ApprovalStatus::Draft, // Par défaut en brouillon
            submitted_at: None,
            approved_by: None,
            approved_at: None,
            rejection_reason: None,
            payment_status: PaymentStatus::Pending,
            supplier,
            invoice_number,
            account_code,
            created_at: now,
            updated_at: now,
        })
    }

    /// Crée une facture avec gestion complète de la TVA
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_vat(
        organization_id: Uuid,
        building_id: Uuid,
        category: ExpenseCategory,
        description: String,
        amount_excl_vat: f64,
        vat_rate: f64,
        invoice_date: DateTime<Utc>,
        due_date: Option<DateTime<Utc>>,
        supplier: Option<String>,
        invoice_number: Option<String>,
        account_code: Option<String>,
    ) -> Result<Self, String> {
        if description.is_empty() {
            return Err("Description cannot be empty".to_string());
        }
        if amount_excl_vat <= 0.0 {
            return Err("Amount (excl. VAT) must be greater than 0".to_string());
        }
        if !(0.0..=100.0).contains(&vat_rate) {
            return Err("VAT rate must be between 0 and 100".to_string());
        }

        // Calcul automatique de la TVA
        let vat_amount = (amount_excl_vat * vat_rate) / 100.0;
        let amount_incl_vat = amount_excl_vat + vat_amount;

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            category,
            description,
            amount: amount_incl_vat, // Backward compatibility
            amount_excl_vat: Some(amount_excl_vat),
            vat_rate: Some(vat_rate),
            vat_amount: Some(vat_amount),
            amount_incl_vat: Some(amount_incl_vat),
            expense_date: invoice_date, // Backward compatibility
            invoice_date: Some(invoice_date),
            due_date,
            paid_date: None,
            approval_status: ApprovalStatus::Draft,
            submitted_at: None,
            approved_by: None,
            approved_at: None,
            rejection_reason: None,
            payment_status: PaymentStatus::Pending,
            supplier,
            invoice_number,
            account_code,
            created_at: now,
            updated_at: now,
        })
    }

    /// Recalcule la TVA si le montant HT ou le taux change
    pub fn recalculate_vat(&mut self) -> Result<(), String> {
        if let (Some(amount_excl_vat), Some(vat_rate)) = (self.amount_excl_vat, self.vat_rate) {
            if amount_excl_vat <= 0.0 {
                return Err("Amount (excl. VAT) must be greater than 0".to_string());
            }
            if !(0.0..=100.0).contains(&vat_rate) {
                return Err("VAT rate must be between 0 and 100".to_string());
            }

            let vat_amount = (amount_excl_vat * vat_rate) / 100.0;
            let amount_incl_vat = amount_excl_vat + vat_amount;

            self.vat_amount = Some(vat_amount);
            self.amount_incl_vat = Some(amount_incl_vat);
            self.amount = amount_incl_vat; // Backward compatibility
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err("Cannot recalculate VAT: amount_excl_vat or vat_rate is missing".to_string())
        }
    }

    /// Soumet la facture pour validation (Draft → PendingApproval)
    pub fn submit_for_approval(&mut self) -> Result<(), String> {
        match self.approval_status {
            ApprovalStatus::Draft => {
                self.approval_status = ApprovalStatus::PendingApproval;
                self.submitted_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            ApprovalStatus::Rejected => {
                // Permet de re-soumettre une facture rejetée
                self.approval_status = ApprovalStatus::PendingApproval;
                self.submitted_at = Some(Utc::now());
                self.rejection_reason = None; // Efface la raison du rejet précédent
                self.updated_at = Utc::now();
                Ok(())
            }
            ApprovalStatus::PendingApproval => {
                Err("Invoice is already pending approval".to_string())
            }
            ApprovalStatus::Approved => Err("Cannot submit an approved invoice".to_string()),
        }
    }

    /// Approuve la facture (PendingApproval → Approved)
    pub fn approve(&mut self, approved_by_user_id: Uuid) -> Result<(), String> {
        match self.approval_status {
            ApprovalStatus::PendingApproval => {
                self.approval_status = ApprovalStatus::Approved;
                self.approved_by = Some(approved_by_user_id);
                self.approved_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            ApprovalStatus::Draft => {
                Err("Cannot approve a draft invoice (must be submitted first)".to_string())
            }
            ApprovalStatus::Approved => Err("Invoice is already approved".to_string()),
            ApprovalStatus::Rejected => {
                Err("Cannot approve a rejected invoice (resubmit first)".to_string())
            }
        }
    }

    /// Rejette la facture avec une raison (PendingApproval → Rejected)
    pub fn reject(&mut self, rejected_by_user_id: Uuid, reason: String) -> Result<(), String> {
        if reason.trim().is_empty() {
            return Err("Rejection reason cannot be empty".to_string());
        }

        match self.approval_status {
            ApprovalStatus::PendingApproval => {
                self.approval_status = ApprovalStatus::Rejected;
                self.approved_by = Some(rejected_by_user_id); // Celui qui a rejeté
                self.approved_at = Some(Utc::now());
                self.rejection_reason = Some(reason);
                self.updated_at = Utc::now();
                Ok(())
            }
            ApprovalStatus::Draft => {
                Err("Cannot reject a draft invoice (not submitted)".to_string())
            }
            ApprovalStatus::Approved => Err("Cannot reject an approved invoice".to_string()),
            ApprovalStatus::Rejected => Err("Invoice is already rejected".to_string()),
        }
    }

    /// Vérifie si la facture peut être modifiée (uniquement en Draft ou Rejected)
    pub fn can_be_modified(&self) -> bool {
        matches!(
            self.approval_status,
            ApprovalStatus::Draft | ApprovalStatus::Rejected
        )
    }

    /// Vérifie si la facture est approuvée
    pub fn is_approved(&self) -> bool {
        self.approval_status == ApprovalStatus::Approved
    }

    pub fn mark_as_paid(&mut self) -> Result<(), String> {
        match self.payment_status {
            PaymentStatus::Pending | PaymentStatus::Overdue => {
                self.payment_status = PaymentStatus::Paid;
                self.paid_date = Some(Utc::now()); // Enregistre la date de paiement effectif
                self.updated_at = Utc::now();
                Ok(())
            }
            PaymentStatus::Paid => Err("Expense is already paid".to_string()),
            PaymentStatus::Cancelled => Err("Cannot mark a cancelled expense as paid".to_string()),
        }
    }

    pub fn mark_as_overdue(&mut self) -> Result<(), String> {
        match self.payment_status {
            PaymentStatus::Pending => {
                self.payment_status = PaymentStatus::Overdue;
                self.updated_at = Utc::now();
                Ok(())
            }
            PaymentStatus::Overdue => Err("Expense is already overdue".to_string()),
            PaymentStatus::Paid => Err("Cannot mark a paid expense as overdue".to_string()),
            PaymentStatus::Cancelled => {
                Err("Cannot mark a cancelled expense as overdue".to_string())
            }
        }
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        match self.payment_status {
            PaymentStatus::Pending | PaymentStatus::Overdue => {
                self.payment_status = PaymentStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(())
            }
            PaymentStatus::Paid => Err("Cannot cancel a paid expense".to_string()),
            PaymentStatus::Cancelled => Err("Expense is already cancelled".to_string()),
        }
    }

    pub fn reactivate(&mut self) -> Result<(), String> {
        match self.payment_status {
            PaymentStatus::Cancelled => {
                self.payment_status = PaymentStatus::Pending;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err("Can only reactivate cancelled expenses".to_string()),
        }
    }

    pub fn unpay(&mut self) -> Result<(), String> {
        match self.payment_status {
            PaymentStatus::Paid => {
                self.payment_status = PaymentStatus::Pending;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err("Can only unpay paid expenses".to_string()),
        }
    }

    pub fn is_paid(&self) -> bool {
        self.payment_status == PaymentStatus::Paid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_expense_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Entretien ascenseur".to_string(),
            500.0,
            Utc::now(),
            Some("ACME Elevators".to_string()),
            Some("INV-2024-001".to_string()),
            Some("611002".to_string()), // Elevator maintenance account (Belgian PCMN)
        );

        assert!(expense.is_ok());
        let expense = expense.unwrap();
        assert_eq!(expense.organization_id, org_id);
        assert_eq!(expense.amount, 500.0);
        assert_eq!(expense.payment_status, PaymentStatus::Pending);
        assert_eq!(expense.account_code, Some("611002".to_string()));
    }

    #[test]
    fn test_create_expense_without_account_code() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Other,
            "Miscellaneous expense".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            None, // No account code
        );

        assert!(expense.is_ok());
        let expense = expense.unwrap();
        assert_eq!(expense.account_code, None);
    }

    #[test]
    fn test_create_expense_empty_account_code_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            Some("".to_string()), // Empty account code should fail
        );

        assert!(expense.is_err());
        assert!(expense
            .unwrap_err()
            .contains("Account code cannot be empty"));
    }

    #[test]
    fn test_create_expense_long_account_code_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let long_code = "a".repeat(41); // 41 characters, exceeds limit
        let expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            Some(long_code),
        );

        assert!(expense.is_err());
        assert!(expense
            .unwrap_err()
            .contains("Account code cannot exceed 40 characters"));
    }

    #[test]
    fn test_create_expense_negative_amount_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            -100.0,
            Utc::now(),
            None,
            None,
            None,
        );

        assert!(expense.is_err());
    }

    #[test]
    fn test_mark_expense_as_paid() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let mut expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            None,
        )
        .unwrap();

        assert!(!expense.is_paid());
        let result = expense.mark_as_paid();
        assert!(result.is_ok());
        assert!(expense.is_paid());
    }

    #[test]
    fn test_mark_paid_expense_as_paid_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let mut expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            None,
        )
        .unwrap();

        expense.mark_as_paid().unwrap();
        let result = expense.mark_as_paid();
        assert!(result.is_err());
    }

    #[test]
    fn test_cancel_expense() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let mut expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            None,
        )
        .unwrap();

        let result = expense.cancel();
        assert!(result.is_ok());
        assert_eq!(expense.payment_status, PaymentStatus::Cancelled);
    }

    #[test]
    fn test_reactivate_expense() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let mut expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            None,
        )
        .unwrap();

        expense.cancel().unwrap();
        let result = expense.reactivate();
        assert!(result.is_ok());
        assert_eq!(expense.payment_status, PaymentStatus::Pending);
    }

    // ========== Tests pour gestion TVA ==========

    #[test]
    fn test_create_invoice_with_vat_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let invoice_date = Utc::now();
        let due_date = invoice_date + chrono::Duration::days(30);

        let invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Réparation toiture".to_string(),
            1000.0, // HT
            21.0,   // TVA 21%
            invoice_date,
            Some(due_date),
            Some("BatiPro SPRL".to_string()),
            Some("INV-2025-042".to_string()),
            None, // account_code
        );

        assert!(invoice.is_ok());
        let invoice = invoice.unwrap();
        assert_eq!(invoice.amount_excl_vat, Some(1000.0));
        assert_eq!(invoice.vat_rate, Some(21.0));
        assert_eq!(invoice.vat_amount, Some(210.0));
        assert_eq!(invoice.amount_incl_vat, Some(1210.0));
        assert_eq!(invoice.amount, 1210.0); // Backward compatibility
        assert_eq!(invoice.approval_status, ApprovalStatus::Draft);
    }

    #[test]
    fn test_create_invoice_with_vat_6_percent() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Works,
            "Rénovation énergétique".to_string(),
            5000.0, // HT
            6.0,    // TVA réduite 6%
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        assert_eq!(invoice.vat_amount, Some(300.0));
        assert_eq!(invoice.amount_incl_vat, Some(5300.0));
    }

    #[test]
    fn test_create_invoice_negative_vat_rate_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            -5.0, // Taux négatif invalide
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        );

        assert!(invoice.is_err());
        assert_eq!(invoice.unwrap_err(), "VAT rate must be between 0 and 100");
    }

    #[test]
    fn test_create_invoice_vat_rate_above_100_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            150.0, // Taux > 100% invalide
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        );

        assert!(invoice.is_err());
    }

    #[test]
    fn test_recalculate_vat_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        // Modifier le montant HT
        invoice.amount_excl_vat = Some(1500.0);
        let result = invoice.recalculate_vat();

        assert!(result.is_ok());
        assert_eq!(invoice.vat_amount, Some(315.0)); // 1500 * 21% = 315
        assert_eq!(invoice.amount_incl_vat, Some(1815.0));
    }

    #[test]
    fn test_recalculate_vat_without_vat_data_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Créer une expense classique sans TVA
        let mut expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            None, // account_code
        )
        .unwrap();

        let result = expense.recalculate_vat();
        assert!(result.is_err());
    }

    // ========== Tests pour workflow de validation ==========

    #[test]
    fn test_submit_draft_invoice_for_approval() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        assert_eq!(invoice.approval_status, ApprovalStatus::Draft);
        assert!(invoice.submitted_at.is_none());

        let result = invoice.submit_for_approval();
        assert!(result.is_ok());
        assert_eq!(invoice.approval_status, ApprovalStatus::PendingApproval);
        assert!(invoice.submitted_at.is_some());
    }

    #[test]
    fn test_submit_already_pending_invoice_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        invoice.submit_for_approval().unwrap();
        let result = invoice.submit_for_approval();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invoice is already pending approval");
    }

    #[test]
    fn test_resubmit_rejected_invoice() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        invoice.submit_for_approval().unwrap();
        invoice
            .reject(user_id, "Montant incorrect".to_string())
            .unwrap();
        assert_eq!(invoice.approval_status, ApprovalStatus::Rejected);

        // Re-soumettre une facture rejetée devrait fonctionner
        let result = invoice.submit_for_approval();
        assert!(result.is_ok());
        assert_eq!(invoice.approval_status, ApprovalStatus::PendingApproval);
        assert!(invoice.rejection_reason.is_none()); // Raison effacée
    }

    #[test]
    fn test_approve_pending_invoice() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let syndic_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        invoice.submit_for_approval().unwrap();
        let result = invoice.approve(syndic_id);

        assert!(result.is_ok());
        assert_eq!(invoice.approval_status, ApprovalStatus::Approved);
        assert_eq!(invoice.approved_by, Some(syndic_id));
        assert!(invoice.approved_at.is_some());
        assert!(invoice.is_approved());
    }

    #[test]
    fn test_approve_draft_invoice_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let syndic_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        // Ne PAS soumettre, tenter d'approuver directement
        let result = invoice.approve(syndic_id);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be submitted first"));
    }

    #[test]
    fn test_reject_pending_invoice_with_reason() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let syndic_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        invoice.submit_for_approval().unwrap();
        let result = invoice.reject(
            syndic_id,
            "Le montant ne correspond pas au devis".to_string(),
        );

        assert!(result.is_ok());
        assert_eq!(invoice.approval_status, ApprovalStatus::Rejected);
        assert_eq!(invoice.approved_by, Some(syndic_id));
        assert_eq!(
            invoice.rejection_reason,
            Some("Le montant ne correspond pas au devis".to_string())
        );
    }

    #[test]
    fn test_reject_invoice_without_reason_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let syndic_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        invoice.submit_for_approval().unwrap();
        let result = invoice.reject(syndic_id, "".to_string());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Rejection reason cannot be empty");
    }

    #[test]
    fn test_can_be_modified_draft() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        assert!(invoice.can_be_modified()); // Draft peut être modifié
    }

    #[test]
    fn test_can_be_modified_rejected() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let syndic_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        invoice.submit_for_approval().unwrap();
        invoice.reject(syndic_id, "Erreur".to_string()).unwrap();

        assert!(invoice.can_be_modified()); // Rejected peut être modifié
    }

    #[test]
    fn test_cannot_modify_approved_invoice() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let syndic_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            1000.0,
            21.0,
            Utc::now(),
            None,
            None,
            None,
            None, // account_code
        )
        .unwrap();

        invoice.submit_for_approval().unwrap();
        invoice.approve(syndic_id).unwrap();

        assert!(!invoice.can_be_modified()); // Approved ne peut PAS être modifié
    }

    #[test]
    fn test_mark_as_paid_sets_paid_date() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut expense = Expense::new(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Test".to_string(),
            100.0,
            Utc::now(),
            None,
            None,
            None, // account_code
        )
        .unwrap();

        assert!(expense.paid_date.is_none());
        expense.mark_as_paid().unwrap();
        assert!(expense.paid_date.is_some());
        assert!(expense.is_paid());
    }

    #[test]
    fn test_workflow_complete_cycle() {
        // Test du cycle complet : Draft → Submit → Approve → Pay
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let syndic_id = Uuid::new_v4();

        let mut invoice = Expense::new_with_vat(
            org_id,
            building_id,
            ExpenseCategory::Maintenance,
            "Entretien annuel".to_string(),
            2000.0,
            21.0,
            Utc::now(),
            Some(Utc::now() + chrono::Duration::days(30)),
            Some("MaintenancePro".to_string()),
            Some("INV-2025-100".to_string()),
            None, // account_code
        )
        .unwrap();

        // Étape 1: Draft
        assert_eq!(invoice.approval_status, ApprovalStatus::Draft);
        assert!(invoice.can_be_modified());

        // Étape 2: Soumettre
        invoice.submit_for_approval().unwrap();
        assert_eq!(invoice.approval_status, ApprovalStatus::PendingApproval);
        assert!(!invoice.can_be_modified());

        // Étape 3: Approuver
        invoice.approve(syndic_id).unwrap();
        assert_eq!(invoice.approval_status, ApprovalStatus::Approved);
        assert!(invoice.is_approved());

        // Étape 4: Payer
        invoice.mark_as_paid().unwrap();
        assert!(invoice.is_paid());
        assert!(invoice.paid_date.is_some());
    }
}
