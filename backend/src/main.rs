use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use dotenvy::dotenv;
use env_logger::Env;
use koprogo_api::application::use_cases::*;
use koprogo_api::infrastructure::database::*;
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

    // Initialize repositories
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let building_repo = Arc::new(PostgresBuildingRepository::new(pool.clone()));
    let unit_repo = Arc::new(PostgresUnitRepository::new(pool.clone()));
    let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
    let expense_repo = Arc::new(PostgresExpenseRepository::new(pool.clone()));

    // Initialize use cases
    let auth_use_cases = AuthUseCases::new(user_repo, jwt_secret);
    let building_use_cases = BuildingUseCases::new(building_repo);
    let unit_use_cases = UnitUseCases::new(unit_repo);
    let owner_use_cases = OwnerUseCases::new(owner_repo);
    let expense_use_cases = ExpenseUseCases::new(expense_repo);

    let app_state = web::Data::new(AppState::new(
        auth_use_cases,
        building_use_cases,
        unit_use_cases,
        owner_use_cases,
        expense_use_cases,
        pool.clone(),
    ));

    log::info!("Starting server at {}:{}", server_host, server_port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .configure(configure_routes)
    })
    .bind((server_host.as_str(), server_port))?
    .run()
    .await
}
