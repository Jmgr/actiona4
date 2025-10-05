use std::{
    io::Cursor,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use bytes::Bytes;
use convert_case::{Case, Casing};
use derive_more::Display;
use encoding_rs::{Encoding, UTF_8};
use eyre::Result;
use futures::{Stream, StreamExt, TryStreamExt};
use http_body::Frame;
use http_body_util::{BodyExt, StreamBody};
use indexmap::IndexMap;
use macros::ExposeEnum;
use mime::Mime;
use reqwest::{
    RequestBuilder, Response,
    header::{self, CONTENT_TYPE, HeaderMap},
    multipart::{Form, Part},
};
use rquickjs::{JsLifetime, class::Trace};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
    select,
    sync::watch,
};
use tokio_util::{io::ReaderStream, sync::CancellationToken, task::TaskTracker};

use crate::{
    cancel_on,
    core::{image::Image, js::task::IsDone},
    sized_body::{BoxError, SizedBody},
};

pub mod js;

#[derive(Clone, Copy, Debug, Default, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
#[rquickjs::class]
pub enum Method {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Patch,
    Trace,
}

impl From<Method> for reqwest::Method {
    fn from(value: Method) -> Self {
        use Method::*;
        match value {
            Get => reqwest::Method::GET,
            Post => reqwest::Method::POST,
            Put => reqwest::Method::PUT,
            Delete => reqwest::Method::DELETE,
            Head => reqwest::Method::HEAD,
            Options => reqwest::Method::OPTIONS,
            Connect => reqwest::Method::CONNECT,
            Patch => reqwest::Method::PATCH,
            Trace => reqwest::Method::TRACE,
        }
    }
}

/// Web options
#[derive(Clone, Default)]
pub struct WebOptions {
    pub user_name: Option<String>,
    pub password: Option<String>,
    pub headers: HeaderMap,
    pub method: Method,
    pub progress: Option<watch::Sender<Progress>>,
    pub timeout: Option<Duration>,
    pub query: IndexMap<String, String>,
    pub request_body: Body,
}

#[derive(Clone)]
pub struct FormField {
    pub name: String,
    pub value: String,
}

#[derive(Clone)]
pub struct MultipartField {
    pub name: String,
    pub value: MultipartValue,
    pub filename: Option<String>,
    pub mimetype: Option<Mime>,
}

#[derive(Clone)]
pub enum MultipartValue {
    Text(String),
    File(PathBuf),
    Bytes(Bytes),
}

#[derive(Clone, Default)]
pub enum Body {
    #[default]
    None,
    Text {
        text: String,
        content_type: Option<Mime>,
    },
    File {
        path: PathBuf,
        content_type: Option<Mime>,
    },
    Bytes {
        bytes: Bytes,
        content_type: Option<Mime>,
    },
    Form(Vec<FormField>),
    Multipart(Vec<MultipartField>),
}

#[derive(Default)]
struct ProgressReporter {
    current: Arc<AtomicU64>,
    sender: watch::Sender<u64>,
}

impl ProgressReporter {
    pub fn increment(&self, value: u64) {
        let previous_value = self.current.fetch_add(value, Ordering::Relaxed);
        let new_value = previous_value + value;
        self.sender.send_replace(new_value);
    }

    pub fn receiver(&self) -> watch::Receiver<u64> {
        self.sender.subscribe()
    }
}

impl Body {
    fn maybe_set_content_type(
        mut request: RequestBuilder,
        content_type: Option<Mime>,
    ) -> RequestBuilder {
        if let Some(content_type) = content_type {
            request = request.header(CONTENT_TYPE, content_type.to_string());
        }

        request
    }

    fn guess_mime_from_bytes(bytes: &Bytes) -> Option<&str> {
        infer::get(&bytes).map(|guess| guess.mime_type())
    }

