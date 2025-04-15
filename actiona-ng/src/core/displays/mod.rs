use std::sync::{Arc, Mutex};

use display_info::error::DIError;
use rand::Rng;
use thiserror::Error;
use tokio::select;

use crate::runtime::{DisplayInfo, DisplayInfoVec, RecordEvent, Runtime};

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
    _implementation: DisplaysImpl,
    displays_info: Arc<Mutex<DisplayInfoVec>>,
}

impl Displays {
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        let mut event_receiver = runtime.subcribe_events();

        let displays_info = Arc::new(Mutex::new(display_info::DisplayInfo::all()?.into()));
        let local_displays_info = displays_info.clone();

        let cancellation_token = runtime.cancellation_token();

        runtime.task_tracker().spawn(async move {
            loop {
                select! {
                    _ = cancellation_token.cancelled() => { break; }
                    event = event_receiver.recv() => {
                        let Ok(event) = event else {
                            break;
                        };

                        if let RecordEvent::DisplayChanged(infos) = event {
                            let mut displays_info = local_displays_info.lock().unwrap();
                            *displays_info = infos;
                        }
                    }
                }
            }
        });

        Ok(Self {
            _implementation: DisplaysImpl::new(runtime)?,
            displays_info,
        })
    }

    pub fn random_point(&self) -> Result<Point> {
        let displays_info = self.displays_info.lock().unwrap();
        let mut rng = rand::rng();

        let random_display = displays_info
            .0
            .get(rng.random_range(0..displays_info.len()))
            .ok_or(DisplaysError::NoDisplays)?;
        let rect = random_display.rect;

        let random_point = point(
            rng.random_range(rect.x..(rect.x + rect.width as i32)),
            rng.random_range(rect.y..(rect.y + rect.height as i32)),
        );

        Ok(random_point)
    }

    pub fn primary_display(&self) -> Result<DisplayInfo> {
        let displays_info = self.displays_info.lock().unwrap();
        displays_info
            .iter()
            .find(|display| display.is_primary)
            .cloned()
            .ok_or(DisplaysError::NoPrimaryDisplay)
    }

    pub fn displays_info(&self) -> &Arc<Mutex<DisplayInfoVec>> {
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
    use std::time::Duration;

    use tokio::time::sleep;
    use tracing_test::traced_test;

    use crate::{core::displays::Displays, runtime::Runtime};

    #[test]
    #[traced_test]
    fn test_displays() {
        Runtime::test(async |runtime| {
            //let mut mouse = Mouse::new().unwrap();

            let displays = Displays::new(runtime).unwrap();
            let displays_info = displays.displays_info().lock().unwrap();
            for display_info in displays_info.iter() {
                println!("display_info {display_info:?}");
            }

            /*
            display_info DisplayInfo { id: 65537, name: "\\\\.\\DISPLAY1", friendly_name: "Acer XB281HK", raw_handle: HMONITOR(0x10001), x: 0, y: 0, width: 3840, height: 2160, width_mm: 621, height_mm: 341, rotation: 0.0, scale_factor: 1.5, frequency: 60.0, is_primary: true }
            display_info DisplayInfo { id: 65539, name: "\\\\.\\DISPLAY2", friendly_name: "2490W1", raw_handle: HMONITOR(0x10003), x: -1920, y: 541, width: 1920, height: 1080, width_mm: 527, height_mm: 296, rotation: 0.0, scale_factor: 1.0, frequency: 60.0, is_primary: false }
            display_info DisplayInfo { id: 65541, name: "\\\\.\\DISPLAY3", friendly_name: "SyncMaster", raw_handle: HMONITOR(0x10005), x: 3840, y: 558, width: 1920, height: 1080, width_mm: 510, height_mm: 287, rotation: 0.0, scale_factor: 1.0, frequency: 60.0, is_primary: false }

            display_info DisplayInfo { id: 474, name: "DP-2", friendly_name: "DP-2", x: 1920, y: 0, width: 3840, height: 2160, width_mm: 621, height_mm: 341, rotation: 0.0, scale_factor: 1.0, frequency: 59.996624, is_primary: true }
            display_info DisplayInfo { id: 469, name: "DP-0", friendly_name: "DP-0", x: 0, y: 649, width: 1920, height: 1080, width_mm: 527, height_mm: 296, rotation: 0.0, scale_factor: 1.0, frequency: 60.0, is_primary: false }
            display_info DisplayInfo { id: 444, name: "HDMI-0", friendly_name: "HDMI-0", x: 5760, y: 601, width: 1920, height: 1080, width_mm: 510, height_mm: 287, rotation: 0.0, scale_factor: 1.0, frequency: 60.0, is_primary: false }
                         */

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
