use std::{hint::black_box, io, time::Duration};

use bytes::Bytes;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use file_format::database::{Database, ReadWriteDatabase};
use futures::{StreamExt, stream};
use tempfile::TempDir;
use tokio::runtime::{Builder, Runtime};

const CHUNK_SIZES: &[usize] = &[
    16 * 1024,
    64 * 1024,
    256 * 1024,
    1024 * 1024,
    4 * 1024 * 1024,
];
const PAYLOAD_SIZES: &[(&str, usize)] = &[
    ("1KiB", 1024),
    ("1MiB", 1024 * 1024),
    ("32MiB", 32 * 1024 * 1024),
];

struct Fixture {
    _directory: TempDir,
    database: ReadWriteDatabase,
}

impl Fixture {
    fn new(max_chunk_size: usize) -> Self {
        let directory = tempfile::tempdir().expect("create benchmark directory");
        let database = Database::create_with_max_chunk_size(
            &directory.path().join("attachment.sqlite"),
            max_chunk_size,
        )
        .expect("create benchmark database");

        Self {
            _directory: directory,
            database,
        }
    }
}

fn write_attachment(runtime: &Runtime, database: &ReadWriteDatabase, payload: Bytes) -> uuid::Uuid {
    runtime
        .block_on(database.write_attachment(Box::pin(stream::iter([Ok::<_, io::Error>(payload)]))))
        .expect("write attachment")
}

fn read_attachment(
    runtime: &Runtime,
    database: &ReadWriteDatabase,
    attachment_id: uuid::Uuid,
) -> usize {
    runtime.block_on(async {
        database
            .read_attachment(attachment_id)
            .fold(0, |size, chunk| async move {
                size + black_box(chunk.expect("read attachment chunk")).len()
            })
            .await
    })
}

fn make_payload(size: usize) -> Bytes {
    Bytes::from(
        (0..size)
            .map(|index| (index % 251) as u8)
            .collect::<Vec<_>>(),
    )
}

fn attachment_chunk_benches(criterion: &mut Criterion) {
    let runtime = Builder::new_current_thread()
        .build()
        .expect("create Tokio runtime");
    let payloads = PAYLOAD_SIZES
        .iter()
        .map(|(name, size)| (*name, make_payload(*size)))
        .collect::<Vec<_>>();
    let mut write_group = criterion.benchmark_group("attachment_chunk_write");
    write_group.sample_size(10);
    write_group.measurement_time(Duration::from_secs(10));

    for (payload_name, payload) in &payloads {
        write_group.throughput(Throughput::Bytes(payload.len() as u64));
        for &chunk_size in CHUNK_SIZES {
            write_group.bench_with_input(
                BenchmarkId::new("write", format!("{payload_name}/{}KiB", chunk_size / 1024)),
                &(payload, chunk_size),
                |bench, (payload, chunk_size)| {
                    bench.iter_batched(
                        || Fixture::new(*chunk_size),
                        |fixture| {
                            black_box(write_attachment(
                                &runtime,
                                &fixture.database,
                                (*payload).clone(),
                            ));
                        },
                        BatchSize::PerIteration,
                    );
                },
            );
        }
    }
    write_group.finish();

    let mut read_group = criterion.benchmark_group("attachment_chunk_read");
    read_group.sample_size(10);
    read_group.measurement_time(Duration::from_secs(10));

    for (payload_name, payload) in &payloads {
        read_group.throughput(Throughput::Bytes(payload.len() as u64));
        for &chunk_size in CHUNK_SIZES {
            read_group.bench_with_input(
                BenchmarkId::new("read", format!("{payload_name}/{}KiB", chunk_size / 1024)),
                &(payload, chunk_size),
                |bench, (payload, chunk_size)| {
                    bench.iter_batched(
                        || {
                            let fixture = Fixture::new(*chunk_size);
                            let attachment_id =
                                write_attachment(&runtime, &fixture.database, (*payload).clone());
                            (fixture, attachment_id)
                        },
                        |(fixture, attachment_id)| {
                            black_box(read_attachment(&runtime, &fixture.database, attachment_id))
                        },
                        BatchSize::PerIteration,
                    );
                },
            );
        }
    }
    read_group.finish();
}

criterion_group!(benches, attachment_chunk_benches);
criterion_main!(benches);
