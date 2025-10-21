//
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use tiberius::{AuthMethod, Config};

//use crate::utils::errors::app_errors::{AppError, ErrCtx as AppErrCtx};
use crate::utils::errors::db_errors::{DbError, ErrCtx as DbErrCtx};

pub type ConnectionPool = Pool<ConnectionManager>;

#[derive(Clone)]
pub struct DbPools {
    pub incoming_invoice_pool: ConnectionPool,
    //pub xslt_pool: ConnectionPool,
    //pub cert_pool: ConnectionPool,
}

pub async fn init_db_connection_pool(name: &str) -> Result<ConnectionPool, DbError> {
    let mut config = Config::new();
    config.host("192.168.3.28");
    config.port(1433);
    config.database("uut_24_6");
    config.authentication(AuthMethod::sql_server("uut", "uut"));
    config.trust_cert(); // Only for development

    check_database_name("uut_24_6".to_string())?;
    let manager = ConnectionManager::new(config);

    let pool = Pool::builder()
        .max_size(10) // Maximum number of connections in the pool
        .min_idle(Some(2)) // Minimum idle connections to maintain
        .max_lifetime(Some(std::time::Duration::from_secs(3600))) // 1 hour max lifetime
        .idle_timeout(Some(std::time::Duration::from_secs(600))) // 10 minutes idle timeout
        .connection_timeout(std::time::Duration::from_secs(5)) // 30 seconds connection timeout
        .build(manager)
        .await
        .ctx("init_db_connection_pool:build")?;

    println!("✅ {name} created successfully.");
    Ok(pool)
}

pub async fn init_db_connection_pools() -> Result<DbPools, DbError> {
    let incoming_invoice_pool = init_db_connection_pool("incoming_invoice_pool").await?;

    /*
    let xslt_pool = init_db_connection_pool("xslt_pool")
        .await
        .map_err(AppError::from)
        .ctx("init_db_connection_pools/xslt")?;

    let cert_pool = init_db_connection_pool("cert_pool")
        .await
        .map_err(AppError::from)
        .ctx("init_db_connection_pools/cert")?;
        */
    Ok(DbPools {
        incoming_invoice_pool,
        //xslt_pool,
        //cert_pool,
    })
}

pub fn check_database_name(db_name: String) -> Result<(), DbError> {
    let expected = "uut_24_6";

    if db_name != expected {
        return Err(DbError::WrongDatabaseName {
            expected,
            found: db_name.to_string(),
        })
        .ctx(format!("{}/{}", module_path!(), "check_database_name").leak());
    }

    Ok(())
}
