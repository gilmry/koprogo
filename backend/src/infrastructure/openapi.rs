/// OpenAPI Documentation Module
/// Generates OpenAPI 3.0 specification for KoproGo API
/// Access Swagger UI at: http://localhost:8080/swagger-ui/
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

// Handler imports not needed — utoipa paths() uses full module paths

/// Main OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "KoproGo API",
        version = "1.0.0",
        description = "Belgian Property Management SaaS Platform\n\n\
            # Features\n\
            - 🏢 Building & Unit Management\n\
            - 👥 Multi-owner & Multi-role Support\n\
            - 💰 Financial Management (Belgian PCMN)\n\
            - 🗳️ Meeting & Voting System\n\
            - 📄 Document Management\n\
            - 📊 Budget & État Daté Generation\n\
            - 🔔 Notifications & Payment Recovery\n\
            - 🤝 Community Features (SEL, Notices, Skills)\n\
            - 🎮 Gamification & Achievements\n\
            - 🔐 GDPR Compliant\n\n\
            # Authentication\n\
            All endpoints (except /health and /public/*) require JWT Bearer token.\n\
            Get token via POST /api/v1/auth/login\n\n\
            # Complete API Documentation\n\
            90 of 511 endpoints annotated with utoipa (Swagger UI live spec).\n\
            Full 495-endpoint OpenAPI 3.0.3 spec available at docs/api/openapi.yaml.\n\n\
            Progressive annotation ongoing — see handlers for pattern.",
        contact(
            name = "KoproGo Support",
            email = "support@koprogo.com"
        ),
        license(
            name = "AGPL-3.0-or-later",
            url = "https://www.gnu.org/licenses/agpl-3.0.en.html"
        ),
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development"),
        (url = "https://api.koprogo.com", description = "Production"),
    ),
    paths(
        // Health
        crate::infrastructure::web::handlers::health::health_check,
        // Auth
        crate::infrastructure::web::handlers::auth_handlers::login,
        crate::infrastructure::web::handlers::auth_handlers::register,
        crate::infrastructure::web::handlers::auth_handlers::refresh_token,
        crate::infrastructure::web::handlers::auth_handlers::switch_role,
        crate::infrastructure::web::handlers::auth_handlers::get_current_user,
        // Buildings
        crate::infrastructure::web::handlers::building_handlers::create_building,
        crate::infrastructure::web::handlers::building_handlers::list_buildings,
        crate::infrastructure::web::handlers::building_handlers::get_building,
        crate::infrastructure::web::handlers::building_handlers::update_building,
        crate::infrastructure::web::handlers::building_handlers::delete_building,
        crate::infrastructure::web::handlers::building_handlers::export_annual_report_pdf,
        // Payments
        crate::infrastructure::web::handlers::payment_handlers::create_payment,
        crate::infrastructure::web::handlers::payment_handlers::get_payment,
        crate::infrastructure::web::handlers::payment_handlers::get_payment_by_stripe_intent,
        crate::infrastructure::web::handlers::payment_handlers::list_owner_payments,
        crate::infrastructure::web::handlers::payment_handlers::list_building_payments,
        crate::infrastructure::web::handlers::payment_handlers::list_expense_payments,
        crate::infrastructure::web::handlers::payment_handlers::list_organization_payments,
        crate::infrastructure::web::handlers::payment_handlers::list_payments_by_status,
        crate::infrastructure::web::handlers::payment_handlers::list_pending_payments,
        crate::infrastructure::web::handlers::payment_handlers::list_failed_payments,
        crate::infrastructure::web::handlers::payment_handlers::mark_payment_processing,
        crate::infrastructure::web::handlers::payment_handlers::mark_payment_requires_action,
        crate::infrastructure::web::handlers::payment_handlers::mark_payment_succeeded,
        crate::infrastructure::web::handlers::payment_handlers::mark_payment_failed,
        crate::infrastructure::web::handlers::payment_handlers::mark_payment_cancelled,
        crate::infrastructure::web::handlers::payment_handlers::refund_payment,
        crate::infrastructure::web::handlers::payment_handlers::delete_payment,
        crate::infrastructure::web::handlers::payment_handlers::get_owner_payment_stats,
        crate::infrastructure::web::handlers::payment_handlers::get_building_payment_stats,
        crate::infrastructure::web::handlers::payment_handlers::get_expense_total_paid,
        crate::infrastructure::web::handlers::payment_handlers::get_owner_total_paid,
        crate::infrastructure::web::handlers::payment_handlers::get_building_total_paid,
        // Tickets
        crate::infrastructure::web::handlers::ticket_handlers::create_ticket,
        crate::infrastructure::web::handlers::ticket_handlers::get_ticket,
        crate::infrastructure::web::handlers::ticket_handlers::delete_ticket,
        crate::infrastructure::web::handlers::ticket_handlers::list_my_tickets,
        crate::infrastructure::web::handlers::ticket_handlers::list_assigned_tickets,
        crate::infrastructure::web::handlers::ticket_handlers::list_building_tickets,
        crate::infrastructure::web::handlers::ticket_handlers::list_organization_tickets,
        crate::infrastructure::web::handlers::ticket_handlers::list_tickets_by_status,
        crate::infrastructure::web::handlers::ticket_handlers::assign_ticket,
        crate::infrastructure::web::handlers::ticket_handlers::start_work,
        crate::infrastructure::web::handlers::ticket_handlers::resolve_ticket,
        crate::infrastructure::web::handlers::ticket_handlers::close_ticket,
        crate::infrastructure::web::handlers::ticket_handlers::cancel_ticket,
        crate::infrastructure::web::handlers::ticket_handlers::reopen_ticket,
        crate::infrastructure::web::handlers::ticket_handlers::get_ticket_statistics,
        crate::infrastructure::web::handlers::ticket_handlers::get_ticket_statistics_org,
        crate::infrastructure::web::handlers::ticket_handlers::get_overdue_tickets,
        crate::infrastructure::web::handlers::ticket_handlers::get_overdue_tickets_org,
        // Polls
        crate::infrastructure::web::handlers::poll_handlers::create_poll,
        crate::infrastructure::web::handlers::poll_handlers::get_poll,
        crate::infrastructure::web::handlers::poll_handlers::update_poll,
        crate::infrastructure::web::handlers::poll_handlers::list_polls,
        crate::infrastructure::web::handlers::poll_handlers::find_active_polls,
        crate::infrastructure::web::handlers::poll_handlers::publish_poll,
        crate::infrastructure::web::handlers::poll_handlers::close_poll,
        crate::infrastructure::web::handlers::poll_handlers::cancel_poll,
        crate::infrastructure::web::handlers::poll_handlers::delete_poll,
        crate::infrastructure::web::handlers::poll_handlers::cast_poll_vote,
        crate::infrastructure::web::handlers::poll_handlers::get_poll_results,
        crate::infrastructure::web::handlers::poll_handlers::get_poll_building_statistics,
        // Resolutions
        crate::infrastructure::web::handlers::resolution_handlers::create_resolution,
        crate::infrastructure::web::handlers::resolution_handlers::get_resolution,
        crate::infrastructure::web::handlers::resolution_handlers::list_meeting_resolutions,
        crate::infrastructure::web::handlers::resolution_handlers::delete_resolution,
        crate::infrastructure::web::handlers::resolution_handlers::cast_vote,
        crate::infrastructure::web::handlers::resolution_handlers::list_resolution_votes,
        crate::infrastructure::web::handlers::resolution_handlers::change_vote,
        crate::infrastructure::web::handlers::resolution_handlers::close_voting,
        crate::infrastructure::web::handlers::resolution_handlers::get_meeting_vote_summary,
        // Notifications
        crate::infrastructure::web::handlers::notification_handlers::create_notification,
        crate::infrastructure::web::handlers::notification_handlers::get_notification,
        crate::infrastructure::web::handlers::notification_handlers::list_my_notifications,
        crate::infrastructure::web::handlers::notification_handlers::list_unread_notifications,
        crate::infrastructure::web::handlers::notification_handlers::mark_notification_read,
        crate::infrastructure::web::handlers::notification_handlers::mark_all_notifications_read,
        crate::infrastructure::web::handlers::notification_handlers::delete_notification,
        crate::infrastructure::web::handlers::notification_handlers::get_notification_stats,
        crate::infrastructure::web::handlers::notification_handlers::get_user_preferences,
        crate::infrastructure::web::handlers::notification_handlers::get_preference,
        crate::infrastructure::web::handlers::notification_handlers::update_preference,
        // GDPR
        crate::infrastructure::web::handlers::gdpr_handlers::export_user_data,
        crate::infrastructure::web::handlers::gdpr_handlers::erase_user_data,
        crate::infrastructure::web::handlers::gdpr_handlers::can_erase_user,
        crate::infrastructure::web::handlers::gdpr_handlers::rectify_user_data,
        crate::infrastructure::web::handlers::gdpr_handlers::restrict_user_processing,
        crate::infrastructure::web::handlers::gdpr_handlers::set_marketing_preference,
        // Consent (GDPR)
        crate::infrastructure::web::handlers::consent_handlers::record_consent,
        crate::infrastructure::web::handlers::consent_handlers::get_consent_status,
        // Legal Reference
        crate::infrastructure::web::handlers::legal_handlers::list_legal_rules,
        crate::infrastructure::web::handlers::legal_handlers::get_legal_rule,
        crate::infrastructure::web::handlers::legal_handlers::get_ag_sequence,
        crate::infrastructure::web::handlers::legal_handlers::get_majority_for,
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "System health and monitoring"),
        (name = "Auth", description = "Authentication and authorization"),
        (name = "Buildings", description = "Building management"),
        (name = "Units", description = "Unit management"),
        (name = "Owners", description = "Owner management"),
        (name = "Expenses", description = "Expense and invoice management"),
        (name = "Meetings", description = "General assembly management"),
        (name = "Budgets", description = "Annual budget management"),
        (name = "Documents", description = "Document upload/download"),
        (name = "GDPR", description = "Data privacy compliance"),
        (name = "Payments", description = "Payment processing"),
        (name = "PaymentMethods", description = "Stored payment methods"),
        (name = "LocalExchanges", description = "SEL time-based exchange system"),
        (name = "Notifications", description = "Multi-channel notifications"),
        (name = "Tickets", description = "Maintenance request system"),
        (name = "Resolutions", description = "Meeting voting system"),
        (name = "BoardMembers", description = "Board of directors management"),
        (name = "Quotes", description = "Contractor quote management"),
        (name = "EtatsDates", description = "Property sale documentation"),
        (name = "PaymentRecovery", description = "Automated payment reminders"),
        (name = "Consent", description = "User consent management (GDPR Art. 7)"),
        (name = "Legal Reference", description = "Belgian legal reference rules and majority types"),
    )
)]
pub struct ApiDoc;

