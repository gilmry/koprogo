use crate::infrastructure::web::handlers::*;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(metrics_endpoint);

    cfg.service(
        web::scope("/api/v1")
            .service(health_check)
            // Authentication
            .service(login)
            .service(register)
            .service(refresh_token)
            .service(switch_role)
            .service(get_current_user)
            // Buildings
            .service(create_building)
            .service(list_buildings)
            .service(get_building)
            .service(update_building)
            .service(delete_building)
            // Units
            .service(create_unit)
            .service(list_units)
            .service(get_unit)
            .service(list_units_by_building)
            .service(assign_owner)
            // Owners
            .service(create_owner)
            .service(list_owners)
            .service(get_owner)
            .service(update_owner)
            // Unit-Owner Relationships
            .service(add_owner_to_unit)
            .service(remove_owner_from_unit)
            .service(update_unit_owner)
            .service(get_unit_owners)
            .service(get_owner_units)
            .service(get_unit_ownership_history)
            .service(get_owner_ownership_history)
            .service(transfer_ownership)
            .service(get_total_ownership_percentage)
            // Expenses
            .service(create_expense)
            .service(list_expenses)
            .service(get_expense)
            .service(list_expenses_by_building)
            .service(mark_expense_paid)
            // Meetings
            .service(create_meeting)
            .service(list_meetings)
            .service(get_meeting)
            .service(list_meetings_by_building)
            .service(update_meeting)
            .service(add_agenda_item)
            .service(complete_meeting)
            .service(cancel_meeting)
            .service(delete_meeting)
            // Documents
            .service(upload_document)
            .service(list_documents)
            .service(get_document)
            .service(download_document)
            .service(list_documents_by_building)
            .service(list_documents_by_meeting)
            .service(link_document_to_meeting)
            .service(link_document_to_expense)
            .service(delete_document)
            // PCN (Belgian Chart of Accounts)
            .service(generate_pcn_report)
            .service(export_pcn_pdf)
            .service(export_pcn_excel)
            // Seed (SuperAdmin only) - ONE seed only
            .service(seed_demo_data)
            // .service(seed_realistic_data) // Disabled: we only use ONE seed
            .service(clear_demo_data)
            // Stats (SuperAdmin only)
            .service(get_dashboard_stats)
            .service(get_seed_data_stats)
            // Stats (Syndic/Accountant)
            .service(get_syndic_stats)
            .service(get_syndic_urgent_tasks)
            // Organizations (SuperAdmin only)
            .service(list_organizations)
            .service(create_organization)
            .service(update_organization)
            .service(activate_organization)
            .service(suspend_organization)
            .service(delete_organization)
            // Users (SuperAdmin only)
            .service(list_users)
            .service(create_user)
            .service(update_user)
            .service(activate_user)
            .service(deactivate_user)
            .service(delete_user),
    );
}
