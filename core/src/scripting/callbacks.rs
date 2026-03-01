use std::{sync::Arc, time::Instant};

use derive_where::derive_where;
use humantime::format_duration;
use parking_lot::Mutex;
use rquickjs::{
    AsyncContext, Ctx, Exception, Function, Persistent, Result, Value, async_with, function::Args,
};
use slotmap::{SlotMap, new_key_type};
use tokio::{
    select,
    sync::{
        mpsc::{self},
        oneshot,
    },
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, warn};

use crate::runtime::WithUserData;

struct Call {
    function_key: FunctionKey,
    parameters: Option<Persistent<Vec<Value<'static>>>>,
    result: Option<Persistent<Value<'static>>>,
    finished: Option<oneshot::Sender<()>>,
}

new_key_type! { pub struct FunctionKey; }
new_key_type! { struct CallKey; }

#[derive_where(Debug)]
pub struct Callbacks {
    /// Callback functions
    functions: Arc<Mutex<SlotMap<FunctionKey, Persistent<Function<'static>>>>>,

    /// Function calls
    #[derive_where(skip)]
    calls: Arc<Mutex<SlotMap<CallKey, Call>>>,

    call_sender: mpsc::UnboundedSender<CallKey>,
}
impl Callbacks {
    pub fn new(
        context: AsyncContext,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Self {
        info!("Callbacks::new");

        let (call_sender, mut call_receiver) = mpsc::unbounded_channel();
        let functions = Default::default();
        let calls = Default::default();

        task_tracker.spawn(async move {
            info!("Callbacks::worker start");
            loop {
                let call_key = select! {
                    _ = cancellation_token.cancelled() => { break; },
                    call_key = call_receiver.recv() => call_key,
                };

                let Some(call_key) = call_key else {
                    break;
                };

                info!("Callbacks::call");

                async_with!(context => |ctx| {
                    let user_data = ctx.user_data();
                    let mut result = Value::new_undefined(ctx.clone());

                    let (function_key, parameters) = {
                        let calls = &user_data.callbacks().calls;
                        let mut calls_guard = calls.lock();
                        let Some(call) = calls_guard.get_mut(call_key) else {
                            warn!(?call_key, "callback call state was missing before execution");
                            return;
                        };
                        (call.function_key, call.parameters.take())
                    };

                    let parameters = parameters.map_or_else(
                        || {
                            warn!(
                                ?call_key,
                                ?function_key,
                                "callback call has no parameters"
                            );
                            None
                        },
                        |parameters| match parameters.restore(&ctx) {
                            Ok(parameters) => Some(parameters),
                            Err(error) => {
                                warn!(
                                    ?call_key,
                                    ?function_key,
                                    error = %error,
                                    "failed to restore callback parameters"
                                );
                                None
                            }
                        },
                    );

                    if let Some(parameters) = parameters {
                        let functions = &user_data.callbacks().functions;
                        let functions_guard = functions.lock();
                        if let Some(function) = functions_guard.get(function_key) {
                            match function.clone().restore(&ctx) {
                                Ok(function) => {
                                    let mut args = Args::new(ctx.clone(), parameters.len());
                                    if let Err(error) = args.push_args(parameters.iter()) {
                                        warn!(
                                            ?call_key,
                                            ?function_key,
                                            error = %error,
                                            argument_count = parameters.len(),
                                            "failed to push callback arguments"
                                        );
                                    } else {
                                        match function.call_arg::<Value<'_>>(args) {
                                            Ok(call_result) => result = call_result,
                                            Err(error) => {
                                                warn!(
                                                    ?call_key,
                                                    ?function_key,
                                                    error = %error,
                                                    "callback function call failed"
                                                );
                                            }
                                        }
                                    }
                                }
                                Err(error) => {
                                    warn!(
                                        ?call_key,
                                        ?function_key,
                                        error = %error,
                                        "failed to restore callback function"
                                    );
                                }
                            }
                        } else {
                            warn!(
                                ?call_key,
                                ?function_key,
                                "callback function is not registered"
                            );
                        }
                    }

                    let result = if let Some(promise) = result.as_promise() {
                        let promise = promise.clone();
                        let promise_ctx = ctx.clone();
                        let (sender, receiver) = oneshot::channel();
                        let promise_call_key = call_key;
                        ctx.spawn(async move {
                            let result = promise
                                .into_future::<Value<'_>>()
                                .await
                                .unwrap_or_else(|error| {
                                    warn!(
                                        ?promise_call_key,
                                        error = %error,
                                        fallback = "undefined",
                                        "callback promise failed; defaulting to undefined"
                                    );
                                    Value::new_undefined(promise_ctx)
                                });
                            if sender.send(result).is_err() {
                                warn!(
                                    ?promise_call_key,
                                    "callback promise result receiver was dropped"
                                );
                            }
                        });
                        receiver
                            .await
                            .unwrap_or_else(|error| {
                                warn!(
                                    ?call_key,
                                    error = %error,
                                    fallback = "undefined",
                                    "failed to receive callback promise result"
                                );
                                Value::new_undefined(ctx.clone())
                            })
                    } else {
                        result
                    };

                    let calls = &user_data.callbacks().calls;
                    let mut calls_guard = calls.lock();
                    let Some(call) = calls_guard.get_mut(call_key) else {
                        warn!(?call_key, "callback call state was missing after execution");
                        return;
                    };

                    call.result = Some(Persistent::save(&ctx, result));

                    if let Some(finished) = call.finished.take()
                        && finished.send(()).is_err()
                    {
                        warn!(?call_key, "callback completion receiver was dropped");
                    }
                })
                .await;
            }
        });

        Self {
            functions,
            calls,
            call_sender,
        }
    }

