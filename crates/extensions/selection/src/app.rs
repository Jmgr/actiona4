use std::sync::Arc;
#[cfg(not(windows))]
use std::sync::atomic::{AtomicBool, Ordering};

use pixels::{Pixels, PixelsBuilder, SurfaceTexture, wgpu};
use tokio::sync::oneshot;
use types::{
    point::{Point, point},
    rect::{Rect, rect},
    size::size,
};
#[cfg(not(windows))]
use winit::platform::x11::WindowAttributesExtX11;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId, WindowLevel},
};

use crate::{
    events::AppEvent,
    magnifier::{
        MAGNIFIER_BOX_SIZE, MAGNIFIER_OFFSET, MagnifierPipeline, MagnifierRenderInput,
        compute_magnifier_origin, create_magnifier_pipeline, update_magnifier_params,
    },
    screenshot::Screenshot,
    text::{draw_text, line_height},
};

const CROSSHAIR_GAP: u32 = 5;
const CROSSHAIR_COLOR: [u8; 4] = [255, 255, 255, 220];
const SELECTION_BORDER_COLOR: [u8; 4] = [255, 255, 255, 255];
/// How much to darken pixels outside the in-progress rect selection (0 = no
/// change, 255 = fully black). Applied as a per-channel multiplier; the alpha
/// channel of the resulting overlay is left at fully opaque.
const RECT_OVERLAY_DARKEN: u8 = 140;
const DEFAULT_ZOOM: f32 = 10.0;

#[derive(Clone, Copy)]
enum SelectionMode {
    Rect,
    Position,
}

enum SelectionResponse {
    Rect(oneshot::Sender<Option<Rect>>),
    Position(oneshot::Sender<Option<Point>>),
}

struct ActiveSelection {
    mode: SelectionMode,
    response: SelectionResponse,
}

pub struct App {
    active_selection: Option<ActiveSelection>,
    screenshot: Option<Screenshot>,
    #[cfg(not(windows))]
    proxy: EventLoopProxy<AppEvent>,
    #[cfg(not(windows))]
    cursor_tracker_stop: Option<Arc<AtomicBool>>,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    /// Pre-computed darkened-screenshot overlay, only used in rect mode.
    overlay: Vec<u8>,
    magnifier: Option<MagnifierPipeline>,
    desktop_origin: PhysicalPosition<i32>,
    drag_start: Option<PhysicalPosition<f64>>,
    current_cursor: PhysicalPosition<f64>,
}

impl App {
    pub fn new(proxy: EventLoopProxy<AppEvent>) -> Self {
        #[cfg(windows)]
        let _ = proxy;
        Self {
            active_selection: None,
            screenshot: None,
            #[cfg(not(windows))]
            proxy,
            #[cfg(not(windows))]
            cursor_tracker_stop: None,
            window: None,
            pixels: None,
            overlay: Vec::new(),
            magnifier: None,
            desktop_origin: PhysicalPosition::new(0, 0),
            drag_start: None,
            current_cursor: PhysicalPosition::new(0.0, 0.0),
        }
    }

    fn render(&mut self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let window_size = window.inner_size();
        let window_width = window_size.width;
        let window_height = window_size.height;

        self.render_frame(window_width, window_height);
        self.render_surface(window_width, window_height);
    }

    fn render_frame(&mut self, window_width: u32, window_height: u32) {
        let active_mode = self.active_mode();
        let drag_start = self.drag_start;
        let Some(pixels) = self.pixels.as_mut() else {
            return;
        };
        let frame = pixels.frame_mut();
        let screenshot_rgba = self
            .screenshot
            .as_ref()
            .map_or(&[] as &[u8], |s| s.rgba.as_slice());

        match (active_mode, drag_start) {
            (Some(SelectionMode::Rect), Some(_)) => {
                draw_rect_selection(
                    frame,
                    &self.overlay,
                    screenshot_rgba,
                    self.drag_start,
                    self.current_cursor,
                    window_width,
                    window_height,
                );
            }
            (Some(SelectionMode::Rect), None) | (Some(SelectionMode::Position), _) => {
                draw_crosshair(
                    frame,
                    screenshot_rgba,
                    window_width,
                    window_height,
                    self.current_cursor,
                );
            }
            (None, _) => {}
        }

        let global_x = self.current_cursor.x as i32 + self.desktop_origin.x;
        let global_y = self.current_cursor.y as i32 + self.desktop_origin.y;
        let rect_size = self.drag_start.map(|start| {
            let w = (start.x - self.current_cursor.x).abs() as i32;
            let h = (start.y - self.current_cursor.y).abs() as i32;
            (w, h)
        });
        draw_cursor_coords(
            frame,
            window_width,
            window_height,
            self.current_cursor,
            global_x,
            global_y,
            rect_size,
        );
    }

