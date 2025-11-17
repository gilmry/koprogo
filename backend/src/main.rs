use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware, web, App, HttpServer};
use dotenvy::dotenv;
use env_logger::Env;
use koprogo_api::application::use_cases::*;
use koprogo_api::infrastructure::audit_logger::AuditLogger;
use koprogo_api::infrastructure::database::*;
use koprogo_api::infrastructure::email::EmailService;
use koprogo_api::infrastructure::storage::{
    FileStorage, S3Storage, S3StorageConfig, StorageProvider,
};
use koprogo_api::infrastructure::web::{
    configure_routes, AppState, GdprRateLimit, GdprRateLimitConfig, LoginRateLimiter,
    SecurityHeaders,
};
use std::env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // JWT Secret with production validation
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| {
            log::warn!("JWT_SECRET not set, using default (INSECURE - only for development!)");
            "super-secret-key-change-in-production".to_string()
        });

    // Validate JWT secret strength in production
    validate_jwt_secret(&jwt_secret)?;
    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid number");

    let actix_workers = env::var("ACTIX_WORKERS")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<usize>()
        .unwrap_or(2);

    // Rate limiting configuration
    let enable_rate_limiting = env::var("ENABLE_RATE_LIMITING")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        .parse::<bool>()
        .unwrap_or(true);

    // Parse allowed CORS origins from environment
    let allowed_origins: Vec<String> = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| {
            log::warn!("CORS_ALLOWED_ORIGINS not set, defaulting to localhost (dev only!)");
            "http://localhost:3000".to_string()
        })
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Validate CORS origins (no wildcards in production)
    validate_cors_origins(&allowed_origins)?;

    log::info!("CORS allowed origins: {:?}", allowed_origins);
    log::info!("Rate limiting enabled: {}", enable_rate_limiting);

    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Seed superadmin account
    log::info!("Seeding superadmin account...");
    let seeder = DatabaseSeeder::new(pool.clone());
    match seeder.seed_superadmin().await {
        Ok(user) => log::info!("SuperAdmin account ready: {}", user.email),
        Err(e) => log::error!("Failed to seed superadmin: {}", e),
    }

    // Initialize storage provider (local filesystem by default)
    let file_storage = initialize_storage_provider()
        .await
        .map_err(std::io::Error::other)?;

    // Initialize repositories
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let user_role_repo = Arc::new(PostgresUserRoleRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let unit_owner_repo = Arc::new(PostgresUnitOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));
    let etat_date_repo = Arc::new(PostgresEtatDateRepository::new(pool.clone()));
    let budget_repo = Arc::new(PostgresBudgetRepository::new(pool.clone()));
    let board_member_repo = Arc::new(PostgresBoardMemberRepository::new(pool.clone()));
    let board_decision_repo = Arc::new(PostgresBoardDecisionRepository::new(pool.clone()));
    let gdpr_repo = Arc::new(PostgresGdprRepository::new(Arc::new(pool.clone())));
    let audit_log_repo = Arc::new(PostgresAuditLogRepository::new(pool.clone()));
    let charge_distribution_repo =
        Arc::new(PostgresChargeDistributionRepository::new(pool.clone()));
    let payment_reminder_repo = Arc::new(PostgresPaymentReminderRepository::new(pool.clone()));
    let account_repo = Arc::new(PostgresAccountRepository::new(pool.clone()));
    let resolution_repo = Arc::new(PostgresResolutionRepository::new(pool.clone()));
    let vote_repo = Arc::new(PostgresVoteRepository::new(pool.clone()));
    let ticket_repo = Arc::new(PostgresTicketRepository::new(pool.clone()));
    let notification_repo = Arc::new(PostgresNotificationRepository::new(pool.clone()));
    let notification_preference_repo =
        Arc::new(PostgresNotificationPreferenceRepository::new(pool.clone()));
    let payment_repo = Arc::new(PostgresPaymentRepository::new(pool.clone()));
    let payment_method_repo = Arc::new(PostgresPaymentMethodRepository::new(pool.clone()));
    let quote_repo = Arc::new(PostgresQuoteRepository::new(pool.clone()));
    let local_exchange_repo = Arc::new(PostgresLocalExchangeRepository::new(pool.clone()));
    let owner_credit_balance_repo =
        Arc::new(PostgresOwnerCreditBalanceRepository::new(pool.clone()));
    let notice_repo = Arc::new(PostgresNoticeRepository::new(pool.clone()));
    let skill_repo = Arc::new(PostgresSkillRepository::new(pool.clone()));
    let convocation_repo = Arc::new(PostgresConvocationRepository::new(pool.clone()));
    let convocation_recipient_repo =
        Arc::new(PostgresConvocationRecipientRepository::new(pool.clone()));

    // Initialize audit logger with database persistence
    let audit_logger = AuditLogger::new(Some(audit_log_repo.clone()));

    // Initialize use cases
    let auth_use_cases =
        AuthUseCases::new(user_repo, refresh_token_repo, user_role_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo.clone());
    let unit_use_cases = UnitUseCases::new(unit_repo.clone());
    let owner_use_cases = OwnerUseCases::new(owner_repo.clone());
    let unit_owner_use_cases = UnitOwnerUseCases::new(
        unit_owner_repo.clone(),
        unit_repo.clone(),
        owner_repo.clone(),
    );
    let expense_use_cases = ExpenseUseCases::new(expense_repo.clone());
    let charge_distribution_use_cases = ChargeDistributionUseCases::new(
        charge_distribution_repo,
        expense_repo.clone(),
        unit_owner_repo.clone(),
    );
    let meeting_use_cases = MeetingUseCases::new(meeting_repo.clone());
    let convocation_use_cases = ConvocationUseCases::new(
        convocation_repo,
        convocation_recipient_repo,
        owner_repo.clone(),
    );
    let resolution_use_cases = ResolutionUseCases::new(resolution_repo, vote_repo);
    let ticket_use_cases = TicketUseCases::new(ticket_repo);
    let notification_use_cases =
        NotificationUseCases::new(notification_repo, notification_preference_repo);
    let payment_use_cases =
        PaymentUseCases::new(payment_repo.clone(), payment_method_repo.clone());
    let payment_method_use_cases = PaymentMethodUseCases::new(payment_method_repo);
    let quote_use_cases = QuoteUseCases::new(quote_repo);
    let local_exchange_use_cases = LocalExchangeUseCases::new(
        local_exchange_repo,
        owner_credit_balance_repo,
        owner_repo.clone(),
    );
    let notice_use_cases = NoticeUseCases::new(notice_repo, owner_repo.clone());
    let skill_use_cases = SkillUseCases::new(skill_repo, owner_repo.clone());
    let document_use_cases = DocumentUseCases::new(document_repo, file_storage.clone());
    let etat_date_use_cases = EtatDateUseCases::new(
        etat_date_repo,
        unit_repo.clone(),
        building_repo.clone(),
        unit_owner_repo.clone(),
    );
    let budget_use_cases = BudgetUseCases::new(
        budget_repo,
        building_repo.clone(),
        expense_repo.clone(),
    );
    let pcn_use_cases = PcnUseCases::new(expense_repo.clone());
    let payment_reminder_use_cases =
        PaymentReminderUseCases::new(payment_reminder_repo, expense_repo.clone());
    let gdpr_use_cases = GdprUseCases::new(gdpr_repo, user_repo.clone());
    let board_member_use_cases =
        BoardMemberUseCases::new(board_member_repo.clone(), building_repo.clone());
    let board_decision_use_cases = BoardDecisionUseCases::new(
        board_decision_repo.clone(),
        building_repo.clone(),
        meeting_repo,
    );
    let board_dashboard_use_cases = BoardDashboardUseCases::new(
        board_member_repo,
        board_decision_repo,
        building_repo.clone(),
    );
    let account_use_cases = AccountUseCases::new(account_repo.clone());
    let financial_report_use_cases =
        FinancialReportUseCases::new(account_repo.clone(), expense_repo.clone());

    // Initialize email service
    let email_service = EmailService::from_env().expect("Failed to initialize email service");

    let app_state = web::Data::new(AppState::new(
        account_use_cases,
        auth_use_cases,
        building_use_cases,
        budget_use_cases,
        unit_use_cases,
        owner_use_cases,
        unit_owner_use_cases,
        expense_use_cases,
        charge_distribution_use_cases,
        meeting_use_cases,
        convocation_use_cases,
        resolution_use_cases,
        ticket_use_cases,
        notification_use_cases,
        payment_use_cases,
        payment_method_use_cases,
        quote_use_cases,
        local_exchange_use_cases,
        notice_use_cases,
        skill_use_cases,
        document_use_cases,
        etat_date_use_cases,
        pcn_use_cases,
        payment_reminder_use_cases,
        gdpr_use_cases,
        board_member_use_cases,
        board_decision_use_cases,
        board_dashboard_use_cases,
        financial_report_use_cases,
        audit_logger,
        email_service,
        pool.clone(),
    ));

    log::info!(
        "Starting server at {}:{} with {} workers",
        server_host,
        server_port,
        actix_workers
    );

    // Configure rate limiter: 100 requests per minute per IP
    // Allows bursts up to 100 requests, then refills at 100/60000ms rate
    // When rate limiting is disabled, set a very high limit (effectively unlimited)
    let rate_limit_ms = if enable_rate_limiting {
        100 * 60 * 1000 // 100 requests per minute (60,000ms)
    } else {
        1 // 1ms = 1000 requests per second (effectively unlimited)
    };
    let burst_size = if enable_rate_limiting {
        100
    } else {
        u32::MAX // Effectively unlimited burst
    };

    let governor_conf = GovernorConfigBuilder::default()
        .milliseconds_per_request(rate_limit_ms)
        .burst_size(burst_size)
        .finish()
        .unwrap();

    // GDPR-specific rate limiting (10 requests/hour per user for GDPR endpoints)
    let gdpr_rate_limit = GdprRateLimit::new(GdprRateLimitConfig::default());

    // Login rate limiting (5 attempts per 15 minutes per IP - anti-brute-force)
    let login_rate_limiter = LoginRateLimiter::default();

    HttpServer::new(move || {
        // Configure CORS with allowed origins from environment
        let mut cors = Cors::default();
        for origin in &allowed_origins {
            cors = cors.allowed_origin(origin);
        }
        let cors = cors
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::ACCEPT,
            ])
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .wrap(gdpr_rate_limit.clone())
            .wrap(login_rate_limiter.clone()) // Login brute-force protection (5/15min)
            .wrap(Governor::new(&governor_conf))
            .wrap(cors)
            .wrap(SecurityHeaders) // Security headers for all responses
            .wrap(middleware::Logger::default())
            .configure(configure_routes)
    })
    .bind((server_host.as_str(), server_port))?
    .workers(actix_workers)
    .run()
    .await
}