    /// Call a registered function synchronously within the current JS context.
    ///
    /// Unlike [`call`], this executes directly without going through the callback worker, so it
    /// does not yield inside an `async_with!` block. This preserves the rquickjs scheduler's
    /// queue waker registration, which `call` would otherwise overwrite.
    ///
    /// If the callback returns a Promise it is spawned into the scheduler for background
    /// execution.
    pub fn call_sync<'js>(&self, ctx: &Ctx<'js>, function_key: FunctionKey) {
        let function = {
            let functions = self.functions.lock();
            functions.get(function_key).cloned()
        };
        let Some(function) = function else {
            warn!(
                ?function_key,
                "call_sync: callback function is not registered"
            );
            return;
        };
        let function = match function.restore(ctx) {
            Ok(function) => function,
            Err(error) => {
                warn!(
                    ?function_key,
                    error = %error,
                    "call_sync: failed to restore callback function"
                );
                return;
            }
        };
        match function.call_arg::<Value<'_>>(Args::new(ctx.clone(), 0)) {
            Ok(result) => {
                if let Some(promise) = result.as_promise() {
                    let promise = promise.clone();
                    ctx.spawn(async move {
                        if let Err(error) = promise.into_future::<Value<'_>>().await {
                            warn!(
                                ?function_key,
                                error = %error,
                                "call_sync: async callback failed"
                            );
                        }
                    });
                }
            }
            Err(error) => {
                warn!(?function_key, error = %error, "call_sync: callback failed");
            }
        }
    }

    pub fn register<'js>(&self, ctx: &Ctx<'js>, function: Function<'js>) -> FunctionKey {
        let mut functions = self.functions.lock();
        let key = functions.insert(Persistent::save(ctx, function));
        info!("register function with key {:?}", key);
        key
    }

    pub async fn call<'js>(
        &self,
        ctx: &Ctx<'js>,
        function_key: FunctionKey,
        args: Vec<Value<'js>>, // TODO: Option
    ) -> Result<Value<'js>> {
        let (finished_sender, finished_receiver) = oneshot::channel();
        let call_id = {
            let mut calls = self.calls.lock();
            calls.insert(Call {
                function_key,
                parameters: Some(Persistent::save(ctx, args)),
                result: None,
                finished: Some(finished_sender),
            })
        };
        info!("calling function {:?}, call id {:?}", function_key, call_id);

        let start = Instant::now();

        if self.call_sender.send(call_id).is_err() {
            warn!(
                ?function_key,
                ?call_id,
                "failed to queue callback call because callback worker is not running"
            );
            let mut calls = self.calls.lock();
            _ = calls.remove(call_id);
            return Err(Exception::throw_message(
                ctx,
                "Callback worker is not running",
            ));
        }

        finished_receiver.await.map_err(|error| {
            warn!(
                ?function_key,
                ?call_id,
                error = %error,
                "callback worker dropped before finishing"
            );
            Exception::throw_message(ctx, "Callback worker dropped before finishing")
        })?;

        info!(
            "call {:?} finished, duration: {}",
            call_id,
            format_duration(Instant::now() - start)
        );

        let result = {
            let mut calls = self.calls.lock();
            let call = calls.remove(call_id).ok_or_else(|| {
                warn!(
                    ?function_key,
                    ?call_id,
                    "callback call state not found after worker completion"
                );
                Exception::throw_message(ctx, "Callback call state not found")
            })?;
            let result = call.result.ok_or_else(|| {
                warn!(
                    ?function_key,
                    ?call_id,
                    "callback call completed without a result"
                );
                Exception::throw_message(ctx, "Callback call completed without a result")
            })?;
            result.restore(ctx)?
        };

        Ok(result)
    }
}