/// Add JWT Bearer authentication to OpenAPI spec
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some(
                            "JWT token obtained from /api/v1/auth/login.\n\n\
                            Example: `Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`\n\n\
                            To authenticate:\n\
                            1. Click 'Authorize' button above\n\
                            2. Enter token (with or without 'Bearer ' prefix)\n\
                            3. Click 'Authorize' in dialog\n\
                            4. Try endpoints",
                        ))
                        .build(),
                ),
            )
        }
    }
}

/// Configure Swagger UI service
///
/// Swagger UI will be available at: http://localhost:8080/swagger-ui/
pub fn configure_swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui/{_:.*}")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .config(
            utoipa_swagger_ui::Config::default()
                .try_it_out_enabled(true)
                .persist_authorization(true)
                .display_request_duration(true)
                .deep_linking(true)
                .display_operation_id(true)
                .default_models_expand_depth(1)
                .default_model_expand_depth(1), // .doc_expansion(utoipa_swagger_ui::DocExpansion::List) // Removed: DocExpansion no longer exists in utoipa_swagger_ui
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_generation() {
        let spec = ApiDoc::openapi();

        // Verify basic info
        assert_eq!(spec.info.title, "KoproGo API");
        assert_eq!(spec.info.version, "1.0.0");

        // Verify servers
        assert!(spec.servers.is_some());
        let servers = spec.servers.unwrap();
        assert_eq!(servers.len(), 2);

        // Verify security scheme
        assert!(spec.components.is_some());
        let components = spec.components.unwrap();
        assert!(components.security_schemes.contains_key("bearer_auth"));

        // Verify tags
        assert!(spec.tags.is_some());
        let tags = spec.tags.unwrap();
        assert!(tags.len() >= 15);
    }

    #[test]
    fn test_swagger_ui_configuration() {
        let _swagger = configure_swagger_ui();
        // SwaggerUi is configured, this test ensures it compiles
    }

    #[test]
    fn test_openapi_json_is_valid() {
        let spec = ApiDoc::openapi();

        // Serialize to JSON to ensure it's valid
        let json = serde_json::to_string(&spec).expect("Should serialize to JSON");
        assert!(json.contains("\"title\":\"KoproGo API\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
    }
}
