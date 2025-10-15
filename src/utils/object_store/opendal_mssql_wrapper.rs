use crate::utils::errors::db_errors::DbError;
use crate::utils::{
    database_manager::init_database::init_db_connection_pool,
    errors::object_store_errors::{ErrCtx as ObjErrCtx, ObjectStoreError},
};

use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::NaiveDateTime;
use tiberius::Query;

pub type ObjectStoreConnectionPool = Pool<ConnectionManager>;
pub struct MssqlStore {
    pool: ObjectStoreConnectionPool,
}

#[derive(Debug, Clone)]
pub struct ObjectStoreRecord {
    pub bucket: String,
    pub object_id: String,
    pub metadata: Vec<u8>,
    pub objcontent: Vec<u8>,
    pub original_size: i64,
    pub compressed_size: i64,
    pub lmts: NaiveDateTime,
}
impl MssqlStore {
    pub async fn new_mssql() -> Result<Self, DbError> {
        let pool = match init_db_connection_pool("MsSqlStore").await {
            Ok(p) => {p},
            Err(e) => {
                eprintln!("❌ Failed to create MsSqlStore pool: {}", e);
                return Err(e);
            }
        }

    }

    pub async fn get(
        &self,
        bucket: &str,
        key: &str,
        year: &str,
    ) -> Result<ObjectStoreRecord, ObjectStoreError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(ObjectStoreError::from)
            .ctx("MsSqlStore : get : get conn from pool")?;

        let sql_sentence = format!(
            "SELECT OBJCONTENT, ORIGINALSIZE, COMPRESSEDSIZE
             FROM {}.dbo.{}
             WHERE BUCKET = @P1 AND OBJECTID > @P2",
            self.get_dbname(year),
            format!("OBJECTSTORE_{}", year)
        );

        let mut query = Query::new(sql_sentence);
        query.bind(bucket.to_string());
        query.bind(key.to_string());

        let stream = query
            .query(&mut *conn)
            .await
            .map_err(ObjectStoreError::from)
            .ctx("MsSqlStore : get : quert")?;
        let rows = stream
            .into_first_result()
            .await
            .map_err(ObjectStoreError::from)
            .ctx("MsSqlStore : get : stream")?;

        if rows.len() > 1 {
            return Err(ObjectStoreError::MultipleRecordsFound(
                bucket.to_string(),
                key.to_string(),
            ));
        }

        let row = &rows[0];
        let object_content = row
            .get::<&[u8], _>(0)
            .ok_or_else(|| ObjectStoreError::MissingField("missing object_content".to_string()))?;
        let original_size = row
            .get::<i64, _>(1)
            .ok_or_else(|| ObjectStoreError::MissingField("missing original_size".to_string()))?;
        let compressed_size = row
            .get::<i64, _>(2)
            .ok_or_else(|| ObjectStoreError::MissingField("missing compressed_size".to_string()))?;
        /*
                for row in rows {
                    // Try to extract all fields
                    let Some(object_content) = row.get::<&[u8], _>(0) else {
                        eprintln!("⚠️ Skipping row: missing object_content");
                        continue;
                    };
                    let Some(original_size) = row.get::<i64, _>(1) else {
                        eprintln!("⚠️ Skipping row: missing original_size");
                        continue;
                    };
                    let Some(compressed_size) = row.get::<i64, _>(2) else {
                        eprintln!("⚠️ Skipping row: missing original_size");
                        continue;
                    };
                }
        */
        Ok(ObjectStoreRecord {
            bucket: bucket.to_string(),
            object_id: key.to_string(),
            metadata: vec![],
            objcontent: object_content.to_vec(),
            original_size: original_size,
            compressed_size: compressed_size,
            lmts: chrono::Local::now().naive_local(),
        })
    }

    fn get_dbname(&self, year: &str) -> String {
        format!("uut_{}", year)
    }
}
