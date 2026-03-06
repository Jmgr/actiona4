use std::{collections::HashSet, fmt, sync::Arc};

use color_eyre::Result;
use macros::options;
use parking_lot::Mutex;
use rquickjs::{AsyncContext, async_with};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    api::{
        js::event_handle::{HandleId, HandleRegistry},
        mouse::Button,
    },
    cancel_on,
    runtime::{Runtime, WithUserData, events::MouseButtonEvent},
    scripting::callbacks::FunctionKey,
};

struct ClickHandler {
    id: HandleId,
    button: Button,
    context: AsyncContext,
    function_key: FunctionKey,
    options: OnButtonOptions,
}

type TriggerList = Vec<ClickHandler>;

/// Options for a click trigger.
#[options]
#[derive(Clone, Copy, Debug)]
pub struct OnButtonOptions {
    /// Require exactly this button and no others to be pressed.
    pub exclusive: bool,
}

#[derive(Clone)]
pub struct ClickTriggers {
    triggers: Arc<Mutex<TriggerList>>,
    runtime: Arc<Runtime>,
}

impl fmt::Debug for ClickTriggers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClickTriggers").finish_non_exhaustive()
    }
}

impl ClickTriggers {
    pub fn new(
        runtime: Arc<Runtime>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Self {
        let triggers: Arc<Mutex<TriggerList>> = Arc::new(Mutex::new(Vec::new()));

        let local_runtime = runtime.clone();
        let local_triggers = triggers.clone();

        task_tracker.spawn(async move {
            let guard = local_runtime.mouse_buttons();
            let mut receiver = guard.subscribe();
            let mut pressed_buttons: HashSet<Button> = HashSet::new();
            // Track which handle IDs have already fired this press cycle.
            let mut fired: HashSet<(Button, HandleId)> = HashSet::new();

            loop {
                let Ok(event) = cancel_on(&cancellation_token, receiver.recv()).await? else {
                    break;
                };

                Self::on_button(event, &mut pressed_buttons, &mut fired, &local_triggers).await?;
            }

            Result::<()>::Ok(())
        });

        Self { triggers, runtime }
    }

    async fn on_button(
        event: MouseButtonEvent,
        pressed_buttons: &mut HashSet<Button>,
        fired: &mut HashSet<(Button, HandleId)>,
        triggers: &Arc<Mutex<TriggerList>>,
    ) -> Result<()> {
        if event.is_injected {
            return Ok(());
        }

        if event.direction.is_release() {
            pressed_buttons.remove(&event.button);
            fired.retain(|(b, _)| b != &event.button);
            return Ok(());
        }

        pressed_buttons.insert(event.button);

        // Collect handlers to fire: those whose trigger matches and haven't fired yet.
        let to_fire: Vec<(HandleId, AsyncContext, FunctionKey)> = {
            let trigger_registry = triggers.lock();

            trigger_registry
                .iter()
                .filter(|handler| {
                    !fired.contains(&(handler.button, handler.id))
                        && handler.button == event.button
                        && (!handler.options.exclusive || pressed_buttons.len() == 1)
                })
                .map(|handler| (handler.id, handler.context.clone(), handler.function_key))
                .collect()
        };

        for (handle_id, context, function_key) in to_fire {
            fired.insert((event.button, handle_id));

            // Keep this synchronous to avoid yielding inside the callback dispatch loop.
            async_with!(context => |ctx| {
                ctx.user_data().callbacks().call_sync(&ctx, function_key);
            })
            .await;
        }

        Ok(())
    }

    pub fn add(
        &self,
        id: HandleId,
        button: Button,
        context: AsyncContext,
        function_key: FunctionKey,
        options: OnButtonOptions,
    ) {
        let mut triggers = self.triggers.lock();
        let was_empty = triggers.is_empty();

        triggers.push(ClickHandler {
            id,
            button,
            context,
            function_key,
            options,
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

impl HandleRegistry for ClickTriggers {
    fn remove_handle(&self, id: HandleId) {
        self.remove(id);
    }
}
