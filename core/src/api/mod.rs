use rquickjs::{Ctx, Exception, Result, Value};

pub mod app;
pub mod audio;
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
pub mod macros;
pub mod mouse;
pub mod name;
pub mod notification;
pub mod path;
pub mod point;
pub mod process;
pub mod random;
pub mod rect;
pub mod screen;
pub mod size;
pub mod standardpaths;
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

#[cfg(test)]
pub(crate) mod test_helpers {
    use std::{
        env::temp_dir,
        path::{Path, PathBuf},
    };

    use macros::{js_class, js_methods};
    use rand::RngExt;
    use rquickjs::{JsLifetime, atom::PredefinedAtom, class::Trace};

    use crate::{api::js::classes::ValueClass, types::display::display_with_type};

    pub fn random_name() -> String {
        use rand::distr::Alphanumeric;

        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect()
    }

    pub fn random_temp_filename() -> PathBuf {
        temp_dir().join(format!("text_{}.txt", random_name()))
    }

    pub fn js_string(value: impl AsRef<str>) -> String {
        serde_json::to_string(value.as_ref()).unwrap()
    }

    pub fn js_path(path: impl AsRef<Path>) -> String {
        js_string(path.as_ref().to_string_lossy().as_ref())
    }

    #[derive(Clone, Debug, Default, JsLifetime, Trace)]
    #[js_class]
    pub struct JsCounter {
        count: u64,
    }

    impl<'js> ValueClass<'js> for JsCounter {}

    #[js_methods]
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

        #[qjs(rename = PredefinedAtom::ToString)]
        #[must_use]
        pub fn to_string_js(&self) -> String {
            display_with_type("Counter", self.count)
        }
    }
}
