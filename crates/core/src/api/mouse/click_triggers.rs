use std::{collections::HashSet, fmt, sync::Arc};

use color_eyre::Result;
use macros::options;
use parking_lot::Mutex;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    api::{
        js::event_handle::{HandleId, HandleRegistry},
        macros::player::MacroPlayer,
        mouse::Button,
        triggers::TriggerAction,
    },
    cancel_on,
    runtime::{Runtime, events::MouseButtonEvent},
};

struct ClickHandler {
    id: HandleId,
    button: Button,
    action: TriggerAction,
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
    macro_player: Arc<MacroPlayer>,
    task_tracker: TaskTracker,
    cancellation_token: CancellationToken,
    listener_cancellation_token: Arc<Mutex<Option<CancellationToken>>>,
}

impl fmt::Debug for ClickTriggers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClickTriggers").finish_non_exhaustive()
    }
}

impl ClickTriggers {
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

    async fn on_button(
        event: MouseButtonEvent,
        pressed_buttons: &mut HashSet<Button>,
        fired: &mut HashSet<(Button, HandleId)>,
        triggers: &Arc<Mutex<TriggerList>>,
        macro_player: &Arc<MacroPlayer>,
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
        let to_fire: Vec<(HandleId, TriggerAction)> = {
            let trigger_registry = triggers.lock();

            trigger_registry
                .iter()
                .filter(|handler| {
                    !fired.contains(&(handler.button, handler.id))
                        && handler.button == event.button
                        && (!handler.options.exclusive || pressed_buttons.len() == 1)
                })
                .map(|handler| (handler.id, handler.action.clone()))
                .collect()
        };

        for (handle_id, action) in to_fire {
            fired.insert((event.button, handle_id));
            action.fire(macro_player, "onButton").await;
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
            let guard = local_runtime.mouse_buttons();
            let mut receiver = guard.subscribe();
            let mut pressed_buttons: HashSet<Button> = HashSet::new();
            // Track which handle IDs have already fired this press cycle.
            let mut fired: HashSet<(Button, HandleId)> = HashSet::new();

            loop {
                let event = match cancel_on(&worker_cancellation_token, receiver.recv()).await {
                    Ok(Ok(event)) => event,
                    Ok(Err(_)) | Err(_) => break,
                };

                Self::on_button(
                    event,
                    &mut pressed_buttons,
                    &mut fired,
                    &local_triggers,
                    &local_macro_player,
                )
                .await?;
            }

            Result::<()>::Ok(())
        });
    }

    fn stop_listener_if_running(&self) {
        let cancellation_token = self.listener_cancellation_token.lock().take();
        if let Some(cancellation_token) = cancellation_token {
            cancellation_token.cancel();
        }
    }

    pub fn add(
        &self,
        id: HandleId,
        button: Button,
        action: TriggerAction,
        options: OnButtonOptions,
    ) {
        let was_empty = {
            let mut triggers = self.triggers.lock();
            let was_empty = triggers.is_empty();

            triggers.push(ClickHandler {
                id,
                button,
                action,
                options,
            });

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

impl HandleRegistry for ClickTriggers {
    fn remove_handle(&self, id: HandleId) {
        self.remove(id);
    }
}
