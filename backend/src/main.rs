use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{middleware, web, App, HttpServer};
use dotenvy::dotenv;
use env_logger::Env;
use koprogo_api::application::use_cases::*;
use koprogo_api::infrastructure::database::*;
use koprogo_api::infrastructure::storage::FileStorage;
use koprogo_api::infrastructure::web::{configure_routes, AppState};
use std::env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-key-change-in-production".to_string());
    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid number");

    let actix_workers = env::var("ACTIX_WORKERS")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<usize>()
        .unwrap_or(2);

    // Parse allowed CORS origins from environment
    let allowed_origins: Vec<String> = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    log::info!("CORS allowed origins: {:?}", allowed_origins);

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

    // Initialize file storage
    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
    let file_storage = FileStorage::new(&upload_dir).expect("Failed to initialize file storage");

    // Initialize repositories
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool.clone()));
    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));
    let meeting_repo = Arc::new(PostgresMeetingRepository::new(pool.clone()));
    let document_repo = Arc::new(PostgresDocumentRepository::new(pool.clone()));

    // Initialize use cases
    let auth_use_cases = AuthUseCases::new(user_repo, refresh_token_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo);
    let unit_use_cases = UnitUseCases::new(unit_repo);
    let owner_use_cases = OwnerUseCases::new(owner_repo);
    let expense_use_cases = ExpenseUseCases::new(expense_repo.clone());
    let meeting_use_cases = MeetingUseCases::new(meeting_repo);
    let document_use_cases = DocumentUseCases::new(document_repo, file_storage);
    let pcn_use_cases = PcnUseCases::new(expense_repo);

    let app_state = web::Data::new(AppState::new(
        auth_use_cases,
        building_use_cases,
        unit_use_cases,
        owner_use_cases,
        expense_use_cases,
        meeting_use_cases,
        document_use_cases,
        pcn_use_cases,
        pool.clone(),
    ));

    log::info!("Starting server at {}:{} with {} workers", server_host, server_port, actix_workers);

    // Configure rate limiter: 100 requests per minute per IP
    // Allows bursts up to 100 requests, then refills at 100/60000ms rate
    let governor_conf = GovernorConfigBuilder::default()
        .milliseconds_per_request(100 * 60 * 1000) // 100 requests per minute (60,000ms)
        .burst_size(100) // Allow initial burst of 100 requests
        .finish()
        .unwrap();

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
            .wrap(Governor::new(&governor_conf))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(configure_routes)
    })
    .bind((server_host.as_str(), server_port))?
    .workers(actix_workers)
    .run()
    .await
}