    fn render_surface(&mut self, window_width: u32, window_height: u32) {
        let Some(pixels) = self.pixels.as_ref() else {
            return;
        };

        let cursor_position = self.current_cursor;
        let desktop_origin = self.desktop_origin;
        let magnifier = self.magnifier.as_ref();
        let screenshot_size =
            self.screenshot
                .as_ref()
                .map_or((window_width, window_height), |screenshot| {
                    (
                        screenshot.size.width.into_inner(),
                        screenshot.size.height.into_inner(),
                    )
                });
        let zoom = DEFAULT_ZOOM;

        let _ = pixels.render_with(|encoder, render_target, context| {
            context.scaling_renderer.render(encoder, render_target);

            if let Some(magnifier) = magnifier {
                update_magnifier_params(
                    &context.queue,
                    magnifier,
                    MagnifierRenderInput {
                        cursor_position: [cursor_position.x as f32, cursor_position.y as f32],
                        desktop_origin: [desktop_origin.x as f32, desktop_origin.y as f32],
                        screenshot_size: [screenshot_size.0 as f32, screenshot_size.1 as f32],
                        window_size: [window_width as f32, window_height as f32],
                        zoom,
                    },
                );

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: render_target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                render_pass.set_pipeline(&magnifier.pipeline);
                render_pass.set_bind_group(0, &magnifier.bind_group, &[]);
                render_pass.draw(0..4, 0..1);
            }

            Ok(())
        });
    }

    fn position_selection(&self, position: PhysicalPosition<f64>) -> Point {
        let global_x = position.x as i32 + self.desktop_origin.x;
        let global_y = position.y as i32 + self.desktop_origin.y;
        point(global_x, global_y)
    }

    fn rect_selection(&self) -> Option<Rect> {
        let drag_start = self.drag_start?;

        let global_x =
            (drag_start.x.min(self.current_cursor.x) + f64::from(self.desktop_origin.x)) as i32;
        let global_y =
            (drag_start.y.min(self.current_cursor.y) + f64::from(self.desktop_origin.y)) as i32;
        let width = (drag_start.x - self.current_cursor.x).abs() as u32;
        let height = (drag_start.y - self.current_cursor.y).abs() as u32;
        Some(rect(point(global_x, global_y), size(width, height)))
    }

    fn active_mode(&self) -> Option<SelectionMode> {
        self.active_selection
            .as_ref()
            .map(|selection| selection.mode)
    }

    fn start_rect_selection(
        &mut self,
        event_loop: &ActiveEventLoop,
        screenshot: Screenshot,
        response: oneshot::Sender<Option<Rect>>,
    ) {
        if self.active_selection.is_some() {
            let _ = response.send(None);
            return;
        }

        self.active_selection = Some(ActiveSelection {
            mode: SelectionMode::Rect,
            response: SelectionResponse::Rect(response),
        });
        self.screenshot = Some(screenshot);
        self.create_selection_window(event_loop);
    }

    fn start_position_selection(
        &mut self,
        event_loop: &ActiveEventLoop,
        screenshot: Screenshot,
        response: oneshot::Sender<Option<Point>>,
    ) {
        if self.active_selection.is_some() {
            let _ = response.send(None);
            return;
        }

        self.active_selection = Some(ActiveSelection {
            mode: SelectionMode::Position,
            response: SelectionResponse::Position(response),
        });
        self.screenshot = Some(screenshot);
        self.create_selection_window(event_loop);
    }

    fn finish_position_selection(&mut self, selection: Option<Point>) {
        if let Some(active_selection) = self.active_selection.take() {
            match active_selection.response {
                SelectionResponse::Position(response) => {
                    let _ = response.send(selection);
                }
                SelectionResponse::Rect(response) => {
                    let _ = response.send(None);
                }
            }
        }
        self.clear_selection_window();
    }

