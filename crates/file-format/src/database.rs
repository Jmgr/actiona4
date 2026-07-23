use std::{fmt, path::Path, pin::Pin, str::FromStr, sync::Arc};

use bytes::{Bytes, BytesMut};
use const_format::formatcp;
use futures::{Stream, StreamExt, pin_mut};
use jiff::Timestamp;
use rusqlite::{Connection, OpenFlags, params, types::FromSql};
use rusqlite_migration::{M, Migrations};
use sha2::{Digest, Sha256};
use strum::EnumString;
use tokio::sync::Mutex;
use uuid::Uuid;

const STORAGE_FORMAT: &str = "actiona";
const STORAGE_VERSION: u32 = 0;
const MAX_CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB
const DEFAULT_CHUNK_SIZE: usize = 256 * 1024; // 256 KiB

const MIGRATIONS_SLICE: &[M<'_>] = &[M::up(formatcp!(
    "
    CREATE TABLE document (
        id INTEGER PRIMARY KEY CHECK (id = 0),
        json TEXT NOT NULL CHECK (json_valid(json))
    );
    CREATE TABLE meta (
        key TEXT PRIMARY KEY NOT NULL,
        value BLOB NOT NULL
    );
    CREATE TABLE attachment (
        id BLOB PRIMARY KEY NOT NULL CHECK (length(id) = 16),
        state TEXT NOT NULL CHECK (state IN ('staging', 'ready')),
        storage_codec TEXT NOT NULL,
        created_at TEXT NOT NULL,
        size INTEGER CHECK (size IS NULL OR size >= 0),
        sha256 BLOB CHECK (sha256 IS NULL OR length(sha256) = 32),
        CHECK (
            state = 'staging'
            OR (
                size IS NOT NULL
                AND sha256 IS NOT NULL
            )
        )
    );
    CREATE TABLE attachment_chunk (
        attachment_id BLOB NOT NULL
            REFERENCES attachment(id) ON DELETE CASCADE,
        chunk_index INTEGER NOT NULL CHECK (chunk_index >= 0),
        data BLOB NOT NULL CHECK (length(data) > 0 AND length(data) <= {MAX_CHUNK_SIZE}),
        UNIQUE (attachment_id, chunk_index)
    );
    "
))];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AttachmentState {
    Staging,
    Ready,
}

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AttachmentCodec {
    Copy,
}

#[derive(Debug)]
pub struct AttachmentDetails {
    pub state: AttachmentState,
    pub codec: AttachmentCodec,
    pub created_at: Timestamp,
    pub size: Option<i64>,
    pub sha256: Option<[u8; 32]>,
    pub chunks: i64,
}

pub type ByteStream<E> = Pin<Box<dyn Stream<Item = Result<Bytes, E>>>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    RusqliteMigration(#[from] rusqlite_migration::Error),
    #[error("invalid format, got {0} expected {1}")]
    Format(String, String),
    #[error("invalid version, got {0} expected {1}")]
    Version(u32, u32),
    #[error("input file is too large: {0}")]
    InputFileIsTooLarge(String),
    #[error("invalid maximum chunk size: {0}")]
    InvalidMaxChunkSize(usize),
    #[error("attachment input failed: {0}")]
    AttachmentInput(String),
    #[error("invalid attachment state: {0}")]
    AttachmentState(String),
    #[error("invalid attachment codec: {0}")]
    AttachmentCodec(String),
    #[error("invalid sha256: {0:?}")]
    Sha256(Vec<u8>),
}

fn chunk_attachment_stream<E>(
    stream: ByteStream<E>,
    max_chunk_size: usize,
) -> impl Stream<Item = Result<Bytes, Error>>
where
    E: fmt::Display,
{
    async_stream::try_stream! {
        pin_mut!(stream);

        let mut buffer = BytesMut::new();
        while let Some(chunk) = stream.next().await {
            let mut chunk = chunk.map_err(|error| Error::AttachmentInput(error.to_string()))?;

            if !buffer.is_empty() {
                let count = (max_chunk_size - buffer.len()).min(chunk.len());
                buffer.extend_from_slice(&chunk.split_to(count));
                if buffer.len() == max_chunk_size {
                    yield buffer.split().freeze();
                }
            }

            while chunk.len() >= max_chunk_size {
                yield chunk.split_to(max_chunk_size);
            }

            if !chunk.is_empty() {
                buffer.extend_from_slice(&chunk);
            }
        }

        if !buffer.is_empty() {
            yield buffer.freeze();
        }
    }
}

pub struct Database {
    connection: Arc<Mutex<Connection>>,
    max_chunk_size: usize,
}

impl Database {
    pub fn create_in_memory() -> Result<ReadWriteDatabase, Error> {
        Self::create_in_memory_with_max_chunk_size(DEFAULT_CHUNK_SIZE)
    }

