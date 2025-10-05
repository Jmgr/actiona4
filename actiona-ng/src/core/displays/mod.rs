use std::sync::{Arc, Mutex};

use display_info::error::DIError;
use thiserror::Error;
use tokio::select;

use crate::runtime::{
    Runtime,
    events::{DisplayInfo, DisplayInfoVec},
    shared_rng::SharedRng,
};

pub(crate) mod platform;

#[cfg(windows)]
use platform::win::DisplaysImpl;
#[cfg(unix)]
use platform::x11::DisplaysImpl;

use super::point::{Point, point};

pub mod js;

#[derive(Debug, Error)]
pub enum DisplaysError {
    #[error("Connection to the X11 server failed: {0}")]
    ConnectionError(String),

    #[error("Display info error: {0}")]
    DisplayInfoError(#[from] DIError),

    #[error("No displays detected")]
    NoDisplays,

    #[error("No primary display found")]
    NoPrimaryDisplay,
}

pub type Result<T> = std::result::Result<T, DisplaysError>;

#[derive(Debug)]
pub struct Displays {
    implementation: DisplaysImpl,
    displays_info: Arc<Mutex<DisplayInfoVec>>,
}

impl Displays {
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        let displays_info = Arc::new(Mutex::new(display_info::DisplayInfo::all()?.into()));
        let local_displays_info = displays_info.clone();

        let cancellation_token = runtime.cancellation_token();
        let screen_change_guard = runtime.platform().subscribe_screen_change();
        let mut screen_change_receiver = screen_change_guard.subscribe();

        runtime.task_tracker().spawn(async move {
            loop {
                select! {
                    _ = cancellation_token.cancelled() => { break; }
                    event = screen_change_receiver.changed() => {
                        if event.is_err() { break; }
                    }
                }

                let mut displays_info = local_displays_info.lock().unwrap();
                *displays_info = screen_change_receiver.borrow_and_update().clone();
            }
        });

        Ok(Self {
            implementation: DisplaysImpl::new(runtime)?,
            displays_info,
        })
    }

    pub fn random_point(&self, rng: SharedRng) -> Result<Point> {
        let displays_info = self.displays_info.lock().unwrap();
        // Total area across all displays (skip zero-area just in case)
        let mut total_area: u64 = 0;
        for display_info in &displays_info.0 {
            let rect = display_info.rect;
            total_area += (rect.width as u64) * (rect.height as u64);
        }
        if total_area == 0 {
            return Err(DisplaysError::NoDisplays);
        }

        // Pick a display with probability proportional to its area
        let pick = rng.random_range(0..total_area); // [0, total_area)
        let mut acc = 0;
        let mut chosen = None;
        for display_info in &displays_info.0 {
            let rect = display_info.rect;
            let area = (rect.width as u64) * (rect.height as u64);
            if area == 0 {
                continue;
            }
            acc += area;
            if pick < acc {
                chosen = Some(rect);
                break;
            }
        }

        let rect = chosen.ok_or(DisplaysError::NoDisplays)?;
        drop(displays_info); // release the lock before sampling inside the rect

        // Sample uniformly inside the chosen rect.
        // Use i64 for the range math to avoid overflows on x + width, etc.
        let x_end = rect.x as i64 + rect.width as i64;
        let y_end = rect.y as i64 + rect.height as i64;

        let x = rng.random_range(rect.x as i64..x_end) as i32;
        let y = rng.random_range(rect.y as i64..y_end) as i32;

        Ok(point(x, y))
    }

    pub fn primary_display(&self) -> Result<DisplayInfo> {
        let displays_info = self.displays_info.lock().unwrap();
        displays_info
            .iter()
            .find(|display| display.is_primary)
            .cloned()
            .ok_or(DisplaysError::NoPrimaryDisplay)
    }

    pub const fn displays_info(&self) -> &Arc<Mutex<DisplayInfoVec>> {
        &self.displays_info
    }