    fn finish_rect_selection(&mut self, selection: Option<Rect>) {
        if let Some(active_selection) = self.active_selection.take() {
            match active_selection.response {
                SelectionResponse::Rect(response) => {
                    let _ = response.send(selection);
                }
                SelectionResponse::Position(response) => {
                    let _ = response.send(None);
                }
            }
        }
        self.clear_selection_window();
    }

    fn cancel_selection(&mut self) {
        if let Some(active_selection) = self.active_selection.take() {
            match active_selection.response {
                SelectionResponse::Rect(response) => {
                    let _ = response.send(None);
                }
                SelectionResponse::Position(response) => {
                    let _ = response.send(None);
                }
            }
        }
        self.clear_selection_window();
    }

    fn clear_selection_window(&mut self) {
        #[cfg(not(windows))]
        if let Some(stop) = self.cursor_tracker_stop.take() {
            stop.store(true, Ordering::Relaxed);
        }

        if let Some(window) = &self.window {
            window.set_visible(false);
        }
        self.window = None;
        self.pixels = None;
        self.overlay.clear();
        self.magnifier = None;
        self.screenshot = None;
        self.drag_start = None;
        self.current_cursor = PhysicalPosition::new(0.0, 0.0);
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn create_selection_window(&mut self, event_loop: &ActiveEventLoop) {
        if self.active_selection.is_none() {
            return;
        }
        if self.window.is_some() {
            return;
        }

        let (desktop_origin, desktop_size) = compute_desktop_bounds(event_loop);
        self.desktop_origin = desktop_origin;

        let window_attributes = Window::default_attributes()
            .with_position(desktop_origin)
            .with_inner_size(desktop_size)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_decorations(false)
            .with_resizable(false)
            .with_visible(false)
            .with_title("Select");
        #[cfg(not(windows))]
        let window_attributes = window_attributes.with_override_redirect(true);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let window_size = window.inner_size();

        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, Arc::clone(&window));
        let pixels = PixelsBuilder::new(window_size.width, window_size.height, surface_texture)
            .build()
            .unwrap();

        if matches!(self.active_mode(), Some(SelectionMode::Rect)) {
            let screenshot_rgba = self
                .screenshot
                .as_ref()
                .map_or(&[] as &[u8], |s| s.rgba.as_slice());
            self.overlay =
                build_rect_overlay(screenshot_rgba, window_size.width, window_size.height);
        }

        if let Some(screenshot) = &self.screenshot {
            let surface_format = pixels.render_texture_format();
            let context = pixels.context();
            self.magnifier = Some(create_magnifier_pipeline(
                context,
                surface_format,
                screenshot,
            ));
        }

        window.set_cursor_visible(false);

        #[cfg(not(windows))]
        if matches!(self.active_mode(), Some(SelectionMode::Position))
            && let Some(window_xid) = crate::cursor_tracker::get_window_xid(&window)
        {
            let stop = Arc::new(AtomicBool::new(false));
            crate::cursor_tracker::spawn_cursor_tracker(
                self.proxy.clone(),
                window_xid,
                self.desktop_origin,
                Arc::clone(&stop),
            );
            self.cursor_tracker_stop = Some(stop);
        }

        self.window = Some(window);
        self.pixels = Some(pixels);
        self.render();
        if let Some(w) = &self.window {
            w.set_visible(true);
            w.request_redraw();
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::SelectRect {
                screenshot,
                response,
            } => self.start_rect_selection(event_loop, screenshot, response),
            AppEvent::SelectPosition {
                screenshot,
                response,
            } => self.start_position_selection(event_loop, screenshot, response),
            AppEvent::Shutdown => {
                self.cancel_selection();
                event_loop.exit();
            }
            #[cfg(not(windows))]
            AppEvent::CursorMoved(position) => {
                self.current_cursor = position;
                self.request_redraw();
            }
            #[cfg(not(windows))]
            AppEvent::Click(position)
                if matches!(self.active_mode(), Some(SelectionMode::Position)) =>
            {
                let selection = self.position_selection(position);
                self.finish_position_selection(Some(selection));
            }
            #[cfg(not(windows))]
            AppEvent::Click(_) => {}
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_selection_window(event_loop);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.cancel_selection(),
            WindowEvent::Moved(position) => {
                self.desktop_origin = position;
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    self.cancel_selection();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.current_cursor = position;
                self.request_redraw();
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match (self.active_mode(), state) {
                (Some(SelectionMode::Position), ElementState::Released) => {
                    let selection = self.position_selection(self.current_cursor);
                    self.finish_position_selection(Some(selection));
                }
                (Some(SelectionMode::Rect), ElementState::Pressed) => {
                    self.drag_start = Some(self.current_cursor);
                }
                (Some(SelectionMode::Rect), ElementState::Released)
                    if self.drag_start.is_some() =>
                {
                    let selection = self.rect_selection();
                    self.finish_rect_selection(selection);
                }
                _ => {}
            },
            WindowEvent::RedrawRequested => self.render(),
            WindowEvent::Resized(size) => {
                if let Some(pixels) = &mut self.pixels {
                    let _ = pixels.resize_surface(size.width, size.height);
                    let _ = pixels.resize_buffer(size.width, size.height);
                }
                self.request_redraw();
            }
            _ => {}
        }
    }
}

fn draw_crosshair(
    frame: &mut [u8],
    screenshot_rgba: &[u8],
    window_width: u32,
    window_height: u32,
    cursor_position: PhysicalPosition<f64>,
) {
    let copy_len = frame.len().min(screenshot_rgba.len());
    frame[..copy_len].copy_from_slice(&screenshot_rgba[..copy_len]);
    frame[copy_len..].fill(0);

    let cursor_x = cursor_position.x as u32;
    let cursor_y = cursor_position.y as u32;
    let stride = window_width as usize * 4;

    if cursor_y < window_height {
        let row_start = cursor_y as usize * stride;
        for x_position in 0..window_width {
            if x_position.abs_diff(cursor_x) > CROSSHAIR_GAP {
                let pixel_index = row_start + x_position as usize * 4;
                if let Some(pixel) = frame.get_mut(pixel_index..pixel_index + 4) {
                    pixel.copy_from_slice(&CROSSHAIR_COLOR);
                }
            }
        }
    }

    let column_offset = cursor_x as usize * 4;
    for y_position in 0..window_height {
        if y_position.abs_diff(cursor_y) > CROSSHAIR_GAP {
            let pixel_index = y_position as usize * stride + column_offset;
            if let Some(pixel) = frame.get_mut(pixel_index..pixel_index + 4) {
                pixel.copy_from_slice(&CROSSHAIR_COLOR);
            }
        }
    }
}

fn draw_rect_selection(
    frame: &mut [u8],
    overlay: &[u8],
    screenshot_rgba: &[u8],
    drag_start: Option<PhysicalPosition<f64>>,
    cursor_position: PhysicalPosition<f64>,
    window_width: u32,
    window_height: u32,
) {
    let copy_length = frame.len().min(overlay.len());
    frame[..copy_length].copy_from_slice(&overlay[..copy_length]);

    let Some(drag_start) = drag_start else {
        return;
    };

    let left = drag_start.x.min(cursor_position.x) as u32;
    let top = drag_start.y.min(cursor_position.y) as u32;
    let right = (drag_start.x.max(cursor_position.x) as u32).min(window_width.saturating_sub(1));
    let bottom = (drag_start.y.max(cursor_position.y) as u32).min(window_height.saturating_sub(1));

    let stride = window_width as usize * 4;
    let row_length = (right - left + 1) as usize * 4;
    let left_offset = left as usize * 4;
    for y_position in top..=bottom {
        let row_start = y_position as usize * stride + left_offset;
        if let Some(selection_row) = frame.get_mut(row_start..row_start + row_length) {
            if let Some(src) = screenshot_rgba.get(row_start..row_start + row_length) {
                selection_row.copy_from_slice(src);
            } else {
                selection_row.fill(0);
            }
        }
    }

    for x_position in left..=right {
        paint_pixel(frame, window_width, x_position, top, SELECTION_BORDER_COLOR);
        paint_pixel(
            frame,
            window_width,
            x_position,
            bottom,
            SELECTION_BORDER_COLOR,
        );
    }

    for y_position in (top + 1)..bottom {
        paint_pixel(
            frame,
            window_width,
            left,
            y_position,
            SELECTION_BORDER_COLOR,
        );
        paint_pixel(
            frame,
            window_width,
            right,
            y_position,
            SELECTION_BORDER_COLOR,
        );
    }
}

fn paint_pixel(
    frame: &mut [u8],
    window_width: u32,
    x_position: u32,
    y_position: u32,
    color: [u8; 4],
) {
    let pixel_index = (y_position as usize * window_width as usize + x_position as usize) * 4;
    if let Some(pixel) = frame.get_mut(pixel_index..pixel_index + 4) {
        pixel.copy_from_slice(&color);
    }
}

const COORD_TEXT_COLOR: [u8; 4] = [255, 255, 255, 255];
const COORD_SHADOW_COLOR: [u8; 4] = [0, 0, 0, 255];
const COORD_BELOW_GAP: i32 = 4;

fn draw_cursor_coords(
    frame: &mut [u8],
    window_width: u32,
    window_height: u32,
    cursor: PhysicalPosition<f64>,
    global_x: i32,
    global_y: i32,
    rect_size: Option<(i32, i32)>,
) {
    let [mag_x, mag_y] = compute_magnifier_origin(
        [cursor.x as f32, cursor.y as f32],
        [window_width as f32, window_height as f32],
        MAGNIFIER_BOX_SIZE,
        MAGNIFIER_OFFSET,
    );

    let text_x = mag_x as i32;
    let text_y = mag_y as i32 + MAGNIFIER_BOX_SIZE as i32 + COORD_BELOW_GAP;

    let coord_text = format!("X:{global_x}  Y:{global_y}");
    draw_text(
        frame,
        window_width,
        window_height,
        text_x + 1,
        text_y + 1,
        &coord_text,
        COORD_SHADOW_COLOR,
    );
    draw_text(
        frame,
        window_width,
        window_height,
        text_x,
        text_y,
        &coord_text,
        COORD_TEXT_COLOR,
    );

    if let Some((w, h)) = rect_size {
        let size_text = format!("{w} x {h}");
        let size_y = text_y + line_height() + 2;
        draw_text(
            frame,
            window_width,
            window_height,
            text_x + 1,
            size_y + 1,
            &size_text,
            COORD_SHADOW_COLOR,
        );
        draw_text(
            frame,
            window_width,
            window_height,
            text_x,
            size_y,
            &size_text,
            COORD_TEXT_COLOR,
        );
    }
}

fn build_rect_overlay(screenshot_rgba: &[u8], window_width: u32, window_height: u32) -> Vec<u8> {
    let size = window_width as usize * window_height as usize * 4;
    let mut overlay = vec![0u8; size];
    let factor = u32::from(255 - RECT_OVERLAY_DARKEN);
    for (i, pixel) in overlay.chunks_exact_mut(4).enumerate() {
        let src = i * 4;
        if src + 3 < screenshot_rgba.len() {
            pixel[0] = (u32::from(screenshot_rgba[src]) * factor / 255) as u8;
            pixel[1] = (u32::from(screenshot_rgba[src + 1]) * factor / 255) as u8;
            pixel[2] = (u32::from(screenshot_rgba[src + 2]) * factor / 255) as u8;
        }
        pixel[3] = 255;
    }
    overlay
}

fn compute_desktop_bounds(
    event_loop: &ActiveEventLoop,
) -> (PhysicalPosition<i32>, PhysicalSize<u32>) {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for monitor in event_loop.available_monitors() {
        let monitor_position = monitor.position();
        let monitor_size = monitor.size();
        min_x = min_x.min(monitor_position.x);
        min_y = min_y.min(monitor_position.y);
        max_x = max_x.max(monitor_position.x + monitor_size.width as i32);
        max_y = max_y.max(monitor_position.y + monitor_size.height as i32);
    }

    if min_x == i32::MAX {
        min_x = 0;
        min_y = 0;
        max_x = 1920;
        max_y = 1080;
    }

    (
        PhysicalPosition::new(min_x, min_y),
        PhysicalSize::new((max_x - min_x) as u32, (max_y - min_y) as u32),
    )
}
