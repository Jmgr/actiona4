use std::{path::PathBuf, str::FromStr};

use indexmap::IndexMap;
use macros::FromJsObject;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use rquickjs::{
    Ctx, JsLifetime, Promise, Result,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::sync::watch;

use crate::{
    IntoJsResult,
    core::{
        convert_watch_receiver,
        image::js::JsImage,
        js::{
            abort_controller::JsAbortSignal,
            classes::{SingletonClass, ValueClass},
            duration::JsDuration,
            task::{IsDone, progress_task_with_token},
        },
        web::{Progress, WebOptions},
    },
    error::CommonError,
};

pub type JsMethod = super::Method;

// TODO: Options
/// Multipart form
#[derive(Clone, Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "MultipartForm")]
pub struct JsMultipartForm {
    //inner: MultipartForm,
}

impl<'js> Trace<'js> for JsMultipartForm {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl ValueClass<'_> for JsMultipartForm {}

/// Web options
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsWebOptions {
    /// @default undefined
    pub signal: Option<JsAbortSignal>,

    /// @default undefined
    pub user_name: Option<String>,

    /// @default undefined
    pub password: Option<String>,

    /// @type Record<string, string>
    /// @default undefined
    pub headers: Option<IndexMap<String, String>>,

    /// @default Method.GET
    pub method: JsMethod,

    /// @default undefined
    pub timeout: Option<JsDuration>,

    /// Sets the content-type header.
    /// Overrides any content-type set by other fields.
    ///
    /// @default undefined
    pub content_type: Option<String>,

    /// Form data as strings.
    /// Sets content-type to "application/x-www-form-urlencoded".
    ///
    /// @type Record<string, string>
    /// @default undefined
    pub form: Option<IndexMap<String, String>>,

    /// Additional query parameters.
    ///
    /// @type Record<string, string>
    /// @default undefined
    pub query: Option<IndexMap<String, String>>,

    /// Form multipart data.
    /// Sets content-type and content-length appropriately.
    ///
    /// @default undefined
    pub multipart: Option<JsMultipartForm>,
}

impl JsWebOptions {
    fn into_super(self, ctx: &Ctx<'_>) -> Result<WebOptions> {
        let headers = if let Some(headers) = self.headers {
            Some(
                headers
                    .into_iter()
                    .map(|(key, value)| {
                        Ok((
                            HeaderName::from_str(&key).map_err(|_| CommonError::Unexpected)?, // TODO: error
                            HeaderValue::from_str(&value).map_err(|_| CommonError::Unexpected)?,
                        ))
                    })
                    .collect::<std::result::Result<HeaderMap<_>, CommonError>>()
                    .into_js(&ctx)?,
            )
        } else {
            None
        };

        todo!();

        /*

        Ok(WebOptions {
            user_name: self.user_name,
            password: self.password,
            headers,
            method: self.method,
            progress: None,
            timeout: self.timeout.map(|timeout| timeout.into()),
            content_type: self.content_type,
            form: self.form,
            query: self.query,
            multipart: self.multipart.map(|multipart| multipart.inner),
        })
        */
    }
}

#[derive(Debug, Clone, Copy, JsLifetime, Trace, Default, PartialEq, Eq)]
#[rquickjs::class(rename = "WebProgress")]
pub struct JsWebProgress {
    total: u64,
    current: u64,
    finished: bool,
}

impl ValueClass<'_> for JsWebProgress {}

impl IsDone for JsWebProgress {
    fn is_done(&self) -> bool {
        self.finished
    }
}

impl From<Progress> for JsWebProgress {
    fn from(value: Progress) -> Self {
        Self {
            total: 0,       //value.total().unwrap_or_default(),
            current: 0,     //value.current(),
            finished: true, //value.is_finished(),
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWebProgress {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    #[qjs(get)]
    pub fn total(&self) -> u64 {
        self.total
    }

    #[qjs(get)]
    pub fn current(&self) -> u64 {
        self.current
    }

    #[qjs(get)]
    pub fn finished(&self) -> bool {
        self.finished
    }
}

/// @singleton
#[derive(JsLifetime)]
#[rquickjs::class(rename = "Web")]
pub struct JsWeb {
    inner: super::Web,
}

impl SingletonClass<'_> for JsWeb {
    fn register_dependencies(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        //JsMultipartForm::register(&ctx)?;
        JsMethod::register(&ctx)?;

        Ok(())
    }
}

impl<'js> Trace<'js> for JsWeb {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl Default for JsWeb {
    fn default() -> Self {
        Self {
            inner: super::Web::default(),
        }
    }
}

impl JsWeb {
    fn make_progress_receiver<'js>(
        ctx: &Ctx<'js>,
        options: &mut WebOptions,
    ) -> Result<watch::Receiver<JsWebProgress>> {
        let (progress_sender, progress_receiver) = watch::channel(Progress::Inactive);
        options.progress = Some(progress_sender);

        Ok(convert_watch_receiver(&ctx, progress_receiver))
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWeb {
    /// Downloads a binary file.
    ///
    /// @returns ProgressTask<Uint8Array, WebProgress>
    pub fn download<'js>(
        &self,
        ctx: Ctx<'js>,
        url: String,
        options: Opt<JsWebOptions>,
    ) -> Result<Promise<'js>> {
        let local_inner = self.inner.clone();
        let options = options.0.unwrap_or_default();
        let mut local_options = options.clone().into_super(&ctx)?;
        let progress_receiver = Self::make_progress_receiver(&ctx, &mut local_options)?;

        progress_task_with_token(
            ctx,
            options.signal,
            progress_receiver,
            async move |ctx, token| {
                let result = local_inner
                    .download(&url, token, Some(local_options))
                    .await
                    .into_js(&ctx)?;
                Ok(result)
            },
        )
    }

    /// Downloads a text file.
    ///
    /// @returns ProgressTask<string, WebProgress>
    pub fn download_text<'js>(
        &self,
        ctx: Ctx<'js>,
        url: String,
        options: Opt<JsWebOptions>,
    ) -> Result<Promise<'js>> {
        let local_inner = self.inner.clone();
        let options = options.0.unwrap_or_default();
        let mut local_options = options.clone().into_super(&ctx)?;
        let progress_receiver = Self::make_progress_receiver(&ctx, &mut local_options)?;

        progress_task_with_token(
            ctx,
            options.signal,
            progress_receiver,
            async move |ctx, token| {
                local_inner
                    .download_text(&url, token, Some(local_options))
                    .await
                    .into_js(&ctx)
            },
        )
    }

    /// Downloads an image.
    ///
    /// @returns ProgressTask<Image, WebProgress>
    pub fn download_image<'js>(
        &self,
        ctx: Ctx<'js>,
        url: String,
        options: Opt<JsWebOptions>,
    ) -> Result<Promise<'js>> {
        let local_inner = self.inner.clone();
        let options = options.0.unwrap_or_default();
        let mut local_options = options.clone().into_super(&ctx)?;
        let progress_receiver = Self::make_progress_receiver(&ctx, &mut local_options)?;

        progress_task_with_token(
            ctx,
            options.signal,
            progress_receiver,
            async move |ctx, token| {
                local_inner
                    .download_image(&url, token, Some(local_options))
                    .await
                    .map(|image| JsImage::from(image))
                    .into_js(&ctx)
            },
        )
    }

    /// Downloads a file to a directory.
    ///
    /// @returns ProgressTask<string, WebProgress>
    pub fn download_file<'js>(
        &self,
        ctx: Ctx<'js>,
        url: String,
        directory: Opt<String>,
        options: Opt<JsWebOptions>,
    ) -> Result<Promise<'js>> {
        let local_inner = self.inner.clone();
        let options = options.0.unwrap_or_default();
        let mut local_options = options.clone().into_super(&ctx)?;
        let directory = directory.as_deref().map(PathBuf::from);
        let progress_receiver = Self::make_progress_receiver(&ctx, &mut local_options)?;

        progress_task_with_token(
            ctx,
            options.signal,
            progress_receiver,
            async move |ctx, token| {
                local_inner
                    .download_file(&url, token, directory.as_deref(), Some(local_options))
                    .await
                    .into_js(&ctx)
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    use httptest::{
        Expectation, Server, all_of,
        matchers::{contains, request},
        responders::status_code,
    };

    use super::*;
    use crate::{core::web::helper::TestImage, runtime::Runtime};

    #[test]
    fn test_download_text() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let server = Server::run();

            server.expect(
                Expectation::matching(request::method_path("GET", "/foo"))
                    .respond_with(status_code(200).body("hello")),
            );

            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"await web.downloadText("{}")"#,
                    server.url("/foo")
                ))
                .await
                .unwrap();

            assert_eq!(result, "hello");
        });
    }

    #[test]
    fn test_download_image() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let server = Server::run();

            let test_image = TestImage::default();

            server.expect(
                Expectation::matching(request::method_path("GET", "/image.png")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "image/png")
                        .body(test_image.bytes),
                ),
            );

            let result = script_engine
                .eval_async::<JsImage>(&format!(
                    r#"await web.downloadImage("{}")"#,
                    server.url("/image.png")
                ))
                .await
                .unwrap();

            assert_eq!(result.into_inner().into_inner(), test_image.image);
        });
    }

    #[test]
    fn test_download_binary() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let server = Server::run();

            let test_image = TestImage::default();

            server.expect(
                Expectation::matching(request::method_path("GET", "/binary")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "application/octet-stream")
                        .body(test_image.bytes.clone()),
                ),
            );

            let result = script_engine
                .eval_async::<Vec<u8>>(&format!(
                    r#"await web.download("{}")"#,
                    server.url("/binary")
                ))
                .await
                .unwrap();

            assert_eq!(result, test_image.bytes);
        });
    }

    #[test]
    fn test_download_basic_auth() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let server = Server::run();

            let auth_header = format!("Basic {}", BASE64.encode("user:password"));

            server.expect(
                Expectation::matching(all_of![
                    request::method_path("POST", "/foo"),
                    request::headers(contains(("authorization", auth_header)))
                ])
                .respond_with(status_code(200).body("hello")),
            );

            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"await web.downloadText("{}", {{
                        userName: "user",
                        password: "password",
                        method: Method.POST,
                    }})"#,
                    server.url("/foo")
                ))
                .await
                .unwrap();

            assert_eq!(result, "hello");
        });
    }

    #[test]
    fn test_download_progress() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let server = Server::run();

            let test_image = TestImage::default();
            let total_size = test_image.bytes.len() as u64;

            server.expect(
                Expectation::matching(request::method_path("GET", "/binary")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "application/octet-stream")
                        .body(test_image.bytes.clone()),
                ),
            );

            let received_progress = script_engine
                .eval_async::<Vec<JsWebProgress>>(&format!(
                    r#"
                    const download = web.download("{}");
                    let result = [];
                    for await (const progress of download) {{
                        result.push(progress);
                    }}
                    await download;
                    result
                    "#,
                    server.url("/binary")
                ))
                .await
                .unwrap();

            assert!(received_progress.contains(&JsWebProgress {
                total: 0,
                current: 0,
                finished: false,
            }));
            assert!(received_progress.contains(&JsWebProgress {
                total: total_size,
                current: total_size,
                finished: true,
            }));
        });
    }

    #[test]
    fn test_download_file() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let server = Server::run();

            let test_image = TestImage::default();

            let directory = std::env::temp_dir();

            server.expect(
                Expectation::matching(request::method_path("GET", "/image")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "image/png")
                        .append_header(
                            "Content-Disposition",
                            r#"attachment; filename="example.png""#,
                        )
                        .body(test_image.bytes.clone()),
                ),
            );

            let filepath = script_engine
                .eval_async::<String>(&format!(
                    r#"await web.downloadFile("{}")"#,
                    server.url("/image")
                ))
                .await
                .unwrap();

            let expected_filepath = directory.join("example.png");
            assert_eq!(filepath, expected_filepath.as_os_str().to_string_lossy());

            let bytes = fs::read(expected_filepath).unwrap();
            assert_eq!(bytes, test_image.bytes);
        });
    }
}
