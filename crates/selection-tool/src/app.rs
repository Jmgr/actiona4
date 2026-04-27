use std::sync::Arc;

use actiona_common::selection::{Color, PositionSelection, RectSelection};
use pixels::{Pixels, PixelsBuilder, SurfaceTexture, wgpu};
use types::{point::point, size::size};
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
    cli::Mode,
    events::AppEvent,
    magnifier::{
        MAGNIFIER_BOX_SIZE, MAGNIFIER_OFFSET, MagnifierPipeline, MagnifierRenderInput,
        compute_magnifier_origin, create_magnifier_pipeline, update_magnifier_params,
    },
    screenshot::{Screenshot, screenshot_color_at},
    text::{draw_text, line_height},
};

const CROSSHAIR_GAP: u32 = 5;
const CROSSHAIR_COLOR: [u8; 4] = [255, 255, 255, 220];
const SELECTION_BORDER_COLOR: [u8; 4] = [255, 255, 255, 255];
const RECT_OVERLAY_ALPHA: u8 = 140;
const DEFAULT_ZOOM: f32 = 10.0;

pub struct App {
    mode: Mode,
    screenshot: Option<Screenshot>,
    #[cfg(not(windows))]
    proxy: EventLoopProxy<AppEvent>,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    /// Pre-computed darkened-screenshot overlay, only used in rect mode.
    overlay: Vec<u8>,
    magnifier: Option<MagnifierPipeline>,
    desktop_origin: PhysicalPosition<i32>,
    drag_start: Option<PhysicalPosition<f64>>,
    current_cursor: PhysicalPosition<f64>,
    is_dragging: bool,
}

