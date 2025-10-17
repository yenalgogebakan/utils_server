use crate::utils::errors::db_errors::DbError;
use crate::utils::{
    database_manager::init_database::init_db_connection_pool,
    errors::object_store_errors::{ErrCtx as ObjErrCtx, ObjectStoreError},
};

use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use chrono::NaiveDateTime;
use tiberius::Query;

#[derive(Debug, Clone)]
pub struct ObjectStoreRecord {
    pub bucket: String,
    pub object_id: String,
    pub metadata: Vec<u8>,
    pub objcontent: Vec<u8>,
    pub original_size: i32,
    pub compressed_size: i32,
    pub lmts: NaiveDateTime,
}

pub type ObjectStoreConnectionPool = Pool<ConnectionManager>;
#[derive(Clone, Debug)]
pub struct MssqlStore {
    object_store_conn_pool: ObjectStoreConnectionPool,
}
impl MssqlStore {
    pub async fn new_mssql() -> Result<Self, DbError> {
        let object_store_conn_pool = init_db_connection_pool("MsSqlStore").await?;
        Ok(Self {
            object_store_conn_pool,
        })
    }

    pub async fn get(
        &self,
        bucket: &str,
        key: &str,
        year: &str,
    ) -> Result<ObjectStoreRecord, ObjectStoreError> {
        let mut conn = self
            .object_store_conn_pool
            .get()
            .await
            .map_err(ObjectStoreError::from)
            .ctx("MsSqlStore : get : get conn from pool")?;

        let sql_sentence = format!(
            "SELECT OBJCONTENT, ORIGINALSIZE, COMPRESSEDSIZE
             FROM {}.dbo.{}
             WHERE BUCKET = @P1 AND OBJECTID = @P2",
            self.get_dbname(year),
            format!("OBJECTSTORE_{}", year)
        );
        println!("Sql_sentence: {}", sql_sentence);

        let mut query = Query::new(sql_sentence);
        query.bind(bucket.to_string());
        query.bind(key.to_string());

        let stream = query
            .query(&mut *conn)
            .await
            .map_err(ObjectStoreError::from)
            .ctx("MsSqlStore : get : query")?;
        let rows = stream
            .into_first_result()
            .await
            .map_err(ObjectStoreError::from)
            .ctx("MsSqlStore : get : stream")?;

        println!("number of rows: {}", rows.len());
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
            .get::<i32, _>(1)
            .ok_or_else(|| ObjectStoreError::MissingField("missing original_size".to_string()))?;
        let compressed_size = row
            .get::<i32, _>(2)
            .ok_or_else(|| ObjectStoreError::MissingField("missing compressed_size".to_string()))?;

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
    pub async fn object_exists(
        &self,
        bucket: &str,
        key: &str,
        year: &str,
    ) -> Result<bool, ObjectStoreError> {
        let mut conn = self
            .object_store_conn_pool
            .get()
            .await
            .map_err(ObjectStoreError::from)
            .ctx("MsSqlStore : object_exists : get conn from pool")?;

        let sql_sentence = format!(
            "SELECT TOP 1 OBJECTID
            FROM {}.dbo.{}
            WHERE BUCKET = @P1 AND OBJECTID = @P2",
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
            .ctx("MsSqlStore : get : query")?;
        let rows = stream
            .into_first_result()
            .await
            .map_err(ObjectStoreError::from)
            .ctx("MsSqlStore : get : stream")?;

        println!("number of rows: {}", rows.len());
        if rows.len() > 0 { Ok(true) } else { Ok(false) }
    }

    fn get_dbname(&self, year: &str) -> String {
        format!("EFaturaDB01_{}", year)
    }
}
