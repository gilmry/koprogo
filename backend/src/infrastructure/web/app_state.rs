use crate::application::use_cases::{
    AccountUseCases, AuthUseCases, BoardDashboardUseCases, BoardDecisionUseCases,
    BoardMemberUseCases, BudgetUseCases, BuildingUseCases, ChargeDistributionUseCases,
    DocumentUseCases, EtatDateUseCases, ExpenseUseCases, FinancialReportUseCases, GdprUseCases,
    MeetingUseCases, OwnerUseCases, PaymentReminderUseCases, PcnUseCases, UnitOwnerUseCases,
    UnitUseCases,
};
use crate::infrastructure::audit_logger::AuditLogger;
use crate::infrastructure::email::EmailService;
use crate::infrastructure::pool::DbPool;
use std::sync::Arc;

pub struct AppState {
    pub account_use_cases: Arc<AccountUseCases>,
    pub auth_use_cases: Arc<AuthUseCases>,
    pub building_use_cases: Arc<BuildingUseCases>,
    pub budget_use_cases: Arc<BudgetUseCases>,
    pub unit_use_cases: Arc<UnitUseCases>,
    pub owner_use_cases: Arc<OwnerUseCases>,
    pub unit_owner_use_cases: Arc<UnitOwnerUseCases>,
    pub expense_use_cases: Arc<ExpenseUseCases>,
    pub charge_distribution_use_cases: Arc<ChargeDistributionUseCases>,
    pub meeting_use_cases: Arc<MeetingUseCases>,
    pub document_use_cases: Arc<DocumentUseCases>,
    pub etat_date_use_cases: Arc<EtatDateUseCases>,
    pub pcn_use_cases: Arc<PcnUseCases>,
    pub payment_reminder_use_cases: Arc<PaymentReminderUseCases>,
    pub gdpr_use_cases: Arc<GdprUseCases>,
    pub board_member_use_cases: Arc<BoardMemberUseCases>,
    pub board_decision_use_cases: Arc<BoardDecisionUseCases>,
    pub board_dashboard_use_cases: Arc<BoardDashboardUseCases>,
    pub financial_report_use_cases: Arc<FinancialReportUseCases>,
    pub audit_logger: Arc<AuditLogger>,
    pub email_service: Arc<EmailService>,
    pub pool: DbPool, // For seeding operations
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account_use_cases: AccountUseCases,
        auth_use_cases: AuthUseCases,
        building_use_cases: BuildingUseCases,
        budget_use_cases: BudgetUseCases,
        unit_use_cases: UnitUseCases,
        owner_use_cases: OwnerUseCases,
        unit_owner_use_cases: UnitOwnerUseCases,
        expense_use_cases: ExpenseUseCases,
        charge_distribution_use_cases: ChargeDistributionUseCases,
        meeting_use_cases: MeetingUseCases,
        document_use_cases: DocumentUseCases,
        etat_date_use_cases: EtatDateUseCases,
        pcn_use_cases: PcnUseCases,
        payment_reminder_use_cases: PaymentReminderUseCases,
        gdpr_use_cases: GdprUseCases,
        board_member_use_cases: BoardMemberUseCases,
        board_decision_use_cases: BoardDecisionUseCases,
        board_dashboard_use_cases: BoardDashboardUseCases,
        financial_report_use_cases: FinancialReportUseCases,
        audit_logger: AuditLogger,
        email_service: EmailService,
        pool: DbPool,
    ) -> Self {
        Self {
            account_use_cases: Arc::new(account_use_cases),
            auth_use_cases: Arc::new(auth_use_cases),
            building_use_cases: Arc::new(building_use_cases),
            budget_use_cases: Arc::new(budget_use_cases),
            unit_use_cases: Arc::new(unit_use_cases),
            owner_use_cases: Arc::new(owner_use_cases),
            unit_owner_use_cases: Arc::new(unit_owner_use_cases),
            expense_use_cases: Arc::new(expense_use_cases),
            charge_distribution_use_cases: Arc::new(charge_distribution_use_cases),
            meeting_use_cases: Arc::new(meeting_use_cases),
            document_use_cases: Arc::new(document_use_cases),
            etat_date_use_cases: Arc::new(etat_date_use_cases),
            pcn_use_cases: Arc::new(pcn_use_cases),
            payment_reminder_use_cases: Arc::new(payment_reminder_use_cases),
            gdpr_use_cases: Arc::new(gdpr_use_cases),
            board_member_use_cases: Arc::new(board_member_use_cases),
            board_decision_use_cases: Arc::new(board_decision_use_cases),
            board_dashboard_use_cases: Arc::new(board_dashboard_use_cases),
            financial_report_use_cases: Arc::new(financial_report_use_cases),
            audit_logger: Arc::new(audit_logger),
            email_service: Arc::new(email_service),
            pool,
        }
    }
}
