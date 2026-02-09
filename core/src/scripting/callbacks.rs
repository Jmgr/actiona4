use std::{sync::Arc, time::Instant};

use derive_where::derive_where;
use humantime::format_duration;
use parking_lot::Mutex;
use rquickjs::{
    AsyncContext, Ctx, Function, Persistent, Result, Value, async_with, function::Args,
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
use tracing::info;

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

                    let result = {
                        let calls = &user_data.callbacks().calls;
                        let mut calls_guard = calls.lock();
                        let call = calls_guard.get_mut(call_key).unwrap(); // TODO: error
                        let parameters = call.parameters.take().unwrap().restore(&ctx).unwrap();

                        let functions = &user_data.callbacks().functions;
                        let functions_guard = functions.lock();
                        let function = functions_guard.get(call.function_key).unwrap();
                        let function = function.clone().restore(&ctx).unwrap();

                        let mut args = Args::new(ctx.clone(), parameters.len());
                        args.push_args(parameters.iter()).unwrap();
                        function.call_arg::<Value<'_>>(args).unwrap() // TODO: send Result?
                    };

                    let result = if let Some(promise) = result.as_promise() {
                        let promise = promise.clone();
                        let (sender, receiver) = oneshot::channel();
                        ctx.spawn(async move {
                            let result = promise.into_future::<Value<'_>>().await.unwrap();
                            _ = sender.send(result);
                        });
                        receiver.await.unwrap()
                    } else {
                        result
                    };

                    let calls = &user_data.callbacks().calls;
                    let mut calls_guard = calls.lock();
                    let call = calls_guard.get_mut(call_key).unwrap(); // TODO: error

                    call.result = Some(Persistent::save(&ctx, result));

                    _ = call.finished.take().unwrap().send(());
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

        _ = self.call_sender.send(call_id);

        _ = finished_receiver.await;

        info!(
            "call {:?} finished, duration: {}",
            call_id,
            format_duration(Instant::now() - start)
        );

        let result = {
            let mut calls = self.calls.lock();
            let call = calls.remove(call_id).unwrap(); // TODO: errors
            call.result.unwrap().restore(ctx)? // TODO: errors
        };

        Ok(result)
    }
}
