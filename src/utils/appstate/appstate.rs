use crate::utils::database_manager::init_database;

use std::sync::Arc;

pub type SharedState = Arc<AppState>;

#[derive(Clone)]
pub struct AppState {
    pub db_pools: init_database::DbPools,
}
