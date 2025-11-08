use std::sync::Arc;

use rquickjs::{
    Ctx, Exception, JsLifetime, Result, Value,
    class::{Trace, Tracer},
};

use crate::{
    built_info,
    core::js::classes::{SingletonClass, register_enum},
    runtime::{Runtime, WaitAtEnd},
};

pub type JsWaitAtEnd = WaitAtEnd;

/// @singleton
/// @prop waitAtEnd: WaitAtEnd | boolean // Should the app wait at the end of execution
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "App")]
pub struct JsApp {
    runtime: Arc<Runtime>,
}

impl<'js> Trace<'js> for JsApp {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl SingletonClass<'_> for JsApp {
    fn register_dependencies(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        register_enum::<JsWaitAtEnd>(ctx)?;
        Ok(())
    }
}

impl JsApp {
    /// @skip
    #[must_use]
    pub const fn new(runtime: Arc<Runtime>) -> Self {
        Self { runtime }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsApp {
    /// Should the script pause at the end?
    /// @skip
    #[must_use]
    #[qjs(get, rename = "waitAtEnd")]
    pub fn get_wait_at_end(&self) -> JsWaitAtEnd {
        self.runtime.wait_at_end()
    }

    /// Should the script pause at the end?
    /// @skip
    #[qjs(set, rename = "waitAtEnd")]
    pub fn set_wait_at_end(&self, ctx: Ctx<'_>, value: Value<'_>) -> Result<()> {
        let value = if let Ok(value) = value.get::<JsWaitAtEnd>() {
            value
        } else if let Some(value) = value.as_bool() {
            if value {
                JsWaitAtEnd::Yes
            } else {
                JsWaitAtEnd::No
            }
        } else {
            return Err(Exception::throw_type(
                &ctx,
                "expected either WaitAtEnd or a boolean",
            ));
        };

        self.runtime.set_wait_at_end(value);

        Ok(())
    }

    /// Version of Actiona-cli
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn version(&self) -> &str {
        built_info::PKG_VERSION
    }
}

#[cfg(test)]
mod tests {
    use crate::{built_info, core::app::js::JsWaitAtEnd, runtime::Runtime};

    #[test]
    fn test_wait_at_end() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval::<JsWaitAtEnd>(
                    r#"
            app.waitAtEnd = true;
            app.waitAtEnd
                "#,
                )
                .await
                .unwrap();
            assert_eq!(result, JsWaitAtEnd::Yes);

            let result = script_engine
                .eval::<JsWaitAtEnd>(
                    r#"
            app.waitAtEnd = WaitAtEnd.Automatic;
            app.waitAtEnd
                "#,
                )
                .await
                .unwrap();
            assert_eq!(result, JsWaitAtEnd::Automatic);
        });
    }

    #[test]
    fn test_version() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine.eval::<String>("app.version").await.unwrap();
            assert_eq!(result, built_info::PKG_VERSION);
        });
    }
}