async fn initialize_storage_provider() -> Result<Arc<dyn StorageProvider>, String> {
    let provider = env::var("STORAGE_PROVIDER")
        .unwrap_or_else(|_| "local".to_string())
        .to_lowercase();

    match provider.as_str() {
        "s3" | "minio" => {
            let config = S3StorageConfig::from_env()?;
            let storage = S3Storage::from_config(config).await?;
            Ok(Arc::new(storage))
        }
        _ => {
            let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
            let storage = FileStorage::new(&upload_dir)?;
            Ok(Arc::new(storage))
        }
    }
}

/// Validate JWT secret strength for production security
///
/// Requirements:
/// - Minimum 32 characters
/// - Not the default insecure value
/// - Contains mix of characters (recommended)
fn validate_jwt_secret(secret: &str) -> std::io::Result<()> {
    // Check minimum length (256 bits = 32 bytes minimum)
    if secret.len() < 32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "JWT_SECRET must be at least 32 characters for security. Current length: {}. \
                 Generate a strong secret with: openssl rand -base64 32",
                secret.len()
            ),
        ));
    }

    // Check if using the dangerous default value
    if secret == "super-secret-key-change-in-production" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "JWT_SECRET is set to the default insecure value. \
             Set a strong secret in environment: export JWT_SECRET=$(openssl rand -base64 32)",
        ));
    }

    // Warn if secret appears weak (all same character, sequential, etc.)
    let unique_chars: std::collections::HashSet<char> = secret.chars().collect();
    if unique_chars.len() < 10 {
        log::warn!(
            "JWT_SECRET has low character diversity ({} unique chars). \
             Consider using a stronger random secret.",
            unique_chars.len()
        );
    }

    log::info!("✓ JWT_SECRET validation passed (length: {} chars)", secret.len());
    Ok(())
}