    pub fn create_in_memory_with_max_chunk_size(
        max_chunk_size: usize,
    ) -> Result<ReadWriteDatabase, Error> {
        let connection = Connection::open_in_memory_with_flags(
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        Self::create_impl(connection, max_chunk_size)
    }

    pub fn open(path: &Path) -> Result<ReadOnlyDatabase, Error> {
        let connection = Connection::open(path)?;
        Self::open_impl(connection)
    }

    pub fn create(path: &Path) -> Result<ReadWriteDatabase, Error> {
        Self::create_with_max_chunk_size(path, DEFAULT_CHUNK_SIZE)
    }

    pub fn create_with_max_chunk_size(
        path: &Path,
        max_chunk_size: usize,
    ) -> Result<ReadWriteDatabase, Error> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        Self::create_impl(connection, max_chunk_size)
    }

    fn new(connection: Connection, max_chunk_size: usize) -> Self {
        Self {
            connection: Arc::new(Mutex::new(connection)),
            max_chunk_size,
        }
    }

    async fn set_document(&self, document: &serde_json::Value) -> Result<(), Error> {
        self.connection.lock().await.execute(
            "INSERT INTO document (id, json) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET json=?2",
            params![0, document],
        )?;

        Ok(())
    }

    async fn get_document(&self) -> Result<serde_json::Value, Error> {
        Ok(self.connection.lock().await.query_one(
            "SELECT json FROM document",
            params![],
            |row| row.get(0),
        )?)
    }

    fn open_impl(mut connection: Connection) -> Result<ReadOnlyDatabase, Error> {
        Self::setup_connection(&connection)?;
        Self::migrate(&mut connection)?;
        Self::check(&connection)?;

        Ok(ReadOnlyDatabase {
            inner: Self::new(connection, DEFAULT_CHUNK_SIZE),
        })
    }

    fn create_impl(
        mut connection: Connection,
        max_chunk_size: usize,
    ) -> Result<ReadWriteDatabase, Error> {
        if max_chunk_size == 0 || max_chunk_size > MAX_CHUNK_SIZE {
            return Err(Error::InvalidMaxChunkSize(max_chunk_size));
        }

        Self::setup_connection(&connection)?;
        Self::migrate(&mut connection)?;

        connection.execute(
            "INSERT INTO meta (key, value) VALUES (?, ?)",
            params!["format", STORAGE_FORMAT],
        )?;
        connection.execute(
            "INSERT INTO meta (key, value) VALUES (?, ?)",
            params!["version", STORAGE_VERSION],
        )?;

        Ok(ReadWriteDatabase {
            inner: Self::new(connection, max_chunk_size),
        })
    }

    fn setup_connection(connection: &Connection) -> Result<(), Error> {
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "trusted_schema", false)?;