    async fn guess_mime_from_path(path: &Path, is_multipart: bool) -> Option<String> {
        const MAGIC_NUMBER_MAX_LEN: usize = 8192; // Copied from the infer crate

        // The infer crate only offers sync I/O, so we manually get the magic number
        let mut file = File::open(path).await.ok()?;

        let mut buffer = vec![0u8; MAGIC_NUMBER_MAX_LEN];

        let n = file.read(&mut buffer).await.ok()?;
        let head = &buffer[..n];

        // Try to guess using the magic number first
        if let Some(mime) = infer::get(&head) {
            return Some(mime.mime_type().to_string());
        }

        // If that fails, fall back to the file extension
        let mime = mime_guess::from_path(path);

        // Default mime type depends on the context
        Some(
            if is_multipart {
                mime.first_or_octet_stream()
            } else {
                mime.first_or_text_plain()
            }
            .to_string(),
        )
    }

    fn stream_to_body<S, E>(
        stream: S,
        size: u64,
        progress_reporter: Arc<ProgressReporter>,
    ) -> (reqwest::Body, u64)
    where
        S: Stream<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let async_stream = stream
            .map_ok(move |bytes| {
                progress_reporter.increment(bytes.len() as u64);
                Frame::data(bytes)
            })
            .map_err(|e| -> BoxError { Box::new(e) });
        let body = StreamBody::new(async_stream);
        let boxed_body = BodyExt::boxed(body);
        let sized_body = SizedBody::new(boxed_body, size);

        (reqwest::Body::wrap(sized_body), size)
    }

    async fn file_to_body(
        path: &Path,
        progress_reporter: Arc<ProgressReporter>,
    ) -> Result<(reqwest::Body, u64)> {
        let file = File::open(path).await?;
        let len = file.metadata().await?.len();

        Ok(Self::stream_to_body(
            ReaderStream::new(file),
            len,
            progress_reporter,
        ))
    }

    async fn bytes_to_body(
        bytes: Bytes,
        progress_reporter: Arc<ProgressReporter>,
    ) -> Result<(reqwest::Body, u64)> {
        let len = bytes.len() as u64;

        Ok(Self::stream_to_body(
            ReaderStream::new(Cursor::new(bytes)),
            len,
            progress_reporter.clone(),
        ))
    }

    async fn apply_to(
        self,
        mut request: RequestBuilder,
    ) -> Result<(RequestBuilder, watch::Receiver<u64>, u64)> {
        let mut total_size = 0;
        let progress_reporter = Arc::new(ProgressReporter::default());
        match self {
            Body::None => {}
            Body::Text { text, content_type } => {
                request = request.body(text);
                request = Self::maybe_set_content_type(request, content_type);
            }
            Body::File { path, content_type } => {
                if let Some(mime) = Self::guess_mime_from_path(&path, false).await {
                    request = request.header(CONTENT_TYPE, mime);
                }

                let (body, size) = Self::file_to_body(&path, progress_reporter.clone()).await?;

                request = request.body(body);
                total_size += size;

                request = Self::maybe_set_content_type(request, content_type);
            }
            Body::Bytes {
                bytes,
                content_type,
            } => {
                if let Some(mime) = Self::guess_mime_from_bytes(&bytes) {
                    request = request.header(CONTENT_TYPE, mime);
                }

                let (body, size) = Self::bytes_to_body(bytes, progress_reporter.clone()).await?;

                request = request.body(body);
                total_size += size;

                request = Self::maybe_set_content_type(request, content_type);
            }
            Body::Form(form_fields) => {
                request = request.form(
                    &form_fields
                        .into_iter()
                        .map(|field| (field.name, field.value))
                        .collect::<Vec<_>>(),
                );
            }
            Body::Multipart(multipart_fields) => {
                let mut form = Form::new();
                for field in multipart_fields {
                    let mut part = match field.value {
                        MultipartValue::Text(text) => Part::text(text),
                        MultipartValue::File(path_buf) => {
                            let (body, size) =
                                Self::file_to_body(&path_buf, progress_reporter.clone()).await?;

                            total_size += size;

                            Part::stream_with_length(body, size)
                        }
                        MultipartValue::Bytes(bytes) => {
                            if let Some(mime) = Self::guess_mime_from_bytes(&bytes) {
                                request = request.header(CONTENT_TYPE, mime);
                            }

                            let (body, size) =
                                Self::bytes_to_body(bytes, progress_reporter.clone()).await?;

                            total_size += size;

                            Part::stream_with_length(body, size)
                        }
                    };

                    if let Some(filename) = field.filename {
                        part = part.file_name(filename);
                    }
                    if let Some(mimetype) = field.mimetype {
                        part = part.mime_str(&mimetype.to_string())?;
                    }

                    form = form.part(field.name, part);
                }
                request = request.multipart(form);
            }
        }

        Ok((request, progress_reporter.receiver(), total_size))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Progress {
    #[default]
    Inactive,
    Uploading {
        current: u64,
        total: u64,
    },
    Downloading {
        current: u64,
        total: Option<u64>,
    },
    Finished,
}

impl IsDone for Progress {
    fn is_done(&self) -> bool {
        self.is_finished()
    }
}

impl Progress {
    /*
    pub fn total(&self) -> Option<u64> {
        match self {
            Self::Inactive | Self::Uploading { .. } => None,
            Self::Downloading { total, .. } => *total,
            Self::Finished { total, .. } => *total,
        }
    }

    pub fn current(&self) -> u64 {
        match self {
            Self::Inactive | Self::Uploading { .. } => 0,
            Self::Downloading { current, .. } => *current,
            Self::Finished { current, .. } => *current,
        }
    }
    */// TODO

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Finished { .. })
    }
}

