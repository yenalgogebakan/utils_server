use crate::utils::database_manager::init_database;
use crate::utils::object_store::object_store::Store;
use crate::utils::rest_handlers::docs_from_objstore_spawn_handler;
use axum::routing::*;
use tokio::sync::Semaphore;

use std::sync::Arc;

pub type SharedState = Arc<AppState>;

#[derive(Clone)]
pub struct AppState {
    pub db_pools: init_database::DbPools,
    pub object_store: Store,
    pub blocking_limiter: Arc<Semaphore>, // NEW
}

pub fn create_app(state: SharedState) -> Router {
    let api_v1 = Router::new().route(
        "/docs_from_objstore",
        get(docs_from_objstore_spawn_handler::docs_from_objstore_spawn_handler),
    );
    //.route("/upload", post(upload_handler));
    // Main router
    Router::new()
        .route("/healthcheck", get(health_check))
        .nest("/api/v1", api_v1) // Version 1 of your API
        .fallback(fallback)
        .with_state(state)
}

/// axum handler for any request that fails to match the router routes.
/// This implementation responds with HTTP status code NOT FOUND (404).
pub async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    eprint!("fallback");
    (axum::http::StatusCode::NOT_FOUND, uri.to_string())
}

pub async fn health_check() -> Result<String, axum::http::StatusCode> {
    Ok("Health : Ok".into())
}
