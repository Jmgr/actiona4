use std::{fmt::Display, sync::Arc};

use color_eyre::eyre::eyre;

pub type Result<T> = color_eyre::Result<T>;
use itertools::Itertools;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, instrument};

use super::point::Point;
use crate::{
    api::{point::try_point, rect::Rect},
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

pub mod display_selector;
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

    pub async fn primary(&self) -> Result<DisplayInfo> {
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

    pub async fn leftmost(&self) -> Result<Option<DisplayInfo>> {
        let displays_infos = self.get_info().await?;
        Ok(displays_infos
            .iter()
            .min_by_key(|d| d.rect.top_left.x)
            .cloned())
    }

    pub async fn rightmost(&self) -> Result<Option<DisplayInfo>> {
        let displays_infos = self.get_info().await?;
        Ok(displays_infos
            .iter()
            .max_by_key(|d| d.rect.top_left.x + d.rect.size.width)
            .cloned())
    }

    pub async fn topmost(&self) -> Result<Option<DisplayInfo>> {
        let displays_infos = self.get_info().await?;
        Ok(displays_infos
            .iter()
            .min_by_key(|d| d.rect.top_left.y)
            .cloned())
    }

    pub async fn bottommost(&self) -> Result<Option<DisplayInfo>> {
        let displays_infos = self.get_info().await?;
        Ok(displays_infos
            .iter()
            .max_by_key(|d| d.rect.top_left.y + d.rect.size.height)
            .cloned())
    }

    pub async fn center(&self) -> Result<Option<DisplayInfo>> {
        let displays_infos = self.get_info().await?;
        let mut iter = displays_infos.iter();
        let Some(first) = iter.next() else {
            return Ok(None);
        };
        let desktop: Rect = iter.fold(first.rect, |acc, d| acc.union(d.rect));
        // Use 2*center to avoid division: 2*c = top_left*2 + size (preserves ordering)
        let desktop_c2 = desktop.top_left * 2 + desktop.size;
        Ok(displays_infos
            .iter()
            .min_by_key(|d| {
                let diff = (d.rect.top_left * 2 + d.rect.size) - desktop_c2;
                diff.length_squared()
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
    use rstest::rstest;

    use super::*;
    use crate::{
        api::{point::point, rect::Rect, size::size},
        runtime::events::{DisplayInfo, DisplayInfoVec},
    };

    fn make_display(id: u32, x: i32, y: i32, w: u32, h: u32) -> DisplayInfo {
        DisplayInfo {
            id,
            name: id.to_string(),
            friendly_name: id.to_string(),
            rect: Rect::new(point(x, y), size(w, h)),
            width_mm: 0,
            height_mm: 0,
            rotation: 0.0,
            scale_factor: 1.0,
            frequency: 60.0,
            is_primary: false,
        }
    }

    fn with_displays(infos: Vec<DisplayInfo>) -> Displays {
        let ct = CancellationToken::new();
        let tracker = TaskTracker::new();
        let displays = Displays::new(ct, tracker).unwrap();
        displays.displays_info.set(DisplayInfoVec(infos));
        displays
    }

    // ---- empty / single -------------------------------------------------------

    #[tokio::test]
    async fn empty_returns_none_for_all() {
        let d = with_displays(vec![]);
        assert!(d.leftmost().await.unwrap().is_none());
        assert!(d.rightmost().await.unwrap().is_none());
        assert!(d.topmost().await.unwrap().is_none());
        assert!(d.bottommost().await.unwrap().is_none());
        assert!(d.center().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn single_display_returned_for_all() {
        let d = with_displays(vec![make_display(1, 100, 200, 1920, 1080)]);
        assert_eq!(Some(1), d.leftmost().await.unwrap().map(|d| d.id));
        assert_eq!(Some(1), d.rightmost().await.unwrap().map(|d| d.id));
        assert_eq!(Some(1), d.topmost().await.unwrap().map(|d| d.id));
        assert_eq!(Some(1), d.bottommost().await.unwrap().map(|d| d.id));
        assert_eq!(Some(1), d.center().await.unwrap().map(|d| d.id));
    }

    // ---- leftmost / rightmost -------------------------------------------------

    #[rstest]
    // id=2 has x=-500, the smallest left edge
    #[case::negative_x(vec![
        make_display(1,  100, 0, 1920, 1080),
        make_display(2, -500, 0, 1920, 1080),
        make_display(3,  200, 0, 1920, 1080),
    ], 2)]
    // leftmost is the one whose left edge is furthest left, not the narrowest
    #[case::positive_only(vec![
        make_display(1, 300, 0, 1920, 1080),
        make_display(2,   0, 0, 1920, 1080),
        make_display(3, 150, 0, 1920, 1080),
    ], 2)]
    #[tokio::test]
    async fn leftmost(#[case] infos: Vec<DisplayInfo>, #[case] want_id: u32) {
        let d = with_displays(infos);
        assert_eq!(Some(want_id), d.leftmost().await.unwrap().map(|d| d.id));
    }

    #[rstest]
    // id=2: x=2000 + w=800 = right edge 2800
    #[case::offset_wide(vec![
        make_display(1,    0, 0, 1920, 1080),
        make_display(2, 2000, 0,  800, 1080),
        make_display(3, -100, 0,  200, 1080),
    ], 2)]
    // id=1: x=0 + w=3840 = 3840 beats id=2: x=1920 + w=1920 = 3840 (tie → first wins in stable iter)
    // but id=3: x=3840 + w=1920 = 5760
    #[case::large_offset(vec![
        make_display(1,    0, 0, 1920, 1080),
        make_display(2, 1920, 0, 1920, 1080),
        make_display(3, 3840, 0, 1920, 1080),
    ], 3)]
    #[tokio::test]
    async fn rightmost(#[case] infos: Vec<DisplayInfo>, #[case] want_id: u32) {
        let d = with_displays(infos);
        assert_eq!(Some(want_id), d.rightmost().await.unwrap().map(|d| d.id));
    }

    // ---- topmost / bottommost -------------------------------------------------

    #[rstest]
    #[case::negative_y(vec![
        make_display(1, 0,    0, 1920, 1080),
        make_display(2, 0, -200, 1920, 1080),
        make_display(3, 0,  500, 1920, 1080),
    ], 2)]
    #[tokio::test]
    async fn topmost(#[case] infos: Vec<DisplayInfo>, #[case] want_id: u32) {
        let d = with_displays(infos);
        assert_eq!(Some(want_id), d.topmost().await.unwrap().map(|d| d.id));
    }

    #[rstest]
    // id=2: y=500 + h=800 = bottom 1300
    #[case::offset_tall(vec![
        make_display(1, 0,    0, 1920, 1080),
        make_display(2, 0,  500, 1920,  800),
        make_display(3, 0, -100, 1920,  400),
    ], 2)]
    #[tokio::test]
    async fn bottommost(#[case] infos: Vec<DisplayInfo>, #[case] want_id: u32) {
        let d = with_displays(infos);
        assert_eq!(Some(want_id), d.bottommost().await.unwrap().map(|d| d.id));
    }

    // ---- center ---------------------------------------------------------------

    #[rstest]
    // Three equal-sized displays side by side; middle is exactly at desktop center
    #[case::three_horizontal(vec![
        make_display(1,    0, 0, 1920, 1080),
        make_display(2, 1920, 0, 1920, 1080),
        make_display(3, 3840, 0, 1920, 1080),
    ], 2)]
    // Asymmetric: desktop spans (-1920,0)-(3840+1920), center ≈ x=1920; id=2 is closest
    #[case::asymmetric(vec![
        make_display(1, -1920, 0, 1920, 1080),
        make_display(2,     0, 0, 3840, 1080),
        make_display(3,  3840, 0, 1920, 1080),
    ], 2)]
    #[tokio::test]
    async fn center(#[case] infos: Vec<DisplayInfo>, #[case] want_id: u32) {
        let d = with_displays(infos);
        assert_eq!(Some(want_id), d.center().await.unwrap().map(|d| d.id));
    }
}
