use axum::routing::*;
use std::sync::Arc;

use libs::utils::appstate::appstate::AppState;
use libs::utils::appstate::appstate::SharedState;
use libs::utils::database_manager;
use libs::utils::rest_handlers::download_docs_handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("rest server");

    //let db_pools: database_manager::init_database::DbPools =
    //    database_manager::init_database::init_db_connection_pools().await?;
    let db_pools = match database_manager::init_database::init_db_connection_pools().await {
        Ok(pools) => {
            println!("✅ All DB pools initialized successfully.");
            pools
        }
        //Err(DbError::WrongDatabaseName { expected, found }) => {
        //    eprintln!("Wrong DB name! expected {expected}, found {found}");
        //}
        Err(err) => {
            eprintln!("❌ Database initialization failed: {err}");
            //for cause in anyhow::Chain::new(&err) {
            //    eprintln!("  caused by: {cause}");
            //}
            std::process::exit(1);
        }
    };

    let app_state = Arc::new(AppState { db_pools });
    let app = create_app(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3090").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

fn create_app(state: SharedState) -> Router {
    let api_v1 = Router::new().route(
        "/download_docs",
        get(download_docs_handler::download_docs_handler),
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
