use std::{path::PathBuf, str::FromStr};

use bytes::Bytes;
use indexmap::IndexMap;
use macros::FromJsObject;
use mime::Mime;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use rquickjs::{
    Ctx, Exception, JsLifetime, Promise, Result, TypedArray,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::sync::watch;
use tokio_util::task::TaskTracker;

use crate::{
    IntoJsResult,
    core::{
        image::js::JsImage,
        js::{
            abort_controller::JsAbortSignal,
            classes::{
                HostClass, SingletonClass, ValueClass, register_enum, register_host_class,
                register_value_class,
            },
            duration::JsDuration,
            task::{IsDone, progress_task_with_token},
        },
        web::{Body, FormField, MultipartField, MultipartValue, Progress, WebOptions},
    },
    error::CommonError,
};

pub type JsMethod = super::Method;

/// Multipart form
#[derive(Clone, Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "MultipartForm")]
pub struct JsMultipartForm {
    fields: Vec<MultipartField>,
}

impl<'js> Trace<'js> for JsMultipartForm {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl ValueClass<'_> for JsMultipartForm {}

impl JsMultipartForm {
    fn parse_mimetype(ctx: &Ctx<'_>, mimetype: Opt<String>) -> Result<Option<Mime>> {
        if let Some(mimetype) = mimetype.0 {
            let parsed = mimetype.parse::<Mime>().map_err(|err| {
                Exception::throw_message(ctx, &format!("Invalid mime type '{mimetype}': {err}"))
            })?;
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMultipartForm {
    /// @constructor
    #[qjs(constructor)]
    #[must_use]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Adds a text field.
    pub fn add_text(
        &mut self,
        ctx: Ctx<'_>,
        name: String,
        value: String,
        filename: Opt<String>,
        mimetype: Opt<String>,
    ) -> Result<()> {
        let mimetype = Self::parse_mimetype(&ctx, mimetype)?;
        self.fields.push(MultipartField {
            name,
            value: MultipartValue::Text(value),
            filename: filename.0,
            mimetype,
        });
        Ok(())
    }

    /// Adds a file field.
    pub fn add_file(
        &mut self,
        ctx: Ctx<'_>,
        name: String,
        path: String,
        filename: Opt<String>,
        mimetype: Opt<String>,
    ) -> Result<()> {
        let mimetype = Self::parse_mimetype(&ctx, mimetype)?;
        let path_buf = PathBuf::from(&path);
        let filename = filename.0.or_else(|| {
            path_buf
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        });
        self.fields.push(MultipartField {
            name,
            value: MultipartValue::File(path_buf),
            filename,
            mimetype,
        });
        Ok(())
    }

    /// Adds a byte field.
    pub fn add_bytes<'js>(
        &mut self,
        ctx: Ctx<'js>,
        name: String,
        bytes: TypedArray<'js, u8>,
        filename: Opt<String>,
        mimetype: Opt<String>,
    ) -> Result<()> {
        let bytes = bytes
            .as_bytes()
            .ok_or(CommonError::DetachedArrayBuffer)
            .into_js_result(&ctx)?;
        let mimetype = Self::parse_mimetype(&ctx, mimetype)?;
        self.fields.push(MultipartField {
            name,
            value: MultipartValue::Bytes(Bytes::copy_from_slice(bytes)),
            filename: filename.0,
            mimetype,
        });
        Ok(())
    }
}

/// Web options
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsWebOptions {
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,

    /// @default `undefined`
    pub user_name: Option<String>,

    /// @default `undefined`
    pub password: Option<String>,

    /// @default `undefined`
    pub headers: Option<IndexMap<String, String>>,

    /// @default `Method.Get`
    pub method: JsMethod,

    /// @default `undefined`
    pub timeout: Option<JsDuration>,

    /// Sets the content-type header.
    /// Overrides any content-type set by other fields.
    ///
    /// @default `undefined`
    pub content_type: Option<String>,

    /// Form data as strings.
    /// Sets content-type to "application/x-www-form-urlencoded".
    ///
    /// @default `undefined`
    pub form: Option<IndexMap<String, String>>,

    /// Additional query parameters.
    ///
    /// @default `undefined`
    pub query: Option<IndexMap<String, String>>,

    /// Form multipart data.
    /// Sets content-type and content-length appropriately.
    ///
    /// @default `undefined`
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
                    .into_js_result(ctx)?,
            )
        } else {
            None
        };

        let mut headers = headers.unwrap_or_default();
        if let Some(content_type) = self.content_type {
            let header_value = HeaderValue::from_str(&content_type)
                .map_err(|_| CommonError::Unexpected)
                .into_js_result(ctx)?;
            headers.insert(CONTENT_TYPE, header_value);
        }

        let request_body = match (self.form, self.multipart) {
            (Some(_), Some(_)) => {
                return Err(CommonError::Unsupported(
                    "Cannot use both form and multipart".to_string(),
                ))
                .into_js_result(ctx);
            }
            (Some(form), None) => Body::Form(
                form.into_iter()
                    .map(|(name, value)| FormField { name, value })
                    .collect(),
            ),
            (None, Some(multipart)) => Body::Multipart(multipart.fields),
            (None, None) => Body::None,
        };

        Ok(WebOptions {
            user_name: self.user_name,
            password: self.password,
            headers,
            method: self.method,
            progress: None,
            timeout: self.timeout.map(|timeout| timeout.into()),
            query: self.query.unwrap_or_default(),
            request_body,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "WebProgress")]
pub struct JsWebProgress {
    total: u64,
    current: u64,
    finished: bool,
}

impl HostClass<'_> for JsWebProgress {}

impl IsDone for JsWebProgress {
    fn is_done(&self) -> bool {
        self.finished
    }
}

impl From<Progress> for JsWebProgress {
    fn from(value: Progress) -> Self {
        match value {
            Progress::Inactive => Self {
                total: 0,
                current: 0,
                finished: false,
            },
            Progress::Uploading { current, total } => Self {
                total,
                current,
                finished: false,
            },
            Progress::Downloading { current, total } => {
                let (total, finished) = match total {
                    Some(total) => (total, current >= total),
                    None => (0, false),
                };

                Self {
                    total,
                    current,
                    finished,
                }
            }
            Progress::Finished => Self {
                total: 0,
                current: 0,
                finished: true,
            },
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWebProgress {
    #[qjs(get)]
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.total
    }

    #[qjs(get)]
    #[must_use]
    pub const fn current(&self) -> u64 {
        self.current
    }

    #[qjs(get)]
    #[must_use]
    pub const fn finished(&self) -> bool {
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
        register_value_class::<JsMultipartForm>(ctx)?;
        register_host_class::<JsWebProgress>(ctx)?;
        register_enum::<JsMethod>(ctx)?;

        Ok(())
    }
}

impl<'js> Trace<'js> for JsWeb {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsWeb {
    /// @skip
    #[must_use]
    pub fn new(task_tracker: TaskTracker) -> Self {
        Self {
            inner: super::Web::new(task_tracker),
        }
    }

    fn make_progress_receiver(options: &mut WebOptions) -> watch::Receiver<Progress> {
        let (progress_sender, progress_receiver) = watch::channel(Progress::Inactive);
        options.progress = Some(progress_sender);

        progress_receiver
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsWeb {
    /// Downloads a binary file.
    ///
    /// ```ts
    /// const bytes = await web.download("https://example.com/file.bin");
    /// ```
    ///
    /// ```ts
    /// const task = web.download("https://example.com/file.bin");
    /// for await (const progress of task) {
    ///   console.log(`${progress.current}/${progress.total} bytes`);
    /// }
    /// const bytes = await task;
    /// ```
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
        let progress_receiver = Self::make_progress_receiver(&mut local_options);

        progress_task_with_token::<_, _, _, _, _, JsWebProgress>(
            ctx,
            options.signal,
            progress_receiver,
            async move |ctx, token| {
                let result = local_inner
                    .download(&url, token, Some(local_options))
                    .await
                    .into_js_result(&ctx)?;
                Ok(result)
            },
        )
    }

    /// Downloads a text file.
    ///
    /// ```ts
    /// const text = await web.downloadText("https://example.com/data.json");
    /// ```
    ///
    /// ```ts
    /// const task = web.downloadText("https://example.com/data.json");
    /// for await (const progress of task) {
    ///   console.log(`${progress.current}/${progress.total} bytes`);
    /// }
    /// const text = await task;
    /// ```
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
        let progress_receiver = Self::make_progress_receiver(&mut local_options);

        progress_task_with_token::<_, _, _, _, _, JsWebProgress>(
            ctx,
            options.signal,
            progress_receiver,
            async move |ctx, token| {
                local_inner
                    .download_text(&url, token, Some(local_options))
                    .await
                    .into_js_result(&ctx)
            },
        )
    }

    /// Downloads an image.
    ///
    /// ```ts
    /// const image = await web.downloadImage("https://example.com/photo.png");
    /// ```
    ///
    /// ```ts
    /// const task = web.downloadImage("https://example.com/photo.png");
    /// for await (const progress of task) {
    ///   console.log(`${progress.current}/${progress.total} bytes`);
    /// }
    /// const image = await task;
    /// ```
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
        let progress_receiver = Self::make_progress_receiver(&mut local_options);

        progress_task_with_token::<_, _, _, _, _, JsWebProgress>(
            ctx,
            options.signal,
            progress_receiver,
            async move |ctx, token| {
                local_inner
                    .download_image(&url, token, Some(local_options))
                    .await
                    .map(JsImage::from)
                    .into_js_result(&ctx)
            },
        )
    }

    /// Downloads a file to a directory.
    ///
    /// ```ts
    /// const filePath = await web.downloadFile("https://example.com/file.zip");
    /// ```
    ///
    /// ```ts
    /// const task = web.downloadFile("https://example.com/file.zip", "/tmp");
    /// for await (const progress of task) {
    ///   console.log(`${progress.current}/${progress.total} bytes`);
    /// }
    /// const filePath = await task;
    /// ```
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
        let progress_receiver = Self::make_progress_receiver(&mut local_options);

        progress_task_with_token::<_, _, _, _, _, JsWebProgress>(
            ctx,
            options.signal,
            progress_receiver,
            async move |ctx, token| {
                local_inner
                    .download_file(&url, token, directory.as_deref(), Some(local_options))
                    .await
                    .into_js_result(&ctx)
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
        matchers::{contains, matches, request},
        responders::status_code,
    };

    use super::*;
    use crate::{core::web::helper::TestImage, runtime::Runtime};

    #[test]
    fn test_download_text() {
        Runtime::test_with_script_engine(|script_engine| async move {
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
        Runtime::test_with_script_engine(|script_engine| async move {
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

            assert_eq!(result.into_inner().into_rgba8(), test_image.image);
        });
    }

    #[test]
    fn test_download_binary() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let server = Server::run();

            let test_image = TestImage::default();
            let total_size = test_image.bytes.len();

            server.expect(
                Expectation::matching(request::method_path("GET", "/binary")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "application/octet-stream")
                        .append_header("Content-Length", total_size.to_string())
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
        Runtime::test_with_script_engine(|script_engine| async move {
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
                        method: Method.Post,
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
        Runtime::test_with_script_engine(|script_engine| async move {
            let server = Server::run();

            let test_image = TestImage::default();
            let total_size = u64::try_from(test_image.bytes.len()).unwrap();

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
            assert!(!received_progress.is_empty());
            assert!(received_progress.iter().any(|progress| progress.finished));
            assert!(
                received_progress
                    .iter()
                    .filter(|progress| progress.total > 0)
                    .all(|progress| progress.total == total_size)
            );
        });
    }

    #[test]
    fn test_download_file() {
        Runtime::test_with_script_engine(|script_engine| async move {
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

    #[test]
    fn test_multipart_text_field() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let server = Server::run();

            server.expect(
                Expectation::matching(all_of![
                    request::method_path("POST", "/upload"),
                    request::headers(contains(("content-type", matches("multipart/form-data")))),
                    request::body(matches("name=\"title\"")),
                    request::body(matches("hello multipart")),
                ])
                .respond_with(status_code(200).body("ok")),
            );

            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                    const form = new MultipartForm();
                    form.addText("title", "hello multipart");
                    await web.downloadText("{}", {{
                        method: Method.Post,
                        multipart: form,
                    }});
                    "#,
                    server.url("/upload")
                ))
                .await
                .unwrap();

            assert_eq!(result, "ok");
        });
    }

    #[test]
    fn test_multipart_bytes_field() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let server = Server::run();

            server.expect(
                Expectation::matching(all_of![
                    request::method_path("POST", "/upload"),
                    request::headers(contains(("content-type", matches("multipart/form-data")))),
                    request::body(matches("name=\"payload\"")),
                    request::body(matches("hello-bytes")),
                ])
                .respond_with(status_code(200).body("ok")),
            );

            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                    const form = new MultipartForm();
                    const bytes = new Uint8Array([104, 101, 108, 108, 111, 45, 98, 121, 116, 101, 115]);
                    form.addBytes("payload", bytes, "payload.bin", "application/octet-stream");
                    await web.downloadText("{}", {{
                        method: Method.Post,
                        multipart: form,
                    }});
                    "#,
                    server.url("/upload")
                ))
                .await
                .unwrap();

            assert_eq!(result, "ok");
        });
    }

    #[test]
    fn test_multipart_file_field() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let server = Server::run();
            let directory = std::env::temp_dir();
            let filepath = directory.join("multipart_test.txt");
            fs::write(&filepath, "file-content").unwrap();

            server.expect(
                Expectation::matching(all_of![
                    request::method_path("POST", "/upload"),
                    request::headers(contains(("content-type", matches("multipart/form-data")))),
                    request::body(matches("name=\"file\"")),
                    request::body(matches("filename=\"multipart_test.txt\"")),
                    request::body(matches("file-content")),
                ])
                .respond_with(status_code(200).body("ok")),
            );

            let result = script_engine
                .eval_async::<String>(&format!(
                    r#"
                    const form = new MultipartForm();
                    form.addFile("file", "{}", "multipart_test.txt", "text/plain");
                    await web.downloadText("{}", {{
                        method: Method.Post,
                        multipart: form,
                    }});
                    "#,
                    filepath.to_string_lossy(),
                    server.url("/upload")
                ))
                .await
                .unwrap();

            assert_eq!(result, "ok");
        });
    }
}
