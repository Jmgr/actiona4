use std::sync::Arc;

use tokio::select;
use x11rb_async::protocol::xinput::XIEventMask;

use super::{JsKey, KeyboardImplTrait, Result};
use crate::runtime::Runtime;

#[derive(Debug)]
pub struct KeyboardImpl {
    _runtime: Arc<Runtime>,
}

impl KeyboardImpl {
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        let mut event_receiver = runtime.subcribe_events();

        let cancellation_token = runtime.cancellation_token();

        runtime.task_tracker().spawn(async move {
            loop {
                select! {
                    _ = cancellation_token.cancelled() => { break; }
                    event = event_receiver.recv() => {
                        let Ok(_event) = event else {
                            break;
                        };

                        // TODO
                        /*
                        match event {

                            _ => (),
                        }
                        */
                    }
                }
            }
        });

        Ok(Self { _runtime: runtime })
    }

    pub fn xinput_event_mask() -> XIEventMask {
        XIEventMask::RAW_KEY_PRESS | XIEventMask::RAW_KEY_RELEASE
    }
}

impl KeyboardImplTrait for KeyboardImpl {
    fn is_key_pressed(&self, _key: JsKey) -> Result<bool> {
        // TODO
        Ok(false)
    }
}
