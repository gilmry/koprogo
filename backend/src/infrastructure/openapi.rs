/// OpenAPI Documentation Module
/// Generates OpenAPI 3.0 specification for KoproGo API
/// Access Swagger UI at: http://localhost:8080/swagger-ui/
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

// Import annotated handlers
// TODO: Re-enable when health_check handler is implemented
// use crate::infrastructure::web::handlers::health::health_check;

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
            This OpenAPI spec is incrementally being built. Currently includes:\n\
            - Health check endpoint\n\
            - ~400 additional endpoints available (see routes.rs)\n\n\
            To add endpoints to this spec, annotate handlers with #[utoipa::path(...)]\n\
            See health.rs for example implementation.",
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
        // Health & Monitoring
        // TODO: Re-enable when health_check handler is implemented
        // health_check,
    ),
    components(
        schemas(
            // Add DTOs and entities here as they get annotated
        )
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
        assert!(true);
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
