use crate::application::dto::{
    ApproveInvoiceDto, CreateExpenseDto, CreateInvoiceDraftDto, ExpenseFilters, ExpenseResponseDto,
    InvoiceResponseDto, PageRequest, PendingInvoicesListDto, RejectInvoiceDto, SortOrder,
    SubmitForApprovalDto, UpdateInvoiceDraftDto,
};
use crate::application::ports::ExpenseRepository;
use crate::application::services::expense_accounting_service::ExpenseAccountingService;
use crate::domain::entities::{ApprovalStatus, Expense};
use chrono::DateTime;
use std::sync::Arc;
use uuid::Uuid;

pub struct ExpenseUseCases {
    repository: Arc<dyn ExpenseRepository>,
    accounting_service: Option<Arc<ExpenseAccountingService>>,
}

impl ExpenseUseCases {
    pub fn new(repository: Arc<dyn ExpenseRepository>) -> Self {
        Self {
            repository,
            accounting_service: None,
        }
    }

    pub fn with_accounting_service(
        repository: Arc<dyn ExpenseRepository>,
        accounting_service: Arc<ExpenseAccountingService>,
    ) -> Self {
        Self {
            repository,
            accounting_service: Some(accounting_service),
        }
    }

    pub async fn create_expense(
        &self,
        dto: CreateExpenseDto,
    ) -> Result<ExpenseResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building ID format".to_string())?;

        let expense_date = DateTime::parse_from_rfc3339(&dto.expense_date)
            .map_err(|_| "Invalid date format".to_string())?
            .with_timezone(&chrono::Utc);

        let expense = Expense::new(
            organization_id,
            building_id,
            dto.category,
            dto.description,
            dto.amount,
            expense_date,
            dto.supplier,
            dto.invoice_number,
            dto.account_code,
        )?;

        let created = self.repository.create(&expense).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_expense(&self, id: Uuid) -> Result<Option<ExpenseResponseDto>, String> {
        let expense = self.repository.find_by_id(id).await?;
        Ok(expense.map(|e| self.to_response_dto(&e)))
    }

    pub async fn list_expenses_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<ExpenseResponseDto>, String> {
        let expenses = self.repository.find_by_building(building_id).await?;
        Ok(expenses.iter().map(|e| self.to_response_dto(e)).collect())
    }

    pub async fn list_expenses_paginated(
        &self,
        page_request: &PageRequest,
        organization_id: Option<Uuid>,
    ) -> Result<(Vec<ExpenseResponseDto>, i64), String> {
        let filters = ExpenseFilters {
            organization_id,
            ..Default::default()
        };

        let (expenses, total) = self
            .repository
            .find_all_paginated(page_request, &filters)
            .await?;

        let dtos = expenses.iter().map(|e| self.to_response_dto(e)).collect();
        Ok((dtos, total))
    }

    /// Marquer une charge comme payée
    ///
    /// Crée automatiquement l'écriture comptable de paiement (FIN - Financier)
    pub async fn mark_as_paid(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.mark_as_paid()?;

        let updated = self.repository.update(&expense).await?;

        // Générer automatiquement l'écriture comptable de paiement
        if let Some(ref accounting_service) = self.accounting_service {
            if let Err(e) = accounting_service
                .generate_payment_entry(&updated, None, None)
                .await
            {
                log::warn!(
                    "Failed to generate payment journal entry for expense {}: {}",
                    updated.id,
                    e
                );
                // Ne pas échouer le paiement si la création de l'écriture échoue
                // L'écriture peut être créée manuellement plus tard
            }
        }

        Ok(self.to_response_dto(&updated))
    }