#[derive(Clone)]
pub struct Web {
    inner: reqwest::Client,
    task_tracker: TaskTracker,
}

impl Web {
    pub fn new(task_tracker: TaskTracker) -> Self {
        Self {
            inner: reqwest::Client::new(),
            task_tracker,
        }
    }

    async fn build_request(
        &self,
        url: &str,
        options: WebOptions,
    ) -> Result<(RequestBuilder, watch::Receiver<u64>, u64)> {
        let mut request_builder = self.inner.request(options.method.into(), url);

        if let Some(user_name) = options.user_name {
            request_builder = request_builder.basic_auth(user_name, options.password);
        }

        request_builder = request_builder.headers(options.headers);

        if let Some(timeout) = options.timeout {
            request_builder = request_builder.timeout(timeout);
        }

        let (new_request_builder, progress_receiver, total_size) =
            options.request_body.apply_to(request_builder).await?;

        request_builder = new_request_builder.query(&options.query);

        Ok((request_builder, progress_receiver, total_size))
    }

    /// Uploads some data.
    pub async fn upload(
        &self,
        url: &str,
        token: CancellationToken,
        options: Option<WebOptions>,
    ) -> Result<()> {
        let mut options = options.unwrap_or_default();
        let progress = options.progress.take();
        self.fetch_response(url, &token, &progress, options).await?;
        Ok(())
    }

    /// Downloads a binary file.
    pub async fn download(
        &self,
        url: &str,
        token: CancellationToken,
        options: Option<WebOptions>,
    ) -> Result<Vec<u8>> {
        let mut options = options.unwrap_or_default();
        let progress = options.progress.take();
        let response = self.fetch_response(url, &token, &progress, options).await?;
        let buffer = self
            .download_impl_with_buffer(token, response, &progress)
            .await?;
        Ok(buffer)
    }

