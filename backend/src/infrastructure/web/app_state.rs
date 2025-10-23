use crate::application::use_cases::{
    AuthUseCases, BuildingUseCases, ExpenseUseCases, MeetingUseCases, OwnerUseCases, UnitUseCases,
};
use crate::infrastructure::pool::DbPool;
use std::sync::Arc;

pub struct AppState {
    pub auth_use_cases: Arc<AuthUseCases>,
    pub building_use_cases: Arc<BuildingUseCases>,
    pub unit_use_cases: Arc<UnitUseCases>,
    pub owner_use_cases: Arc<OwnerUseCases>,
    pub expense_use_cases: Arc<ExpenseUseCases>,
    pub meeting_use_cases: Arc<MeetingUseCases>,
    pub pool: DbPool, // For seeding operations
}

impl AppState {
    pub fn new(
        auth_use_cases: AuthUseCases,
        building_use_cases: BuildingUseCases,
        unit_use_cases: UnitUseCases,
        owner_use_cases: OwnerUseCases,
        expense_use_cases: ExpenseUseCases,
        meeting_use_cases: MeetingUseCases,
        pool: DbPool,
    ) -> Self {
        Self {
            auth_use_cases: Arc::new(auth_use_cases),
            building_use_cases: Arc::new(building_use_cases),
            unit_use_cases: Arc::new(unit_use_cases),
            owner_use_cases: Arc::new(owner_use_cases),
            expense_use_cases: Arc::new(expense_use_cases),
            meeting_use_cases: Arc::new(meeting_use_cases),
            pool,
        }
    }
}
