use std::sync::Arc;

use rquickjs::{
    Ctx, JsLifetime, Promise, Result,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::sync::watch;
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{
        color::js::JsColor,
        displays::Displays,
        image::{
            find_image::{FindImageProgress, FindImageStage, FindImageTemplateOptions, Template},
            js::{JsFindImageOptions, JsFindImageProgress, JsImage, JsMatch},
        },
        js::{classes::SingletonClass, task::progress_task_with_token},
        point::js::JsPointLike,
        rect::js::JsRectLike,
        screenshot::Screenshot,
    },
    runtime::{Runtime, WithUserData},
};

impl<'js> Trace<'js> for super::Screenshot {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

/// Screenshot capture and image search.
///
/// Provides methods to capture screen regions, displays, and individual pixels,
/// as well as finding images on screen.
///
/// ```ts
/// const image = await screenshot.captureDisplay(0);
/// println(image.size().toString());
/// ```
///
/// ```ts
/// const pixel = await screenshot.capturePixel(100, 100);
/// println(pixel.toString());
/// ```
///
/// @singleton
#[derive(Debug, JsLifetime, Trace)]
#[rquickjs::class(rename = "Screenshot")]
pub struct JsScreenshot {
    inner: super::Screenshot,
}

impl SingletonClass<'_> for JsScreenshot {}

impl JsScreenshot {
    /// @skip
    #[instrument(skip_all)]
    pub async fn new(runtime: Arc<Runtime>, displays: Displays) -> super::Result<Self> {
        Ok(Self {
            inner: super::Screenshot::new(runtime, displays).await?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsScreenshot {
    /// Captures a screenshot of a screen rectangle.
    ///
    /// ```ts
    /// const image = await screenshot.captureRect(0, 0, 1920, 1080);
    /// ```
    pub async fn capture_rect(&self, ctx: Ctx<'_>, rect: JsRectLike) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner.capture_rect(rect.0).await.into_js_result(&ctx)?,
        ))
    }

    /// Captures a screenshot of an entire display.
    ///
    /// ```ts
    /// const image = await screenshot.captureDisplay(0);
    /// ```
    pub async fn capture_display(&self, ctx: Ctx<'_>, display_id: u32) -> Result<JsImage> {
        Ok(JsImage::new(
            self.inner
                .capture_display(display_id)
                .await
                .into_js_result(&ctx)?,
        ))
    }

    /// Captures the color of a single pixel on screen.
    ///
    /// ```ts
    /// const color = await screenshot.capturePixel(100, 200);
    /// println(color.toString());
    /// ```
    pub async fn capture_pixel(&self, ctx: Ctx<'_>, position: JsPointLike) -> Result<JsColor> {
        Ok(self
            .inner
            .capture_pixel(position.0)
            .await
            .into_js_result(&ctx)?
            .into())
    }

    /// Finds the best match of an image on a screen rectangle.
    ///
    /// ```ts
    /// const match = await screenshot.findImageOnRect(0, 0, 1920, 1080, template);
    /// ```
    ///
    /// ```ts
    /// const task = screenshot.findImageOnRect(0, 0, 1920, 1080, template);
    /// for await (const progress of task) {
    ///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
    /// }
    /// const match = await task;
    /// ```
    /// @returns ProgressTask<Match | undefined, FindImageProgress>
    pub fn find_image_on_rect<'js>(
        &self,
        ctx: Ctx<'js>,
        rect: JsRectLike,
        image: JsImage,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Promise<'js>> {
        let rect = rect.0;
        find_image_task(
            ctx,
            &self.inner,
            image,
            options,
            move |inner, template, opts, token, progress| async move {
                let result = inner
                    .find_image_on_rect(rect, &template, opts, token, progress)
                    .await?;
                Ok(result.map(JsMatch::from))
            },
        )
    }

    /// Finds all occurrences of an image on a screen rectangle.
    ///
    /// ```ts
    /// const matches = await screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
    /// ```
    ///
    /// ```ts
    /// const task = screenshot.findImageOnRectAll(0, 0, 1920, 1080, template);
    /// for await (const progress of task) {
    ///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
    /// }
    /// const matches = await task;
    /// ```
    /// @returns ProgressTask<Match[], FindImageProgress>
    pub fn find_image_on_rect_all<'js>(
        &self,
        ctx: Ctx<'js>,
        rect: JsRectLike,
        image: JsImage,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Promise<'js>> {
        let rect = rect.0;
        find_image_task(
            ctx,
            &self.inner,
            image,
            options,
            move |inner, template, opts, token, progress| async move {
                let results = inner
                    .find_image_on_rect_all(rect, &template, opts, token, progress)
                    .await?;
                Ok(results.into_iter().map(JsMatch::from).collect::<Vec<_>>())
            },
        )
    }

