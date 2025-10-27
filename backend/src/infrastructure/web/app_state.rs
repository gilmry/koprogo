use crate::application::use_cases::{
    AuthUseCases, BuildingUseCases, DocumentUseCases, ExpenseUseCases, MeetingUseCases,
    OwnerUseCases, PcnUseCases, UnitOwnerUseCases, UnitUseCases,
};
use crate::infrastructure::pool::DbPool;
use std::sync::Arc;

pub struct AppState {
    pub auth_use_cases: Arc<AuthUseCases>,
    pub building_use_cases: Arc<BuildingUseCases>,
    pub unit_use_cases: Arc<UnitUseCases>,
    pub owner_use_cases: Arc<OwnerUseCases>,
    pub unit_owner_use_cases: Arc<UnitOwnerUseCases>,
    pub expense_use_cases: Arc<ExpenseUseCases>,
    pub meeting_use_cases: Arc<MeetingUseCases>,
    pub document_use_cases: Arc<DocumentUseCases>,
    pub pcn_use_cases: Arc<PcnUseCases>,
    pub pool: DbPool, // For seeding operations
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auth_use_cases: AuthUseCases,
        building_use_cases: BuildingUseCases,
        unit_use_cases: UnitUseCases,
        owner_use_cases: OwnerUseCases,
        unit_owner_use_cases: UnitOwnerUseCases,
        expense_use_cases: ExpenseUseCases,
        meeting_use_cases: MeetingUseCases,
        document_use_cases: DocumentUseCases,
        pcn_use_cases: PcnUseCases,
        pool: DbPool,
    ) -> Self {
        Self {
            auth_use_cases: Arc::new(auth_use_cases),
            building_use_cases: Arc::new(building_use_cases),
            unit_use_cases: Arc::new(unit_use_cases),
            owner_use_cases: Arc::new(owner_use_cases),
            unit_owner_use_cases: Arc::new(unit_owner_use_cases),
            expense_use_cases: Arc::new(expense_use_cases),
            meeting_use_cases: Arc::new(meeting_use_cases),
            document_use_cases: Arc::new(document_use_cases),
            pcn_use_cases: Arc::new(pcn_use_cases),
            pool,
        }
    }
}
