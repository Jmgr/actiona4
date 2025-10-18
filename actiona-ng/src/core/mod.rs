use eyre::eyre;
use rquickjs::{Ctx, Exception, Result, Value};
use tokio::{select, sync::watch};

use crate::{core::js::task::IsDone, runtime::WithUserData};

pub mod clipboard;
pub mod color;
pub mod console;
pub mod directory;
pub mod displays;
pub mod file;
pub mod filesystem;
pub mod image;
pub mod js;
pub mod keyboard;
pub mod mouse;
pub mod name;
pub mod path;
pub mod point;
pub mod random;
pub mod rect;
pub mod screenshot;
pub mod size;
pub mod system;
pub mod ui;
pub mod web;
pub mod windows;

pub trait ResultExt<T> {
    fn or_throw_message(self, ctx: &Ctx, msg: &str) -> Result<T>;
}

impl<T> ResultExt<T> for Option<T> {
    fn or_throw_message(self, ctx: &Ctx, msg: &str) -> Result<T> {
        self.ok_or_else(|| Exception::throw_message(ctx, msg))
    }
}

pub fn check_min_arg_count(min: usize, ctx: &Ctx, args: &[Value<'_>]) -> Result<()> {
    if args.len() < min {
        return Err(Exception::throw_message(
            ctx,
            &format!(
                "Expected at least {min} arguments, but {} were provided",
                args.len()
            ),
        ));
    }

    Ok(())
}

pub fn convert_watch_receiver<'js, FromT, ToT>(
    ctx: &Ctx<'js>,
    mut from_receiver: watch::Receiver<FromT>,
) -> watch::Receiver<ToT>
where
    ToT: Default + From<FromT> + Sync + Send + 'static,
    FromT: IsDone + Clone + Sync + Send + 'static,
{
    let (new_sender, to_receiver) = watch::channel(ToT::default());
    let token = ctx.user_data().cancellation_token().clone();
    ctx.user_data().task_tracker().spawn(async move {
        loop {
            select! {
                _ = token.cancelled() => { break; },
                _ = from_receiver.changed() => {
                    let value = from_receiver.borrow_and_update().clone();
                    let is_done = value.is_done();
                    new_sender.send_replace(value.into());

                    if is_done {
                        break;
                    }
                },
            }
        }
    });

    to_receiver
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use std::{env::temp_dir, path::PathBuf};

    use rquickjs::{JsLifetime, class::Trace};

    use crate::core::js::classes::ValueClass;

    pub(crate) fn random_name() -> String {
        use rand::{Rng, distr::Alphanumeric};

        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect()
    }

    pub(crate) fn random_temp_filename() -> PathBuf {
        temp_dir().join(format!("text_{}.txt", random_name()))
    }

    #[derive(Clone, Debug, Default, JsLifetime, Trace)]
    #[rquickjs::class(rename = "Counter")]
    pub struct JsCounter {
        count: u64,
    }

    impl<'js> ValueClass<'js> for JsCounter {}

    #[rquickjs::methods(rename_all = "camelCase")]
    impl JsCounter {
        #[qjs(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        pub fn increase(&mut self) {
            self.count += 1;
        }

        pub fn value(&self) -> u64 {
            self.count
        }
    }
}

pub trait ToIntClamped {
    fn to_i32_clamped(self) -> eyre::Result<i32>;
    fn to_u32_clamped(self) -> eyre::Result<u32>;
}

impl ToIntClamped for f64 {
    #[inline]
    fn to_i32_clamped(self) -> eyre::Result<i32> {
        if self.is_nan() {
            return Err(eyre!("value is not a number"));
        }

        let value = self
            .round()
            .clamp(Self::from(i32::MIN), Self::from(i32::MAX));

        #[allow(clippy::as_conversions, reason = "range checked via clamp")]
        {
            Ok(value as i32)
        }
    }

    #[inline]
    fn to_u32_clamped(self) -> eyre::Result<u32> {
        if self.is_nan() {
            return Err(eyre!("value is not a number"));
        }

        let value = self
            .round()
            .clamp(Self::from(u32::MIN), Self::from(u32::MAX));

        #[allow(clippy::as_conversions, reason = "range checked via clamp")]
        {
            Ok(value as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamped() {
        assert_eq!(10.0.to_i32_clamped().unwrap(), 10);
        assert!(f64::NAN.to_i32_clamped().is_err());
        assert_eq!(f64::INFINITY.to_i32_clamped().unwrap(), i32::MAX);
        assert_eq!(f64::NEG_INFINITY.to_i32_clamped().unwrap(), i32::MIN);
    }
}
