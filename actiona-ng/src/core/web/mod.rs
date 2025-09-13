use std::{
    cmp,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use bytes::Bytes;
use convert_case::{Case, Casing};
use derive_more::Display;
use encoding_rs::{Encoding, UTF_8};
use eyre::Result;
use futures_util::StreamExt;
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
    io::{AsyncWrite, AsyncWriteExt},
    sync::watch,
};
use tokio_util::{io::ReaderStream, sync::CancellationToken};

use crate::{
    cancel_on,
    core::{image::Image, js::task::IsDone},
};

pub mod js;

#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace, Default)]
#[rquickjs::class]
pub enum Method {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
}

impl From<Method> for reqwest::Method {
    fn from(value: Method) -> Self {
        use Method::*;
        match value {
            Get => reqwest::Method::GET,
            Post => reqwest::Method::POST,
            Put => reqwest::Method::PUT,
            Patch => reqwest::Method::PATCH,
            Delete => reqwest::Method::DELETE,
            Head => reqwest::Method::HEAD,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct MultipartForm {
    texts: IndexMap<String, String>,
    files: IndexMap<String, PathBuf>,
    bytes: IndexMap<String, Vec<u8>>,
}

impl MultipartForm {
    async fn into_reqwest_form(self) -> Result<reqwest::multipart::Form> {
        let mut result = reqwest::multipart::Form::new();

        for (name, text) in self.texts {
            result = result.text(name, text);
        }
        for (name, filepath) in self.files {
            result = result.file(name, filepath).await?; // TODO: async?
        }
        for (name, bytes) in self.bytes {
            result = result.part(name, Part::bytes(bytes).file_name("TODO")); // TODO: add filename option and MIME option to all
        }

        Ok(result)
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
    Bytes {
        bytes: Bytes,
        content_type: Option<Mime>,
    },
    Form(Vec<FormField>),
    Multipart(Vec<MultipartField>),
}

impl Body {
    fn set_content_type(mut request: RequestBuilder, content_type: Option<Mime>) -> RequestBuilder {
        if let Some(content_type) = content_type {
            request = request.header(CONTENT_TYPE, content_type.to_string());
        }

        request
    }

    fn apply_to(self, mut request: RequestBuilder) -> Result<RequestBuilder> {
        match self {
            Body::None => {}
            Body::Text { text, content_type } => {
                request = request.body(text);
                request = Self::set_content_type(request, content_type);
            }
            Body::Bytes {
                bytes,
                content_type,
            } => {
                request = request.body(bytes);
                request = Self::set_content_type(request, content_type);
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
                let form = Form::new();
                for field in multipart_fields {
                    let mut part = match field.value {
                        MultipartValue::Text(text) => Part::text(text),
                        MultipartValue::File(path_buf) => Part::stream(path_buf), // TODO: streaming
                        MultipartValue::Bytes(bytes) => Part::bytes(bytes.into()),
                    };

                    if let Some(filename) = field.filename {
                        part = part.file_name(filename);
                    }
                    if let Some(mimetype) = field.mimetype {
                        part = part.mime_str(&mimetype.to_string())?;
                    }

                    form.part(field.name, part);
                }
                request = request.multipart(form);
            }
        }

        Ok(request)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Progress {
    #[default]
    Inactive,
    InitialRequest,
    ReceivedResponse {
        total: Option<u64>,
    },
    Downloading {
        current: u64,
        total: Option<u64>,
    },
    Finished {
        current: u64,
        total: Option<u64>,
    },
}

impl IsDone for Progress {
    fn is_done(&self) -> bool {
        self.is_finished()
    }
}

impl Progress {
    pub fn total(&self) -> Option<u64> {
        match self {
            Self::Inactive | Self::InitialRequest => None,
            Self::ReceivedResponse { total } => *total,
            Self::Downloading { total, .. } => *total,
            Self::Finished { total, .. } => *total,
        }
    }

    pub fn current(&self) -> u64 {
        match self {
            Self::Inactive | Self::InitialRequest | Self::ReceivedResponse { .. } => 0,
            Self::Downloading { current, .. } => *current,
            Self::Finished { current, .. } => *current,
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Finished { .. })
    }
}

#[derive(Clone)]
pub struct Web {
    inner: reqwest::Client,
}

impl Default for Web {
    fn default() -> Self {
        Self {
            inner: reqwest::Client::new(),
        }
    }
}

impl Web {
    async fn build_request(&self, url: &str, options: WebOptions) -> Result<RequestBuilder> {
        let mut result = self.inner.request(options.method.into(), url);

        if let Some(user_name) = options.user_name {
            result = result.basic_auth(user_name, options.password);
        }
        result = result.headers(options.headers);
        if let Some(timeout) = options.timeout {
            result = result.timeout(timeout);
        }
        result = options.request_body.apply_to(result)?;
        result = result.query(&options.query);

        Ok(result)
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
            progress.send_replace(Progress::InitialRequest);
        }

        let request = self.build_request(url, options).await?;

        let file = tokio::fs::File::open("")
            .await
            .expect("Cannot open input file for HTTPS read");
        let mut reader_stream = ReaderStream::with_capacity(file, 2 * 8192); // 16 KiB
        let async_stream = async_stream::stream! {
            while let Some(chunk) = reader_stream.next().await {
                if let Ok(chunk) = &chunk {
                    /*
                    let new = cmp::min(uploaded + (chunk.len() as u64), total_size);
                    uploaded = new;
                    bar.set_position(new);
                    if uploaded >= total_size {
                        bar.finish_upload(&input_, &output_);
                    }
                    */
                }
                yield chunk;
            }
        };

        let request = request.body(reqwest::Body::wrap_stream(async_stream));

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
            progress.send_replace(Progress::ReceivedResponse { total: total_size });
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
            progress.send_replace(Progress::Finished {
                current,
                total: total_size,
            });
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
        Runtime::test(async move |_| {
            let server = Server::run();

            server.expect(
                Expectation::matching(request::method_path("GET", "/foo"))
                    .respond_with(status_code(200).body("hello")),
            );

            let web = Web::default();
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
        Runtime::test(async move |_| {
            let server = Server::run();

            let test_image = TestImage::default();

            server.expect(
                Expectation::matching(request::method_path("GET", "/image.png")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "image/png")
                        .body(test_image.bytes),
                ),
            );

            let web = Web::default();
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
        Runtime::test(async move |_| {
            let server = Server::run();

            let test_image = TestImage::default();

            server.expect(
                Expectation::matching(request::method_path("GET", "/binary")).respond_with(
                    status_code(200)
                        .append_header("Content-Type", "application/octet-stream")
                        .body(test_image.bytes.clone()),
                ),
            );

            let web = Web::default();
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
        Runtime::test(async move |_| {
            let server = Server::run();

            let auth_header = format!("Basic {}", BASE64.encode("user:password"));

            server.expect(
                Expectation::matching(all_of![
                    request::method_path("POST", "/foo"),
                    request::headers(contains(("authorization", auth_header)))
                ])
                .respond_with(status_code(200).body("hello")),
            );

            let web = Web::default();
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
        Runtime::test(async move |_| {
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

            let web = Web::default();
            let (sender, mut receiver) = watch::channel(Progress::Inactive);

            let received_progress = Arc::new(Mutex::new(Vec::new()));

            let local_received_progress = received_progress.clone();
            let receiver_handle = tokio::spawn(async move {
                local_received_progress
                    .lock()
                    .unwrap()
                    .push(*receiver.borrow_and_update());

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
                        ..Default::default()
                    }),
                )
                .await
                .unwrap();
            assert_eq!(result, test_image.bytes);

            drop(web);
            receiver_handle.await.unwrap();

            let received_progress = received_progress.lock().unwrap();
            assert!(received_progress.contains(&Progress::Inactive));
            assert!(received_progress.contains(&Progress::InitialRequest));
            assert!(received_progress.contains(&Progress::ReceivedResponse {
                total: Some(total_size)
            }));
            assert!(received_progress.contains(&Progress::Finished {
                current: total_size,
                total: Some(total_size),
            }));
        });
    }

    #[test]
    fn test_download_file() {
        Runtime::test(async move |_| {
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

            let web = Web::default();
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
}