/// Validate CORS origins for production security
///
/// Requirements:
/// - No wildcard (*) origins allowed
/// - Valid URL format
/// - HTTPS required in production (optional check)
fn validate_cors_origins(origins: &[String]) -> std::io::Result<()> {
    if origins.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "CORS_ALLOWED_ORIGINS cannot be empty. Specify allowed origins explicitly.",
        ));
    }

    for origin in origins {
        // Reject wildcard origins (security risk)
        if origin == "*" || origin.contains("*") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Wildcard CORS origins not allowed for security: '{}'. \
                     Specify exact origins like: http://localhost:3000,https://app.example.com",
                    origin
                ),
            ));
        }

        // Validate URL format
        if !origin.starts_with("http://") && !origin.starts_with("https://") {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Invalid CORS origin format '{}'. Must start with http:// or https://",
                    origin
                ),
            ));
        }

        // Warn about HTTP in production (HTTPS recommended)
        if origin.starts_with("http://") && !origin.contains("localhost") && !origin.contains("127.0.0.1") {
            log::warn!(
                "CORS origin '{}' uses HTTP (not HTTPS). \
                 HTTPS is strongly recommended for production.",
                origin
            );
        }
    }

    log::info!("✓ CORS origins validation passed ({} origins)", origins.len());
    Ok(())
}
