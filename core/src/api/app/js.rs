use std::{collections::BTreeMap, sync::Arc};

use rquickjs::{
    Ctx, Exception, JsLifetime, Result, Value,
    class::{Trace, Tracer},
};
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{
        app::App,
        js::classes::{SingletonClass, register_enum},
    },
    built_info,
    runtime::{Runtime, WaitAtEnd},
};

pub type JsWaitAtEnd = WaitAtEnd;

/// The global application singleton, providing access to environment information
/// and execution settings.
///
/// ```ts
/// // Get the current version
/// console.log(app.version);
///
/// // Read environment variables
/// const home = app.env["HOME"];
///
/// // Change working directory
/// app.setCwd("/tmp");
/// console.log(app.cwd);
///
/// // Control whether the script waits at the end
/// app.waitAtEnd = true;
/// app.waitAtEnd = WaitAtEnd.Automatic;
/// ```
///
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
    #[instrument(skip_all)]
    pub fn new(runtime: Arc<Runtime>) -> Self {
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

    /// The version of Actiona-cli.
    ///
    /// ```ts
    /// console.log(app.version); // e.g. "0.1.0"
    /// ```
    ///
    /// @get
    #[must_use]
    #[qjs(get)]
    pub fn version(&self) -> &str {
        built_info::PKG_VERSION
    }

    /// All environment variables as a readonly key-value map.
    ///
    /// ```ts
    /// const env = app.env;
    /// console.log(env["HOME"]);
    /// console.log(env["PATH"]);
    /// ```
    ///
    /// @get
    /// @readonly
    #[must_use]
    #[qjs(get)]
    pub fn env(&self) -> BTreeMap<String, String> {
        App::env_vars()
    }

    /// The current working directory.
    ///
    /// ```ts
    /// console.log(app.cwd); // e.g. "/home/user/project"
    /// ```
    ///
    /// @get
    #[qjs(get)]
    pub fn cwd(&self, ctx: Ctx<'_>) -> Result<String> {
        std::env::current_dir()
            .map(|dir| dir.to_string_lossy().to_string())
            .into_js_result(&ctx)
    }

    /// Sets the current working directory.
    ///
    /// ```ts
    /// app.setCwd("/tmp");
    /// ```
    pub fn set_cwd(&self, ctx: Ctx<'_>, cwd: String) -> Result<()> {
        std::env::set_current_dir(&cwd).into_js_result(&ctx)?;
        Ok(())
    }

    /// The path to the running executable.
    ///
    /// ```ts
    /// console.log(app.executablePath); // e.g. "/usr/bin/actiona4-run"
    /// ```
    ///
    /// @get
    #[qjs(get)]
    pub fn executable_path(&self, ctx: Ctx<'_>) -> Result<String> {
        std::env::current_exe()
            .map(|dir| dir.to_string_lossy().to_string())
            .into_js_result(&ctx)
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::app::js::JsWaitAtEnd, built_info, runtime::Runtime};

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
