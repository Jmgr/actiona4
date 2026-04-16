use std::{fmt, sync::Arc};

use color_eyre::Result;
use parking_lot::Mutex;
use rquickjs::IntoJs;
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
    macro_player: Arc<MacroPlayer>,
    task_tracker: TaskTracker,
    cancellation_token: CancellationToken,
    listener_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
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
        Self {
            triggers: Arc::new(Mutex::new(Vec::new())),
            runtime,
            macro_player,
            task_tracker,
            cancellation_token,
            listener_cancellation_token: Arc::new(Mutex::new(None)),
        }
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

                    context
                        .async_with(async |ctx| {
                            let args = length.into_js(&ctx).map_or_else(
                                |_| {
                                    warn!(
                                        ?function_key,
                                        "failed to convert scroll length for callback"
                                    );
                                    vec![]
                                },
                                |v| vec![v],
                            );
                            fire_callback(&ctx, function_key, &player_clone, args, "onScroll");
                        })
                        .await;
                }
            }
        }

        Ok(())
    }

    fn ensure_listener_started(&self) {
        let mut listener_cancellation_token = self.listener_cancellation_token.lock();
        if listener_cancellation_token.is_some() {
            return;
        }

        let worker_cancellation_token = self.cancellation_token.child_token();
        *listener_cancellation_token = Some(worker_cancellation_token.clone());
        drop(listener_cancellation_token);

        let local_runtime = self.runtime.clone();
        let local_triggers = self.triggers.clone();
        let local_macro_player = self.macro_player.clone();
        self.task_tracker.spawn(async move {
            let guard = local_runtime.mouse_scroll();
            let mut receiver = guard.subscribe();

            loop {
                let event = match cancel_on(&worker_cancellation_token, receiver.recv()).await {
                    Ok(Ok(event)) => event,
                    Ok(Err(_)) | Err(_) => break,
                };

                Self::on_scroll(event, &local_triggers, &local_macro_player).await?;
            }

            Result::<()>::Ok(())
        });
    }

    fn stop_listener_if_running(&self) {
        if let Some(cancellation_token) = self.listener_cancellation_token.lock().take() {
            cancellation_token.cancel();
        }
    }

    pub fn add(&self, id: HandleId, axis: Axis, action: TriggerAction) {
        let was_empty = {
            let mut triggers = self.triggers.lock();
            let was_empty = triggers.is_empty();

            triggers.push(ScrollHandler { id, axis, action });

            was_empty
        };

        if was_empty {
            self.ensure_listener_started();
            self.runtime.increase_background_tasks_counter();
        }
    }

    pub fn remove(&self, id: HandleId) {
        let became_empty = {
            let mut triggers = self.triggers.lock();
            let was_empty = triggers.is_empty();

            triggers.retain(|handler| handler.id != id);

            !was_empty && triggers.is_empty()
        };

        if became_empty {
            self.stop_listener_if_running();
            self.runtime.decrease_background_tasks_counter();
        }
    }

    pub fn clear(&self) {
        {
            let mut triggers = self.triggers.lock();
            if triggers.is_empty() {
                return;
            }

            triggers.clear();
        }

        self.stop_listener_if_running();
        self.runtime.decrease_background_tasks_counter();
    }
}

impl HandleRegistry for ScrollTriggers {
    fn remove_handle(&self, id: HandleId) {
        self.remove(id);
    }
}