    pub async fn mark_as_overdue(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.mark_as_overdue()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn cancel_expense(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.cancel()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn reactivate_expense(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.reactivate()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn unpay_expense(&self, id: Uuid) -> Result<ExpenseResponseDto, String> {
        let mut expense = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Expense not found".to_string())?;

        expense.unpay()?;

        let updated = self.repository.update(&expense).await?;
        Ok(self.to_response_dto(&updated))
    }

    // ========== Invoice Workflow Methods (Issue #73) ==========

    /// Créer une facture brouillon avec gestion TVA
    pub async fn create_invoice_draft(
        &self,
        dto: CreateInvoiceDraftDto,
    ) -> Result<InvoiceResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building ID format".to_string())?;

        let invoice_date = DateTime::parse_from_rfc3339(&dto.invoice_date)
            .map_err(|_| "Invalid invoice_date format".to_string())?
            .with_timezone(&chrono::Utc);

        let due_date = dto
            .due_date
            .map(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .map_err(|_| "Invalid due_date format".to_string())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            })
            .transpose()?;

        let invoice = Expense::new_with_vat(
            organization_id,
            building_id,
            dto.category,
            dto.description,
            dto.amount_excl_vat,
            dto.vat_rate,
            invoice_date,
            due_date,
            dto.supplier,
            dto.invoice_number,
            None, // account_code (can be added later)
        )?;

        let created = self.repository.create(&invoice).await?;
        Ok(self.to_invoice_response_dto(&created))
    }

    /// Modifier une facture brouillon ou rejetée
    pub async fn update_invoice_draft(
        &self,
        invoice_id: Uuid,
        dto: UpdateInvoiceDraftDto,
    ) -> Result<InvoiceResponseDto, String> {
        let mut invoice = self
            .repository
            .find_by_id(invoice_id)
            .await?
            .ok_or_else(|| "Invoice not found".to_string())?;

        // Vérifier que la facture peut être modifiée
        if !invoice.can_be_modified() {
            return Err(format!(
                "Invoice cannot be modified (status: {:?})",
                invoice.approval_status
            ));
        }

        // Appliquer les modifications
        if let Some(desc) = dto.description {
            invoice.description = desc;
        }
        if let Some(cat) = dto.category {
            invoice.category = cat;
        }
        if let Some(amount_ht) = dto.amount_excl_vat {
            invoice.amount_excl_vat = Some(amount_ht);
        }
        if let Some(vat_rate) = dto.vat_rate {
            invoice.vat_rate = Some(vat_rate);
        }

        // Recalculer la TVA si nécessaire
        if dto.amount_excl_vat.is_some() || dto.vat_rate.is_some() {
            invoice.recalculate_vat()?;
        }

        if let Some(inv_date) = dto.invoice_date {
            let parsed_date = DateTime::parse_from_rfc3339(&inv_date)
                .map_err(|_| "Invalid invoice_date format".to_string())?
                .with_timezone(&chrono::Utc);
            invoice.invoice_date = Some(parsed_date);
        }

        if let Some(due_date_str) = dto.due_date {
            let parsed_date = DateTime::parse_from_rfc3339(&due_date_str)
                .map_err(|_| "Invalid due_date format".to_string())?
                .with_timezone(&chrono::Utc);
            invoice.due_date = Some(parsed_date);
        }

        if dto.supplier.is_some() {
            invoice.supplier = dto.supplier;
        }
        if dto.invoice_number.is_some() {
            invoice.invoice_number = dto.invoice_number;
        }

        invoice.updated_at = chrono::Utc::now();

        let updated = self.repository.update(&invoice).await?;
        Ok(self.to_invoice_response_dto(&updated))
    }

    /// Soumettre une facture pour validation (Draft → PendingApproval)
    pub async fn submit_for_approval(
        &self,
        invoice_id: Uuid,
        _dto: SubmitForApprovalDto,
    ) -> Result<InvoiceResponseDto, String> {
        let mut invoice = self
            .repository
            .find_by_id(invoice_id)
            .await?
            .ok_or_else(|| "Invoice not found".to_string())?;

        invoice.submit_for_approval()?;

        let updated = self.repository.update(&invoice).await?;
        Ok(self.to_invoice_response_dto(&updated))
    }

    /// Approuver une facture (PendingApproval → Approved)
    ///
    /// Crée automatiquement l'écriture comptable correspondante (ACH - Achats)
    pub async fn approve_invoice(
        &self,
        invoice_id: Uuid,
        dto: ApproveInvoiceDto,
    ) -> Result<InvoiceResponseDto, String> {
        let mut invoice = self
            .repository
            .find_by_id(invoice_id)
            .await?
            .ok_or_else(|| "Invoice not found".to_string())?;

        let approved_by_user_id = Uuid::parse_str(&dto.approved_by_user_id)
            .map_err(|_| "Invalid approved_by_user_id format".to_string())?;

        invoice.approve(approved_by_user_id)?;

        let updated = self.repository.update(&invoice).await?;

        // Générer automatiquement l'écriture comptable pour la facture approuvée
        if let Some(ref accounting_service) = self.accounting_service {
            if let Err(e) = accounting_service
                .generate_journal_entry_for_expense(&updated, Some(approved_by_user_id))
                .await
            {
                log::warn!(
                    "Failed to generate journal entry for approved expense {}: {}",
                    updated.id,
                    e
                );
                // Ne pas échouer l'approbation si la création de l'écriture échoue
                // L'écriture peut être créée manuellement plus tard
            }
        }

        Ok(self.to_invoice_response_dto(&updated))
    }

    /// Rejeter une facture avec raison (PendingApproval → Rejected)
    pub async fn reject_invoice(
        &self,
        invoice_id: Uuid,
        dto: RejectInvoiceDto,
    ) -> Result<InvoiceResponseDto, String> {
        let mut invoice = self
            .repository
            .find_by_id(invoice_id)
            .await?
            .ok_or_else(|| "Invoice not found".to_string())?;

        let rejected_by_user_id = Uuid::parse_str(&dto.rejected_by_user_id)
            .map_err(|_| "Invalid rejected_by_user_id format".to_string())?;

        invoice.reject(rejected_by_user_id, dto.rejection_reason)?;

        let updated = self.repository.update(&invoice).await?;
        Ok(self.to_invoice_response_dto(&updated))
    }

    /// Récupérer toutes les factures en attente d'approbation (pour syndics)
    pub async fn get_pending_invoices(
        &self,
        organization_id: Uuid,
    ) -> Result<PendingInvoicesListDto, String> {
        let filters = ExpenseFilters {
            organization_id: Some(organization_id),
            approval_status: Some(ApprovalStatus::PendingApproval),
            ..Default::default()
        };

        // Utiliser une pagination large pour récupérer toutes les factures pending
        let page_request = PageRequest {
            page: 1,
            per_page: 1000, // Limite raisonnable
            sort_by: None,
            order: SortOrder::default(),
        };

        let (expenses, _total) = self
            .repository
            .find_all_paginated(&page_request, &filters)
            .await?;

        let invoices: Vec<InvoiceResponseDto> = expenses
            .iter()
            .map(|e| self.to_invoice_response_dto(e))
            .collect();

        Ok(PendingInvoicesListDto {
            count: invoices.len(),
            invoices,
        })
    }

    /// Récupérer une facture avec tous les détails (enrichi)
    pub async fn get_invoice(&self, id: Uuid) -> Result<Option<InvoiceResponseDto>, String> {
        let expense = self.repository.find_by_id(id).await?;
        Ok(expense.map(|e| self.to_invoice_response_dto(&e)))
    }

    // ========== Helper Methods ==========

    fn to_response_dto(&self, expense: &Expense) -> ExpenseResponseDto {
        ExpenseResponseDto {
            id: expense.id.to_string(),
            building_id: expense.building_id.to_string(),
            category: expense.category.clone(),
            description: expense.description.clone(),
            amount: expense.amount,
            expense_date: expense.expense_date.to_rfc3339(),
            payment_status: expense.payment_status.clone(),
            approval_status: expense.approval_status.clone(),
            supplier: expense.supplier.clone(),
            invoice_number: expense.invoice_number.clone(),
            account_code: expense.account_code.clone(),
        }
    }

    fn to_invoice_response_dto(&self, expense: &Expense) -> InvoiceResponseDto {
        InvoiceResponseDto {
            id: expense.id.to_string(),
            organization_id: expense.organization_id.to_string(),
            building_id: expense.building_id.to_string(),
            category: expense.category.clone(),
            description: expense.description.clone(),

            // Montants
            amount: expense.amount,
            amount_excl_vat: expense.amount_excl_vat,
            vat_rate: expense.vat_rate,
            vat_amount: expense.vat_amount,
            amount_incl_vat: expense.amount_incl_vat,

            // Dates
            expense_date: expense.expense_date.to_rfc3339(),
            invoice_date: expense.invoice_date.map(|d| d.to_rfc3339()),
            due_date: expense.due_date.map(|d| d.to_rfc3339()),
            paid_date: expense.paid_date.map(|d| d.to_rfc3339()),

            // Workflow
            approval_status: expense.approval_status.clone(),
            submitted_at: expense.submitted_at.map(|d| d.to_rfc3339()),
            approved_by: expense.approved_by.map(|u| u.to_string()),
            approved_at: expense.approved_at.map(|d| d.to_rfc3339()),
            rejection_reason: expense.rejection_reason.clone(),

            // Payment
            payment_status: expense.payment_status.clone(),
            supplier: expense.supplier.clone(),
            invoice_number: expense.invoice_number.clone(),

            created_at: expense.created_at.to_rfc3339(),
            updated_at: expense.updated_at.to_rfc3339(),
        }
    }
}
