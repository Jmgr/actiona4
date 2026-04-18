use std::{path::PathBuf, str::FromStr};

use bytes::Bytes;
use indexmap::IndexMap;
use macros::{FromJsObject, js_class, js_methods, options};
use mime::Mime;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use rquickjs::{
    Ctx, Exception, JsLifetime, Promise, Result, TypedArray,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::sync::mpsc;
use tokio_util::task::TaskTracker;

use crate::{
    IntoJsResult,
    api::{
        image::js::JsImage,
        js::{
            abort_controller::JsAbortSignal,
            classes::{
                HostClass, SingletonClass, ValueClass, register_enum, register_host_class,
                register_value_class,
            },
            duration::JsDuration,
            task::progress_task_with_token,
        },
        web::{Body, FormField, MultipartField, MultipartValue, Progress, WebOptions},
    },
    error::CommonError,
    types::display::{DisplayFields, display_list, display_with_type},
};

pub type JsMethod = super::Method;

/// Multipart form for uploading files and data.
///
/// ```ts
/// const form = new MultipartForm();
/// form.addText("title", "My Upload");
/// form.addFile("file", "/path/to/file.txt");
/// const result = await web.downloadText("https://example.com/upload", {
///   method: Method.Post,
///   multipart: form,
/// });
/// ```
#[derive(Clone, Debug, Default, JsLifetime)]
#[js_class]
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

#[js_methods]
impl JsMultipartForm {
    /// @constructor
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Adds a text field.
    ///
    /// ```ts
    /// const form = new MultipartForm();
    /// form.addText("username", "john");
    /// ```
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
    ///
    /// ```ts
    /// const form = new MultipartForm();
    /// form.addFile("document", "/path/to/report.pdf");
    /// ```
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
    ///
    /// ```ts
    /// const form = new MultipartForm();
    /// const bytes = new Uint8Array([72, 101, 108, 108, 111]);
    /// form.addBytes("data", bytes, "hello.bin");
    /// ```
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

    /// Returns a string representation of this multipart form.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type(
            "MultipartForm",
            DisplayFields::default()
                .display("fields", display_list(self.fields.as_slice()))
                .finish_as_string(),
        )
    }
}

/// Web request options.
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsWebOptions {
    /// Abort signal to cancel the request.
    pub signal: Option<JsAbortSignal>,

    /// User name for HTTP basic authentication.
    pub user_name: Option<String>,

    /// Password for HTTP basic authentication.
    pub password: Option<String>,

    /// Additional HTTP headers to send with the request.
    pub headers: Option<IndexMap<String, String>>,

    /// HTTP method to use for the request.
    #[default(JsMethod::Get, ts = "Method.Get")]
    pub method: JsMethod,

    /// Request timeout duration.
    pub timeout: Option<JsDuration>,

    /// Sets the content-type header.
    /// Overrides any content-type set by other fields.
    ///
    pub content_type: Option<String>,

    /// Form data as strings.
    /// Sets content-type to "application/x-www-form-urlencoded".
    ///
    pub form: Option<IndexMap<String, String>>,

    /// Additional query parameters.
    ///
    pub query: Option<IndexMap<String, String>>,

    /// Form multipart data.
    /// Sets content-type and content-length appropriately.
    ///
    pub multipart: Option<JsMultipartForm>,
}

impl JsWebOptions {
    fn into_super(self, ctx: &Ctx<'_>) -> Result<WebOptions> {
        let headers = if let Some(headers) = self.headers {
            Some(
                headers
                    .into_iter()
                    .map(|(key, value)| {
                        let header_name = HeaderName::from_str(&key).map_err(|error| {
                            CommonError::Unknown(format!(
                                "Invalid HTTP header name '{key}': {error}"
                            ))
                        })?;
                        let header_value = HeaderValue::from_str(&value).map_err(|error| {
                            CommonError::Unknown(format!(
                                "Invalid HTTP header value for '{key}': {error}"
                            ))
                        })?;

                        Ok((header_name, header_value))
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
                .map_err(|error| {
                    CommonError::Unknown(format!(
                        "Invalid content-type header value '{content_type}': {error}"
                    ))
                })
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

/// Progress information for web downloads and uploads.
///
/// ```ts
/// const task = web.download("https://example.com/file.bin");
/// for await (const progress of task) {
///   println(
///     formatBytes(progress.current),
///     formatBytes(progress.total),
///     progress.finished,
///   );
/// }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, JsLifetime, PartialEq, Trace)]
#[js_class]
pub struct JsWebProgress {
    total: u64,
    current: u64,
    finished: bool,
}

impl HostClass<'_> for JsWebProgress {}

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
                #[allow(clippy::option_if_let_else)]
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

#[js_methods]
impl JsWebProgress {
    /// Total bytes expected (0 if unknown).
    #[get]
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.total
    }

    /// Bytes transferred so far.
    #[get]
    #[must_use]
    pub const fn current(&self) -> u64 {
        self.current
    }

    /// Whether the transfer is complete.
    #[get]
    #[must_use]
    pub const fn finished(&self) -> bool {
        self.finished
    }

    /// Returns a string representation of this web transfer progress.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type(
            "WebProgress",
            DisplayFields::default()
                .display("total", self.total)
                .display("current", self.current)
                .display("finished", self.finished)
                .finish_as_string(),
        )
    }
}

/// HTTP client for downloading files, text, images, and binary data.
///
/// Supports progress tracking, authentication, custom headers, and multipart uploads.
///
/// ```ts
/// const text = await web.downloadText("https://example.com/data.json");
/// ```
///
/// ```ts
/// const image = await web.downloadImage("https://example.com/photo.png");
/// println(image.size());
/// ```
///
/// @singleton
#[derive(JsLifetime)]
#[js_class]
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

    fn make_progress_receiver(options: &mut WebOptions) -> mpsc::UnboundedReceiver<Progress> {
        let (progress_sender, progress_receiver) = mpsc::unbounded_channel::<Progress>();
        options.progress = Some(progress_sender);

        progress_receiver
    }
}

#[js_methods]
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
    ///   println(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
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
    ///   println(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
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
    ///   println(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
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
    ///   println(`${formatBytes(progress.current)}/${formatBytes(progress.total)}`);
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

    /// Returns a string representation of the `web` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Web".to_string()
    }
}
