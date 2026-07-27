// Benchmark fixtures are real-world-sized screenshots; the lint's default 1 MB cap doesn't apply here.
#![allow(clippy::large_include_file)]

use std::{hint::black_box, sync::Arc, time::Duration};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use extension::protocols::opencv::FindImageTemplateOptions;
use extension_opencv::find_image::{Source, Template};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use types::{Size, size};

static SOURCE_BYTES: &[u8] = include_bytes!("../../../core/test-data/input.png");
static SOURCE_BYTES_2X: &[u8] = include_bytes!("../../../core/test-data/input_2x.png");
static TEMPLATE_BYTES: &[u8] = include_bytes!("../../../core/test-data/Crown_icon_transparent.png");
static TEMPLATE_BYTES_2X: &[u8] =
    include_bytes!("../../../core/test-data/Crown_icon_transparent_2x.png");

/// Decode a PNG fixture into (RGBA8 bytes, size), matching what the host
/// uploads over IPC.
fn rgba(bytes: &[u8]) -> (Vec<u8>, Size) {
    let image = image::load_from_memory(bytes)
        .expect("benchmark fixture is a valid image")
        .into_rgba8();
    let dimensions = size(image.width(), image.height());
    (image.into_raw(), dimensions)
}

fn make_source(bytes: &[u8]) -> Arc<Source> {
    let (pixels, dimensions) = rgba(bytes);
    Source::from_rgba(&pixels, dimensions).expect("benchmark source image is valid")
}

fn make_template(bytes: &[u8]) -> Arc<Template> {
    let (pixels, dimensions) = rgba(bytes);
    Template::from_rgba(&pixels, dimensions).expect("benchmark template image is valid")
}

fn find_image_benches(c: &mut Criterion) {
    let datasets: &[(&str, &[u8], &[u8])] = &[
        ("original", SOURCE_BYTES, TEMPLATE_BYTES),
        ("scaled_2x", SOURCE_BYTES_2X, TEMPLATE_BYTES_2X),
    ];

    let options_variants: &[(&str, FindImageTemplateOptions)] = &[
        (
            "colors+transparency",
            FindImageTemplateOptions {
                use_colors: true,
                enable_gpu: false,
                use_transparency: true,
                match_threshold: 0.8,
                non_maximum_suppression_radius: Some(10),
                downscale: 0,
            },
        ),
        (
            "no_colors",
            FindImageTemplateOptions {
                use_colors: false,
                enable_gpu: false,
                use_transparency: true,
                match_threshold: 0.8,
                non_maximum_suppression_radius: Some(10),
                downscale: 0,
            },
        ),
        (
            "downscale_2",
            FindImageTemplateOptions {
                use_colors: true,
                enable_gpu: false,
                use_transparency: true,
                match_threshold: 0.8,
                non_maximum_suppression_radius: Some(10),
                downscale: 2,
            },
        ),
    ];

    let mut group = c.benchmark_group("find_image");
    group.measurement_time(Duration::from_secs(15));

    for (dataset_name, source_bytes, template_bytes) in datasets {
        let source = make_source(source_bytes);
        let template = make_template(template_bytes);

        for (name, opts) in options_variants {
            group.bench_with_input(
                BenchmarkId::new("find_template_all", format!("{dataset_name}/{name}")),
                opts,
                |b, opts| {
                    b.iter(|| {
                        let (tx, _rx) = mpsc::unbounded_channel();
                        let cancellation_token = CancellationToken::new();
                        black_box(
                            source
                                .find_template_all(&template, *opts, &cancellation_token, &tx)
                                .expect("find-image benchmark succeeds"),
                        )
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new("find_template_all_gpu", format!("{dataset_name}/{name}")),
                opts,
                |b, opts| {
                    b.iter(|| {
                        let (tx, _rx) = mpsc::unbounded_channel();
                        let cancellation_token = CancellationToken::new();
                        let mut gpu_opts = *opts;
                        gpu_opts.enable_gpu = true;
                        black_box(
                            source
                                .find_template_all(&template, gpu_opts, &cancellation_token, &tx)
                                .expect("GPU find-image benchmark succeeds"),
                        )
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new("find_template", format!("{dataset_name}/{name}")),
                opts,
                |b, opts| {
                    b.iter(|| {
                        let (tx, _rx) = mpsc::unbounded_channel();
                        let cancellation_token = CancellationToken::new();
                        black_box(
                            source
                                .find_template(&template, *opts, &cancellation_token, &tx)
                                .expect("find-image benchmark succeeds"),
                        )
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new("find_template_gpu", format!("{dataset_name}/{name}")),
                opts,
                |b, opts| {
                    b.iter(|| {
                        let (tx, _rx) = mpsc::unbounded_channel();
                        let cancellation_token = CancellationToken::new();
                        let mut gpu_opts = *opts;
                        gpu_opts.enable_gpu = true;
                        black_box(
                            source
                                .find_template(&template, gpu_opts, &cancellation_token, &tx)
                                .expect("GPU find-image benchmark succeeds"),
                        )
                    });
                },
            );
        }
    }

    group.finish();
}

fn prepare_benches(c: &mut Criterion) {
    let datasets: &[(&str, &[u8], &[u8])] = &[
        ("original", SOURCE_BYTES, TEMPLATE_BYTES),
        ("scaled_2x", SOURCE_BYTES_2X, TEMPLATE_BYTES_2X),
    ];

    let mut group = c.benchmark_group("find_image_prepare");

    for (dataset_name, source_bytes, template_bytes) in datasets {
        let source_pixels = rgba(source_bytes);
        let template_pixels = rgba(template_bytes);

        group.bench_with_input(
            BenchmarkId::new("prepare_source", dataset_name),
            &source_pixels,
            |b, (pixels, dimensions)| {
                b.iter(|| {
                    black_box(
                        Source::from_rgba(black_box(pixels), *dimensions)
                            .expect("benchmark source image has source-compatible format"),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("prepare_template", dataset_name),
            &template_pixels,
            |b, (pixels, dimensions)| {
                b.iter(|| {
                    black_box(
                        Template::from_rgba(black_box(pixels), *dimensions)
                            .expect("benchmark template image has template-compatible format"),
                    )
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, find_image_benches, prepare_benches);
criterion_main!(benches);
