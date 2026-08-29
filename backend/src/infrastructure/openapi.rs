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
        // ACP (Association des Copropriétaires)
        //
        // Les 5 handlers portaient DÉJÀ `#[utoipa::path]` et leurs DTO
        // `#[derive(ToSchema)]`, mais rien n'était déclaré ici — utoipa ne
        // collecte QUE ce qui est listé. Les annotations étaient donc mortes :
        // aucun path `/acps` dans `docs/api/openapi.json`, aucun type dans
        // `api.d.ts`, et un `CreateAcpDto` recopié à la main côté frontend qui
        // avait déjà divergé (il omettait `total_tantiemes`). Même défaut que
        // les 16 endpoints `payment-methods` (#732).
        crate::infrastructure::web::handlers::acp_handlers::create_acp,
        crate::infrastructure::web::handlers::acp_handlers::list_acps,
        crate::infrastructure::web::handlers::acp_handlers::get_acp,
        crate::infrastructure::web::handlers::acp_handlers::update_acp,
        crate::infrastructure::web::handlers::acp_handlers::archive_acp,
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
        crate::infrastructure::web::handlers::ticket_handlers::update_ticket_fields,
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
        // ContractorEvaluation (Story 3.9 — FR34 FR35 INV-21 INV-24)
        crate::infrastructure::web::handlers::contractor_evaluation_handlers::create_contractor_evaluation,
        crate::infrastructure::web::handlers::contractor_evaluation_handlers::get_contractor_evaluation,
        crate::infrastructure::web::handlers::contractor_evaluation_handlers::list_contractor_evaluations,
        // MagicLink (Story 3.2 — FR6 INV-13 INV-17)
        crate::infrastructure::web::handlers::magic_link_handlers::issue_magic_link,
        crate::infrastructure::web::handlers::magic_link_handlers::consume_magic_link,
        // Mandate (Story 3.4 — FR7 INV-14)
        crate::infrastructure::web::handlers::mandate_handlers::issue_mandate,
        crate::infrastructure::web::handlers::mandate_handlers::list_mandates,
        crate::infrastructure::web::handlers::mandate_handlers::get_mandate,
        crate::infrastructure::web::handlers::mandate_handlers::revoke_mandate,
        // Users — org-scoped listing (syndic org-users-endpoint)
        crate::infrastructure::web::handlers::user_handlers::list_organization_users,
        // RoleAssignment — CRUD REST (Story B0bis — gap Story 3.1)
        crate::infrastructure::web::handlers::role_assignment_handlers::assign_role,
        crate::infrastructure::web::handlers::role_assignment_handlers::list_role_assignments_for_user,
        crate::infrastructure::web::handlers::role_assignment_handlers::revoke_role_assignment,
        crate::infrastructure::web::handlers::role_assignment_handlers::list_role_assignments_admin,
        // RoleDelegation (Story 3.5 — FR8 INV-8)
        crate::infrastructure::web::handlers::role_delegation_handlers::create_role_delegation,
        crate::infrastructure::web::handlers::role_delegation_handlers::revoke_role_delegation,
        crate::infrastructure::web::handlers::role_delegation_handlers::list_role_delegations,
        // SyndicResponse (Story 3.7 — FR32 INV-23)
        crate::infrastructure::web::handlers::syndic_response_handlers::create_syndic_response,
        crate::infrastructure::web::handlers::syndic_response_handlers::list_syndic_responses,
        // TechnicalSpec (Story 3.8 — FR33)
        crate::infrastructure::web::handlers::technical_spec_handlers::create_technical_spec,
        crate::infrastructure::web::handlers::technical_spec_handlers::bump_technical_spec,
        crate::infrastructure::web::handlers::technical_spec_handlers::submit_technical_spec,
        crate::infrastructure::web::handlers::technical_spec_handlers::sign_technical_spec,
        crate::infrastructure::web::handlers::technical_spec_handlers::get_technical_spec,
        crate::infrastructure::web::handlers::technical_spec_handlers::list_technical_specs,
    ),
    components(schemas(
        // ACP — voir la note dans `paths()` ci-dessus.
        crate::application::dto::acp_dto::CreateAcpDto,
        crate::application::dto::acp_dto::UpdateAcpDto,
        crate::application::dto::acp_dto::AcpResponseDto,
        crate::domain::entities::acp::AcpLegalStatus,
        // Pagination primitives — referenced by query params on list endpoints
        crate::application::dto::pagination::SortOrder,
        // STORY-P7-701/702: all enums used by frontend wrappers are exposed
        // here so `openapi-typescript` emits them into api.d.ts for re-export.
        crate::domain::entities::resolution::ResolutionType,
        crate::domain::entities::resolution::MajorityType,
        crate::domain::entities::ticket::TicketCategory,
        crate::domain::entities::ticket::TicketPriority,
        crate::domain::entities::ticket::TicketStatus,
        crate::domain::entities::poll::PollStatus,
        crate::domain::entities::poll::PollType,
        crate::domain::entities::meeting::MeetingType,
        crate::domain::entities::meeting::MeetingStatus,
        crate::domain::entities::expense::ExpenseCategory,
        crate::domain::entities::expense::PaymentStatus,
        crate::domain::entities::expense::ApprovalStatus,
        crate::domain::entities::resource_booking::ResourceType,
        crate::domain::entities::resource_booking::BookingStatus,
        crate::domain::entities::resource_booking::RecurringPattern,
        crate::domain::entities::shared_object::SharedObjectCategory,
        crate::domain::entities::shared_object::ObjectCondition,
        crate::domain::entities::budget::BudgetStatus,
        crate::domain::entities::convocation::ConvocationType,
        crate::domain::entities::convocation::ConvocationStatus,
        crate::domain::entities::convocation_recipient::AttendanceStatus,
        crate::domain::entities::energy_campaign::CampaignType,
        crate::domain::entities::energy_campaign::CampaignStatus,
        crate::domain::entities::energy_campaign::EnergyType,
        crate::domain::entities::energy_campaign::ContractType,
        crate::domain::entities::etat_date::EtatDateStatus,
        crate::domain::entities::etat_date::EtatDateLanguage,
        crate::domain::entities::achievement::AchievementCategory,
        crate::domain::entities::achievement::AchievementTier,
        crate::domain::entities::challenge::ChallengeStatus,
        crate::domain::entities::challenge::ChallengeType,
        crate::domain::entities::technical_inspection::InspectionType,
        crate::domain::entities::technical_inspection::InspectionStatus,
        crate::domain::entities::local_exchange::ExchangeType,
        crate::domain::entities::local_exchange::ExchangeStatus,
        crate::domain::entities::owner_credit_balance::CreditStatus,
        crate::domain::entities::owner_credit_balance::ParticipationLevel,
        crate::domain::entities::notice::NoticeType,
        crate::domain::entities::notice::NoticeCategory,
        crate::domain::entities::notice::NoticeStatus,
        crate::domain::entities::payment_reminder::ReminderLevel,
        crate::domain::entities::payment_reminder::ReminderStatus,
        crate::domain::entities::payment_reminder::DeliveryMethod,
        crate::domain::entities::quote::QuoteStatus,
        crate::domain::entities::skill::SkillCategory,
        crate::domain::entities::skill::ExpertiseLevel,
        crate::domain::entities::work_report::WorkType,
        crate::domain::entities::work_report::WarrantyType,
        crate::domain::entities::resolution::ResolutionStatus,
        crate::domain::entities::notification::NotificationStatus,
        crate::domain::entities::notification::NotificationPriority,
        crate::domain::entities::payment::TransactionStatus,
        crate::domain::entities::payment_method::PaymentMethodType,
    )),
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
        (name = "ContractorEvaluation", description = "Contractor performance evaluations (Story 3.9 — FR34 FR35 INV-21 INV-24)"),
        (name = "MagicLink", description = "Single-use scoped magic links for public access (Story 3.2 — FR6 INV-13 INV-17)"),
        (name = "Mandate", description = "Time-bounded mandates for mandataire delegation (Story 3.4 — FR7 INV-14)"),
        (name = "RoleAssignment", description = "CRUD REST on user_role_assignments (Story B0bis — gap fill Story 3.1)"),
        (name = "RoleDelegation", description = "Temporary role delegation between users (Story 3.5 — FR8 INV-8)"),
        (name = "SyndicResponse", description = "Append-only syndic replies to tickets (Story 3.7 — FR32 INV-23)"),
        (name = "TechnicalSpec", description = "Versionable technical specifications with multi-party signatures (Story 3.8 — FR33)"),
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
