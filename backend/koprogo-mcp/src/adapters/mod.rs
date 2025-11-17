// Adapters module - Infrastructure implementations
pub mod postgres_repository;
pub mod edge_client;
pub mod actix_handlers;

pub use postgres_repository::*;
pub use edge_client::*;
pub use actix_handlers::*;