    /// Finds the best match of an image on a display.
    ///
    /// ```ts
    /// const match = await screenshot.findImageOnDisplay(0, template);
    /// ```
    ///
    /// ```ts
    /// const task = screenshot.findImageOnDisplay(0, template);
    /// for await (const progress of task) {
    ///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
    /// }
    /// const match = await task;
    /// ```
    /// @returns ProgressTask<Match | undefined, FindImageProgress>
    pub fn find_image_on_display<'js>(
        &self,
        ctx: Ctx<'js>,
        display_id: u32,
        image: JsImage,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Promise<'js>> {
        find_image_task(
            ctx,
            &self.inner,
            image,
            options,
            move |inner, template, opts, token, progress| async move {
                let result = inner
                    .find_image_on_display(display_id, &template, opts, token, progress)
                    .await?;
                Ok(result.map(JsMatch::from))
            },
        )
    }

    /// Finds all occurrences of an image on a display.
    ///
    /// ```ts
    /// const matches = await screenshot.findImageOnDisplayAll(0, template);
    /// ```
    ///
    /// ```ts
    /// const task = screenshot.findImageOnDisplayAll(0, template);
    /// for await (const progress of task) {
    ///   println(`${progress.stage}: ${formatPercent(progress.percent)}`);
    /// }
    /// const matches = await task;
    /// ```
    /// @returns ProgressTask<Match[], FindImageProgress>
    pub fn find_image_on_display_all<'js>(
        &self,
        ctx: Ctx<'js>,
        display_id: u32,
        image: JsImage,
        options: Opt<JsFindImageOptions>,
    ) -> Result<Promise<'js>> {
        find_image_task(
            ctx,
            &self.inner,
            image,
            options,
            move |inner, template, opts, token, progress| async move {
                let results = inner
                    .find_image_on_display_all(display_id, &template, opts, token, progress)
                    .await?;
                Ok(results.into_iter().map(JsMatch::from).collect::<Vec<_>>())
            },
        )
    }
}

/// Shared helper for all `find_image_on_*` JS bindings.
fn find_image_task<'js, R, F, Fut>(
    ctx: Ctx<'js>,
    inner: &Screenshot,
    image: JsImage,
    options: Opt<JsFindImageOptions>,
    search: F,
) -> Result<Promise<'js>>
where
    R: for<'a> rquickjs::IntoJs<'a> + Send + 'static,
    F: FnOnce(
            Screenshot,
            Arc<Template>,
            FindImageTemplateOptions,
            tokio_util::sync::CancellationToken,
            watch::Sender<FindImageProgress>,
        ) -> Fut
        + Send
        + 'static,
    Fut: Future<Output = color_eyre::Result<R>> + Send + 'static,
{
    let options = options.0.unwrap_or_default();
    let signal = options.signal.clone();
    let template = Arc::<Template>::try_from(image.to_inner()).into_js_result(&ctx)?;
    let inner = inner.clone();
    let (progress_sender, progress_receiver) =
        watch::channel(FindImageProgress::new(FindImageStage::Capturing, 0));

    progress_task_with_token::<_, _, _, _, _, JsFindImageProgress>(
        ctx,
        signal,
        progress_receiver,
        async move |ctx, token| {
            let task_tracker = ctx.user_data().task_tracker();

            let result = task_tracker
                .spawn(search(
                    inner,
                    template,
                    options.into_inner(),
                    token,
                    progress_sender,
                ))
                .await
                .map_err(|e| {
                    rquickjs::Exception::throw_message(&ctx, &format!("Task join error: {e}"))
                })?
                .into_js_result(&ctx)?;

            Ok(result)
        },
    )
}
