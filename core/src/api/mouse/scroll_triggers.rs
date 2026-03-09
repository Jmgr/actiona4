use std::{fmt, sync::Arc};

use color_eyre::Result;
use parking_lot::Mutex;
use rquickjs::{IntoJs, async_with};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::warn;

use crate::{
    api::{
        js::event_handle::{HandleId, HandleRegistry},
        macros::{PlayConfig, player::MacroPlayer},
        mouse::Axis,
        triggers::{TriggerAction, fire_callback},
    },
    cancel_on,
    runtime::{Runtime, events::MouseScrollEvent},
};

struct ScrollHandler {
    id: HandleId,
    axis: Axis,
    action: TriggerAction,
}

type TriggerList = Vec<ScrollHandler>;

#[derive(Clone)]
pub struct ScrollTriggers {
    triggers: Arc<Mutex<TriggerList>>,
    runtime: Arc<Runtime>,
}

impl fmt::Debug for ScrollTriggers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScrollTriggers").finish_non_exhaustive()
    }
}

impl ScrollTriggers {
    pub fn new(
        runtime: Arc<Runtime>,
        macro_player: Arc<MacroPlayer>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        let triggers: Arc<Mutex<TriggerList>> = Arc::new(Mutex::new(Vec::new()));

        let local_runtime = runtime.clone();
        let local_macro_player = macro_player;
        let local_triggers = triggers.clone();

        task_tracker.spawn(async move {
            let guard = local_runtime.mouse_scroll();
            let mut receiver = guard.subscribe();

            loop {
                let Ok(event) = cancel_on(&cancellation_token, receiver.recv()).await? else {
                    break;
                };

                Self::on_scroll(event, &local_triggers, &local_macro_player).await?;
            }

            Result::<()>::Ok(())
        });

        Self { triggers, runtime }
    }

    async fn on_scroll(
        event: MouseScrollEvent,
        triggers: &Arc<Mutex<TriggerList>>,
        macro_player: &Arc<MacroPlayer>,
    ) -> Result<()> {
        if event.is_injected {
            return Ok(());
        }

        let to_fire: Vec<TriggerAction> = {
            let trigger_registry = triggers.lock();

            trigger_registry
                .iter()
                .filter(|handler| handler.axis == event.axis)
                .map(|handler| handler.action.clone())
                .collect()
        };

        let length = event.length;

        for action in to_fire {
            match action {
                TriggerAction::Macro(data) => {
                    macro_player.play_detached(data, PlayConfig::default())
                }
                TriggerAction::Callback(context, function_key) => {
                    let player_clone = macro_player.clone();
                    async_with!(context => |ctx| {
                        let args = length.into_js(&ctx).map_or_else(|_| {
                            warn!(?function_key, "failed to convert scroll length for callback");
                            vec![]
                        }, |v| vec![v]);
                        fire_callback(&ctx, function_key, &player_clone, args, "onScroll");
                    })
                    .await;
                }
            }
        }

        Ok(())
    }

    pub fn add(&self, id: HandleId, axis: Axis, action: TriggerAction) {
        let mut triggers = self.triggers.lock();
        let was_empty = triggers.is_empty();

        triggers.push(ScrollHandler { id, axis, action });

        if was_empty {
            self.runtime.increase_background_tasks_counter();
        }
    }

    pub fn remove(&self, id: HandleId) {
        let mut triggers = self.triggers.lock();
        let was_empty = triggers.is_empty();

        triggers.retain(|handler| handler.id != id);

        if !was_empty && triggers.is_empty() {
            self.runtime.decrease_background_tasks_counter();
        }
    }

    pub fn clear(&self) {
        let mut triggers = self.triggers.lock();
        if triggers.is_empty() {
            return;
        }

        triggers.clear();
        self.runtime.decrease_background_tasks_counter();
    }
}

impl HandleRegistry for ScrollTriggers {
    fn remove_handle(&self, id: HandleId) {
        self.remove(id);
    }
}
