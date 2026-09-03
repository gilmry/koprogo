//! Le pool de connexions à la base.
//!
//! Un second `create_pool` a longtemps vécu dans `src/db.rs`, plafonné à cinq
//! connexions et sans réglage de délai. Il n'était déclaré dans aucun `mod`,
//! donc jamais compilé ni appelé — mais il se lisait comme la configuration du
//! pool, et quiconque cherchait à régler les connexions tombait dessus en
//! premier. Supprimé (#720) : deux définitions du même nom, dont l'une inerte,
//! sont plus trompeuses qu'un fichier manquant.
//!
//! Les réglages sont pilotés par l'environnement, avec des valeurs par défaut
//! prudentes plutôt qu'optimales — un pool trop large sur un VPS partagé épuise
//! les connexions du serveur avant celles de l'application.

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::time::Duration;

pub type DbPool = Pool<Postgres>;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let max_connections = env::var("DB_POOL_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u32>()
        .unwrap_or(10);

    let min_connections = env::var("DB_POOL_MIN_CONNECTIONS")
        .unwrap_or_else(|_| "2".to_string())
        .parse::<u32>()
        .unwrap_or(2);

    let acquire_timeout = env::var("DB_POOL_ACQUIRE_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse::<u64>()
        .unwrap_or(30);

    log::info!(
        "Creating database pool: max_connections={}, min_connections={}, acquire_timeout={}s",
        max_connections,
        min_connections,
        acquire_timeout
    );

    PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(acquire_timeout))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .connect(database_url)
        .await
}
