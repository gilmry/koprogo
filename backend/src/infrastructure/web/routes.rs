use crate::infrastructure::web::handlers::*;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(health_check)
            // Authentication
            .service(login)
            .service(register)
            .service(get_current_user)
            // Buildings
            .service(create_building)
            .service(list_buildings)
            .service(get_building)
            .service(update_building)
            .service(delete_building)
            // Units
            .service(create_unit)
            .service(get_unit)
            .service(list_units_by_building)
            .service(assign_owner)
            // Owners
            .service(create_owner)
            .service(list_owners)
            .service(get_owner)
            // Expenses
            .service(create_expense)
            .service(get_expense)
            .service(list_expenses_by_building)
            .service(mark_expense_paid)
            // Seed (SuperAdmin only)
            .service(seed_demo_data)
            .service(clear_demo_data),
    );
}
