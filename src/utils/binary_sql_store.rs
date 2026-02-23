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
    pub fn new(sqlite_file: std::path::PathBuf) -> Result<BinarySQLStore, BinarySQLStoreError> {
        let brand_new = !sqlite_file.exists();

        let sqlite_connection =
            rusqlite::Connection::open(sqlite_file).map_err(BinarySQLStoreError::SQLiteError)?;

        if brand_new {
            sqlite_connection
                .execute(
                    "CREATE TABLE blobs(
                        blob_name TEXT NOT NULL,
                        data BLOB
                    )",
                    (),
                )
                .map_err(BinarySQLStoreError::SQLiteError)?;
        }

        Ok(BinarySQLStore { sqlite_connection })
    }

    pub fn write(
        &self,
        name: &str,
        data: impl serde::Serialize,
    ) -> Result<(), BinarySQLStoreError> {
        let data_blob =
            postcard::to_allocvec(&data).map_err(BinarySQLStoreError::PostcardEncodeError)?;

        self.sqlite_connection
            .execute(
                "INSERT INTO blobs (blob_name, data) VALUES (?1, ?2)
                    ON CONFLICT (blob_name) DO UPDATE SET
                        data = ECLUDED.data",
                (name, data_blob),
            )
            .map_err(BinarySQLStoreError::SQLiteError)?;

        Ok(())
    }

    pub fn read<T: for<'a> serde::Deserialize<'a>>(
        &self,
        name: &str,
    ) -> Result<T, BinarySQLStoreError> {
        let mut statement = self
            .sqlite_connection
            .prepare("SELECT data FROM blobs WHERE blob_name = ?1 LIMIT 1")
            .map_err(BinarySQLStoreError::SQLiteError)?;

        let mut results = statement
            .query_map((name,), |row| row.get::<_, Vec<u8>>(0))
            .map_err(BinarySQLStoreError::SQLiteError)?;

        let data_blob = results
            .nth(0)
            .ok_or_else(|| BinarySQLStoreError::NoSuchAnalysis(name.to_owned()))?
            .map_err(BinarySQLStoreError::SQLiteError)?;

        let data = postcard::from_bytes::<T>(&data_blob)
            .map_err(BinarySQLStoreError::PostcardDecodeError)?;

        Ok(data)
    }
}