    /// Downloads a text file.
    pub async fn download_text(
        &self,
        url: &str,
        token: CancellationToken,
        options: Option<WebOptions>,
    ) -> Result<String> {
        let mut options = options.unwrap_or_default();
        let progress = options.progress.take();
        let response = self.fetch_response(url, &token, &progress, options).await?;

        // This is basically the same as in reqwest::async_impl::response::Response::text_with_charset
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<Mime>().ok());
        let encoding_name = content_type
            .as_ref()
            .and_then(|mime| mime.get_param("charset").map(|charset| charset.as_str()))
            .unwrap_or("utf-8");
        let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);

        let buffer = self
            .download_impl_with_buffer(token, response, &progress)
            .await?;
        let (text, _, _) = encoding.decode(&buffer);

        Ok(text.into_owned())
    }

    /// Downloads an image.
    pub async fn download_image(
        &self,
        url: &str,
        token: CancellationToken,
        options: Option<WebOptions>,
    ) -> Result<Image> {
        let mut options = options.unwrap_or_default();
        let progress = options.progress.take();
        let response = self.fetch_response(url, &token, &progress, options).await?;
        let buffer = self
            .download_impl_with_buffer(token, response, &progress)
            .await?;
        let image = Image::from_bytes(&buffer)?;

        Ok(image)
    }

    /// Downloads a file to a directory.
    pub async fn download_file(
        &self,
        url: &str,
        token: CancellationToken,
        directory: Option<&Path>,
        options: Option<WebOptions>,
    ) -> Result<String> {
        let mut options = options.unwrap_or_default();
        let progress = options.progress.take();
        let response = self.fetch_response(url, &token, &progress, options).await?;

        let directory = directory
            .map(|directory| directory.to_path_buf())
            .unwrap_or(std::env::temp_dir());

        let filename = Self::guess_filename(&response, url);
        let filepath = directory.join(&filename);
        let tmp_filepath = directory.join(&format!("{}.part", filename));

        let file = cancel_on(&token, File::create(&tmp_filepath)).await??;

        self.download_impl(token.clone(), response, &progress, file)
            .await?;

        cancel_on(&token, fs::rename(&tmp_filepath, &filepath)).await??;

        Ok(filepath.to_string_lossy().to_string())
    }

    async fn fetch_response(
        &self,
        url: &str,
        token: &CancellationToken,
        progress: &Option<watch::Sender<Progress>>,
        options: WebOptions,
    ) -> Result<Response> {
        if let Some(progress) = &progress {
            progress.send_replace(Progress::Uploading {
                current: 0,
                total: 0,
            });
        }

        let (request, mut upload_progress, total_upload) = self.build_request(url, options).await?;

        let local_token = token.clone();
        let local_progress = progress.clone();
        self.task_tracker.spawn(async move {
            loop {
                select! {
                    _ = local_token.cancelled() => {
                        break;
                    },
                    changed = upload_progress.changed() => {
                        if changed.is_err() { // Sender closed
                            break;
                        }

                        let progress_value = *upload_progress.borrow_and_update();

                        if let Some(progress) = &local_progress {
                            progress.send_replace(Progress::Uploading {
                                current: progress_value,
                                total: total_upload,
                            });
                        }
                    },
                }
            }
        });

        let response = cancel_on(token, request.send()).await??;
        let response = response.error_for_status()?;

        Ok(response)
    }

    async fn download_impl<W: AsyncWrite + Unpin>(
        &self,
        token: CancellationToken,
        response: Response,
        progress: &Option<watch::Sender<Progress>>,
        mut writer: W,
    ) -> Result<()> {
        let total_size = response.content_length();
        let mut current = 0;

        if let Some(progress) = &progress {
            progress.send_replace(Progress::Downloading {
                current,
                total: total_size,
            });
        }

        let mut stream = response.bytes_stream();

        loop {
            let chunk = cancel_on(&token, stream.next()).await?;
            let Some(chunk) = chunk else {
                break;
            };

            let chunk = chunk?;

            cancel_on(&token, writer.write_all(&chunk)).await??;

            current += chunk.len() as u64;

            if let Some(progress) = &progress {
                progress.send_replace(Progress::Downloading {
                    current,
                    total: total_size,
                });
            }
        }

        if let Some(progress) = &progress {
            progress.send_replace(Progress::Finished);
        }

        Ok(())
    }

    async fn download_impl_with_buffer(
        &self,
        token: CancellationToken,
        response: Response,
        progress: &Option<watch::Sender<Progress>>,
    ) -> Result<Vec<u8>> {
        let mut buffer = if let Some(length) = response.content_length() {
            Vec::with_capacity(length as usize)
        } else {
            Vec::new()
        };

        self.download_impl(token, response, progress, &mut buffer)
            .await?;

        Ok(buffer)
    }

    fn guess_filename(resp: &Response, url: &str) -> String {
        if let Some(cd) = resp
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
        {
            if let Some(name) = cd
                .split(';')
                .find_map(|p| p.trim().strip_prefix("filename="))
                .map(|s| s.trim_matches('"'))
            {
                return name.to_string();
            }
        }
        url.split('/')
            .last()
            .filter(|s| !s.is_empty())
            .unwrap_or("download.bin")
            .to_string()
    }
}

