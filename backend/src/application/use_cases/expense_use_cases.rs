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
            contractor_report_id: expense.contractor_report_id.map(|id| id.to_string()),
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

            contractor_report_id: expense.contractor_report_id.map(|id| id.to_string()),

            created_at: expense.created_at.to_rfc3339(),
            updated_at: expense.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{ExpenseFilters, PageRequest};
    use crate::application::ports::ExpenseRepository;
    use crate::domain::entities::{ApprovalStatus, ExpenseCategory, PaymentStatus};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ========== Mock Repository ==========

    struct MockExpenseRepository {
        expenses: Mutex<HashMap<Uuid, Expense>>,
    }

    impl MockExpenseRepository {
        fn new() -> Self {
            Self {
                expenses: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ExpenseRepository for MockExpenseRepository {
        async fn create(&self, expense: &Expense) -> Result<Expense, String> {
            let mut expenses = self.expenses.lock().unwrap();
            expenses.insert(expense.id, expense.clone());
            Ok(expense.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Expense>, String> {
            let expenses = self.expenses.lock().unwrap();
            Ok(expenses.get(&id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String> {
            let expenses = self.expenses.lock().unwrap();
            Ok(expenses
                .values()
                .filter(|e| e.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            filters: &ExpenseFilters,
        ) -> Result<(Vec<Expense>, i64), String> {
            let expenses = self.expenses.lock().unwrap();
            let filtered: Vec<Expense> = expenses
                .values()
                .filter(|e| {
                    if let Some(org_id) = filters.organization_id {
                        if e.organization_id != org_id {
                            return false;
                        }
                    }
                    if let Some(ref status) = filters.approval_status {
                        if e.approval_status != *status {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect();
            let count = filtered.len() as i64;
            Ok((filtered, count))
        }

        async fn update(&self, expense: &Expense) -> Result<Expense, String> {
            let mut expenses = self.expenses.lock().unwrap();
            expenses.insert(expense.id, expense.clone());
            Ok(expense.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut expenses = self.expenses.lock().unwrap();
            Ok(expenses.remove(&id).is_some())
        }
    }

    // ========== Helpers ==========

    fn make_use_cases(repo: MockExpenseRepository) -> ExpenseUseCases {
        ExpenseUseCases::new(Arc::new(repo))
    }

    fn valid_create_dto(org_id: Uuid, building_id: Uuid) -> CreateExpenseDto {
        CreateExpenseDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            category: ExpenseCategory::Maintenance,
            description: "Elevator maintenance Q1".to_string(),
            amount: 1500.0,
            expense_date: "2026-01-15T10:00:00Z".to_string(),
            supplier: Some("Schindler SA".to_string()),
            invoice_number: Some("INV-2026-001".to_string()),
            account_code: Some("611002".to_string()),
        }
    }

    fn valid_invoice_draft_dto(org_id: Uuid, building_id: Uuid) -> CreateInvoiceDraftDto {
        CreateInvoiceDraftDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            category: ExpenseCategory::Utilities,
            description: "Electricity bill January".to_string(),
            amount_excl_vat: 1000.0,
            vat_rate: 21.0,
            invoice_date: "2026-01-31T10:00:00Z".to_string(),
            due_date: Some("2026-02-28T10:00:00Z".to_string()),
            supplier: Some("Engie Electrabel".to_string()),
            invoice_number: Some("ELEC-2026-001".to_string()),
        }
    }

    // ========== Tests ==========

    #[tokio::test]
    async fn test_create_expense_success() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = uc
            .create_expense(valid_create_dto(org_id, building_id))
            .await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.building_id, building_id.to_string());
        assert_eq!(dto.description, "Elevator maintenance Q1");
        assert_eq!(dto.amount, 1500.0);
        assert_eq!(dto.payment_status, PaymentStatus::Pending);
        assert_eq!(dto.approval_status, ApprovalStatus::Draft);
        assert_eq!(dto.supplier, Some("Schindler SA".to_string()));
        assert_eq!(dto.account_code, Some("611002".to_string()));
    }

    #[tokio::test]
    async fn test_create_expense_invalid_building_id() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);

        let mut dto = valid_create_dto(Uuid::new_v4(), Uuid::new_v4());
        dto.building_id = "not-a-uuid".to_string();

        let result = uc.create_expense(dto).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid building ID format");
    }

    #[tokio::test]
    async fn test_submit_for_approval_success() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Create an expense (starts as Draft)
        let created = uc
            .create_expense(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let expense_id = Uuid::parse_str(&created.id).unwrap();

        // Submit for approval
        let result = uc
            .submit_for_approval(expense_id, SubmitForApprovalDto {})
            .await;

        assert!(result.is_ok());
        let invoice = result.unwrap();
        assert_eq!(invoice.approval_status, ApprovalStatus::PendingApproval);
        assert!(invoice.submitted_at.is_some());
    }

    #[tokio::test]
    async fn test_approve_invoice_success() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();

        // Create and submit
        let created = uc
            .create_expense(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let expense_id = Uuid::parse_str(&created.id).unwrap();
        uc.submit_for_approval(expense_id, SubmitForApprovalDto {})
            .await
            .unwrap();

        // Approve
        let result = uc
            .approve_invoice(
                expense_id,
                ApproveInvoiceDto {
                    approved_by_user_id: approver_id.to_string(),
                },
            )
            .await;

        assert!(result.is_ok());
        let invoice = result.unwrap();
        assert_eq!(invoice.approval_status, ApprovalStatus::Approved);
        assert_eq!(invoice.approved_by, Some(approver_id.to_string()));
        assert!(invoice.approved_at.is_some());
    }

    #[tokio::test]
    async fn test_reject_invoice_success() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let rejector_id = Uuid::new_v4();

        // Create and submit
        let created = uc
            .create_expense(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let expense_id = Uuid::parse_str(&created.id).unwrap();
        uc.submit_for_approval(expense_id, SubmitForApprovalDto {})
            .await
            .unwrap();

        // Reject
        let result = uc
            .reject_invoice(
                expense_id,
                RejectInvoiceDto {
                    rejected_by_user_id: rejector_id.to_string(),
                    rejection_reason: "Missing supporting documents".to_string(),
                },
            )
            .await;

        assert!(result.is_ok());
        let invoice = result.unwrap();
        assert_eq!(invoice.approval_status, ApprovalStatus::Rejected);
        assert_eq!(
            invoice.rejection_reason,
            Some("Missing supporting documents".to_string())
        );
    }

    #[tokio::test]
    async fn test_mark_as_paid_requires_approval() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Create an expense (Draft status, not approved)
        let created = uc
            .create_expense(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let expense_id = Uuid::parse_str(&created.id).unwrap();

        // Attempt to mark as paid without approval should fail
        let result = uc.mark_as_paid(expense_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("invoice must be approved first"));
    }

    #[tokio::test]
    async fn test_mark_as_paid_after_approval() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();

        // Create, submit, and approve
        let created = uc
            .create_expense(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let expense_id = Uuid::parse_str(&created.id).unwrap();
        uc.submit_for_approval(expense_id, SubmitForApprovalDto {})
            .await
            .unwrap();
        uc.approve_invoice(
            expense_id,
            ApproveInvoiceDto {
                approved_by_user_id: approver_id.to_string(),
            },
        )
        .await
        .unwrap();

        // Now mark as paid
        let result = uc.mark_as_paid(expense_id).await;
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.payment_status, PaymentStatus::Paid);
    }

    #[tokio::test]
    async fn test_find_by_building() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_a = Uuid::new_v4();
        let building_b = Uuid::new_v4();

        // Create expenses for two different buildings
        let mut dto_a = valid_create_dto(org_id, building_a);
        dto_a.description = "Building A expense".to_string();
        uc.create_expense(dto_a).await.unwrap();

        let mut dto_b = valid_create_dto(org_id, building_b);
        dto_b.description = "Building B expense".to_string();
        uc.create_expense(dto_b).await.unwrap();

        // Another expense for building A
        let mut dto_a2 = valid_create_dto(org_id, building_a);
        dto_a2.description = "Building A expense 2".to_string();
        uc.create_expense(dto_a2).await.unwrap();

        // Query for building A
        let result = uc.list_expenses_by_building(building_a).await;
        assert!(result.is_ok());
        let expenses = result.unwrap();
        assert_eq!(expenses.len(), 2);
        assert!(expenses
            .iter()
            .all(|e| e.building_id == building_a.to_string()));
    }

    #[tokio::test]
    async fn test_update_invoice_draft_blocked_after_approval() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let approver_id = Uuid::new_v4();

        // Create invoice draft, submit, and approve
        let created = uc
            .create_invoice_draft(valid_invoice_draft_dto(org_id, building_id))
            .await
            .unwrap();
        let invoice_id = Uuid::parse_str(&created.id).unwrap();
        uc.submit_for_approval(invoice_id, SubmitForApprovalDto {})
            .await
            .unwrap();
        uc.approve_invoice(
            invoice_id,
            ApproveInvoiceDto {
                approved_by_user_id: approver_id.to_string(),
            },
        )
        .await
        .unwrap();

        // Attempt to modify the approved invoice
        let update_dto = UpdateInvoiceDraftDto {
            description: Some("Changed description".to_string()),
            category: None,
            amount_excl_vat: None,
            vat_rate: None,
            invoice_date: None,
            due_date: None,
            supplier: None,
            invoice_number: None,
        };

        let result = uc.update_invoice_draft(invoice_id, update_dto).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be modified"));
    }

    #[tokio::test]
    async fn test_create_invoice_draft_vat_calculations() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        // Create invoice with 21% VAT on 1000 EUR HT
        let result = uc
            .create_invoice_draft(valid_invoice_draft_dto(org_id, building_id))
            .await;

        assert!(result.is_ok());
        let invoice = result.unwrap();

        // 1000 * 21% = 210 VAT, total = 1210
        assert_eq!(invoice.amount_excl_vat, Some(1000.0));
        assert_eq!(invoice.vat_rate, Some(21.0));
        assert_eq!(invoice.vat_amount, Some(210.0));
        assert_eq!(invoice.amount_incl_vat, Some(1210.0));
        // backward compat: amount field = TTC
        assert_eq!(invoice.amount, 1210.0);
    }

    #[tokio::test]
    async fn test_reject_then_resubmit() {
        let repo = MockExpenseRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let rejector_id = Uuid::new_v4();

        // Create, submit, reject
        let created = uc
            .create_expense(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let expense_id = Uuid::parse_str(&created.id).unwrap();
        uc.submit_for_approval(expense_id, SubmitForApprovalDto {})
            .await
            .unwrap();
        uc.reject_invoice(
            expense_id,
            RejectInvoiceDto {
                rejected_by_user_id: rejector_id.to_string(),
                rejection_reason: "Incorrect amount".to_string(),
            },
        )
        .await
        .unwrap();

        // Verify rejected state
        let rejected = uc.get_invoice(expense_id).await.unwrap().unwrap();
        assert_eq!(rejected.approval_status, ApprovalStatus::Rejected);
        assert_eq!(
            rejected.rejection_reason,
            Some("Incorrect amount".to_string())
        );

        // Re-submit after rejection (allowed)
        let result = uc
            .submit_for_approval(expense_id, SubmitForApprovalDto {})
            .await;
        assert!(result.is_ok());
        let resubmitted = result.unwrap();
        assert_eq!(resubmitted.approval_status, ApprovalStatus::PendingApproval);
        // rejection_reason should be cleared upon resubmission
        assert_eq!(resubmitted.rejection_reason, None);
    }
}
