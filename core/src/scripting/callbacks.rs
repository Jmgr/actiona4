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
new_key_type! { pub(crate) struct CallKey; }

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

                // SAFETY: Required due to unsafe operations within rquickjs::async_with! macro
                #[allow(unsafe_op_in_unsafe_fn)]
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

    /// Call a registered function synchronously with a single argument within the current JS context.
    ///
    /// Same guarantees as `call_sync`, but passes one argument to the callback.
    pub fn call_sync_with_arg<'js>(
        &self,
        ctx: &Ctx<'js>,
        function_key: FunctionKey,
        arg: Value<'js>,
    ) {
        self.call_sync_fire_and_forget(ctx, function_key, &[arg], "call_sync_with_arg");
    }

    /// Call a registered function synchronously and return its raw result.
    ///
    /// Like `call_sync`, this executes directly without yielding inside `async_with!`, so the
    /// rquickjs scheduler's queue waker is not overwritten.
    ///
    /// Unlike `call_sync`, the return value is given back to the caller. If the function returns
    /// a Promise, the caller is responsible for spawning it (e.g. via `ctx.spawn`). Returns
    /// `undefined` on any error.
    pub fn call_sync_returning<'js>(
        &self,
        ctx: &Ctx<'js>,
        function_key: FunctionKey,
        args: Vec<Value<'js>>,
    ) -> Value<'js> {
        let Some(function) = self.get_function(ctx, function_key, "call_sync_returning") else {
            return Value::new_undefined(ctx.clone());
        };
        let mut args_obj = Args::new(ctx.clone(), args.len());
        if let Err(error) = args_obj.push_args(args.iter()) {
            warn!(
                ?function_key,
                error = %error,
                "call_sync_returning: failed to push arguments"
            );
            return Value::new_undefined(ctx.clone());
        }
        match function.call_arg::<Value<'_>>(args_obj) {
            Ok(result) => result,
            Err(error) => {
                warn!(
                    ?function_key,
                    error = %error,
                    "call_sync_returning: callback failed"
                );
                Value::new_undefined(ctx.clone())
            }
        }
    }

    /// Call a registered function synchronously within the current JS context.
    ///
    /// Unlike `call`, this executes directly without going through the callback worker, so it
    /// does not yield inside an `async_with!` block. This preserves the rquickjs scheduler's
    /// queue waker registration, which `call` would otherwise overwrite.
    ///
    /// If the callback returns a Promise it is spawned into the scheduler for background
    /// execution.
    pub fn call_sync<'js>(&self, ctx: &Ctx<'js>, function_key: FunctionKey) {
        self.call_sync_fire_and_forget(ctx, function_key, &[], "call_sync");
    }

    fn get_function<'js>(
        &self,
        ctx: &Ctx<'js>,
        function_key: FunctionKey,
        caller: &str,
    ) -> Option<Function<'js>> {
        let function = {
            let functions = self.functions.lock();
            functions.get(function_key).cloned()
        };
        let Some(function) = function else {
            warn!(
                ?function_key,
                "{caller}: callback function is not registered"
            );
            return None;
        };
        match function.restore(ctx) {
            Ok(function) => Some(function),
            Err(error) => {
                warn!(
                    ?function_key,
                    error = %error,
                    "{caller}: failed to restore callback function"
                );
                None
            }
        }
    }

    fn call_sync_fire_and_forget<'js>(
        &self,
        ctx: &Ctx<'js>,
        function_key: FunctionKey,
        args: &[Value<'js>],
        caller: &'js str,
    ) {
        let Some(function) = self.get_function(ctx, function_key, caller) else {
            return;
        };
        let mut args_obj = Args::new(ctx.clone(), args.len());
        if let Err(error) = args_obj.push_args(args.iter()) {
            warn!(?function_key, error = %error, "{caller}: failed to push arguments");
            return;
        }
        match function.call_arg::<Value<'_>>(args_obj) {
            Ok(result) => {
                if let Some(promise) = result.as_promise() {
                    let promise = promise.clone();
                    ctx.spawn(async move {
                        if let Err(error) = promise.into_future::<Value<'_>>().await {
                            warn!(?function_key, error = %error, "{caller}: async callback failed");
                        }
                    });
                }
            }
            Err(error) => {
                warn!(?function_key, error = %error, "{caller}: callback failed");
            }
        }
    }

    /// Prepare a callback call without awaiting its completion.
    ///
    /// This is the first phase of the split-call pattern that avoids yielding inside
    /// `async_with!`. Call this synchronously within a non-yielding `async_with!` closure,
    /// then `.await` the returned receiver **outside** any `async_with!`, and finally call
    /// `retrieve_result` inside another non-yielding `async_with!`.
    ///
    /// Returns `None` if the callback worker is not running.
    pub(crate) fn prepare_call<'js>(
        &self,
        ctx: &Ctx<'js>,
        function_key: FunctionKey,
        args: Vec<Value<'js>>,
    ) -> Option<(CallKey, oneshot::Receiver<()>)> {
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

        if self.call_sender.send(call_id).is_err() {
            warn!(
                ?function_key,
                ?call_id,
                "prepare_call: failed to queue callback call because callback worker is not running"
            );
            let mut calls = self.calls.lock();
            _ = calls.remove(call_id);
            return None;
        }

        Some((call_id, finished_receiver))
    }

    /// Retrieve the result of a completed callback call.
    ///
    /// This is the third phase of the split-call pattern. Call this inside a non-yielding
    /// `async_with!` closure after the receiver from `prepare_call` has resolved.
    pub(crate) fn retrieve_result<'js>(
        &self,
        ctx: &Ctx<'js>,
        call_id: CallKey,
    ) -> Result<Value<'js>> {
        let mut calls = self.calls.lock();
        let call = calls.remove(call_id).ok_or_else(|| {
            warn!(
                ?call_id,
                "retrieve_result: callback call state not found after worker completion"
            );
            Exception::throw_message(ctx, "Callback call state not found")
        })?;
        let result = call.result.ok_or_else(|| {
            warn!(
                ?call_id,
                "retrieve_result: callback call completed without a result"
            );
            Exception::throw_message(ctx, "Callback call completed without a result")
        })?;
        result.restore(ctx)
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