#[cfg(test)]
mod helper {
    use std::io::Cursor;

    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use imageproc::{drawing::draw_hollow_rect_mut, rect::Rect};

    pub struct TestImage {
        pub image: DynamicImage,
        pub bytes: Vec<u8>,
    }

    impl Default for TestImage {
        fn default() -> Self {
            let mut image = RgbaImage::from_pixel(2048, 2048, Rgba([255, 255, 255, 255]));
            let rect = Rect::at(5, 0).of_size(30, 35);
            draw_hollow_rect_mut(&mut image, rect, Rgba([255, 0, 0, 255]));
            let image = DynamicImage::ImageRgba8(image);
            let mut image_bytes = Vec::new();
            image
                .write_to(&mut Cursor::new(&mut image_bytes), ImageFormat::Png)
                .unwrap();

            Self {
                image,
                bytes: image_bytes,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{Arc, Mutex},
    };

    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    use httptest::{
        Expectation, Server, all_of,
        matchers::{contains, request},
        responders::status_code,
    };
    use tokio::sync::watch;

    use super::*;
    use crate::{core::web::helper::TestImage, runtime::Runtime};

    #[test]
    fn test_download_text() {
        Runtime::test(async move |runtime| {
            let server = Server::run();

            server.expect(
                Expectation::matching(request::method_path("GET", "/foo"))
                    .respond_with(status_code(200).body("hello")),
            );

            let web = Web::new(runtime.task_tracker());
            let cancellation_token = CancellationToken::new();
            let result = web
                .download_text(&server.url("/foo").to_string(), cancellation_token, None)
                .await
                .unwrap();

            assert_eq!(result, "hello");
        });
    }

    #[test]
    fn test_download_image() {
        Runtime::test(async move |runtime| {
            let server = Server::run();

            let test_image = TestImage::default();

            server.expect(
                Expectation::matching(request::method_path("GET", "/image.png")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "image/png")
                        .body(test_image.bytes),
                ),
            );

            let web = Web::new(runtime.task_tracker());
            let cancellation_token = CancellationToken::new();
            let result = web
                .download_image(
                    &server.url("/image.png").to_string(),
                    cancellation_token,
                    None,
                )
                .await
                .unwrap()
                .into_inner();

            assert_eq!(result, test_image.image);
        });
    }

    #[test]
    fn test_download_binary() {
        Runtime::test(async move |runtime| {
            let server = Server::run();

            let test_image = TestImage::default();

            server.expect(
                Expectation::matching(request::method_path("GET", "/binary")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "application/octet-stream")
                        .body(test_image.bytes.clone()),
                ),
            );

            let web = Web::new(runtime.task_tracker());
            let cancellation_token = CancellationToken::new();
            let result = web
                .download(&server.url("/binary").to_string(), cancellation_token, None)
                .await
                .unwrap();

            assert_eq!(result, test_image.bytes);
        });
    }

    #[test]
    fn test_download_basic_auth() {
        Runtime::test(async move |runtime| {
            let server = Server::run();

            let auth_header = format!("Basic {}", BASE64.encode("user:password"));

            server.expect(
                Expectation::matching(all_of![
                    request::method_path("POST", "/foo"),
                    request::headers(contains(("authorization", auth_header)))
                ])
                .respond_with(status_code(200).body("hello")),
            );

            let web = Web::new(runtime.task_tracker());
            let cancellation_token = CancellationToken::new();
            let result = web
                .download_text(
                    &server.url("/foo").to_string(),
                    cancellation_token,
                    Some(WebOptions {
                        user_name: Some("user".to_string()),
                        password: Some("password".to_string()),
                        method: Method::Post,
                        ..Default::default()
                    }),
                )
                .await
                .unwrap();

            assert_eq!(result, "hello");
        });
    }

