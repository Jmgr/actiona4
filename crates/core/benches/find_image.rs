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
static TEMPLATE_BYTES: &[u8] = include_bytes!("../test-data/Crown_icon_transparent.png");

fn make_source() -> Arc<Source> {
    let image = Image::from_bytes(SOURCE_BYTES).unwrap();
    Arc::<Source>::try_from(&image).unwrap()
}

fn make_template() -> Arc<Template> {
    let image = Image::from_bytes(TEMPLATE_BYTES).unwrap();
    Arc::<Template>::try_from(&image).unwrap()
}

fn find_image_benches(c: &mut Criterion) {
    let source = make_source();
    let template = make_template();

    let options_variants: &[(&str, FindImageTemplateOptions)] = &[
        (
            "colors+transparency",
            FindImageTemplateOptions {
                use_colors: true,
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
                use_transparency: true,
                match_threshold: 0.8,
                non_maximum_suppression_radius: Some(10),
                downscale: 2,
            },
        ),
    ];

    let mut group = c.benchmark_group("find_image");
    group.measurement_time(Duration::from_secs(15));

    for (name, opts) in options_variants {
        group.bench_with_input(
            BenchmarkId::new("find_template_all", name),
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
            BenchmarkId::new("find_template", name),
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
    }

    group.finish();
}

fn prepare_benches(c: &mut Criterion) {
    let source_image = Image::from_bytes(SOURCE_BYTES).unwrap();
    let template_image = Image::from_bytes(TEMPLATE_BYTES).unwrap();

    let mut group = c.benchmark_group("find_image_prepare");

    group.bench_function("prepare_source", |b| {
        b.iter(|| black_box(Arc::<Source>::try_from(black_box(&source_image)).unwrap()));
    });

    group.bench_function("prepare_template", |b| {
        b.iter(|| black_box(Arc::<Template>::try_from(black_box(&template_image)).unwrap()));
    });

    group.finish();
}

criterion_group!(benches, find_image_benches, prepare_benches);
criterion_main!(benches);
