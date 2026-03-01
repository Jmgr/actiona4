use std::{fmt, sync::Arc};

use color_eyre::Result;
use parking_lot::Mutex;
use rquickjs::{AsyncContext, IntoJs, async_with};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    api::{
        js::event_handle::{HandleId, HandleRegistry},
        mouse::Axis,
    },
    cancel_on,
    runtime::{Runtime, WithUserData, events::MouseScrollEvent},
    scripting::callbacks::FunctionKey,
};

struct ScrollHandler {
    id: HandleId,
    axis: Axis,
    context: AsyncContext,
    function_key: FunctionKey,
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
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        let triggers: Arc<Mutex<TriggerList>> = Arc::new(Mutex::new(Vec::new()));

        let local_runtime = runtime.clone();
        let local_triggers = triggers.clone();

        task_tracker.spawn(async move {
            let guard = local_runtime.mouse_scroll();
            let mut receiver = guard.subscribe();

            loop {
                let Ok(event) = cancel_on(&cancellation_token, receiver.recv()).await? else {
                    break;
                };

                Self::on_scroll(event, &local_triggers).await?;
            }

            Result::<()>::Ok(())
        });

        Self { triggers, runtime }
    }

    async fn on_scroll(event: MouseScrollEvent, triggers: &Arc<Mutex<TriggerList>>) -> Result<()> {
        if event.is_injected {
            return Ok(());
        }

        let to_fire: Vec<(AsyncContext, FunctionKey)> = {
            let trigger_registry = triggers.lock();

            trigger_registry
                .iter()
                .filter(|handler| handler.axis == event.axis)
                .map(|handler| (handler.context.clone(), handler.function_key))
                .collect()
        };

        let length = event.length;

        for (context, function_key) in to_fire {
            // Keep this synchronous to avoid yielding inside the callback dispatch loop.
            async_with!(context => |ctx| {
                if let Ok(length_value) = length.into_js(&ctx) {
                    ctx.user_data().callbacks().call_sync_with_arg(&ctx, function_key, length_value);
                }
            })
            .await;
        }

        Ok(())
    }

    pub fn add(&self, id: HandleId, axis: Axis, context: AsyncContext, function_key: FunctionKey) {
        let mut triggers = self.triggers.lock();
        let was_empty = triggers.is_empty();

        triggers.push(ScrollHandler {
            id,
            axis,
            context,
            function_key,
        });

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
