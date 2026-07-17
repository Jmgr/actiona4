use std::{
    fs::File as StdFile,
    io::{self, Cursor, ErrorKind, Write},
    mem::size_of,
    path::{Path, PathBuf},
};

use action_definition::tree::ActionTree;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    select,
    task::JoinError,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use uuid::Uuid;

use crate::compression::{
    MAX_BINARY_COMPRESSED_SIZE, MAX_BINARY_UNCOMPRESSED_SIZE, MAX_JSON_FILE_SIZE, compress,
    decompress,
};

mod attachment;
mod attachment_bytes;
mod compression;
mod json;
mod media_type;

pub use attachment::{Attachment, AttachmentKind};
pub use attachment_bytes::AttachmentBytes;
pub use media_type::MediaType;

const BINARY_VERSION: u16 = 0;
const JSON_VERSION: u16 = 0;
const BINARY_CODEC_ZSTD: u8 = 1;

struct BinaryHeader {
    version: u16,
    codec: u8,
    compressed_size: u64,
    uncompressed_size: u64,
}

impl BinaryHeader {
    const MAGIC: [u8; 12] = *b"ACTIONA\xF0\x9F\xA4\x96\0";

    async fn read_from(reader: &mut (impl AsyncRead + Unpin)) -> Result<Self, Error> {
        let mut magic = [0; Self::MAGIC.len()];
        reader.read_exact(&mut magic).await?;
        if magic != Self::MAGIC {
            return Err(Error::BinaryMagic);
        }

        let mut version = [0; size_of::<u16>()];
        reader.read_exact(&mut version).await?;
        let mut codec = [0; size_of::<u8>()];
        reader.read_exact(&mut codec).await?;
        let mut compressed_size = [0; size_of::<u64>()];
        reader.read_exact(&mut compressed_size).await?;
        let mut uncompressed_size = [0; size_of::<u64>()];
        reader.read_exact(&mut uncompressed_size).await?;

        Ok(Self {
            version: u16::from_le_bytes(version),
            codec: codec[0],
            compressed_size: u64::from_le_bytes(compressed_size),
            uncompressed_size: u64::from_le_bytes(uncompressed_size),
        })
    }

