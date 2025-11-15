use super::handlers::*;
use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/grid")
            .route("/register", web::post().to(register_node))
            .route("/heartbeat", web::post().to(heartbeat))
            .route("/task", web::get().to(get_task))
            .route("/task", web::post().to(create_task))
            .route("/report", web::post().to(report_task))
            .route("/stats", web::get().to(get_stats)),
    );
}
