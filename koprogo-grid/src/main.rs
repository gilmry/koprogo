use actix_web::{middleware, web, App, HttpServer};
use koprogo_grid::adapters::actix::{configure_routes, AppState};
use koprogo_grid::adapters::postgres::*;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment and logging
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting KoproGo Grid Computing Server");

    // Database connection
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    log::info!("Database connection established");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    log::info!("Database migrations completed");

    // Initialize repositories
    let node_repo = Arc::new(PostgresNodeRepository::new(pool.clone())) as Arc<dyn koprogo_grid::NodeRepository>;
    let task_repo = Arc::new(PostgresTaskRepository::new(pool.clone())) as Arc<dyn koprogo_grid::TaskRepository>;
    let proof_repo = Arc::new(PostgresGreenProofRepository::new(pool.clone())) as Arc<dyn koprogo_grid::GreenProofRepository>;
    let credit_repo = Arc::new(PostgresCarbonCreditRepository::new(pool.clone())) as Arc<dyn koprogo_grid::CarbonCreditRepository>;

    // Initialize task distributor
    let distributor = Arc::new(PostgresTaskDistributor::new(
        node_repo.clone(),
        task_repo.clone(),
    )) as Arc<dyn koprogo_grid::TaskDistributor>;

    // Application state
    let app_state = Arc::new(AppState {
        node_repo,
        task_repo,
        proof_repo,
        credit_repo,
        distributor,
    });

    // Server configuration
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number");

    log::info!("Starting server at http://{}:{}", host, port);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(configure_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
