use std::sync::Arc;

use color_eyre::eyre::eyre;
use tokio_util::sync::CancellationToken;
use windows::Win32::{
    Foundation::POINT,
    UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON, VK_XBUTTON1, VK_XBUTTON2,
        },
        WindowsAndMessaging::{GetCursorPos, SetCursorPos},
    },
};

use super::{Button, MouseImplTrait, Result};
use crate::{
    api::{
        mouse::{ButtonConditions, Coordinate},
        point::{Point, point},
    },
    runtime::{Runtime, events::MouseButtonEvent},
};

#[derive(Clone, Debug)]
pub struct MouseImpl {
    runtime: Arc<Runtime>,
}

impl Button {
    fn into_vkey(self) -> i32 {
        match self {
            Self::Left => VK_LBUTTON,
            Self::Middle => VK_MBUTTON,
            Self::Right => VK_RBUTTON,
            Self::Back => VK_XBUTTON1,
            Self::Forward => VK_XBUTTON2,
        }
        .0
        .into()
    }
}

impl MouseImpl {
    pub async fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self { runtime })
    }

    #[allow(unsafe_code)]
    pub fn set_position(&self, position: Point, coordinate: Coordinate) -> Result<()> {
        let target_position = if coordinate == Coordinate::Abs {
            position
        } else {
            self.position()? + position
        };

        // Enigo's Windows absolute move path maps through primary-display metrics,
        // which breaks virtual-desktop coordinates on mixed-DPI multi-monitor setups.
        // Use the native cursor API so mouse positions stay aligned with display/capture rects.
        unsafe { SetCursorPos(target_position.x.into(), target_position.y.into()) }
            .map_err(|error| eyre!("{error}"))?;

        Ok(())
    }

    #[allow(unsafe_code)]
    pub fn position(&self) -> Result<Point> {
        let mut current_position = POINT::default();
        unsafe { GetCursorPos(&mut current_position) }.map_err(|error| eyre!("{error}"))?;
        Ok(point(current_position.x, current_position.y))
    }
}

#[allow(unsafe_code)]
impl MouseImplTrait for MouseImpl {
    fn is_button_pressed(&self, button: Button) -> Result<bool> {
        #[allow(clippy::as_conversions)] // i16 → u16 bitwise check, not a numeric conversion
        Ok(unsafe { GetAsyncKeyState(button.into_vkey()) as u16 & 0x8000u16 != 0 })
    }

    async fn wait_for_button(
        &self,
        conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<MouseButtonEvent> {
        super::wait_for_button(self.runtime.as_ref(), conditions, cancellation_token).await
    }
}
