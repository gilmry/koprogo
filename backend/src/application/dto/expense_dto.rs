//! Expense DTOs — monetary fields use `rust_decimal::Decimal` (cf. ADR-0007).
//!
//! Note : `validator` crate `#[validate(range(...))]` ne support pas Decimal
//! avec literals de type f64. Les invariants montants (> 0, taux VAT 0-100)
//! sont enforced dans `Expense::new` / `Expense::new_with_vat` côté domaine
//! (cf. `domain/entities/expense.rs`).

use crate::domain::entities::{ApprovalStatus, ExpenseCategory, PaymentStatus};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

// ========== Legacy DTOs (backward compatibility) ==========

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateExpenseDto {
    #[serde(default)]
    pub organization_id: String,
    pub building_id: String,
    pub category: ExpenseCategory,

    #[validate(length(min = 1))]
    pub description: String,

    /// Montant TTC (validé > 0 dans Expense::new). Decimal exact (cf. ADR-0007).
    pub amount: Decimal,

    pub expense_date: String,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,

    /// Optional Belgian PCMN account code (e.g., "604001" for electricity)
    /// Must reference an existing account in the organization's chart of accounts
    #[validate(length(max = 40))]
    pub account_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExpenseResponseDto {
    pub id: String,
    pub building_id: String,
    pub category: ExpenseCategory,
    pub description: String,
    pub amount: Decimal,
    pub expense_date: String,
    pub payment_status: PaymentStatus,
    pub approval_status: ApprovalStatus,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
    /// Belgian PCMN account code if linked to chart of accounts
    pub account_code: Option<String>,
    /// Contractor report reference for Works category (Issue #309)
    pub contractor_report_id: Option<String>,
}

// ========== New Invoice DTOs (with VAT & Workflow) ==========

/// Créer une facture brouillon avec gestion TVA.
/// Validation des montants > 0 et taux 0-100 effectuée dans `Expense::new_with_vat`.
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateInvoiceDraftDto {
    #[serde(default)]
    pub organization_id: String,
    pub building_id: String,
    pub category: ExpenseCategory,

    #[validate(length(min = 1))]
    pub description: String,

    /// Montant HT (validé > 0 dans `Expense::new_with_vat`).
    pub amount_excl_vat: Decimal,

    /// Taux TVA en % (validé 0..=100 dans `Expense::new_with_vat`).
    pub vat_rate: Decimal,

    pub invoice_date: String,     // ISO 8601
    pub due_date: Option<String>, // ISO 8601
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
}

/// Modifier une facture brouillon ou rejetée.
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UpdateInvoiceDraftDto {
    #[validate(length(min = 1))]
    pub description: Option<String>,

    pub category: Option<ExpenseCategory>,

    pub amount_excl_vat: Option<Decimal>,
    pub vat_rate: Option<Decimal>,

    pub invoice_date: Option<String>,
    pub due_date: Option<String>,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
}

/// Soumettre une facture pour validation (Draft → PendingApproval).
#[derive(Debug, Deserialize, Clone)]
pub struct SubmitForApprovalDto {
    // Empty body, action via PUT /invoices/:id/submit
}

/// Approuver une facture (PendingApproval → Approved).
#[derive(Debug, Deserialize, Clone)]
pub struct ApproveInvoiceDto {
    pub approved_by_user_id: String, // User ID du syndic/admin
}

/// Rejeter une facture avec raison (PendingApproval → Rejected).
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct RejectInvoiceDto {
    pub rejected_by_user_id: String,

    #[validate(length(min = 1))]
    pub rejection_reason: String,
}

/// Créer une ligne de facture.
/// Validations (quantity > 0, unit_price ≥ 0, vat_rate 0..=100) dans `InvoiceLineItem::new`.
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateInvoiceLineItemDto {
    pub expense_id: String,

    #[validate(length(min = 1))]
    pub description: String,

    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
}

// ========== Response DTOs ==========

/// Response enrichie avec tous les champs invoice/workflow.
#[derive(Debug, Serialize, Clone)]
pub struct InvoiceResponseDto {
    pub id: String,
    pub organization_id: String,
    pub building_id: String,
    pub category: ExpenseCategory,
    pub description: String,

    // Montants — exact decimal (cf. ADR-0007)
    pub amount: Decimal, // TTC (backward compatibility)
    pub amount_excl_vat: Option<Decimal>,
    pub vat_rate: Option<Decimal>,
    pub vat_amount: Option<Decimal>,
    pub amount_incl_vat: Option<Decimal>,

    // Dates
    pub expense_date: String,
    pub invoice_date: Option<String>,
    pub due_date: Option<String>,
    pub paid_date: Option<String>,

    // Workflow
    pub approval_status: ApprovalStatus,
    pub submitted_at: Option<String>,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub rejection_reason: Option<String>,

    // Payment
    pub payment_status: PaymentStatus,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,

    /// Contractor report reference for Works category (Issue #309)
    pub contractor_report_id: Option<String>,

    pub created_at: String,
    pub updated_at: String,
}

/// Response pour une ligne de facture.
#[derive(Debug, Serialize)]
pub struct InvoiceLineItemResponseDto {
    pub id: String,
    pub expense_id: String,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub amount_excl_vat: Decimal,
    pub vat_rate: Decimal,
    pub vat_amount: Decimal,
    pub amount_incl_vat: Decimal,
    pub created_at: String,
}

/// Response pour une répartition de charge.
#[derive(Debug, Serialize)]
pub struct ChargeDistributionResponseDto {
    pub id: String,
    pub expense_id: String,
    pub unit_id: String,
    pub owner_id: String,
    /// Quote-part (e.g., dec!(0.25) pour 25%). Decimal exact (cf. ADR-0007).
    pub quota_percentage: Decimal,
    pub amount_due: Decimal,
    pub created_at: String,
}

/// Liste des factures en attente d'approbation (pour syndics).
#[derive(Debug, Serialize)]
pub struct PendingInvoicesListDto {
    pub invoices: Vec<InvoiceResponseDto>,
    pub count: usize,
}