        Ok(())
    }

    fn migrate(connection: &mut Connection) -> Result<(), Error> {
        MIGRATIONS.to_latest(connection)?;

        Ok(())
    }

    fn check(connection: &Connection) -> Result<(), Error> {
        let format: String = Self::get_meta(connection, "format")?;
        if format != STORAGE_FORMAT {
            return Err(Error::Format(format, STORAGE_FORMAT.to_owned()));
        }

        let version: u32 = Self::get_meta(connection, "version")?;
        if version != STORAGE_VERSION {
            return Err(Error::Version(version, STORAGE_VERSION));
        }

        Ok(())
    }

    fn get_meta<T: FromSql>(connection: &Connection, key: &str) -> Result<T, Error> {
        let result =
            connection.query_one("SELECT value FROM meta WHERE key = ?", [key], |result| {
                result.get::<_, T>(0)
            })?;
        Ok(result)
    }

    async fn get_attachment_details(
        &self,
        attachment_id: Uuid,
    ) -> Result<AttachmentDetails, Error> {
        let connection = self.connection.lock().await;

        let (state, codec, created_at, size, sha256, chunks) = connection.query_one(
            "SELECT state, storage_codec, created_at, size, sha256,
             (SELECT COUNT(*) FROM attachment_chunk WHERE attachment_id = ?1) AS chunks
            FROM attachment
            WHERE id = ?1",
            params!(attachment_id),
            |row| {
                let state = row.get_ref(0)?.as_str()?;
                let state = AttachmentState::from_str(state)
                    .map_err(|_| Error::AttachmentState(state.to_owned()));

                let codec = row.get_ref(1)?.as_str()?;
                let codec = AttachmentCodec::from_str(codec)
                    .map_err(|_| Error::AttachmentCodec(codec.to_owned()));

                let created_at = row.get::<_, Timestamp>(2)?;

                let size = row.get::<_, Option<i64>>(3)?;

                let sha256 = row.get::<_, Option<Vec<u8>>>(4)?;
                let sha256: Result<Option<[u8; 32]>, _> =
                    sha256.map(|sha256| sha256.try_into()).transpose();

                let chunks = row.get::<_, i64>(5)?;

                Ok((state, codec, created_at, size, sha256, chunks))
            },
        )?;

        let state = state?;
        let codec = codec?;
        let sha256 = sha256.map_err(Error::Sha256)?;

        Ok(AttachmentDetails {
            state,
            codec,
            created_at,
            size,
            sha256,
            chunks,
        })
    }

    fn read_attachment(&self, attachment_id: Uuid) -> ByteStream<Error> {
        let connection = self.connection.clone();

        Box::pin(async_stream::try_stream! {
            let connection = connection.lock().await;
            let mut statement = connection.prepare(
                "SELECT data FROM attachment_chunk WHERE attachment_id = ? ORDER BY chunk_index",
            )?;
            let mut rows = statement.query(params!(attachment_id))?;

            while let Some(row) = rows.next()? {
                let data: Vec<u8> = row.get(0)?;
                yield Bytes::from(data);
            }
        })
    }

    async fn write_attachment<E>(&self, stream: ByteStream<E>) -> Result<Uuid, Error>
    where
        E: fmt::Display,
    {
        let chunked_stream = chunk_attachment_stream(stream, self.max_chunk_size);

        let mut connection = self.connection.lock().await;
        let transaction = connection.transaction()?;

        let attachment_id = {
            let mut statement = transaction.prepare(
                "INSERT INTO attachment (id, state, storage_codec, created_at, size, sha256)
                    VALUES(?, 'staging', 'copy', ?, NULL, NULL)",
            )?;
            let attachment_id = Uuid::new_v4();
            let created_at = Timestamp::now();
            statement.execute(params!(attachment_id, created_at))?;

            let mut statement = transaction.prepare(
                "INSERT INTO attachment_chunk (attachment_id, chunk_index, data) VALUES (?, ?, ?)",
            )?;
            let mut index = 0_i64;
            let mut hasher = Sha256::new();
            let mut size = 0_i64;

            pin_mut!(chunked_stream);

            while let Some(chunk) = chunked_stream.next().await {
                let chunk = chunk?;
                statement.execute(params!(attachment_id, index, chunk.as_ref()))?;
                hasher.update(&chunk);
                size = size
                    .checked_add(
                        i64::try_from(chunk.len())
                            .map_err(|err| Error::InputFileIsTooLarge(err.to_string()))?,
                    )
                    .ok_or_else(|| Error::InputFileIsTooLarge(chunk.len().to_string()))?;
                index = index
                    .checked_add(1)
                    .ok_or_else(|| Error::InputFileIsTooLarge(chunk.len().to_string()))?;
            }

            let mut statement = transaction.prepare(
                "UPDATE attachment SET state = 'ready', size = ?, sha256 = ? WHERE id = ?",
            )?;
            statement.execute(params!(size, hasher.finalize().as_slice(), attachment_id))?;

            attachment_id
        };

        transaction.commit()?;

        Ok(attachment_id)
    }
}

pub struct ReadOnlyDatabase {
    inner: Database,
}

impl ReadOnlyDatabase {
    pub async fn get_document(&self) -> Result<serde_json::Value, Error> {
        self.inner.get_document().await
    }

    #[must_use]
    pub fn read_attachment(&self, attachment_id: Uuid) -> ByteStream<Error> {
        self.inner.read_attachment(attachment_id)
    }

    pub async fn get_attachment_details(
        &self,
        attachment_id: Uuid,
    ) -> Result<AttachmentDetails, Error> {
        self.inner.get_attachment_details(attachment_id).await
    }
}

pub struct ReadWriteDatabase {
    inner: Database,
}

impl ReadWriteDatabase {
    pub async fn set_document(&self, document: &serde_json::Value) -> Result<(), Error> {
        self.inner.set_document(document).await
    }

    pub async fn get_document(&self) -> Result<serde_json::Value, Error> {
        self.inner.get_document().await
    }

    pub async fn write_attachment<E>(&self, stream: ByteStream<E>) -> Result<Uuid, Error>
    where
        E: fmt::Display,
    {
        self.inner.write_attachment(stream).await
    }

    #[must_use]
    pub fn read_attachment(&self, attachment_id: Uuid) -> ByteStream<Error> {
        self.inner.read_attachment(attachment_id)
    }