    fn write_to(&self, writer: &mut impl io::Write) -> io::Result<()> {
        writer.write_all(&Self::MAGIC)?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&[self.codec])?;
        writer.write_all(&self.compressed_size.to_le_bytes())?;
        writer.write_all(&self.uncompressed_size.to_le_bytes())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FileMetadata {
    pub created_by: Option<WriterInfo>,
    pub last_saved_by: Option<WriterInfo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WriterInfo {
    pub application: String,
    pub application_version: Option<String>,
    pub os: Option<String>,
    pub architecture: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("invalid binary magic")]
    BinaryMagic,
    #[error("unexpected version {0}")]
    Version(u16),
    #[error("unsupported binary codec {0}")]
    BinaryCodec(u8),
    #[error("{kind} size {size} bytes exceeds the {limit}-byte limit")]
    PayloadTooLarge {
        kind: &'static str,
        size: u64,
        limit: usize,
    },
    #[error("binary file has data after its declared compressed payload")]
    BinaryFileTrailingData,
    #[error("binary payload has data after the action file")]
    BinaryPayloadTrailingData,
    #[error("invalid binary payload: {0}")]
    Postcard(#[from] postcard::Error),
    #[error("invalid JSON payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("file processing task failed: {0}")]
    Task(#[from] JoinError),
    #[error("operation canceled")]
    Canceled,
    #[error("invalid attachment payload: {0}")]
    Attachment(String),
    #[error("image processing task failed: {0}")]
    Image(#[from] image::ImageError),
}

#[derive(Clone, Deserialize, Serialize)]
pub struct File {
    pub actions: ActionTree,
    #[serde(with = "indexmap::map::serde_seq")]
    pub attachments: IndexMap<Uuid, Attachment>,
    #[serde(default)]
    pub metadata: FileMetadata,
}

impl File {
    // Validates the binary envelope before allocating its exact compressed payload length.
    pub async fn read_binary(
        filepath: &Path,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
    ) -> Result<Self, Error> {
        check_canceled(cancellation_token)?;
        let header_read = async {
            let mut file = fs::File::open(filepath).await?;
            let header = BinaryHeader::read_from(&mut file).await?;
            let header_size = file.stream_position().await?;
            let file_size = file.metadata().await?.len();
            Ok::<_, Error>((file, header, header_size, file_size))
        };
        let (mut file, header, header_size, actual_file_size) = select! {
            biased;
            () = cancellation_token.cancelled() => return Err(Error::Canceled),
            result = header_read => result?,
        };
        if header.version != BINARY_VERSION {
            return Err(Error::Version(header.version));
        }
        if header.codec != BINARY_CODEC_ZSTD {
            return Err(Error::BinaryCodec(header.codec));
        }
        let compressed_size = checked_payload_size(
            "compressed binary payload",
            header.compressed_size,
            MAX_BINARY_COMPRESSED_SIZE,
        )?;
        let uncompressed_size = checked_payload_size(
            "uncompressed binary payload",
            header.uncompressed_size,
            MAX_BINARY_UNCOMPRESSED_SIZE,
        )?;

        let declared_file_size =
            header_size
                .checked_add(compressed_size as u64)
                .ok_or(Error::PayloadTooLarge {
                    kind: "compressed binary payload",
                    size: u64::MAX,
                    limit: MAX_BINARY_COMPRESSED_SIZE,
                })?;
        if actual_file_size > declared_file_size {
            return Err(Error::BinaryFileTrailingData);
        }
        if actual_file_size < declared_file_size {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "binary file is shorter than its declared compressed payload",
            )
            .into());
        }

        let mut payload = vec![0; compressed_size];
        select! {
            biased;
            () = cancellation_token.cancelled() => return Err(Error::Canceled),
            result = file.read_exact(&mut payload) => { result?; },
        }

        let task_token = cancellation_token.clone();
        let handle = task_tracker.spawn_blocking(move || {
            check_canceled(&task_token)?;
            let payload = decompress(&payload, uncompressed_size, MAX_BINARY_UNCOMPRESSED_SIZE)?;
            let (file, remaining): (Self, &[u8]) = postcard::take_from_bytes(&payload)?;
            if !remaining.is_empty() {
                return Err(Error::BinaryPayloadTrailingData);
            }

            Ok(file)
        });
        select! {
            biased;
            () = cancellation_token.cancelled() => Err(Error::Canceled),
            result = handle => result?,
        }
    }

    // Serializes first so the header can declare exact compressed and uncompressed lengths.
    pub async fn write_binary(
        &self,
        filepath: &Path,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Error> {
        check_canceled(cancellation_token)?;
        let file = self.clone();
        let task_token = cancellation_token.clone();
        let handle = task_tracker.spawn_blocking(move || {
            check_canceled(&task_token)?;
            let payload = postcard::to_stdvec(&file)?;
            checked_payload_size(
                "uncompressed binary payload",
                payload.len() as u64,
                MAX_BINARY_UNCOMPRESSED_SIZE,
            )?;
            let compressed = compress(&payload)?;
            checked_payload_size(
                "compressed binary payload",
                compressed.len() as u64,
                MAX_BINARY_COMPRESSED_SIZE,
            )?;

            let header = BinaryHeader {
                version: BINARY_VERSION,
                codec: BINARY_CODEC_ZSTD,
                compressed_size: compressed.len() as u64,
                uncompressed_size: payload.len() as u64,
            };
            let mut writer = Cursor::new(Vec::new());
            header.write_to(&mut writer)?;
            writer.get_mut().reserve(compressed.len());
            Write::write_all(&mut writer, &compressed)?;
            Ok::<Vec<u8>, Error>(writer.into_inner())
        });
        let buffer = select! {
            biased;
            () = cancellation_token.cancelled() => return Err(Error::Canceled),
            result = handle => result??,
        };

        select! {
            biased;
            () = cancellation_token.cancelled() => Err(Error::Canceled),
            result = write_atomically(filepath, &buffer, task_tracker, cancellation_token) => result,
        }
    }

    // Moves parsing and attachment decoding off the asynchronous I/O runtime.
    pub async fn read_json(
        filepath: &Path,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
    ) -> Result<Self, Error> {
        check_canceled(cancellation_token)?;
        let file = select! {
            biased;
            () = cancellation_token.cancelled() => return Err(Error::Canceled),
            result = fs::File::open(filepath) => result?,
        };
        let mut data = Vec::new();
        let mut reader = file.take(MAX_JSON_FILE_SIZE as u64 + 1);
        select! {
            biased;
            () = cancellation_token.cancelled() => return Err(Error::Canceled),
            result = reader.read_to_end(&mut data) => { result?; },
        };
        checked_payload_size("JSON file", data.len() as u64, MAX_JSON_FILE_SIZE)?;
        let task_token = cancellation_token.clone();
        let handle = task_tracker.spawn_blocking(move || {
            check_canceled(&task_token)?;
            let contents: json::FileWire = serde_json::from_slice(&data)?;
            if contents.version != JSON_VERSION {
                return Err(Error::Version(contents.version));
            }

            contents.into_file(&task_token)
        });
        select! {
            biased;
            () = cancellation_token.cancelled() => Err(Error::Canceled),
            result = handle => result?,
        }
    }

    // Produces the JSON wire representation before writing it as formatted text.
    pub async fn write_json(
        &self,
        filepath: &Path,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Error> {
        check_canceled(cancellation_token)?;
        let file = self.clone();
        let task_token = cancellation_token.clone();
        let handle = task_tracker.spawn_blocking(move || {
            let contents = json::FileWire::from_file(file, JSON_VERSION, &task_token)?;
            check_canceled(&task_token)?;
            let mut buffer = Vec::new();
            serde_json::to_writer_pretty(&mut buffer, &contents)?;
            checked_payload_size("JSON file", buffer.len() as u64, MAX_JSON_FILE_SIZE)?;
            Ok::<Vec<u8>, Error>(buffer)
        });
        let buffer = select! {
            biased;
            () = cancellation_token.cancelled() => return Err(Error::Canceled),
            result = handle => result??,
        };

        select! {
            biased;
            () = cancellation_token.cancelled() => Err(Error::Canceled),
            result = write_atomically(filepath, &buffer, task_tracker, cancellation_token) => result,
        }
    }
}

// Converts and validates untrusted header lengths before they are used for allocation.
pub(crate) const fn checked_payload_size(
    kind: &'static str,
    size: u64,
    limit: usize,
) -> Result<usize, Error> {
    if size > limit as u64 {
        return Err(Error::PayloadTooLarge { kind, size, limit });
    }

    Ok(size as usize)
}

fn check_canceled(cancellation_token: &CancellationToken) -> Result<(), Error> {
    if cancellation_token.is_cancelled() {
        Err(Error::Canceled)
    } else {
        Ok(())
    }
}

// Writes to a synced temporary in the target directory, then atomically replaces the destination.
async fn write_atomically(
    filepath: &Path,
    data: &[u8],
    task_tracker: &TaskTracker,
    cancellation_token: &CancellationToken,
) -> Result<(), Error> {
    check_canceled(cancellation_token)?;
    let directory = destination_directory(filepath);
    let existing_permissions = select! {
        biased;
        () = cancellation_token.cancelled() => return Err(Error::Canceled),
        result = fs::metadata(filepath) => match result {
            Ok(metadata) => Some(metadata.permissions()),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        },
    };
    let temporary = tempfile::Builder::new()
        .prefix(".")
        .tempfile_in(&directory)?;
    if let Some(permissions) = existing_permissions {
        // Set the destination mode before syncing the temporary inode, so both data and metadata
        // are durable before it replaces the destination.
        temporary.as_file().set_permissions(permissions)?;
    }
    let mut file = fs::File::from_std(temporary.reopen()?);

    select! {
        biased;
        () = cancellation_token.cancelled() => return Err(Error::Canceled),
        result = file.write_all(data) => result?,
    }

    select! {
        biased;
        () = cancellation_token.cancelled() => return Err(Error::Canceled),
        result = file.sync_all() => result?,
    }
    drop(file);

    let filepath = filepath.to_path_buf();
    let task_token = cancellation_token.clone();
    let handle = task_tracker.spawn_blocking(move || {
        check_canceled(&task_token)?;
        temporary
            .persist(filepath)
            .map_err(|error| Error::Io(error.error))?;
        Ok::<(), Error>(())
    });
    // Once persistence starts, cancellation cannot safely claim the write did not happen.
    handle.await??;
    sync_directory(&directory, task_tracker).await?;

    Ok(())
}

fn destination_directory(filepath: &Path) -> PathBuf {
    filepath
        .parent()
        .filter(|directory| !directory.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

// Persists the rename itself on Unix, where the directory entry has separate durability semantics.
#[cfg(unix)]
async fn sync_directory(directory: &Path, task_tracker: &TaskTracker) -> Result<(), Error> {
    let directory = directory.to_path_buf();
    let handle = task_tracker.spawn_blocking(move || {
        StdFile::open(directory)?.sync_all()?;
        Ok(())
    });
    handle.await?
}

#[cfg(not(unix))]
async fn sync_directory(_: &Path, _: &TaskTracker) -> Result<(), Error> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Cursor, ErrorKind},
        mem::size_of,
        path::Path,
    };

    use action_definition::{
        actions::{ActionInstance, Code},
        tree::ActionTree,
    };
    use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};
    use image::{DynamicImage, GenericImageView as _, ImageFormat, Rgba, RgbaImage};
    use indexmap::IndexMap;
    use rstest::rstest;
    use serde_json::json;
    use tokio::{fs, io::AsyncSeekExt as _};
    use tokio_util::{sync::CancellationToken, task::TaskTracker};
    use uuid::Uuid;

    use super::{
        Attachment, Error, File, FileMetadata, JSON_VERSION, WriterInfo,
        attachment::AttachmentKind, json,
    };

    #[derive(Clone, Copy)]
    enum Format {
        Binary,
        Json,
    }

    fn task_context() -> (TaskTracker, CancellationToken) {
        (TaskTracker::new(), CancellationToken::new())
    }

    fn attachment_id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn png_data() -> Vec<u8> {
        let image = RgbaImage::from_pixel(1, 1, Rgba([0x20, 0xA0, 0xF0, 0xFF]));
        let mut data = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut data), ImageFormat::Png)
            .expect("encode test PNG");
        data
    }

