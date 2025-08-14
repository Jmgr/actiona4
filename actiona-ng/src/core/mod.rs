#[cfg(test)]
use std::{env::temp_dir, path::PathBuf};

#[cfg(test)]
use rand::{Rng, distr::Alphanumeric};
use rquickjs::{Ctx, Exception, Result, Value};

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
pub mod rect;
pub mod screenshot;
pub mod ui;
pub mod web;

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

#[cfg(test)]
pub(crate) fn random_name() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}

#[cfg(test)]
pub(crate) fn random_temp_filename() -> PathBuf {
    temp_dir().join(format!("text_{}.txt", random_name()))
}