    #[test]
    fn test_download_progress() {
        Runtime::test(async move |runtime| {
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

            let web = Web::new(runtime.task_tracker());
            let (sender, mut receiver) = watch::channel(Progress::Inactive);

            let received_progress = Arc::new(Mutex::new(Vec::new()));

            let local_received_progress = received_progress.clone();
            let receiver_handle = tokio::spawn(async move {
                while receiver.changed().await.is_ok() {
                    local_received_progress
                        .lock()
                        .unwrap()
                        .push(*receiver.borrow_and_update());
                }
            });

            let cancellation_token = CancellationToken::new();
            let result = web
                .download(
                    &server.url("/binary").to_string(),
                    cancellation_token,
                    Some(WebOptions {
                        progress: Some(sender),
                        request_body: Body::Bytes {
                            bytes: test_image.bytes.clone().into(),
                            content_type: None,
                        },
                        ..Default::default()
                    }),
                )
                .await
                .unwrap();
            assert_eq!(result, test_image.bytes);

            drop(web);
            receiver_handle.await.unwrap();

            let received_progress = received_progress.lock().unwrap();

            println!("{:?}", received_progress);

            assert!(received_progress.contains(&Progress::Finished));
        });
    }

    #[test]
    fn test_download_file() {
        Runtime::test(async move |runtime| {
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

            let web = Web::new(runtime.task_tracker());
            let cancellation_token = CancellationToken::new();
            let filepath = web
                .download_file(
                    &server.url("/image").to_string(),
                    cancellation_token,
                    Some(&directory),
                    None,
                )
                .await
                .unwrap();

            let expected_filepath = directory.join("example.png");
            assert_eq!(filepath, expected_filepath.as_os_str().to_string_lossy());

            let bytes = fs::read(expected_filepath).unwrap();
            assert_eq!(bytes, test_image.bytes);
        });
    }

    #[test]
    fn test_upload_text_body() {
        Runtime::test(async move |runtime| {
            const TEST_STRING: &str = "this is a test";

            let server = Server::run();

            server.expect(
                Expectation::matching(all_of![
                    request::method_path("POST", "/"),
                    request::headers(contains((
                        "content-length",
                        format!("{}", TEST_STRING.len())
                    ))),
                    request::body(TEST_STRING)
                ])
                .respond_with(status_code(200)),
            );

            let web = Web::new(runtime.task_tracker());
            let cancellation_token = CancellationToken::new();
            web.upload(
                &server.url("/").to_string(),
                cancellation_token,
                Some(WebOptions {
                    method: Method::Post,
                    request_body: Body::Text {
                        text: TEST_STRING.to_string(),
                        content_type: None,
                    },
                    ..Default::default()
                }),
            )
            .await
            .unwrap();
        });
    }

    #[test]
    fn test_upload_binary_body() {
        Runtime::test(async move |runtime| {
            let test_image = TestImage::default();

            let server = Server::run();

            server.expect(
                Expectation::matching(all_of![
                    request::method_path("POST", "/"),
                    request::headers(contains((
                        "content-length",
                        format!("{}", test_image.bytes.len())
                    ))),
                    request::headers(contains(("content-type", "image/png"))),
                    request::body(test_image.bytes.clone())
                ])
                .respond_with(status_code(200)),
            );

            let web = Web::new(runtime.task_tracker());
            let cancellation_token = CancellationToken::new();
            web.upload(
                &server.url("/").to_string(),
                cancellation_token,
                Some(WebOptions {
                    method: Method::Post,
                    request_body: Body::Bytes {
                        bytes: test_image.bytes.into(),
                        content_type: None,
                    },
                    ..Default::default()
                }),
            )
            .await
            .unwrap();
        });
    }
}