    pub fn from_point(&self, point: Point) -> Option<DisplayInfo> {
        let displays_info = self.displays_info.lock().unwrap();

        displays_info
            .iter()
            .find(|display_info| display_info.rect.contains(point))
            .cloned()
    }

    pub fn smallest(&self) -> Option<DisplayInfo> {
        let displays_infos = self.displays_info.lock().unwrap();
        displays_infos
            .iter()
            .min_by(|left_display_info, right_display_info| {
                left_display_info
                    .rect
                    .surface()
                    .cmp(&right_display_info.rect.surface())
            })
            .cloned()
    }

    pub fn largest(&self) -> Option<DisplayInfo> {
        let displays_infos = self.displays_info.lock().unwrap();
        displays_infos
            .iter()
            .max_by(|left_display_info, right_display_info| {
                left_display_info
                    .rect
                    .surface()
                    .cmp(&right_display_info.rect.surface())
            })
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{
        core::{displays::Displays, mouse::Mouse},
        runtime::Runtime,
    };

    #[test]
    #[traced_test]
    fn test_displays() {
        Runtime::test(async |runtime| {
            let mouse = Mouse::new(runtime.clone()).await.unwrap();
            let displays = Displays::new(runtime).unwrap();

            /*
            let displays = Displays::new(runtime).unwrap();
            let displays_info = displays.displays_info().lock().unwrap();
            for display_info in displays_info.iter() {
                println!("display_info {display_info:?}");
            }
            */

            /*
            display_info DisplayInfo { id: 65537, name: "\\\\.\\DISPLAY1", friendly_name: "Acer XB281HK", raw_handle: HMONITOR(0x10001), x: 0, y: 0, width: 3840, height: 2160, width_mm: 621, height_mm: 341, rotation: 0.0, scale_factor: 1.5, frequency: 60.0, is_primary: true }
            display_info DisplayInfo { id: 65539, name: "\\\\.\\DISPLAY2", friendly_name: "2490W1", raw_handle: HMONITOR(0x10003), x: -1920, y: 541, width: 1920, height: 1080, width_mm: 527, height_mm: 296, rotation: 0.0, scale_factor: 1.0, frequency: 60.0, is_primary: false }
            display_info DisplayInfo { id: 65541, name: "\\\\.\\DISPLAY3", friendly_name: "SyncMaster", raw_handle: HMONITOR(0x10005), x: 3840, y: 558, width: 1920, height: 1080, width_mm: 510, height_mm: 287, rotation: 0.0, scale_factor: 1.0, frequency: 60.0, is_primary: false }

            display_info DisplayInfo { id: 474, name: "DP-2", friendly_name: "DP-2", x: 1920, y: 0, width: 3840, height: 2160, width_mm: 621, height_mm: 341, rotation: 0.0, scale_factor: 1.0, frequency: 59.996624, is_primary: true }
            display_info DisplayInfo { id: 469, name: "DP-0", friendly_name: "DP-0", x: 0, y: 649, width: 1920, height: 1080, width_mm: 527, height_mm: 296, rotation: 0.0, scale_factor: 1.0, frequency: 60.0, is_primary: false }
            display_info DisplayInfo { id: 444, name: "HDMI-0", friendly_name: "HDMI-0", x: 5760, y: 601, width: 1920, height: 1080, width_mm: 510, height_mm: 287, rotation: 0.0, scale_factor: 1.0, frequency: 60.0, is_primary: false }
                         */

            //mouse
            //    .set_position(displays.random_point().unwrap(), Coordinate::Abs)
            //   .unwrap();

            //for _ in 0..60 {
            //mouse
            //    .r#move(displays.random_point().unwrap(), MoveOptions::default())
            //    .unwrap();

            //let c = runtime.displays().screen_count().unwrap();
            //println!("screen count: {c}");

            //sleep(Duration::from_millis(1000)).await;
            //}
        });
    }
}
