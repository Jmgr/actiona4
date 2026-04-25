use std::sync::Arc;

use actiona_core::api::image::{
    Image,
    find_image::{FindImageTemplateOptions, Source, Template},
};
use std::{hint::black_box, time::Duration};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

static SOURCE_BYTES: &[u8] = include_bytes!("../test-data/input.png");
static SOURCE_BYTES_2X: &[u8] = include_bytes!("../test-data/input_2x.png");
static TEMPLATE_BYTES: &[u8] = include_bytes!("../test-data/Crown_icon_transparent.png");
static TEMPLATE_BYTES_2X: &[u8] = include_bytes!("../test-data/Crown_icon_transparent_2x.png");

fn make_source(bytes: &[u8]) -> Arc<Source> {
    let image = Image::from_bytes(bytes).unwrap();
    Arc::<Source>::try_from(&image).unwrap()
}

fn make_template(bytes: &[u8]) -> Arc<Template> {
    let image = Image::from_bytes(bytes).unwrap();
    Arc::<Template>::try_from(&image).unwrap()
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
                        black_box(
                            source
                                .find_template_all(
                                    &template,
                                    opts.clone(),
                                    CancellationToken::new(),
                                    tx,
                                )
                                .unwrap(),
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
                        let mut gpu_opts = opts.clone();
                        gpu_opts.enable_gpu = true;
                        black_box(
                            source
                                .find_template_all(
                                    &template,
                                    gpu_opts,
                                    CancellationToken::new(),
                                    tx,
                                )
                                .unwrap(),
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
                        black_box(
                            source
                                .find_template(
                                    &template,
                                    opts.clone(),
                                    CancellationToken::new(),
                                    tx,
                                )
                                .unwrap(),
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
                        let mut gpu_opts = opts.clone();
                        gpu_opts.enable_gpu = true;
                        black_box(
                            source
                                .find_template(
                                    &template,
                                    gpu_opts,
                                    CancellationToken::new(),
                                    tx,
                                )
                                .unwrap(),
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
        let source_image = Image::from_bytes(source_bytes).unwrap();
        let template_image = Image::from_bytes(template_bytes).unwrap();

        group.bench_with_input(
            BenchmarkId::new("prepare_source", dataset_name),
            &source_image,
            |b, source_image| {
                b.iter(|| black_box(Arc::<Source>::try_from(black_box(source_image)).unwrap()));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("prepare_template", dataset_name),
            &template_image,
            |b, template_image| {
                b.iter(|| black_box(Arc::<Template>::try_from(black_box(template_image)).unwrap()));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, find_image_benches, prepare_benches);
criterion_main!(benches);