    pub async fn get_attachment_details(
        &self,
        attachment_id: Uuid,
    ) -> Result<AttachmentDetails, Error> {
        self.inner.get_attachment_details(attachment_id).await
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use futures::{StreamExt, stream};
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn database() {
        let database = Database::create_in_memory().unwrap();

        database
            .set_document(&json!({
                "foo": true,
                "bar": 42,
            }))
            .await
            .unwrap();

        let document = database.get_document().await.unwrap();
        assert_eq!(document["foo"], true);
        assert_eq!(document["bar"], 42);
        //db.database.execute("SELECT *", []).unwrap();
    }

    #[tokio::test]
    async fn chunk_attachment_stream_rechunks_input() {
        let first = vec![0xA1; MAX_CHUNK_SIZE - 2];
        let second = vec![0xB2; 4];
        let third = vec![0xC3; MAX_CHUNK_SIZE + 1];
        let mut expected = Vec::with_capacity(first.len() + second.len() + third.len());
        expected.extend_from_slice(&first);
        expected.extend_from_slice(&second);
        expected.extend_from_slice(&third);
        let input = stream::iter([
            Ok::<_, io::Error>(Bytes::from(first)),
            Ok(Bytes::from(second)),
            Ok(Bytes::from(third)),
        ]);

        let chunks = chunk_attachment_stream(Box::pin(input), MAX_CHUNK_SIZE)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(
            chunks.iter().map(Bytes::len).collect::<Vec<_>>(),
            [MAX_CHUNK_SIZE, MAX_CHUNK_SIZE, 3,]
        );
        assert_eq!(chunks.concat(), expected);
    }

    #[tokio::test]
    async fn write_attachment_chunks_large_stream_items() {
        let database = Database::create_in_memory().unwrap();
        let data = Bytes::from(vec![0xAA; DEFAULT_CHUNK_SIZE + 1]);
        let attachment_id = database
            .write_attachment(Box::pin(stream::iter([Ok::<_, io::Error>(data)])))
            .await
            .unwrap();

        let connection = database.inner.connection.lock().await;
        let (state, created_at, size): (String, Timestamp, i64) = connection
            .query_row(
                "SELECT state, created_at, size FROM attachment WHERE id = ?",
                rusqlite::params![attachment_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        let chunk_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM attachment_chunk", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(state, "ready");
        assert!(created_at <= Timestamp::now());
        assert_eq!(size, i64::try_from(DEFAULT_CHUNK_SIZE + 1).unwrap());
        assert_eq!(chunk_count, 2);
    }

    #[tokio::test]
    async fn write_attachment_uses_configured_max_chunk_size() {
        let database = Database::create_in_memory_with_max_chunk_size(2).unwrap();
        let attachment_id = database
            .write_attachment(Box::pin(stream::iter([Ok::<_, io::Error>(
                Bytes::from_static(b"abcde"),
            )])))
            .await
            .unwrap();

        let details = database
            .get_attachment_details(attachment_id)
            .await
            .unwrap();
        assert_eq!(details.chunks, 3);
    }

    #[test]
    fn zero_max_chunk_size_is_rejected() {
        assert!(matches!(
            Database::create_in_memory_with_max_chunk_size(0),
            Err(Error::InvalidMaxChunkSize(0))
        ));
    }

    #[tokio::test]
    async fn attachment_roundtrips_through_database() {
        let database = Database::create_in_memory().unwrap();
        let data = vec![0xA5; DEFAULT_CHUNK_SIZE + 1];
        let expected_sha256: [u8; 32] = Sha256::digest(&data).into();
        let attachment_id = database
            .write_attachment(Box::pin(stream::iter([Ok::<_, io::Error>(Bytes::from(
                data.clone(),
            ))])))
            .await
            .unwrap();

        let details = database
            .get_attachment_details(attachment_id)
            .await
            .unwrap();
        assert!(matches!(details.state, AttachmentState::Ready));
        assert!(matches!(details.codec, AttachmentCodec::Copy));
        assert!(details.created_at <= Timestamp::now());
        assert_eq!(details.size, Some(i64::try_from(data.len()).unwrap()));
        assert_eq!(details.sha256, Some(expected_sha256));
        assert_eq!(details.chunks, 2);

        let chunks = database
            .read_attachment(attachment_id)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(chunks.concat(), data);
    }

    #[tokio::test]
    async fn write_attachment_rolls_back_input_errors() {
        let database = Database::create_in_memory().unwrap();
        let input = stream::iter(vec![
            Ok(Bytes::from_static(b"partial")),
            Err(io::Error::other("upstream failed")),
        ]);

        let error = database
            .write_attachment(Box::pin(input))
            .await
            .unwrap_err();
        assert!(matches!(error, Error::AttachmentInput(message) if message == "upstream failed"));

        let connection = database.inner.connection.lock().await;
        let attachment_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM attachment", [], |row| row.get(0))
            .unwrap();
        assert_eq!(attachment_count, 0);
    }
}