    fn wave_data() -> Vec<u8> {
        const SAMPLE_RATE: u32 = 48_000;
        const CHANNELS: u16 = 1;
        const BITS_PER_SAMPLE: u16 = 16;
        let samples = vec![0_i16; 960];
        let data_size = (samples.len() * size_of::<i16>()) as u32;
        let byte_rate = SAMPLE_RATE * u32::from(CHANNELS) * u32::from(BITS_PER_SAMPLE) / 8;
        let block_align = CHANNELS * BITS_PER_SAMPLE / 8;

        let mut data = Vec::with_capacity(44 + data_size as usize);
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&(36 + data_size).to_le_bytes());
        data.extend_from_slice(b"WAVEfmt ");
        data.extend_from_slice(&16_u32.to_le_bytes());
        data.extend_from_slice(&1_u16.to_le_bytes());
        data.extend_from_slice(&CHANNELS.to_le_bytes());
        data.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
        data.extend_from_slice(&byte_rate.to_le_bytes());
        data.extend_from_slice(&block_align.to_le_bytes());
        data.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
        data.extend_from_slice(b"data");
        data.extend_from_slice(&data_size.to_le_bytes());
        for sample in samples {
            data.extend_from_slice(&sample.to_le_bytes());
        }
        data
    }

    fn binary_header(compressed_size: u64, uncompressed_size: u64) -> Vec<u8> {
        let mut writer = Cursor::new(Vec::new());
        super::BinaryHeader {
            version: super::BINARY_VERSION,
            codec: super::BINARY_CODEC_ZSTD,
            compressed_size,
            uncompressed_size,
        }
        .write_to(&mut writer)
        .expect("write binary test header");
        writer.into_inner()
    }

    fn file_with_content() -> File {
        let mut tree = ActionTree::default();
        tree.append_action_instance(
            ActionInstance::Code(
                Code::new("return 'next';")
                    .with_branches(vec!["next".to_owned()])
                    .into(),
            ),
            tree.root(),
        )
        .expect("append test action");

        let binary: Attachment = serde_json::from_value(json!({
            "filename": "payload.bin",
            "kind": {
                "binary": {
                    "media_type": "application/octet-stream",
                    "data": "AAEC/w==",
                },
            },
        }))
        .expect("deserialize binary attachment");
        let text: Attachment = serde_json::from_value(json!({
            "filename": "notes.txt",
            "kind": {
                "text": {
                    "media_type": "text/plain; charset=utf-8",
                    "text": "A note.",
                },
            },
        }))
        .expect("deserialize text attachment");
        let image = Attachment {
            filename: Some("pixel.png".to_owned()),
            kind: AttachmentKind::Image {
                media_type: "image/png"
                    .parse::<mime::Mime>()
                    .expect("parse test image MIME type")
                    .into(),
                data: png_data().into(),
                dimensions: None,
            },
        };

        File {
            actions: tree,
            attachments: IndexMap::from([
                (attachment_id(12), binary),
                (attachment_id(4), text),
                (attachment_id(27), image),
            ]),
            metadata: FileMetadata {
                created_by: Some(WriterInfo {
                    application: "Actiona".to_owned(),
                    application_version: Some("0.2.0".to_owned()),
                    os: Some("test-os".to_owned()),
                    architecture: Some("test-arch".to_owned()),
                }),
                last_saved_by: None,
            },
        }
    }

    #[rstest]
    #[case::binary(Format::Binary)]
    #[case::json(Format::Json)]
    #[tokio::test]
    async fn files_roundtrip_through_each_format(#[case] format: Format) {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join(match format {
            Format::Binary => "file.actiona",
            Format::Json => "file.actiona.json",
        });
        let expected = file_with_content();
        let (task_tracker, cancellation_token) = task_context();

        let actual = match format {
            Format::Binary => {
                expected
                    .write_binary(&filepath, &task_tracker, &cancellation_token)
                    .await
                    .expect("write binary file");
                File::read_binary(&filepath, &task_tracker, &cancellation_token)
                    .await
                    .expect("read binary file")
            }
            Format::Json => {
                expected
                    .write_json(&filepath, &task_tracker, &cancellation_token)
                    .await
                    .expect("write JSON file");
                File::read_json(&filepath, &task_tracker, &cancellation_token)
                    .await
                    .expect("read JSON file")
            }
        };

        assert_eq!(
            serde_json::to_value(&actual.actions).expect("serialize read tree"),
            serde_json::to_value(&expected.actions).expect("serialize written tree"),
        );
        assert_eq!(
            serde_json::to_value(&actual.metadata).expect("serialize read metadata"),
            serde_json::to_value(&expected.metadata).expect("serialize written metadata"),
        );
        assert_eq!(
            serde_json::to_value(&actual.attachments[&attachment_id(12)])
                .expect("serialize binary attachment"),
            serde_json::to_value(&expected.attachments[&attachment_id(12)])
                .expect("serialize expected binary attachment"),
        );
        assert_eq!(
            serde_json::to_value(&actual.attachments[&attachment_id(4)])
                .expect("serialize text attachment"),
            serde_json::to_value(&expected.attachments[&attachment_id(4)])
                .expect("serialize expected text attachment"),
        );
        if matches!(format, Format::Binary) {
            assert_eq!(
                serde_json::to_value(&actual.attachments[&attachment_id(27)])
                    .expect("serialize binary image attachment"),
                serde_json::to_value(&expected.attachments[&attachment_id(27)])
                    .expect("serialize expected binary image attachment"),
            );
        } else {
            let image = serde_json::to_value(&actual.attachments[&attachment_id(27)])
                .expect("serialize JSON image attachment");
            assert_eq!(image["kind"]["image"]["media_type"], "image/webp");
            let data = Base64
                .decode(image["kind"]["image"]["data"].as_str().expect("image data"))
                .expect("decode WebP image");
            let image = image::load_from_memory(&data).expect("decode WebP image");
            assert_eq!(image.dimensions(), (1, 1));
            assert_eq!(image.to_rgba8().get_pixel(0, 0).0, [0x20, 0xA0, 0xF0, 0xFF]);
        }
        assert_eq!(
            actual.attachments.keys().copied().collect::<Vec<_>>(),
            vec![attachment_id(12), attachment_id(4), attachment_id(27)],
        );
    }

    #[tokio::test]
    async fn binary_payload_is_zstd_compressed() {
        let mut file = file_with_content();
        file.attachments.insert(
            attachment_id(99),
            Attachment {
                filename: None,
                kind: AttachmentKind::Binary {
                    media_type: None,
                    data: vec![0; 64 * 1024].into(),
                },
            },
        );
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join("file.actiona");
        let (task_tracker, cancellation_token) = task_context();

        file.write_binary(&filepath, &task_tracker, &cancellation_token)
            .await
            .expect("write compressed binary file");
        let bytes = fs::read(&filepath)
            .await
            .expect("read compressed binary file");
        let mut reader = fs::File::open(&filepath)
            .await
            .expect("open compressed binary file");
        let header = super::BinaryHeader::read_from(&mut reader)
            .await
            .expect("read binary header");
        let header_size = reader.stream_position().await.expect("get header size");

        assert_eq!(header.codec, super::BINARY_CODEC_ZSTD);
        assert_eq!(
            bytes.len() - header_size as usize,
            header.compressed_size as usize
        );
        assert!(header.compressed_size < header.uncompressed_size);
        let read = File::read_binary(&filepath, &task_tracker, &cancellation_token)
            .await
            .expect("read compressed binary file");
        assert_eq!(
            serde_json::to_value(&read.attachments[&attachment_id(99)])
                .expect("serialize read attachment"),
            serde_json::to_value(&file.attachments[&attachment_id(99)])
                .expect("serialize written attachment"),
        );
    }

    #[tokio::test]
    async fn binary_reader_rejects_oversized_compressed_payload_before_allocation() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join("oversized.actiona");
        let (task_tracker, cancellation_token) = task_context();
        fs::write(
            &filepath,
            binary_header(super::MAX_BINARY_COMPRESSED_SIZE as u64 + 1, 0),
        )
        .await
        .expect("write binary header");

        assert!(matches!(
            File::read_binary(&filepath, &task_tracker, &cancellation_token).await,
            Err(Error::PayloadTooLarge {
                kind: "compressed binary payload",
                ..
            })
        ));
    }

    #[tokio::test]
    async fn binary_reader_rejects_trailing_file_data() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join("trailing.actiona");
        let (task_tracker, cancellation_token) = task_context();
        let mut data = binary_header(0, 0);
        data.push(0);
        fs::write(&filepath, data)
            .await
            .expect("write binary header with trailing byte");

        assert!(matches!(
            File::read_binary(&filepath, &task_tracker, &cancellation_token).await,
            Err(Error::BinaryFileTrailingData)
        ));
    }

    #[tokio::test]
    async fn binary_reader_rejects_short_declared_payload_before_allocation() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join("short.actiona");
        let (task_tracker, cancellation_token) = task_context();
        fs::write(&filepath, binary_header(16 * 1024 * 1024, 0))
            .await
            .expect("write short binary file");

        assert!(matches!(
            File::read_binary(&filepath, &task_tracker, &cancellation_token).await,
            Err(Error::Io(error)) if error.kind() == ErrorKind::UnexpectedEof
        ));
    }

    #[test]
    fn relative_destinations_use_the_current_directory_for_atomic_sync() {
        assert_eq!(
            super::destination_directory(Path::new("file.actiona")),
            Path::new("."),
        );
    }

    #[tokio::test]
    async fn json_reader_rejects_duplicate_attachment_ids() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join("duplicate.actiona.json");
        let (task_tracker, cancellation_token) = task_context();
        let tree = serde_json::to_value(ActionTree::default()).expect("serialize tree");
        let attachment = json!({
            "filename": null,
            "kind": { "type": "text", "text": "one" },
        });
        let contents = json!({
            "version": JSON_VERSION,
            "tree": tree,
            "attachments": [
                [attachment_id(7), attachment.clone()],
                [attachment_id(7), attachment],
            ],
        });
        fs::write(
            &filepath,
            serde_json::to_vec(&contents).expect("serialize JSON"),
        )
        .await
        .expect("write duplicate JSON file");

        assert!(matches!(
            File::read_json(&filepath, &task_tracker, &cancellation_token).await,
            Err(Error::Attachment(message))
                if message == format!("duplicate attachment ID {}", attachment_id(7))
        ));
    }

    #[tokio::test]
    async fn canceled_operations_return_canceled() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join("file.actiona");
        let (task_tracker, cancellation_token) = task_context();
        cancellation_token.cancel();

        assert!(matches!(
            File::read_binary(&filepath, &task_tracker, &cancellation_token).await,
            Err(Error::Canceled)
        ));
        assert!(matches!(
            file_with_content()
                .write_binary(&filepath, &task_tracker, &cancellation_token)
                .await,
            Err(Error::Canceled)
        ));
        assert!(!filepath.exists());
    }

    #[tokio::test]
    async fn json_compresses_large_binary_and_text_attachments() {
        let mut file = File {
            actions: ActionTree::default(),
            attachments: IndexMap::new(),
            metadata: FileMetadata::default(),
        };
        file.attachments.insert(
            attachment_id(1),
            Attachment {
                filename: None,
                kind: AttachmentKind::Binary {
                    media_type: None,
                    data: vec![0xAB; 16 * 1024].into(),
                },
            },
        );
        file.attachments.insert(
            attachment_id(2),
            Attachment {
                filename: None,
                kind: AttachmentKind::Text {
                    media_type: Some(
                        "text/plain"
                            .parse::<mime::Mime>()
                            .expect("parse text MIME type")
                            .into(),
                    ),
                    text: "compressible text ".repeat(1_024),
                },
            },
        );
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join("file.actiona.json");
        let (task_tracker, cancellation_token) = task_context();

        file.write_json(&filepath, &task_tracker, &cancellation_token)
            .await
            .expect("write compressed JSON file");
        let json: serde_json::Value = serde_json::from_slice(
            &fs::read(&filepath)
                .await
                .expect("read compressed JSON file"),
        )
        .expect("parse compressed JSON file");
        let attachments = json["attachments"].as_array().expect("attachments array");
        assert_eq!(attachments[0][1]["kind"]["compression"], "zstd");
        assert_eq!(attachments[1][1]["kind"]["compression"], "zstd");
        assert!(attachments[1][1]["kind"].get("text").is_none());

        let read = File::read_json(&filepath, &task_tracker, &cancellation_token)
            .await
            .expect("read compressed JSON file");
        assert_eq!(
            serde_json::to_value(&read.attachments).expect("serialize read attachments"),
            serde_json::to_value(&file.attachments).expect("serialize written attachments"),
        );
    }

    #[tokio::test]
    async fn json_transcodes_audio_to_ogg_opus() {
        let file = File {
            actions: ActionTree::default(),
            attachments: IndexMap::from([(
                attachment_id(1),
                Attachment {
                    filename: Some("tone.wav".to_owned()),
                    kind: AttachmentKind::Audio {
                        media_type: "audio/wav"
                            .parse::<mime::Mime>()
                            .expect("parse WAV MIME type")
                            .into(),
                        data: wave_data().into(),
                    },
                },
            )]),
            metadata: FileMetadata::default(),
        };
        let directory = tempfile::tempdir().expect("create temporary directory");
        let filepath = directory.path().join("file.actiona.json");
        let (task_tracker, cancellation_token) = task_context();

        file.write_json(&filepath, &task_tracker, &cancellation_token)
            .await
            .expect("write JSON audio file");
        let read = File::read_json(&filepath, &task_tracker, &cancellation_token)
            .await
            .expect("read JSON audio file");
        let attachment = serde_json::to_value(&read.attachments[&attachment_id(1)])
            .expect("serialize Ogg Opus attachment");
        assert_eq!(
            attachment["kind"]["audio"]["media_type"],
            "audio/ogg; codecs=opus"
        );
        let data = Base64
            .decode(
                attachment["kind"]["audio"]["data"]
                    .as_str()
                    .expect("Ogg Opus data"),
            )
            .expect("decode Ogg Opus data");
        assert!(data.starts_with(b"OggS"));

        let second_filepath = directory.path().join("file-resaved.actiona.json");
        read.write_json(&second_filepath, &task_tracker, &cancellation_token)
            .await
            .expect("re-save Ogg Opus audio file");
        File::read_json(&second_filepath, &task_tracker, &cancellation_token)
            .await
            .expect("read re-saved Ogg Opus audio file");
    }

    #[test]
    fn json_omits_default_tree_fields_and_uses_snake_case_tags() {
        let cancellation_token = CancellationToken::new();
        let wire =
            json::FileWire::from_file(file_with_content(), JSON_VERSION, &cancellation_token)
                .expect("serialize test file");
        let json = serde_json::to_value(wire).expect("serialize test file");

        assert!(json["tree"]["map"][3]["value"].get("children").is_none());
        assert!(json["tree"]["map"][3]["value"].get("metadata").is_none());
        assert_eq!(
            json["tree"]["map"][1]["value"]["payload"],
            json!({ "static": "root" }),
        );
        assert_eq!(
            json["tree"]["map"][2]["value"]["payload"]["action"]["kind"],
            "code",
        );
        assert_eq!(
            json["tree"]["map"][3]["value"]["payload"],
            json!({ "static": { "branch": { "named": "next" } } }),
        );
    }

    #[tokio::test]
    async fn write_samples_for_inspection() {
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
        let file = file_with_content();
        let (task_tracker, cancellation_token) = task_context();

        file.write_binary(
            &target.join("file-format-sample.actiona"),
            &task_tracker,
            &cancellation_token,
        )
        .await
        .expect("write binary sample");
        file.write_json(
            &target.join("file-format-sample.json"),
            &task_tracker,
            &cancellation_token,
        )
        .await
        .expect("write JSON sample");
    }
}
