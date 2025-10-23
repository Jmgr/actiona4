use std::{io::Cursor, time::Duration};

use actiona_ng::{core::ui::js::JsMessageBoxResult, runtime::Runtime};
use askama::Template;
use base64::Engine;
use eyre::Result;
use image::{DynamicImage, ImageFormat};
use libtest_mimic_collect::libtest_mimic::Arguments;
use tauri::{WebviewUrl, WebviewWindowBuilder};
use tokio::time::sleep;

#[macro_use]
extern crate libtest_mimic_collect;

#[derive(Template)]
#[template(path = "image.html")]
struct ImagePage<'a> {
    title: &'a str,
    alt: &'a str,
    data_url: &'a str,
}

fn dynamic_image_to_data_url(img: &DynamicImage) -> Result<String> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(format!("data:image/png;base64,{}", b64))
}

/*
#[test]
fn test_success() {
    Runtime::test_with_ui(|runtime, _| async move {
        let tauri = runtime.tauri_app();

        let img = image::load_from_memory(include_bytes!("../icons/icon.png")).unwrap();

        // 1) Build the HTML via Askama
        let data_url = dynamic_image_to_data_url(&img).unwrap();
        let html = ImagePage {
            title: "Preview",
            alt: "Image",
            data_url: &data_url,
        }
        .render()
        .unwrap(); // compile-time checked, no runtime file reads

        // 2) Spawn a minimal webview
        let win = WebviewWindowBuilder::new(
            tauri,
            "image_view",
            // Any local asset will do; we'll replace its content immediately:
            WebviewUrl::App("blank.html".into()),
        )
        .decorations(true)
        .transparent(true)
        .resizable(true)
        .always_on_top(true)
        .center()
        .build()
        .unwrap();

        // 3) Replace the whole document with our rendered HTML
        // Use JSON to safely quote the string for JS
        let html_js = serde_json::to_string(&html).unwrap();
        win.eval(format!(
            "document.open();document.write({html_js});document.close();"
        ))
        .unwrap();

        win.show().unwrap();

        sleep(Duration::from_secs(100)).await;
    });
}
    */
#[test]
fn test_success() {
    Runtime::test_with_ui(|_, scripting_engine| async move {
        let result = scripting_engine
            .eval_async::<JsMessageBoxResult>(
                r#"
            let button = await MessageBox.show(
                "Hello",
                "Some title",
                MessageBoxButtons.yesNoCancelCustom("yeah", "nope", "arg"),
                MessageBoxIcon.Error
                );
            button
        "#,
            )
            .await
            .unwrap();
        println!("result: {result}");
    });
}

pub fn main() {
    let mut args = Arguments::from_args();
    args.test_threads = Some(1);
    libtest_mimic_collect::TestCollection::run_with_args(args);
}
