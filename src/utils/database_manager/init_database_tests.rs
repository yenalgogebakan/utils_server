use super::init_database::{DbPools, check_database_name, init_db_connection_pools};

#[test]
fn check_database_name_test() {
    let db_name = "uut_24_6".to_string();
    match check_database_name(db_name) {
        Ok(_) => println!("✅ Database name is valid."),
        Err(e) => panic!("Reason: {}", e),
    }
}

#[tokio::test]
async fn init_db_connection_pools_test() {
    match init_db_connection_pools().await {
        Ok(pools) => {
            println!("✅ Database connection pools initialized successfully.");
            // Optionally, you can add more checks here to verify the pools
            let _db_pools: DbPools = pools;
        }
        Err(e) => panic!("Reason: {}", e),
    }
}
