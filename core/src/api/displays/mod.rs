use std::{fmt::Display, sync::Arc};

use color_eyre::eyre::eyre;

pub type Result<T> = color_eyre::Result<T>;
use itertools::Itertools;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, instrument};

use super::point::Point;
use crate::{
    api::point::try_point,
    runtime::{
        async_resource::AsyncResource,
        events::{DisplayInfo, DisplayInfoVec},
        shared_rng::SharedRng,
    },
    types::{
        display::{DisplayFields, display_list},
        su32::Su32,
    },
};

pub mod js;

#[derive(Clone, Debug)]
pub struct Displays {
    task_tracker: TaskTracker,
    displays_info: AsyncResource<DisplayInfoVec>,
}

impl Display for Displays {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.displays_info.try_get() {
            Some(info) => DisplayFields::default()
                .display("displays", display_list(info.as_slice()))
                .finish(f),
            None => DisplayFields::default().finish(f),
        }
    }
}

impl Displays {
    #[instrument(skip_all)]
    pub fn new(cancellation_token: CancellationToken, task_tracker: TaskTracker) -> Result<Self> {
        let displays_info = AsyncResource::new(cancellation_token);

        Ok(Self {
            task_tracker,
            displays_info,
        })
    }

    async fn get_info(&self) -> Result<Arc<DisplayInfoVec>> {
        if self.displays_info.try_get().is_none() {
            self.refresh();
        }
        self.displays_info.wait_get().await
    }

    pub async fn random_point(&self, rng: SharedRng) -> Result<Point> {
        let displays_info = self.get_info().await?;
        // Total area across all displays (skip zero-area just in case)
        let mut total_area = Su32::ZERO;
        for display_info in displays_info.iter() {
            let rect = display_info.rect;
            total_area += rect.size.width * rect.size.height;
        }
        if total_area == 0 {
            return Err(eyre!("no displays detected"));
        }

        // Pick a display with probability proportional to its area
        let pick = Su32::from(rng.random_range(0..total_area.into_inner())); // [0, total_area)
        let mut acc = Su32::ZERO;
        let mut chosen = None;
        for display_info in displays_info.iter() {
            let rect = display_info.rect;
            let area = rect.size.width * rect.size.height;
            if area == 0 {
                continue;
            }
            acc += area;
            if pick < acc {
                chosen = Some(rect);
                break;
            }
        }

        let rect = chosen.ok_or_else(|| eyre!("no displays detected"))?;
        drop(displays_info); // release the lock before sampling inside the rect

        // Sample uniformly inside the chosen rect.
        // Use i64 for the range math to avoid overflows on x + width, etc.
        let x_end = i64::from(rect.top_left.x) + i64::from(rect.size.width);
        let y_end = i64::from(rect.top_left.y) + i64::from(rect.size.height);

        let x = rng.random_range(i64::from(rect.top_left.x)..x_end);
        let y = rng.random_range(i64::from(rect.top_left.y)..y_end);

        try_point(x, y)
    }

    pub async fn primary_display(&self) -> Result<DisplayInfo> {
        let displays_info = self.get_info().await?;
        displays_info
            .iter()
            .find(|display| display.is_primary)
            .cloned()
            .ok_or_else(|| eyre!("no primary display found"))
    }

    pub async fn wait_get_info(&self) -> Result<Arc<DisplayInfoVec>> {
        self.get_info().await
    }

    pub async fn changed(&self) -> Result<()> {
        self.displays_info.changed().await
    }

    pub fn refresh(&self) {
        let displays_info = self.displays_info.clone();

        self.task_tracker
            .spawn_blocking(move || match display_info::DisplayInfo::all() {
                Ok(info) => {
                    displays_info.set(DisplayInfoVec::from(info));
                }
                Err(err) => error!("fetching display info failed: {err}"),
            });
    }

    pub async fn from_point(&self, point: Point) -> Result<Option<DisplayInfo>> {
        let displays_info = self.get_info().await?;

        Ok(displays_info
            .iter()
            .find(|display_info| display_info.rect.contains(point))
            .cloned())
    }

    pub async fn smallest(&self) -> Result<Option<DisplayInfo>> {
        let displays_infos = self.get_info().await?;
        Ok(displays_infos
            .iter()
            .min_by(|left_display_info, right_display_info| {
                left_display_info
                    .rect
                    .surface()
                    .cmp(&right_display_info.rect.surface())
            })
            .cloned())
    }

    pub async fn largest(&self) -> Result<Option<DisplayInfo>> {
        let displays_infos = self.get_info().await?;
        Ok(displays_infos
            .iter()
            .max_by(|left_display_info, right_display_info| {
                left_display_info
                    .rect
                    .surface()
                    .cmp(&right_display_info.rect.surface())
            })
            .cloned())
    }

    pub async fn all(&self) -> Result<Vec<DisplayInfo>> {
        let displays_infos = self.get_info().await?;
        Ok(displays_infos.iter().cloned().collect_vec())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::runtime::Runtime;

    #[test]
    #[traced_test]
    fn test_displays() {
        Runtime::test(async |_runtime| {
            /*
            let mouse = Mouse::new(runtime.clone()).await.unwrap();
            let displays = Displays::new(runtime).unwrap();

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
