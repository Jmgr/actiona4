#![allow(clippy::unwrap_used, clippy::expect_used)]

use actiona_core::{
    api::{
        displays::Displays,
        screen::{AskScreenshotOptions, Screen},
        windows::Windows,
    },
    runtime::Runtime,
};
use askama::Template;
use libtest_mimic_collect::libtest_mimic::Arguments;

extern crate libtest_mimic_collect;

macro_rules! ignored_ui_test {
    ($name:ident, $body:expr) => {
        fn $name() {
            $body()
        }

        const _: () = {
            #[libtest_mimic_collect::ctor]
            fn __add_test() {
                use libtest_mimic_collect::ConvertResult;

                let trial =
                    libtest_mimic_collect::libtest_mimic::Trial::test(stringify!($name), || {
                        libtest_mimic_collect::TestCollection::convert_result($name())
                    })
                    .with_ignored_flag(true);

                libtest_mimic_collect::TestCollection::add_test(trial);
            }
        };
    };
}

#[derive(Template)]
#[template(path = "image.html")]
struct ImagePage<'a> {
    title: &'a str,
    alt: &'a str,
    data_url: &'a str,
}

/*
fn dynamic_image_to_data_url(img: &DynamicImage) -> Result<String> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&buf);
    Ok(format!("data:image/png;base64,{}", b64))
}
*/

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

ignored_ui_test!(test_ask_screenshot, || {
    Runtime::test_with_ui(|runtime, _| async move {
        let screen = Screen::new(
            runtime.clone(),
            Displays::new(runtime.cancellation_token(), runtime.task_tracker()).unwrap(),
            Windows::new(runtime.clone()),
        )
        .await
        .unwrap();

        let image = screen
            .ask_screenshot(AskScreenshotOptions {
                method: actiona_core::api::screen::AskScreenshotMethod::Overlay,
            })
            .await
            .unwrap();

        println!("result: {:?}", image.map(|i| i.size()));
    });
});

// Run with cargo test -p core --test ui -- --ignored test_success
ignored_ui_test!(test_success, || {
    Runtime::test_with_ui(|_, scripting_engine| async move {
        scripting_engine
            .eval_async::<()>(r#"sleep(100000)"#)
            .await
            .unwrap();
    });
});

pub fn main() {
    // When Snipping Tool (or similar) re-launches this binary as a deep-link
    // handler, forward the URI to the running first instance and exit.
    #[cfg(windows)]
    if Runtime::relay_deep_link_if_needed() {
        return;
    }

    let mut args = Arguments::from_args();
    args.test_threads = Some(1);
    libtest_mimic_collect::TestCollection::run_with_args(args);
}
