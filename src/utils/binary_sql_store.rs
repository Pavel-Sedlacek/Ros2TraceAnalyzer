use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum BinarySQLStoreError {
    #[error("An error occured in rusqlite {0}")]
    SQLiteError(rusqlite::Error),
    #[error("An error occured during bincode encode {0}")]
    PostcardEncodeError(postcard::Error),
    #[error("An error occured during bincode decode {0}")]
    PostcardDecodeError(postcard::Error),
    #[error("There is no analysis named {0}")]
    NoSuchAnalysis(String),
}

pub struct BinarySQLStore {
    sqlite_connection: rusqlite::Connection,
}

impl BinarySQLStore {
    pub fn new(sqlite_file: &Path) -> Result<BinarySQLStore, BinarySQLStoreError> {
        let sqlite_connection =
            rusqlite::Connection::open(sqlite_file).map_err(BinarySQLStoreError::SQLiteError)?;

        Ok(BinarySQLStore { sqlite_connection })
    }

    pub fn write(
        &mut self,
        table: &str,
        data: Vec<impl StoreEntity>,
    ) -> Result<(), BinarySQLStoreError> {
        let data = data.iter().filter_map(|d| {
            let mut h = std::hash::DefaultHasher::new();
            d.id().hash(&mut h);

            let bin = postcard::to_allocvec(d.data())
                .map_err(BinarySQLStoreError::PostcardEncodeError)
                .ok();

            bin.map(|b| (h.finish() as i64, b))
        });

        self.sqlite_connection
            .execute(
                &format!(
                    "
            CREATE TABLE IF NOT EXISTS {} (
                id      INT PRIMARY KEY,
                data    BLOB
            );
        ",
                    table
                ),
                (),
            )
            .map_err(BinarySQLStoreError::SQLiteError)?;

        let tx = self
            .sqlite_connection
            .transaction()
            .map_err(BinarySQLStoreError::SQLiteError)?;

        {
            let mut query = tx
                .prepare_cached(&format!(
                    "
                    INSERT INTO {} (id, data) VALUES (?1, ?2)
                    ON CONFLICT (id) DO UPDATE SET
                        data = EXCLUDED.data
                ",
                    table
                ))
                .map_err(BinarySQLStoreError::SQLiteError)?;

            for entry in data {
                query
                    .execute(entry)
                    .map_err(BinarySQLStoreError::SQLiteError)?;
            }
        }

        tx.commit().map_err(BinarySQLStoreError::SQLiteError)?;
        Ok(())
    }

    pub fn read<T: for<'a> serde::Deserialize<'a>>(
        &self,
        table: &str,
        id: &str,
    ) -> Result<T, BinarySQLStoreError> {
        let mut query = self
            .sqlite_connection
            .prepare(&format!("SELECT data FROM {} WHERE id = ?1", table))
            .map_err(BinarySQLStoreError::SQLiteError)?;

        let id = {
            let mut h = std::hash::DefaultHasher::new();
            id.hash(&mut h);
            h.finish() as i64
        };

        let mut results = query
            .query_map((id,), |row| row.get::<_, Vec<u8>>(0))
            .map_err(BinarySQLStoreError::SQLiteError)?;

        let data_blob = results
            .nth(0)
            .ok_or_else(|| BinarySQLStoreError::NoSuchAnalysis(table.to_owned()))?
            .map_err(BinarySQLStoreError::SQLiteError)?;

        let data = postcard::from_bytes::<T>(&data_blob)
            .map_err(BinarySQLStoreError::PostcardDecodeError)?;

        Ok(data)
    }
}

pub trait StoreEntity {
    fn id(&self) -> String;
    fn data(&self) -> &impl Serialize;
}
