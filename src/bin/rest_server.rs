use libs::utils::appstate::appstate::{AppState, create_app};
use libs::utils::database_manager;
use libs::utils::object_store::object_store::Store;
use libs::utils::object_store::opendal_mssql_wrapper::MssqlStore;

use std::sync::Arc;
use tokio::sync::Semaphore;

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
    let object_store = Store::Mssql(
        MssqlStore::new_mssql()
            .await
            .expect("Failed to init MSSQL store"),
    );

    // Limit of concurrent heavy blocking tasks.
    const MAX_BLOCKING_TASKS: usize = 64;

    let app_state = Arc::new(AppState {
        db_pools,
        object_store,
        blocking_limiter: Arc::new(Semaphore::new(MAX_BLOCKING_TASKS)), // NEW
    });

    let app = create_app(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3090").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
