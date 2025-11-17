use crate::domain::entities::{ApprovalStatus, ExpenseCategory, PaymentStatus};
use serde::{Deserialize, Serialize};
use validator::Validate;

// ========== Legacy DTOs (backward compatibility) ==========

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateExpenseDto {
    pub organization_id: String,
    pub building_id: String,
    pub category: ExpenseCategory,

    #[validate(length(min = 1))]
    pub description: String,

    #[validate(range(min = 0.01))]
    pub amount: f64,

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
    pub amount: f64,
    pub expense_date: String,
    pub payment_status: PaymentStatus,
    pub approval_status: ApprovalStatus,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
    /// Belgian PCMN account code if linked to chart of accounts
    pub account_code: Option<String>,
}

// ========== New Invoice DTOs (with VAT & Workflow) ==========

/// Créer une facture brouillon avec gestion TVA
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateInvoiceDraftDto {
    pub organization_id: String,
    pub building_id: String,
    pub category: ExpenseCategory,

    #[validate(length(min = 1))]
    pub description: String,

    #[validate(range(min = 0.01))]
    pub amount_excl_vat: f64, // Montant HT

    #[validate(range(min = 0.0, max = 100.0))]
    pub vat_rate: f64, // Taux TVA (21.0 pour 21%)

    pub invoice_date: String,     // ISO 8601
    pub due_date: Option<String>, // ISO 8601
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
}

/// Modifier une facture brouillon ou rejetée
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct UpdateInvoiceDraftDto {
    #[validate(length(min = 1))]
    pub description: Option<String>,

    pub category: Option<ExpenseCategory>,

    #[validate(range(min = 0.01))]
    pub amount_excl_vat: Option<f64>,

    #[validate(range(min = 0.0, max = 100.0))]
    pub vat_rate: Option<f64>,

    pub invoice_date: Option<String>,
    pub due_date: Option<String>,
    pub supplier: Option<String>,
    pub invoice_number: Option<String>,
}

/// Soumettre une facture pour validation (Draft → PendingApproval)
#[derive(Debug, Deserialize, Clone)]
pub struct SubmitForApprovalDto {
    // Empty body, action via PUT /invoices/:id/submit
}

/// Approuver une facture (PendingApproval → Approved)
#[derive(Debug, Deserialize, Clone)]
pub struct ApproveInvoiceDto {
    pub approved_by_user_id: String, // User ID du syndic/admin
}

/// Rejeter une facture avec raison (PendingApproval → Rejected)
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct RejectInvoiceDto {
    pub rejected_by_user_id: String,

    #[validate(length(min = 1))]
    pub rejection_reason: String,
}

/// Créer une ligne de facture
#[derive(Debug, Deserialize, Validate, Clone)]
pub struct CreateInvoiceLineItemDto {
    pub expense_id: String,

    #[validate(length(min = 1))]
    pub description: String,

    #[validate(range(min = 0.01))]
    pub quantity: f64,

    #[validate(range(min = 0.0))]
    pub unit_price: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub vat_rate: f64,
}

// ========== Response DTOs ==========

/// Response enrichie avec tous les champs invoice/workflow
#[derive(Debug, Serialize)]
pub struct InvoiceResponseDto {
    pub id: String,
    pub organization_id: String,
    pub building_id: String,
    pub category: ExpenseCategory,
    pub description: String,

    // Montants
    pub amount: f64, // TTC (backward compatibility)
    pub amount_excl_vat: Option<f64>,
    pub vat_rate: Option<f64>,
    pub vat_amount: Option<f64>,
    pub amount_incl_vat: Option<f64>,

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

    pub created_at: String,
    pub updated_at: String,
}

/// Response pour une ligne de facture
#[derive(Debug, Serialize)]
pub struct InvoiceLineItemResponseDto {
    pub id: String,
    pub expense_id: String,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub amount_excl_vat: f64,
    pub vat_rate: f64,
    pub vat_amount: f64,
    pub amount_incl_vat: f64,
    pub created_at: String,
}

/// Response pour une répartition de charge
#[derive(Debug, Serialize)]
pub struct ChargeDistributionResponseDto {
    pub id: String,
    pub expense_id: String,
    pub unit_id: String,
    pub owner_id: String,
    pub quota_percentage: f64, // Sera converti en pourcentage côté client (0.25 → 25%)
    pub amount_due: f64,
    pub created_at: String,
}

/// Liste des factures en attente d'approbation (pour syndics)
#[derive(Debug, Serialize)]
pub struct PendingInvoicesListDto {
    pub invoices: Vec<InvoiceResponseDto>,
    pub count: usize,
}