impl App {
    pub fn new(
        mode: Mode,
        screenshot: Option<Screenshot>,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Self {
        #[cfg(windows)]
        let _ = proxy;
        Self {
            mode,
            screenshot,
            #[cfg(not(windows))]
            proxy,
            window: None,
            pixels: None,
            overlay: Vec::new(),
            magnifier: None,
            desktop_origin: PhysicalPosition::new(0, 0),
            drag_start: None,
            current_cursor: PhysicalPosition::new(0.0, 0.0),
            is_dragging: false,
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
        let Some(pixels) = self.pixels.as_mut() else {
            return;
        };
        let frame = pixels.frame_mut();
        let screenshot_rgba = self
            .screenshot
            .as_ref()
            .map_or(&[] as &[u8], |s| s.rgba.as_slice());

        match (&self.mode, self.is_dragging) {
            (Mode::Rect, false) | (Mode::Pos { .. }, _) => {
                draw_crosshair(
                    frame,
                    screenshot_rgba,
                    window_width,
                    window_height,
                    self.current_cursor,
                );
            }
            (Mode::Rect, true) => {
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
        }

        let global_x = self.current_cursor.x as i32 + self.desktop_origin.x;
        let global_y = self.current_cursor.y as i32 + self.desktop_origin.y;
        let rect_size = if self.is_dragging {
            self.drag_start.map(|start| {
                let w = (start.x - self.current_cursor.x).abs() as i32;
                let h = (start.y - self.current_cursor.y).abs() as i32;
                (w, h)
            })
        } else {
            None
        };
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
        let screenshot_size = self
            .screenshot
            .as_ref()
            .map_or((window_width, window_height), |screenshot| {
                (screenshot.width, screenshot.height)
            });
        let zoom = match self.mode {
            Mode::Pos { zoom } => zoom,
            Mode::Rect => DEFAULT_ZOOM,
        };

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

    fn print_position_selection(&self, position: PhysicalPosition<f64>) {
        let global_x = position.x as i32 + self.desktop_origin.x;
        let global_y = position.y as i32 + self.desktop_origin.y;
        let color = self
            .screenshot
            .as_ref()
            .and_then(|screenshot| screenshot_color_at(screenshot, global_x, global_y))
            .map(|[r, g, b]| Color { r, g, b });

        let result = PositionSelection {
            point: point(global_x, global_y),
            color,
        };
        println!(
            "{}",
            serde_json::to_string(&result).expect("serialization failed")
        );
    }

    fn print_rect_selection(&self) {
        let Some(drag_start) = self.drag_start else {
            return;
        };

        let global_x =
            (drag_start.x.min(self.current_cursor.x) + f64::from(self.desktop_origin.x)) as i32;
        let global_y =
            (drag_start.y.min(self.current_cursor.y) + f64::from(self.desktop_origin.y)) as i32;
        let width = (drag_start.x - self.current_cursor.x).abs() as i32;
        let height = (drag_start.y - self.current_cursor.y).abs() as i32;
        let result = RectSelection {
            top_left: point(global_x, global_y),
            size: size(width, height),
        };
        println!(
            "{}",
            serde_json::to_string(&result).expect("serialization failed")
        );
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        #[cfg(not(windows))]
        {
            match event {
                AppEvent::CursorMoved(position) => {
                    self.current_cursor = position;
                    self.request_redraw();
                }
                AppEvent::Click(position) if matches!(self.mode, Mode::Pos { .. }) => {
                    self.print_position_selection(position);
                    event_loop.exit();
                }
                AppEvent::Click(_) => {}
            }
        }
        #[cfg(windows)]
        {
            let _ = event_loop;
            let _ = event;
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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

        if matches!(self.mode, Mode::Rect) {
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
        if matches!(self.mode, Mode::Pos { .. })
            && let Some(window_xid) = crate::cursor_tracker::get_window_xid(&window)
        {
            crate::cursor_tracker::spawn_cursor_tracker(
                self.proxy.clone(),
                window_xid,
                self.desktop_origin,
            );
        }

        self.window = Some(window);
        self.pixels = Some(pixels);
        self.render();
        if let Some(w) = &self.window {
            w.set_visible(true);
            w.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Moved(position) => {
                self.desktop_origin = position;
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    event_loop.exit();
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
            } => match (&self.mode, state) {
                (Mode::Pos { .. }, ElementState::Released) => {
                    self.print_position_selection(self.current_cursor);
                    event_loop.exit();
                }
                (Mode::Rect, ElementState::Pressed) => {
                    self.drag_start = Some(self.current_cursor);
                    self.is_dragging = true;
                }
                (Mode::Rect, ElementState::Released) if self.is_dragging => {
                    self.print_rect_selection();
                    event_loop.exit();
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

    if cursor_y < window_height {
        for x_position in 0..window_width {
            if x_position.abs_diff(cursor_x) > CROSSHAIR_GAP {
                let pixel_index = ((cursor_y * window_width + x_position) * 4) as usize;
                if let Some(pixel) = frame.get_mut(pixel_index..pixel_index + 4) {
                    pixel.copy_from_slice(&CROSSHAIR_COLOR);
                }
            }
        }
    }

    for y_position in 0..window_height {
        if y_position.abs_diff(cursor_y) > CROSSHAIR_GAP {
            let pixel_index = ((y_position * window_width + cursor_x) * 4) as usize;
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

    for y_position in top..=bottom {
        let row_start = ((y_position * window_width + left) * 4) as usize;
        let row_length = ((right - left + 1) * 4) as usize;
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
    let pixel_index = ((y_position * window_width + x_position) * 4) as usize;
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
    let size = (window_width * window_height * 4) as usize;
    let mut overlay = vec![0u8; size];
    let factor = (255 - RECT_OVERLAY_ALPHA) as u32;
    for (i, pixel) in overlay.chunks_exact_mut(4).enumerate() {
        let src = i * 4;
        if src + 3 < screenshot_rgba.len() {
            pixel[0] = (screenshot_rgba[src] as u32 * factor / 255) as u8;
            pixel[1] = (screenshot_rgba[src + 1] as u32 * factor / 255) as u8;
            pixel[2] = (screenshot_rgba[src + 2] as u32 * factor / 255) as u8;
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
